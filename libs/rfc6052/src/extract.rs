use crate::{error::Error, ALLOWED_PREFIX_LENS};
use std::cmp::max;
use std::net::{Ipv4Addr, Ipv6Addr};

/// Extracts an IPv4 address from an IPv6 prefix following the method defined in [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)
pub fn extract_ipv4_addr(ipv6_addr: Ipv6Addr, prefix_length: u8) -> Result<Ipv4Addr, Error> {
    // Fail if the prefix length is invalid
    if !ALLOWED_PREFIX_LENS.contains(&prefix_length) {
        return Err(Error::InvalidPrefixLength(prefix_length));
    }

    // Fall through to the unchecked version of this function
    Ok(unsafe { extract_ipv4_addr_unchecked(ipv6_addr, prefix_length) })
}

/// Extracts an IPv4 address from an IPv6 prefix following the method defined in [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)
///
/// **Warning:** This function does not check that the prefix length is valid according to the RFC. Use `extract_ipv4_addr` instead.
#[must_use]
#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe fn extract_ipv4_addr_unchecked(ipv6_addr: Ipv6Addr, prefix_length: u8) -> Ipv4Addr {
    // Convert the IPv6 address to a number for easier manipulation
    let ipv6_addr = u128::from(ipv6_addr);
    let host_part = ipv6_addr & ((1 << (128 - prefix_length)) - 1);

    // Extract the IPv4 address from the IPv6 address
    Ipv4Addr::from(
        (((host_part & 0xffff_ffff_ffff_ffff_0000_0000_0000_0000)
            | (host_part & 0x00ff_ffff_ffff_ffff) << 8)
            >> max(8, 128 - prefix_length - 32)) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_len_32() {
        unsafe {
            assert_eq!(
                extract_ipv4_addr_unchecked("64:ff9b:c000:0201::".parse().unwrap(), 32),
                "192.0.2.1".parse::<Ipv4Addr>().unwrap(),
            )
        }
    }

    #[test]
    fn test_extract_len_40() {
        unsafe {
            assert_eq!(
                extract_ipv4_addr_unchecked("64:ff9b:00c0:0002:0001::".parse().unwrap(), 40),
                "192.0.2.1".parse::<Ipv4Addr>().unwrap(),
            )
        }
    }

    #[test]
    fn test_extract_len_48() {
        unsafe {
            assert_eq!(
                extract_ipv4_addr_unchecked("64:ff9b:0000:c000:0002:0100::".parse().unwrap(), 48),
                "192.0.2.1".parse::<Ipv4Addr>().unwrap(),
            )
        }
    }

    #[test]
    fn test_extract_len_56() {
        unsafe {
            assert_eq!(
                extract_ipv4_addr_unchecked("64:ff9b:0000:00c0:0000:0201::".parse().unwrap(), 56),
                "192.0.2.1".parse::<Ipv4Addr>().unwrap(),
            )
        }
    }

    #[test]
    fn test_extract_len_64() {
        unsafe {
            assert_eq!(
                extract_ipv4_addr_unchecked(
                    "64:ff9b:0000:0000:00c0:0002:0100::".parse().unwrap(),
                    64
                ),
                "192.0.2.1".parse::<Ipv4Addr>().unwrap(),
            )
        }
    }

    #[test]
    fn test_extract_len_96() {
        unsafe {
            assert_eq!(
                extract_ipv4_addr_unchecked(
                    "64:ff9b:0000:0000:0000:0000:c000:0201".parse().unwrap(),
                    96
                ),
                "192.0.2.1".parse::<Ipv4Addr>().unwrap(),
            )
        }
    }
}
