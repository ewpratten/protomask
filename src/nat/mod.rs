use std::net::{Ipv4Addr, Ipv6Addr};

use ipnet::{Ipv4Net, Ipv6Net};
use tokio::process::Command;
use tun_tap::{Iface, Mode};

/// A cleaner way to execute an `ip` command
macro_rules! iproute2 {
    ($($arg:expr),*) => {{
        Command::new("ip")
            $(.arg($arg))*
            .status()
    }}
}

pub struct Nat64 {
    interface: Iface,
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
        for prefix in ipv4_pool {
            log::debug!("Adding route {} via {}", prefix, interface_name);
            iproute2!("route", "add", prefix.to_string(), "dev", interface_name).await?;
        }

        Ok(Self { interface })
    }
}
