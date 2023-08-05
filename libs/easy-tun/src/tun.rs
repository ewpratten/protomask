use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    mem::size_of,
    os::fd::{AsRawFd, IntoRawFd, RawFd},
};

use ioctl_gen::{ioc, iow};
use libc::{__c_anonymous_ifr_ifru, ifreq, ioctl, IFF_NO_PI, IFF_TUN, IF_NAMESIZE};

/// Architecture / target environment specific definitions
mod arch {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    pub type IoctlRequestType = libc::c_ulong;
    #[cfg(all(target_os = "linux", target_env = "musl"))]
    pub type IoctlRequestType = libc::c_int;
    #[cfg(not(any(target_env = "gnu", target_env = "musl")))]
    compile_error!(
        "Unsupported target environment. Only gnu and musl targets are currently supported."
    );
}

/// A TUN device
pub struct Tun {
    /// Internal file descriptor for the TUN device
    fd: File,
    /// Device name
    name: String,
}

impl Tun {
    /// Creates a new Tun device with the given name.
    ///
    /// The `name` argument must be less than the system's `IFNAMSIZ` constant,
    /// and may contain a `%d` format specifier to allow for multiple devices with the same name.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_lossless)]
    pub fn new(dev: &str) -> Result<Self, std::io::Error> {
        log::debug!("Creating new TUN device with requested name:{}", dev);

        // Get a file descriptor for `/dev/net/tun`
        log::trace!("Opening /dev/net/tun");
        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        // Copy the device name into a C string with padding
        // NOTE: No zero padding is needed because we pre-init the array to all 0s
        let mut dev_cstr: [libc::c_char; IF_NAMESIZE] = [0; IF_NAMESIZE];
        let dev_bytes: Vec<libc::c_char> = dev.chars().map(|c| c as libc::c_char).collect();
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
                iow!('T', 202, size_of::<libc::c_int>()) as arch::IoctlRequestType,
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

        // Log the success
        log::debug!("Created TUN device: {}", name);

        // Build the TUN struct
        Ok(Self { fd, name })
    }

    /// Get the name of the TUN device
    #[must_use]
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

impl Read for Tun {
    #[profiling::function]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.fd.read(buf)
    }
}

impl Write for Tun {
    #[profiling::function]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.fd.write(buf)
    }

    #[profiling::function]
    fn flush(&mut self) -> std::io::Result<()> {
        self.fd.flush()
    }
}
