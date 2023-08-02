use std::{
    fs::{File, OpenOptions},
    future::Future,
    mem::size_of,
    os::fd::{AsRawFd, IntoRawFd, RawFd},
};

use ioctl_gen::{ioc, iow};
use libc::{__c_anonymous_ifr_ifru, ifreq, ioctl, IFF_NO_PI, IFF_TUN, IF_NAMESIZE};

/// A TUN device
pub struct Tun {
    /// Internal file descriptor for the TUN device
    fd: File,
    /// Device name
    name: String,
    /// Handle on the RTNETLINK connection
    rt_connection_future: Box<dyn Future<Output = ()>>,
    /// Handle on the RTNETLINK socket
    rt_handle: rtnetlink::Handle,
    /// Link index of the TUN device
    link_index: u32,
}

impl Tun {
    /// Creates a new Tun device with the given name.
    ///
    /// The `name` argument must be less than the system's `IFNAMSIZ` constant,
    /// and may contain a `%d` format specifier to allow for multiple devices with the same name.
    pub fn new(dev: &str) -> Result<Self, std::io::Error> {
        // Get a file descriptor for `/dev/net/tun`
        log::trace!("Opening /dev/net/tun");
        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        // Copy the device name into a C string with padding
        // NOTE: No zero padding is needed because we pre-init the array to all 0s
        let mut dev_cstr = [0i8; IF_NAMESIZE];
        let dev_bytes: Vec<i8> = dev.as_bytes().iter().map(|c| *c as i8).collect();
        let dev_len = dev_bytes.len().min(IF_NAMESIZE);
        log::trace!("Device name length after truncation: {}", dev_len);
        dev_cstr[..dev_len].copy_from_slice(&dev_bytes[..dev_len]);

        // Build an `ifreq` struct to send to the kernel
        let mut ifr = ifreq {
            ifr_name: dev_cstr,
            ifr_ifru: __c_anonymous_ifr_ifru {
                ifru_flags: (IFF_TUN | IFF_NO_PI) as i16,
            },
        };

        // Make an ioctl call to create the TUN device
        log::trace!("Calling ioctl to create TUN device");
        let err = unsafe {
            ioctl(
                fd.as_raw_fd(),
                iow!('T', 202, size_of::<usize>()) as u64,
                &mut ifr,
            )
        };
        log::trace!("ioctl returned: {}", err);

        // Check for errors
        if err < 0 {
            log::error!("ioctl failed: {}", err);
            return Err(std::io::Error::last_os_error());
        }

        // Get the name of the device
        let name = unsafe { std::ffi::CStr::from_ptr(ifr.ifr_name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();

        // Create an rtnetlink connection
        let (rt_connection, rt_handle, _) = rtnetlink::new_connection().map_err(|err| {
            log::error!("Failed to open rtnetlink connection");
            log::error!("{}", err);
            err
        })?;

        // Build the TUN struct
        Ok(Self {
            fd,
            name,
            rt_connection_future: Box::new(rt_connection),
            rt_handle,
            link_index: 0,
        })
    }

    /// Get the name of the TUN device
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AsRawFd for Tun {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl IntoRawFd for Tun {
    fn into_raw_fd(self) -> RawFd {
        self.fd.into_raw_fd()
    }
}
