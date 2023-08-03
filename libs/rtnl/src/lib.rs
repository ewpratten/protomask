#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]

pub mod link;
pub mod ip;
pub mod route;

/// Get a handle on a new rtnetlink connection
#[cfg(feature="tokio")]
pub fn new_handle() -> Result<rtnetlink::Handle, std::io::Error> {
    let (rt_connection, rt_handle, _) = rtnetlink::new_connection().map_err(|err| {
        log::error!("Failed to open rtnetlink connection");
        log::error!("{}", err);
        err
    })?;
    tokio::spawn(rt_connection);
    Ok(rt_handle)
}
