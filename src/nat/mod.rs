use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use bimap::BiMap;
use colored::Colorize;
use ipnet::{Ipv4Net, Ipv6Net};
use tokio::process::Command;
use tun_tap::{Iface, Mode};

use crate::nat::{
    packet::{make_ipv4_packet, make_ipv6_packet},
    utils::{bytes_to_hex_str, bytes_to_ipv4_addr, bytes_to_ipv6_addr, ipv4_to_ipv6},
};

mod packet;
mod utils;

/// A cleaner way to execute an `ip` command
macro_rules! iproute2 {
    ($($arg:expr),*) => {{
        Command::new("ip")
            $(.arg($arg))*
            .status()
    }}
}

pub struct Nat64 {
    /// Handle for the TUN interface
    interface: Iface,
    /// Instance IPv4 address
    instance_v4: Ipv4Addr,
    /// Instance IPv6 address
    instance_v6: Ipv6Addr,
    /// IPv4 pool
    ipv4_pool: Vec<Ipv4Net>,
    /// IPv6 prefix
    ipv6_prefix: Ipv6Net,
    /// A mapping of currently allocated pool reservations
    pool_reservations: BiMap<Ipv4Addr, Ipv6Addr>,
}

impl Nat64 {
    /// Bring up a new NAT64 interface
    ///
    /// **Arguments:**
    /// - `nat_v4`: An IPv4 address to assign to this NAT instance for ICMP and other purposes
    /// - `nat_v6`: An IPv6 address to assign to this NAT instance for ICMP and other purposes
    /// - `ipv4_pool`: A list of IPv4 prefixes to communicate from
    /// - `ipv6_prefix`: The IPv6 prefix to listen on (should generally be `64:ff9b::/96`)
    pub async fn new(
        nat_v4: Ipv4Addr,
        nat_v6: Ipv6Addr,
        ipv4_pool: Vec<Ipv4Net>,
        ipv6_prefix: Ipv6Net,
        static_mappings: Vec<(Ipv4Addr, Ipv6Addr)>,
    ) -> Result<Self, std::io::Error> {
        // Bring up tun interface
        let interface = Iface::new("nat64i%d", Mode::Tun)?;

        // Configure the interface
        let interface_name = interface.name();
        log::info!("Configuring interface {}", interface_name);

        // Add the nat addresses
        log::debug!("Assigning {} to {}", nat_v4, interface_name);
        iproute2!(
            "address",
            "add",
            format!("{}/32", nat_v4),
            "dev",
            interface_name
        )
        .await?;
        log::debug!("Assigning {} to {}", nat_v6, interface_name);
        iproute2!(
            "address",
            "add",
            format!("{}/128", nat_v6),
            "dev",
            interface_name
        )
        .await?;

        // Bring up the interface
        log::debug!("Bringing up {}", interface_name);
        iproute2!("link", "set", "dev", interface_name, "up").await?;

        // Add route for IPv6 prefix
        log::debug!("Adding route {} via {}", ipv6_prefix, interface_name);
        iproute2!(
            "route",
            "add",
            ipv6_prefix.to_string(),
            "dev",
            interface_name
        )
        .await?;

        // Add every IPv4 prefix to the routing table
        for prefix in ipv4_pool.iter() {
            log::debug!("Adding route {} via {}", prefix, interface_name);
            iproute2!("route", "add", prefix.to_string(), "dev", interface_name).await?;
        }

        // Build a reservation list
        let mut pool_reservations = BiMap::new();
        for (v4, v6) in static_mappings {
            pool_reservations.insert(v4, v6);
        }
        pool_reservations.insert(nat_v4, nat_v6);

        Ok(Self {
            interface,
            instance_v4: nat_v4,
            instance_v6: nat_v6,
            ipv4_pool,
            ipv6_prefix,
            pool_reservations,
        })
    }

    /// Block and run the NAT instance. This will handle all packets
    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        // Read the interface MTU
        let mtu: u16 =
            std::fs::read_to_string(format!("/sys/class/net/{}/mtu", self.interface.name()))
                .expect("Failed to read interface MTU")
                .strip_suffix("\n")
                .unwrap()
                .parse()
                .unwrap();

        // Allocate a buffer for incoming packets
        // NOTE: Add 4 to account for the Tun header
        let mut buffer = vec![0; (mtu as usize) + 4];

