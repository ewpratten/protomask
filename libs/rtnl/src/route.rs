//! Utilities for interacting with the routing table

use ipnet::IpNet;
use rtnetlink::Handle;

/// Add a route to a link
pub async fn route_add(
    destination: IpNet,
    rt_handle: &Handle,
    link_index: u32,
) -> Result<(), rtnetlink::Error> {
    log::trace!("Adding route {} to link {}", destination, link_index);
    match destination {
        IpNet::V4(destination) => rt_handle
            .route()
            .add()
            .v4()
            .output_interface(link_index)
            .destination_prefix(destination.addr(), destination.prefix_len())
            .execute()
            .await
            .map_err(|err| {
                log::error!("Failed to add route {} to link", destination);
                log::error!("{}", err);
                err
            }),
        IpNet::V6(destination) => rt_handle
            .route()
            .add()
            .v6()
            .output_interface(link_index)
            .destination_prefix(destination.addr(), destination.prefix_len())
            .execute()
            .await
            .map_err(|err| {
                log::error!("Failed to add route {} to link", destination);
                log::error!("{}", err);
                err
            }),
    }
}
