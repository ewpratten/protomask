# protomask
[![GitHub release](https://img.shields.io/github/v/release/ewpratten/protomask)](https://github.com/ewpratten/protomask/releases/latest)
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/build.yml)
[![Audit](https://github.com/ewpratten/protomask/actions/workflows/audit.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/audit.yml)

**A user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation.**

This repository contains:

- `protomask`: The main NAT64 daemon
- `protomask-clat`: A Customer-side transLATor (CLAT) implementation
- `easy-tun`: A minimal TUN interface library
- `fast-nat`: A library designed for highly efficient mapping and lookup of IP address pairs
- `interproto`: A library for translating packets between protocols
- `rfc6052`: A Rust implementation of RFC6052
- `rtnl`: A high-level wrapper around `rtnetlink`

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
