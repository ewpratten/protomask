//! Utilities for interacting with [RFC6052](https://datatracker.ietf.org/doc/html/rfc6052) "IPv4-Embedded IPv6 Addresses"

use std::{
    cmp::{max, min},
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use ipnet::Ipv6Net;

/// Parses an [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)-compliant IPv6 prefix from a string
pub fn parse_network_specific_prefix(string: &str) -> Result<Ipv6Net, String> {
    // First, parse to an IPv6Net struct
    let net = Ipv6Net::from_str(string).map_err(|err| err.to_string())?;

    // Ensure the prefix length is one of the allowed lengths according to RFC6052 Section 2.2
    if ![32, 40, 48, 56, 64, 96].contains(&net.prefix_len()) {
        return Err("Prefix length must be one of 32, 40, 48, 56, 64, or 96".to_owned());
    }

    // Return the parsed network struct
    Ok(net)
}

/// Embeds an IPv4 address into an IPv6 prefix
pub fn embed_to_ipv6(ipv4_addr: Ipv4Addr, ipv6_prefix: Ipv6Net) -> Ipv6Addr {
    // Convert to integer types
    let ipv4_addr = u32::from(ipv4_addr);
    let prefix_len = ipv6_prefix.prefix_len() as i16;
    let ipv6_prefix = u128::from(ipv6_prefix.addr());

    // According to the RFC, the IPv4 address must be split on the boundary of bits 64..71.
    // To accomplish this, we split the IPv4 address into two parts so we can separately mask
    // and shift them into place on each side of the boundary
    Ipv6Addr::from(
        ipv6_prefix
            | (((ipv4_addr as u128 & (0xffff_ffffu128 << (32 + min(0, prefix_len - 64)))) as u128)
                << (128 - prefix_len - 32))
            | (((ipv4_addr as u128) << max(0, 128 - prefix_len - 32 - 8)) & 0x00ff_ffff_ffff_ffff),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_len_32() {
        assert_eq!(
            embed_to_ipv6(
                "192.0.2.1".parse().unwrap(),
                "64:ff9b::/32".parse().unwrap()
            ),
            "64:ff9b:c000:0201::".parse::<Ipv6Addr>().unwrap()
        );
    }

    #[test]
    fn test_embed_len_40() {
        assert_eq!(
            embed_to_ipv6(
                "192.0.2.1".parse().unwrap(),
                "64:ff9b::/40".parse().unwrap(),
            ),
            "64:ff9b:00c0:0002:0001::".parse::<Ipv6Addr>().unwrap()
        );
    }

    #[test]
    fn test_embed_len_48() {
        assert_eq!(
            embed_to_ipv6(
                "192.0.2.1".parse().unwrap(),
                "64:ff9b::/48".parse().unwrap(),
            ),
            "64:ff9b:0000:c000:0002:0100::".parse::<Ipv6Addr>().unwrap()
        );
    }

    #[test]
    fn test_embed_len_56() {
        assert_eq!(
            embed_to_ipv6(
                "192.0.2.1".parse().unwrap(),
                "64:ff9b::/56".parse().unwrap(),
            ),
            "64:ff9b:0000:00c0:0000:0201::".parse::<Ipv6Addr>().unwrap()
        );
    }

    #[test]
    fn test_embed_len_64() {
        assert_eq!(
            embed_to_ipv6(
                "192.0.2.1".parse().unwrap(),
                "64:ff9b::/64".parse().unwrap(),
            ),
            "64:ff9b:0000:0000:00c0:0002:0100::"
                .parse::<Ipv6Addr>()
                .unwrap()
        );
    }

    #[test]
    fn test_embed_len_96() {
        assert_eq!(
            embed_to_ipv6(
                "192.0.2.1".parse().unwrap(),
                "64:ff9b::/96".parse().unwrap(),
            ),
            "64:ff9b:0000:0000:0000:0000:c000:0201"
                .parse::<Ipv6Addr>()
                .unwrap()
        );
    }
}
