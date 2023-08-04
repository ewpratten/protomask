# `protomask`: Fast & reliable user space NAT64
[![GitHub release](https://img.shields.io/github/v/release/ewpratten/protomask)](https://github.com/ewpratten/protomask/releases/latest)
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/build.yml)
[![Audit](https://github.com/ewpratten/protomask/actions/workflows/audit.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/audit.yml)


Protomask is a user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation geared towards networks that need fast and reliable inter-protocol packet translation. Behind the scenes, protomask uses the [Universal TUN/TAP Device Driver](https://docs.kernel.org/networking/tuntap.html) to translate incoming packets from IPv4 to IPv6 and vice-versa.

## Latest Releases

| Crate | Info |
| -- | -- |
| [`protomask`](./src/protomask.rs) | [![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) |
| [`protomask-clat`](./src/protomask-clat.rs) | [![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) |
| [`easy-tun`](./libs/easy-tun/) | [![Crates.io](https://img.shields.io/crates/v/easy-tun)](https://crates.io/crates/easy-tun) [![Docs.rs](https://docs.rs/easy-tun/badge.svg)](https://docs.rs/easy-tun) |
| [`fast-nat`](./libs/fast-nat/) | [![Crates.io](https://img.shields.io/crates/v/fast-nat)](https://crates.io/crates/fast-nat) [![Docs.rs](https://docs.rs/fast-nat/badge.svg)](https://docs.rs/fast-nat) |
| [`interproto`](./libs/interproto/) | [![Crates.io](https://img.shields.io/crates/v/interproto)](https://crates.io/crates/interproto) [![Docs.rs](https://docs.rs/interproto/badge.svg)](https://docs.rs/interproto) |
| [`rfc6052`](./libs/rfc6052/) | [![Crates.io](https://img.shields.io/crates/v/rfc6052)](https://crates.io/crates/rfc6052) [![Docs.rs](https://docs.rs/rfc6052/badge.svg)](https://docs.rs/rfc6052) |
| [`rtnl`](./libs/rtnl/) | [![Crates.io](https://img.shields.io/crates/v/rtnl)](https://crates.io/crates/rtnl) [![Docs.rs](https://docs.rs/rtnl/badge.svg)](https://docs.rs/rtnl) |

## The protomask tool suite

To accomplish the various translation needs of an IPv6-only or dual-stack ISP, the protomask tool suite includes a few tools:

- **`protomask`**: The main NAT64 daemon
  - Translates IPv6 packets using *RFC6052 IPv4-Embedded IPv6 Addressing* to native IPv4 traffic
  - Can handle high volumes of traffic from multiple clients simultaneously
- **`protomask-clat`**: A Customer-side transLATor (CLAT) implementation
  - Intended to be deployed at the customer edge to pass IPv4 traffic over an IPv6-only ISP's network

Every tool in the protomask suite is easy to deploy and configure, plus supports optionally exposing Prometheus metrics for remote monitoring.

## The protomask library suite

The development of protomask necessitated the creation of a few specialized software libraries. Since the technology developed for protomask is useful outside of the scope of this project, these libraries are also available for general use:

- **`easy-tun`**: A minimal TUN interface library
- **`fast-nat`**: A library designed for highly efficient mapping and lookup of IP address pairs
- **`interproto`**: The heart of protomask, a library for translating many types of packets between layer 3 protocols
- **`rfc6052`**: A Rust implementation of RFC6052
- **`rtnl`**: A high-level wrapper around `rtnetlink`

## Installation

Protomask can be installed using various methods:

### Debian

Head over to the [releases](https://github.com/ewpratten/protomask/releases) page and download the latest release for your architecture.

Then, install with:

```sh
apt install /path/to/protomask_<version>_<arch>.deb

# You can also edit the config file in /etc/protomask.toml
# And once ready, start protomask with
systemctl start protomask
```

### Using Cargo

```bash
cargo install protomask
```
