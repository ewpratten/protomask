# protomask
[![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) 
[![Docs.rs](https://docs.rs/protomask/badge.svg)](https://docs.rs/protomask) 
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/Ewpratten/protomask/actions/workflows/build.yml)

**A user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation.**

Protomask started as a challenge to create a NAT64 implementation in a weekend. The goal of this implementation is to *keep things simple*. There aren't many knobs to tweak, so if you want to do stateful NAT or source address filtering, put something like `iptables` in front of it.

## How it works

Protomask operates by listening on an IPv6 `/96` prefix for incoming traffic.

When a new IPv6 host sends traffic through protomask, it is dynamically assigned an IPv4 address from a pool of addresses on a first-come-first-serve basis.

From then on, all subsequent packets coming from that same IPv6 host will be NATed through the assigned IPv4 address until the reservation period expires. Likewise, a similar process occurs for return traffic.

For hosts that necessitate a consistent IPv4 address, it is possible to configure a static mapping in the configuration file. This ensures it always communicates using the same IPv4 address no matter how long it is offline for. This is useful for single-stack hosts that need IPv4 DNS entries.

## Configuration

Protomask uses a [TOML](https://toml.io) configuration file. Here is a functional example:

```toml
# The NAT64 prefix to route to protomask
Nat64Prefix = "64:ff9b::/96"
# Setting this will enable prometheus metrics
Prometheus = "[::]:8080" # Optional, defaults to disabled

[Pool]
# All prefixes in the pool
Prefixes = ["192.0.2.0/24"]
# The maximum duration a prefix will be reserved for after becoming idle
MaxIdleDuration = 7200 # Optional, seconds. Defaults to 7200 (2 hours)
# Permanent address mappings
Static = [{ v4 = "192.0.2.2", v6 = "2001:db8:1::2" }]
```

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

## Usage

```text
Usage: protomask [OPTIONS] <CONFIG_FILE>

Arguments:
  <CONFIG_FILE>  Path to the config file

Options:
  -v, --verbose  Enable verbose logging
  -h, --help     Print help
  -V, --version  Print version
```
