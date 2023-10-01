//! Utilities for operating on a link/interface/device

use futures::TryStreamExt;
use rtnetlink::Handle;

/// Bring up a link by its link index
pub async fn link_up(rt_handle: &Handle, link_index: u32) -> Result<(), rtnetlink::Error> {
    log::trace!("Bringing up link {}", link_index);
    rt_handle.link().set(link_index).up().execute().await
}

/// Bring down a link by its link index
pub async fn link_down(rt_handle: &Handle, link_index: u32) -> Result<(), rtnetlink::Error> {
    log::trace!("Bringing down link {}", link_index);
    rt_handle.link().set(link_index).down().execute().await
}

/// Get the link index of a link by its name
pub async fn get_link_index(
    rt_handle: &Handle,
    link_name: &str,
) -> Result<Option<u32>, rtnetlink::Error> {
    Ok(rt_handle
        .link()
        .get()
        .match_name(link_name.to_owned())
        .execute()
        .try_next()
        .await?
        .map(|message| message.header.index))
}
