use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use bimap::BiMap;
use colored::Colorize;
use ipnet::{Ipv4Net, Ipv6Net};
use pnet_packet::{
    icmpv6::Icmpv6Packet,
    ip::IpNextHeaderProtocols,
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    Packet, ipv4::{Ipv4Packet, MutableIpv4Packet}, icmp::IcmpPacket,
};
use tokio::process::Command;
use tun_tap::{Iface, Mode};

use crate::nat::packet::{xlat_v6_to_v4, IpPacket};

use self::packet::xlat_v4_to_v6;

mod icmp;
mod packet;

/// A cleaner way to execute a CLI command
macro_rules! command {
    ($cmd:expr, $($arg:expr),*) => {{
        Command::new($cmd)
            $(.arg($arg))*
            .status()
    }}
}

/// Converts bytes to a hex string for debugging
fn bytes_to_hex_str(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|val| format!("{:02x}", val))
        .collect::<Vec<String>>()
        .join(" ")
}

pub struct Nat64 {
    /// Handle for the Tun interface
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
        let interface = Iface::without_packet_info("nat64i%d", Mode::Tun)?;

        // Configure the interface
        let interface_name = interface.name();
        log::info!("Configuring interface {}", interface_name);

        #[cfg_attr(rustfmt, rustfmt_skip)]
        {
            // Add the nat addresses
            log::debug!("Assigning {} to {}", nat_v4, interface_name);
            command!("ip", "address", "add", format!("{}/32", nat_v4), "dev", interface_name).await?;
            log::debug!("Assigning {} to {}", nat_v6, interface_name);
            command!("ip", "address", "add", format!("{}/128", nat_v6), "dev", interface_name ).await?;

            // Bring up the interface
            log::debug!("Bringing up {}", interface_name);
            command!("ip", "link", "set", "dev", interface_name, "up").await?;

            // Add route for IPv6 prefix
            log::debug!("Adding route {} via {}", ipv6_prefix, interface_name);
            command!("ip", "route", "add", ipv6_prefix.to_string(), "dev", interface_name).await?;

            // Configure iptables
            log::debug!("Configuring iptables");
            command!("iptables", "-A", "FORWARD", "-i", interface_name, "-j", "ACCEPT").await?;
            command!("iptables", "-A", "FORWARD", "-o", interface_name, "-j", "ACCEPT").await?;
            command!("ip6tables", "-A", "FORWARD", "-i", interface_name, "-j", "ACCEPT").await?;
            command!("ip6tables", "-A", "FORWARD", "-o", interface_name, "-j", "ACCEPT").await?;
        }