        log::info!("Translating packets");
        loop {
            // Read incoming packet
            let len = self.interface.recv(&mut buffer)?;

            // Process the packet
            let response = self.process(&buffer[..len]).await?;

            // If there is a response, send it
            if let Some(response) = response {
                self.interface.send(&response)?;
            }
        }
    }

    /// Internal function that checks if a destination address is allowed to be processed
    // fn is_dest_allowed(&self, dest: IpAddr) -> bool {
    //     return dest == self.instance_v4
    //         || dest == self.instance_v6
    //         || match dest {
    //             IpAddr::V4(addr) => self.ipv4_pool.iter().any(|prefix| prefix.contains(&addr)),
    //             IpAddr::V6(addr) => self.ipv6_prefix.contains(&addr),
    //         };
    // }

    /// Calculate a unique IPv4 address inside the pool for a given IPv6 address
    fn calculate_ipv4(&self, _addr: Ipv6Addr) -> Option<Ipv4Addr> {
        // Search the list of possible IPv4 addresses
        for prefix in self.ipv4_pool.iter() {
            for addr in prefix.hosts() {
                // If this address is avalible, use it
                if !self.pool_reservations.contains_left(&addr) {
                    return Some(addr);
                }
            }
        }

        None
    }

    /// Internal function to process an incoming packet.
    /// If `Some` is returned, the result is sent back out the interface
    async fn process(&mut self, packet: &[u8]) -> Result<Option<Vec<u8>>, std::io::Error> {
        // Ignore the first 4 bytes, which are the Tun header
        let tun_header = &packet[..4];
        let packet = &packet[4..];

        // Log the packet
        log::debug!("Processing packet with length: {}", packet.len());
        log::debug!(
            "> Tun Header: {}",
            bytes_to_hex_str(tun_header).bright_cyan()
        );
        log::debug!("> IP Header: {}", bytes_to_hex_str(packet).bright_cyan());

        match packet[0] >> 4 {
            4 => {
                // Parse the source and destination addresses
                let source_addr = bytes_to_ipv4_addr(&packet[12..16]);
                let dest_addr = bytes_to_ipv4_addr(&packet[16..20]);
                log::debug!("> Source: {}", source_addr.to_string().bright_cyan());
                log::debug!("> Destination: {}", dest_addr.to_string().bright_cyan());

                // Only accept packets destined to hosts in the reservation list
                // TODO: Should also probably let the nat addr pass
                if !self.pool_reservations.contains_left(&dest_addr) {
                    log::debug!("{}", "Ignoring packet. Invalid destination".yellow());
                    return Ok(None);
                }

                // Get the IPv6 source and destination addresses
                let source_addr_v6 = ipv4_to_ipv6(&source_addr, &self.ipv6_prefix);
                let dest_addr_v6 = self.pool_reservations.get_by_left(&dest_addr).unwrap();
                log::debug!(
                    "> Mapped IPv6 Source: {}",
                    source_addr_v6.to_string().bright_cyan()
                );
                log::debug!(
                    "> Mapped IPv6 Destination: {}",
                    dest_addr_v6.to_string().bright_cyan()
                );

                // Build an IPv6 packet using this information and the original packet's payload
                let translated = make_ipv6_packet(
                    packet[8],
                    match packet[9] {
                        1 => 58,
                        _ => packet[9],
                    },
                    &source_addr_v6,
                    &dest_addr_v6,
                    &packet[20..],
                );
                let mut response = vec![0; 4 + translated.len()];
                response[..4].copy_from_slice(tun_header);
                response[4..].copy_from_slice(&translated);
                log::debug!(
                    "> Translated Header: {}",
                    bytes_to_hex_str(&response[4..40]).bright_cyan()
                );
                log::debug!("{}", "Sending translated packet".bright_green());
                return Ok(Some(response));
            }
            6 => {
                // Parse the source and destination addresses
                let source_addr = bytes_to_ipv6_addr(&packet[8..24]);
                let dest_addr = bytes_to_ipv6_addr(&packet[24..40]);
                log::debug!("> Source: {}", source_addr.to_string().bright_cyan());
                log::debug!("> Destination: {}", dest_addr.to_string().bright_cyan());

                // Only process packets destined for the NAT prefix
                if !self.ipv6_prefix.contains(&dest_addr) {
                    log::debug!("{}", "Ignoring packet. Invalid destination".yellow());
                    return Ok(None);
                }

                // If the source address doesn't have a reservation, calculate its corresponding IPv4 address and insert into the map
                if !self.pool_reservations.contains_right(&source_addr) {
                    let source_addr_v4 = self.calculate_ipv4(source_addr).unwrap();
                    self.pool_reservations.insert(source_addr_v4, source_addr);
                }

                // Get the mapped source address
                let source_addr_v4 = self.pool_reservations.get_by_right(&source_addr).unwrap();
                log::debug!(
                    "> Mapped IPv4 Source: {}",
                    source_addr_v4.to_string().bright_cyan()
                );

                // Convert the destination address to IPv4
                let dest_addr_v4 = Ipv4Addr::new(packet[36], packet[37], packet[38], packet[39]);
                log::debug!(
                    "> Mapped IPv4 Destination: {}",
                    dest_addr_v4.to_string().bright_cyan()
                );

                // Build an IPv4 packet using this information and the original packet's payload
                let translated = make_ipv4_packet(
                    packet[7],
                    match packet[6] {
                        58 => 1,
                        _ => packet[6],
                    },
                    source_addr_v4,
                    &dest_addr_v4,
                    &packet[40..],
                );
                let mut response = vec![0; 4 + translated.len()];
                response[..4].copy_from_slice(tun_header);
                response[4..].copy_from_slice(&translated);
                log::debug!(
                    "> Translated Header: {}",
                    bytes_to_hex_str(&response[4..24]).bright_cyan()
                );
                log::debug!("{}", "Sending translated packet".bright_green());
                return Ok(Some(response));
            }
            _ => {
                log::warn!("Unknown IP version: {}", packet[0] >> 4);
                return Ok(None);
            }
        };
    }
}
