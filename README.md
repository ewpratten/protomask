# protomask
[![GitHub release](https://img.shields.io/github/v/release/ewpratten/protomask)](https://github.com/ewpratten/protomask/releases/latest)
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/build.yml)
[![Audit](https://github.com/ewpratten/protomask/actions/workflows/audit.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/audit.yml)

**Fast & reliable user space [NAT64](https://en.wikipedia.org/wiki/NAT64).**

For user-oriented documentation, see the [protomask website](https://protomask.ewpratten.com).

## Table of Contents

| Crate | Info |
| -- | -- |
| [`protomask`](./src/protomask.rs) | [![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) |
| [`protomask-clat`](./src/protomask-clat.rs) | [![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) |
| [`easy-tun`](./libs/easy-tun/) | [![Crates.io](https://img.shields.io/crates/v/easy-tun)](https://crates.io/crates/easy-tun) [![Docs.rs](https://docs.rs/easy-tun/badge.svg)](https://docs.rs/easy-tun) |
| [`fast-nat`](./libs/fast-nat/) | [![Crates.io](https://img.shields.io/crates/v/fast-nat)](https://crates.io/crates/fast-nat) [![Docs.rs](https://docs.rs/fast-nat/badge.svg)](https://docs.rs/fast-nat) |
| [`interproto`](./libs/interproto/) | [![Crates.io](https://img.shields.io/crates/v/interproto)](https://crates.io/crates/interproto) [![Docs.rs](https://docs.rs/interproto/badge.svg)](https://docs.rs/interproto) |
| [`rfc6052`](./libs/rfc6052/) | [![Crates.io](https://img.shields.io/crates/v/rfc6052)](https://crates.io/crates/rfc6052) [![Docs.rs](https://docs.rs/rfc6052/badge.svg)](https://docs.rs/rfc6052) |
| [`rtnl`](./libs/rtnl/) | [![Crates.io](https://img.shields.io/crates/v/rtnl)](https://crates.io/crates/rtnl) [![Docs.rs](https://docs.rs/rtnl/badge.svg)](https://docs.rs/rtnl) |



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
