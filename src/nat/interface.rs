use futures::stream::TryStreamExt;
use ipnet::{Ipv4Net, Ipv6Net};
use tun_tap::{Iface, Mode};

#[derive(Debug, thiserror::Error)]
pub enum InterfaceError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    NetlinkError(#[from] rtnetlink::Error),
}

/// Wrapper around a TUN interface that automatically configures itself
#[derive(Debug)]
pub struct Nat64Interface {
    /// Underlying TUN interface
    interface: Iface,
    /// Interface MTU
    mtu: usize,
}

impl Nat64Interface {
    /// Create a new NAT64 interface
    pub async fn new(v6_prefix: Ipv6Net, v4_pool: &Vec<Ipv4Net>) -> Result<Self, InterfaceError> {
        // Bring up an rtnetlink connection
        let (rt_connection, rt_handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(rt_connection);

        // Set up the TUN interface
        let interface = Iface::without_packet_info("nat64i%d", Mode::Tun)?;

        // Get access to the new interface through rtnetlink
        let interface_link = rt_handle
            .link()
            .get()
            .match_name(interface.name().to_owned())
            .execute()
            .try_next()
            .await?
            .expect("Interface not found even though it was just created");

        // Bring up the interface
        rt_handle
            .link()
            .set(interface_link.header.index)
            .up()
            .execute()
            .await?;
        log::info!("Created interface: {}", interface.name());

        // Add the v6 prefix as a route
        rt_handle
            .route()
            .add()
            .v6()
            .destination_prefix(v6_prefix.addr(), v6_prefix.prefix_len())
            .output_interface(interface_link.header.index)
            .execute()
            .await
            .map_err(|error| {
                log::error!("Failed to add route for {}: {}", v6_prefix, error);
                error
            })?;
        log::info!("Added route: {} via {}", v6_prefix, interface.name());

        // Add every prefix in the v4 pool as a route
        for prefix in v4_pool {
            rt_handle
                .route()
                .add()
                .v4()
                .destination_prefix(prefix.addr(), prefix.prefix_len())
                .output_interface(interface_link.header.index)
                .execute()
                .await
                .map_err(|error| {
                    log::error!("Failed to add route for {}: {}", prefix, error);
                    error
                })?;
            log::info!("Added route: {} via {}", prefix, interface.name());
        }

        // Read the interface MTU
        let mtu: usize =
            std::fs::read_to_string(format!("/sys/class/net/{}/mtu", interface.name()))
                .expect("Failed to read interface MTU")
                .strip_suffix("\n")
                .unwrap()
                .parse()
                .unwrap();

        Ok(Self { interface, mtu })
    }

    /// Get the interface mode
    pub fn mode(&self) -> Mode {
        self.interface.mode()
    }

    /// Get the interface name
    pub fn name(&self) -> &str {
        self.interface.name()
    }

    /// Get the interface MTU
    pub fn mtu(&self) -> usize {
        self.mtu
    }

    /// Receive a packet from the interface
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.interface.recv(buf)
    }

    /// Send a packet to the interface
    pub fn send(&self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.interface.send(buf)
    }
}