        // Add every IPv4 prefix to the routing table
        for prefix in ipv4_pool.iter() {
            log::debug!("Adding route {} via {}", prefix, interface_name);
            command!(
                "ip",
                "route",
                "add",
                prefix.to_string(),
                "dev",
                interface_name
            )
            .await?;
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
        let mut buffer = vec![0; mtu as usize];

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
    fn is_dest_allowed(&self, dest: IpAddr) -> bool {
        return dest == self.instance_v4
            || dest == self.instance_v6
            || match dest {
                IpAddr::V4(addr) => self.ipv4_pool.iter().any(|prefix| prefix.contains(&addr)),
                IpAddr::V6(addr) => self.ipv6_prefix.contains(&addr),
            };
    }

    /// Calculate a unique IPv4 address inside the pool for a given IPv6 address
    fn calculate_ipv4(&self, _addr: Ipv6Addr) -> Option<Ipv4Addr> {
        // Search the list of possible IPv4 addresses
        for prefix in self.ipv4_pool.iter() {
            for addr in prefix.hosts() {
                // If this address is available, use it
                if !self.pool_reservations.contains_left(&addr) {
                    return Some(addr);
                }
            }
        }

        None
    }

    /// Embeds an IPv4 address into an IPv6 address
    fn embed_v4_into_v6(&self, addr: Ipv4Addr) -> Ipv6Addr {
        let mut octets = [0u8; 16];
        octets[..12].copy_from_slice(&self.ipv6_prefix.network().octets()[..12]);
        octets[12..].copy_from_slice(&addr.octets());
        Ipv6Addr::from(octets)
    }

    /// Extracts an IPv4 address from an IPv6 address
    fn extract_v4_from_v6(&self, addr: Ipv6Addr) -> Ipv4Addr {
        let mut octets = [0u8; 4];
        octets.copy_from_slice(&addr.octets()[12..]);
        Ipv4Addr::from(octets)
    }

    /// Gets or creates a reservation for a given address
    fn get_or_create_reservation(&mut self, addr: IpAddr) -> Option<IpAddr> {
        match addr {
            IpAddr::V4(addr) => {
                if self.pool_reservations.contains_left(&addr) {
                    return Some(IpAddr::V6(
                        *self.pool_reservations.get_by_left(&addr).unwrap(),
                    ));
                } else {
                    return None;
                }
            }
            IpAddr::V6(addr) => {
                // If the address is already reserved, return it
                if self.pool_reservations.contains_right(&addr) {
                    return Some(IpAddr::V4(
                        *self.pool_reservations.get_by_right(&addr).unwrap(),
                    ));
                }

                // Otherwise, calculate a new address
                let new_addr = self.calculate_ipv4(addr)?;
                self.pool_reservations.insert(new_addr, addr);
                return Some(IpAddr::V4(new_addr));
            }
        }
    }

    /// Internal function to process an incoming packet.
    /// If `Some` is returned, the result is sent back out the interface
    async fn process(&mut self, packet: &[u8]) -> Result<Option<Vec<u8>>, std::io::Error> {
        // Parse the packet
        let input_packet = IpPacket::new(&packet);
        if let None = input_packet {
            log::warn!(
                "{}",
                format!(
                    "Malformed packet received: version: {}, len: {}",
                    packet[0] >> 4,
                    packet.len()
                )
                .yellow()
            );
            return Ok(None);
        }
        let input_packet = input_packet.unwrap();

        // Log some info about the packet
        #[cfg_attr(rustfmt, rustfmt_skip)]
        {
            log::debug!("Processing packet with length: {}", input_packet.len().to_string().bright_cyan());
            log::debug!("> IP Header: {}", bytes_to_hex_str(input_packet.get_header()).bright_cyan());
            log::debug!("> Source: {}", input_packet.get_source().to_string().bright_cyan());
            log::debug!("> Destination: {}", input_packet.get_destination().to_string().bright_cyan());
            log::debug!("> Next Header: {}", input_packet.get_next_header().to_string().bright_cyan());
        }

        // Ignore packets that aren't destined for the NAT instance
        if !self.is_dest_allowed(input_packet.get_destination()) {
            log::debug!("{}", "Ignoring packet. Invalid destination".yellow());
            return Ok(None);
        }

        // Drop packets with 0 TTL
        if input_packet.get_ttl() == 0 {
            log::debug!("{}", "Ignoring packet. TTL is 0".yellow());
            return Ok(None);
        }

        // Handle packet translation
        let output_packet = match input_packet {
            IpPacket::V4(packet) => {
                let new_source = self.embed_v4_into_v6(packet.get_source());
                let new_dest =
                    self.get_or_create_reservation(std::net::IpAddr::V4(packet.get_destination()));
                if let Some(IpAddr::V6(new_dest)) = new_dest {
                    // Log the new addresses
                    #[cfg_attr(rustfmt, rustfmt_skip)]
                    {
                        log::debug!("> Mapped IPv6 Source: {}", new_source.to_string().bright_cyan());
                        log::debug!("> Mapped IPv6 Destination: {}", new_dest.to_string().bright_cyan());
                    }

                    // Handle inner packet conversion for protocols that don't support both v4 and v6
                    if let Some(packet) = Ipv4Packet::owned(match packet.get_next_level_protocol() {
                        // ICMP must be translated to ICMPv6
                        IpNextHeaderProtocols::Icmp => {
                            if let Some(new_payload) =
                                icmp::icmp_to_icmpv6(&IcmpPacket::new(packet.payload()).unwrap())
                            {
                                // Mutate the input packet
                                let mut packet =
                                    MutableIpv4Packet::owned(packet.packet().to_vec()).unwrap();
                                packet.set_next_level_protocol(IpNextHeaderProtocols::Icmpv6);
                                packet.set_payload(&new_payload.packet().to_vec());
                                packet.packet().to_vec()
                            } else {
                                return Ok(None);
                            }
                        }

                        // By default, packets can be directly fed to the next function
                        _ => packet.packet().to_vec(),
                    }) {
                        // Translate the packet
                        let translated = xlat_v4_to_v6(&packet, new_source, new_dest, true);

                        // Log the translated packet header
                        log::debug!(
                            "> Translated Header: {}",
                            bytes_to_hex_str(&translated[0..40]).bright_cyan()
                        );

                        // Return the translated packet
                        translated
                    } else {
                        return Ok(None);
                    }

                } else {
                    return Ok(None);
                }
            }
            IpPacket::V6(packet) => {
                let new_source =
                    self.get_or_create_reservation(std::net::IpAddr::V6(packet.get_source()));
                let new_dest = self.extract_v4_from_v6(packet.get_destination());
                if let Some(IpAddr::V4(new_source)) = new_source {
                    // Log the new addresses
                    #[cfg_attr(rustfmt, rustfmt_skip)]
                    {
                        log::debug!("> Mapped IPv4 Source: {}", new_source.to_string().bright_cyan());
                        log::debug!("> Mapped IPv4 Destination: {}", new_dest.to_string().bright_cyan());
                    }

                    // Handle inner packet conversion for protocols that don't support both v4 and v6
                    if let Some(packet) = Ipv6Packet::owned(match packet.get_next_header() {
                        // ICMPv6 must be translated to ICMP
                        IpNextHeaderProtocols::Icmpv6 => {
                            if let Some(new_payload) =
                                icmp::icmpv6_to_icmp(&Icmpv6Packet::new(packet.payload()).unwrap())
                            {
                                // Mutate the input packet
                                let mut packet =
                                    MutableIpv6Packet::owned(packet.packet().to_vec()).unwrap();
                                packet.set_next_header(IpNextHeaderProtocols::Icmp);
                                packet.set_payload(&new_payload.packet().to_vec());
                                packet.packet().to_vec()
                            } else {
                                return Ok(None);
                            }
                        }

                        // By default, packets can be directly fed to the next function
                        _ => packet.packet().to_vec(),
                    }) {
                        // Translate the packet
                        let translated = xlat_v6_to_v4(&packet, new_source, new_dest, true);

                        // Log the translated packet header
                        log::debug!(
                            "> Translated Header: {}",
                            bytes_to_hex_str(&translated[0..20]).bright_cyan()
                        );

                        // Return the translated packet
                        translated
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
        };

        // Build the response
        log::debug!("{}", "Sending translated packet".bright_green());
        return Ok(Some(output_packet));
    }
}
