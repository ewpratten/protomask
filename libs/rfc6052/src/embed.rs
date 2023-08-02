use ipnet::Ipv6Net;
use std::cmp::{max, min};
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::error::Error;
use crate::ALLOWED_PREFIX_LENS;

/// Embeds an IPv4 address into an IPv6 prefix following the method defined in [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)
pub fn embed_ipv4_addr(ipv4_addr: Ipv4Addr, ipv6_prefix: Ipv6Net) -> Result<Ipv6Addr, Error> {
    // Fail if the prefix length is invalid
    if !ALLOWED_PREFIX_LENS.contains(&ipv6_prefix.prefix_len()) {
        return Err(Error::InvalidPrefixLength(ipv6_prefix.prefix_len()));
    }

    // Fall through to the unchecked version of this function
    Ok(unsafe { embed_ipv4_addr_unchecked(ipv4_addr, ipv6_prefix) })
}

/// Embeds an IPv4 address into an IPv6 prefix following the method defined in [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)
///
/// **Warning:** This function does not check that the prefix length is valid according to the RFC. Use `embed_ipv4_addr` instead.
#[must_use]
#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe fn embed_ipv4_addr_unchecked(ipv4_addr: Ipv4Addr, ipv6_prefix: Ipv6Net) -> Ipv6Addr {
    // Convert to integer types
    let ipv4_addr = u32::from(ipv4_addr);
    let prefix_len = ipv6_prefix.prefix_len() as i16;
    let ipv6_prefix = u128::from(ipv6_prefix.addr());

    // According to the RFC, the IPv4 address must be split on the boundary of bits 64..71.
    // To accomplish this, we split the IPv4 address into two parts so we can separately mask
    // and shift them into place on each side of the boundary
    Ipv6Addr::from(
        ipv6_prefix
            | ((ipv4_addr as u128 & (0xffff_ffffu128 << (32 + min(0, prefix_len - 64))))
                << (128 - prefix_len - 32))
            | (((ipv4_addr as u128) << max(0, 128 - prefix_len - 32 - 8)) & 0x00ff_ffff_ffff_ffff),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_len_32() {
        unsafe {
            assert_eq!(
                embed_ipv4_addr_unchecked(
                    "192.0.2.1".parse().unwrap(),
                    "64:ff9b::/32".parse().unwrap()
                ),
                "64:ff9b:c000:0201::".parse::<Ipv6Addr>().unwrap()
            );
        }
    }

    #[test]
    fn test_embed_len_40() {
        unsafe {
            assert_eq!(
                embed_ipv4_addr_unchecked(
                    "192.0.2.1".parse().unwrap(),
                    "64:ff9b::/40".parse().unwrap(),
                ),
                "64:ff9b:00c0:0002:0001::".parse::<Ipv6Addr>().unwrap()
            );
        }
    }
    #[test]
    fn test_embed_len_48() {
        unsafe {
            assert_eq!(
                embed_ipv4_addr_unchecked(
                    "192.0.2.1".parse().unwrap(),
                    "64:ff9b::/48".parse().unwrap(),
                ),
                "64:ff9b:0000:c000:0002:0100::".parse::<Ipv6Addr>().unwrap()
            );
        }
    }

    #[test]
    fn test_embed_len_56() {
        unsafe {
            assert_eq!(
                embed_ipv4_addr_unchecked(
                    "192.0.2.1".parse().unwrap(),
                    "64:ff9b::/56".parse().unwrap(),
                ),
                "64:ff9b:0000:00c0:0000:0201::".parse::<Ipv6Addr>().unwrap()
            );
        }
    }

    #[test]
    fn test_embed_len_64() {
        unsafe {
            assert_eq!(
                embed_ipv4_addr_unchecked(
                    "192.0.2.1".parse().unwrap(),
                    "64:ff9b::/64".parse().unwrap(),
                ),
                "64:ff9b:0000:0000:00c0:0002:0100::"
                    .parse::<Ipv6Addr>()
                    .unwrap()
            );
        }
    }

    #[test]
    fn test_embed_len_96() {
        unsafe {
            assert_eq!(
                embed_ipv4_addr_unchecked(
                    "192.0.2.1".parse().unwrap(),
                    "64:ff9b::/96".parse().unwrap(),
                ),
                "64:ff9b:0000:0000:0000:0000:c000:0201"
                    .parse::<Ipv6Addr>()
                    .unwrap()
            );
        }
    }
}
