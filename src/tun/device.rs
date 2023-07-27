use std::{
    io::{Read, Write},
    net::IpAddr,
    os::fd::{AsRawFd, FromRawFd},
};

use futures::TryStreamExt;
use ipnet::IpNet;
use tokio::{
    sync::{broadcast, mpsc},
    task,
};
use tun_tap::Mode;

use super::TunError;

#[derive(Debug)]
pub struct TunDevice {
    device: tun_tap::Iface,
    rt_handle: rtnetlink::Handle,
    link_index: u32,
    mtu: usize,
}

impl TunDevice {
    /// Create and bring up a new TUN device
    ///
    /// ## Name format
    ///
    /// The name field can be any string. If `%d` is present in the string,
    /// it will be replaced with a unique number.
    pub async fn new(name: &str) -> Result<Self, TunError> {
        // Bring up an rtnetlink connection
        let (rt_connection, rt_handle, _) = rtnetlink::new_connection().map_err(|err| {
            log::error!("Failed to open rtnetlink connection");
            log::error!("{}", err);
            err
        })?;
        tokio::spawn(rt_connection);

        // Create the TUN device
        let tun_device = tun_tap::Iface::without_packet_info(name, Mode::Tun)?;
        log::debug!("Created new TUN device: {}", tun_device.name());

        // Get access to the link through rtnetlink
        // NOTE: I don't think there is any way this can fail, so `except` is probably OK
        let tun_link = rt_handle
            .link()
            .get()
            .match_name(tun_device.name().to_owned())
            .execute()
            .try_next()
            .await?
            .expect("Failed to access newly created TUN device");

        // Bring the link up
        rt_handle
            .link()
            .set(tun_link.header.index)
            .up()
            .execute()
            .await
            .map_err(|err| {
                log::error!("Failed to bring up link");
                log::error!("{}", err);
                err
            })?;
        log::debug!("Brought {} up", tun_device.name());

        // Read the link MTU
        let mtu: usize =
            std::fs::read_to_string(format!("/sys/class/net/{}/mtu", tun_device.name()))
                .expect("Failed to read link MTU")
                .strip_suffix("\n")
                .unwrap()
                .parse()
                .unwrap();

        Ok(Self {
            device: tun_device,
            rt_handle,
            link_index: tun_link.header.index,
            mtu,
        })
    }

    /// Add an IP address to this device
    pub async fn add_address(
        &mut self,
        ip_address: IpAddr,
        prefix_len: u8,
    ) -> Result<(), TunError> {
        self.rt_handle
            .address()
            .add(self.link_index, ip_address, prefix_len)
            .execute()
            .await
            .map_err(|err| {
                log::error!("Failed to add address {} to link", ip_address);
                log::error!("{}", err);
                err
            })?;

        Ok(())
    }

    /// Remove an IP address from this device
    pub async fn remove_address(
        &mut self,
        ip_address: IpAddr,
        prefix_len: u8,
    ) -> Result<(), TunError> {
        // Find the address message that matches the given address
        if let Some(address_message) = self
            .rt_handle
            .address()
            .get()
            .set_link_index_filter(self.link_index)
            .set_address_filter(ip_address)
            .set_prefix_length_filter(prefix_len)
            .execute()
            .try_next()
            .await
            .map_err(|err| {
                log::error!("Failed to find address {} on link", ip_address);
                log::error!("{}", err);
                err
            })?
        {
            // Delete the address
            self.rt_handle
                .address()
                .del(address_message)
                .execute()
                .await
                .map_err(|err| {
                    log::error!("Failed to remove address {} from link", ip_address);
                    log::error!("{}", err);
                    err
                })?;
        }

        Ok(())
    }

    /// Add a route to this device
    pub async fn add_route(&mut self, destination: IpNet) -> Result<(), TunError> {
        match destination {
            IpNet::V4(destination) => {
                self.rt_handle
                    .route()
                    .add()
                    .v4()
                    .output_interface(self.link_index)
                    .destination_prefix(destination.addr(), destination.prefix_len())
                    .execute()
                    .await
                    .map_err(|err| {
                        log::error!("Failed to add route {} to link", destination);
                        log::error!("{}", err);
                        err
                    })?;
            }
            IpNet::V6(destination) => {
                self.rt_handle
                    .route()
                    .add()
                    .v6()
                    .output_interface(self.link_index)
                    .destination_prefix(destination.addr(), destination.prefix_len())
                    .execute()
                    .await
                    .map_err(|err| {
                        log::error!("Failed to add route {} to link", destination);
                        log::error!("{}", err);
                        err
                    })?;
            }
        }

        Ok(())
    }

    /// Spawns worker threads, and returns a tx/rx pair for the caller to interact with them
    pub async fn spawn_worker(&self) -> (mpsc::Sender<Vec<u8>>, broadcast::Receiver<Vec<u8>>) {
        // Create a channel for packets to be sent to the caller
        let (tx_to_caller, rx_from_worker) = broadcast::channel(65535);

        // Create a channel for packets being received from the caller
        let (tx_to_worker, mut rx_from_caller) = mpsc::channel(65535);

        // Clone some values for use in worker threads
        let mtu = self.mtu;
        let device_fd = self.device.as_raw_fd();

        // Create a task that broadcasts all incoming packets
        let _rx_task = task::spawn_blocking(move || {
            // Build a buffer to read packets into
            let mut buffer = vec![0u8; mtu];

            // Create a file to access the TUN device
            let mut device = unsafe { std::fs::File::from_raw_fd(device_fd) };

            loop {
                // Read a packet from the TUN device
                let packet_len = device.read(&mut buffer[..]).unwrap();
                let packet = buffer[..packet_len].to_vec();

                // Broadcast the packet to all listeners
                tx_to_caller.send(packet).unwrap();
            }
        });

        // Create a task that sends all outgoing packets
        let _tx_task = task::spawn(async move {
            // Create a file to access the TUN device
            let mut device = unsafe { std::fs::File::from_raw_fd(device_fd) };

            loop {
                // Wait for a packet to be sent
                let packet: Vec<u8> = rx_from_caller.recv().await.unwrap();

                // Write the packet to the TUN device
                device.write_all(&packet[..]).unwrap();
            }
        });

        // Create a task that sends all outgoing packets
        let _tx_task = task::spawn_blocking(|| {});

        // Return an rx/tx pair for the caller to interact with the workers
        (tx_to_worker, rx_from_worker)
    }
}
