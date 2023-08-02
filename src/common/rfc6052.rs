//! Utilities for interacting with [RFC6052](https://datatracker.ietf.org/doc/html/rfc6052) "IPv4-Embedded IPv6 Addresses"

use std::str::FromStr;

use ipnet::Ipv6Net;

/// Parses an [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)-compliant IPv6 prefix from a string
pub fn parse_network_specific_prefix(string: &str) -> Result<Ipv6Net, String> {
    // First, parse to an IPv6Net struct
    let net = Ipv6Net::from_str(string).map_err(|err| err.to_string())?;

    // Ensure the prefix length is one of the allowed lengths according to RFC6052 Section 2.2
    if !rfc6052::ALLOWED_PREFIX_LENS.contains(&net.prefix_len()) {
        return Err(format!(
            "Prefix length must be one of {:?}",
            rfc6052::ALLOWED_PREFIX_LENS
        ));
    }

    // Return the parsed network struct
    Ok(net)
}
