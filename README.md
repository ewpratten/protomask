# protomask
[![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) 
[![Docs.rs](https://docs.rs/protomask/badge.svg)](https://docs.rs/protomask) 
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/Ewpratten/protomask/actions/workflows/build.yml)

**A user space NAT64 implementation.**

*Protomask* started as a challenge to create a NAT64 implementation in a weekend. The goal of this implementation is to *keep things simple*. There aren't many knobs to tweak, so if you want to do stateful NAT or source address filtering, put something like `iptables` in front of it.

## Configuration

Protomask uses a [TOML](https://toml.io) configuration file. Here is a functional example:

```toml
[Interface]
# A list of IPv4 prefixes to map traffic to
Pool = ["44.31.119.0/24"]
# The IPv6 prefix to listen for traffic on (should generally be the well-known 64:ff9b::/96 prefix)
Prefix = "64:ff9b::/96"

[Rules]
# A static mapping of IPv4 and IPv6 addresses
# These addresses will be exclusively reserved, and not used in the general pool
MapStatic = [{ v4 = "192.0.2.2", v6 = "2001:db8:1::2" }]
```

