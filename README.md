# protomask
[![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) 
[![Docs.rs](https://docs.rs/protomask/badge.svg)](https://docs.rs/protomask) 
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/Ewpratten/protomask/actions/workflows/build.yml)

**A user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation.**

Protomask started as a challenge to create a NAT64 implementation in a weekend. The goal of this implementation is to *keep things simple*. There aren't many knobs to tweak, so if you want to do stateful NAT or source address filtering, put something like `iptables` in front of it.

## How it works

Protomask listens on an IPv6 `/96` prefix for incoming traffic.

When traffic destined for an [embedded IPv4 address](https://datatracker.ietf.org/doc/html/rfc6052) is received, the source IPv6 address is assigned a real IPv4 address from a pool of addresses on a first-come-first-serve basis.

All further packet from that source IPv6 address will be NAT-ed through its assigned IPv4 address until the reservation expires. The reverse process happens for return traffic too.

## Configuration

Protomask uses a [TOML](https://toml.io) configuration file. Here is a functional example:

```toml
[Interface]
# The IPv6 prefix to listen for V6 traffic on
Prefix = "64:ff9b::/96"
# A list of IPv4 prefixes to map V6 traffic to
Pool = ["192.0.2.0/24"]

[Rules]
# A static mapping of IPv4 and IPv6 addresses
# These addresses will not used in the dynamic pool
MapStatic = [{ v4 = "192.0.2.2", v6 = "2001:db8:1::2" }]
# How many seconds to keep a dynamic mapping alive for
ReservationDuration = 7200 # Optional
```
