//! Utilities for manipulating the addresses assigned to links

use std::net::IpAddr;

use futures::TryStreamExt;
use rtnetlink::Handle;

/// Add an IP address to a link
pub async fn addr_add(
    ip_addr: IpAddr,
    prefix_len: u8,
    rt_handle: &Handle,
    link_index: u32,
) -> Result<(), rtnetlink::Error> {
    log::trace!("Adding address {} to link {}", ip_addr, link_index);
    rt_handle
        .address()
        .add(link_index, ip_addr, prefix_len)
        .execute()
        .await
        .map_err(|err| {
            log::error!("Failed to add address {} to link {}", ip_addr, link_index);
            log::error!("{}", err);
            err
        })
}

/// Remove an IP address from a link
pub async fn addr_del(
    ip_addr: IpAddr,
    prefix_len: u8,
    rt_handle: &Handle,
    link_index: u32,
) -> Result<(), rtnetlink::Error> {
    log::trace!("Removing address {} from link {}", ip_addr, link_index);

    // Find the address message that matches the given address
    if let Some(address_message) = rt_handle
        .address()
        .get()
        .set_link_index_filter(link_index)
        .set_address_filter(ip_addr)
        .set_prefix_length_filter(prefix_len)
        .execute()
        .try_next()
        .await
        .map_err(|err| {
            log::error!("Failed to find address {} on link {}", ip_addr, link_index);
            log::error!("{}", err);
            err
        })?
    {
        // Delete the address
        rt_handle
            .address()
            .del(address_message)
            .execute()
            .await
            .map_err(|err| {
                log::error!(
                    "Failed to remove address {} from link {}",
                    ip_addr,
                    link_index
                );
                log::error!("{}", err);
                err
            })?;
    }

    Ok(())
}
