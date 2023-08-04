# protomask
[![GitHub release](https://img.shields.io/github/v/release/ewpratten/protomask)](https://github.com/ewpratten/protomask/releases/latest)
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/build.yml)
[![Audit](https://github.com/ewpratten/protomask/actions/workflows/audit.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/audit.yml)

**A user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation.**

This repository contains:

- `protomask`: The main NAT64 daemon
  - [![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask)
- `protomask-clat`: A Customer-side transLATor (CLAT) implementation
  - [![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask)
- `easy-tun`: A minimal TUN interface library
  - [![Crates.io](https://img.shields.io/crates/v/easy-tun)](https://crates.io/crates/easy-tun) [![Docs.rs](https://docs.rs/easy-tun/badge.svg)](https://docs.rs/easy-tun)
- `fast-nat`: A library designed for highly efficient mapping and lookup of IP address pairs
  - [![Crates.io](https://img.shields.io/crates/v/fast-nat)](https://crates.io/crates/fast-nat) [![Docs.rs](https://docs.rs/fast-nat/badge.svg)](https://docs.rs/fast-nat)
- `interproto`: A library for translating packets between protocols
  - [![Crates.io](https://img.shields.io/crates/v/interproto)](https://crates.io/crates/interproto) [![Docs.rs](https://docs.rs/interproto/badge.svg)](https://docs.rs/interproto)
- `rfc6052`: A Rust implementation of RFC6052
  - [![Crates.io](https://img.shields.io/crates/v/rfc6052)](https://crates.io/crates/rfc6052) [![Docs.rs](https://docs.rs/rfc6052/badge.svg)](https://docs.rs/rfc6052)
- `rtnl`: A high-level wrapper around `rtnetlink`
  - [![Crates.io](https://img.shields.io/crates/v/rtnl)](https://crates.io/crates/rtnl) [![Docs.rs](https://docs.rs/rtnl/badge.svg)](https://docs.rs/rtnl)

For user-oriented documentation, see the [protomask website](https://protomask.ewpratten.com).

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
