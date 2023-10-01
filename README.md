# `protomask`: Fast & reliable user space NAT64
[![GitHub release](https://img.shields.io/github/v/release/ewpratten/protomask)](https://github.com/ewpratten/protomask/releases/latest)
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/build.yml)
[![Audit](https://github.com/ewpratten/protomask/actions/workflows/audit.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/audit.yml)

> The protomask tool suite is a collection of user space tools that translate packets between OSI layer 3 protocol versions

This repository (referred to as the *protomask tool suite*) contains the following sub-projects:

<table>
    <thead>
        <tr>
            <td><strong>Crate</strong></td>
            <td><strong>Info</strong></td>
            <td><strong>Latest Version</strong></td>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><a href="./src/protomask.rs"><code>protomask</code></a></td>
            <td>User space NAT64 implementation</td>
            <td><a href="https://crates.io/crates/protomask"><img src="https://img.shields.io/crates/v/protomask" alt="crates.io"></a></td>
        </tr>
        <tr>
            <td><a href="./src/protomask-clat.rs"><code>protomask-clat</code></a></td>
            <td>User space Customer-side transLATor (CLAT) implementation</td>
            <td><a href="https://crates.io/crates/protomask"><img src="https://img.shields.io/crates/v/protomask" alt="crates.io"></a></td>
        </tr>
        <tr>
            <td><a href="./libs/easy-tun/"><code>easy-tun</code></a></td>
            <td>A pure-rust TUN interface library</td>
            <td>
                <a href="https://crates.io/crates/easy-tun"><img src="https://img.shields.io/crates/v/easy-tun" alt="crates.io"></a>
                <a href="https://docs.rs/easy-tun"><img src="https://docs.rs/easy-tun/badge.svg" alt="docs.rs"></a>
            </td>
        </tr>
        <tr>
            <td><a href="./libs/fast-nat/"><code>fast-nat</code></a></td>
            <td>An OSI layer 3 Network Address Table built for speed</td>
            <td>
                <a href="https://crates.io/crates/fast-nat"><img src="https://img.shields.io/crates/v/fast-nat" alt="crates.io"></a>
                <a href="https://docs.rs/fast-nat"><img src="https://docs.rs/fast-nat/badge.svg" alt="docs.rs"></a>
            </td>
        </tr>
        <tr>
            <td><a href="./libs/interproto/"><code>interproto</code></a></td>
            <td>Utilities for translating packets between IPv4 and IPv6</td>
            <td>
                <a href="https://crates.io/crates/interproto"><img src="https://img.shields.io/crates/v/interproto" alt="crates.io"></a>
                <a href="https://docs.rs/interproto"><img src="https://docs.rs/interproto/badge.svg" alt="docs.rs"></a>
            </td>
        </tr>
        <tr>
            <td><a href="./libs/rfc6052/"><code>rfc6052</code></a></td>
            <td>A Rust implementation of RFC6052</td>
            <td>
                <a href="https://crates.io/crates/rfc6052"><img src="https://img.shields.io/crates/v/rfc6052" alt="crates.io"></a>
                <a href="https://docs.rs/rfc6052"><img src="https://docs.rs/rfc6052/badge.svg" alt="docs.rs"></a>
            </td>
        </tr>
        <tr>
            <td><a href="./libs/rtnl/"><code>rtnl</code></a></td>
            <td>Slightly sane wrapper around rtnetlink</td>
            <td>
                <a href="https://crates.io/crates/rtnl"><img src="https://img.shields.io/crates/v/rtnl" alt="crates.io"></a>
                <a href="https://docs.rs/rtnl"><img src="https://docs.rs/rtnl/badge.svg" alt="docs.rs"></a>
            </td>
    </tbody>
</table>

## Installation

Protomask can be installed using various methods:

### Debian

Head over to the [releases](https://github.com/ewpratten/protomask/releases) page and download the latest release for your architecture.

Then, install with:

```sh
apt install /path/to/protomask_<version>_<arch>.deb

# You can also edit the config file in /etc/protomask.json
# And once ready, start protomask with
systemctl start protomask
```

### Using Cargo

```bash
cargo install protomask
```

## Usage

The `protomask` and `protomask-clat` binaries are mostly self-sufficient.

### Nat64

To start up a NAT64 server on the Well-Known Prefix (WKP), run:

```bash
protomask --pool-prefix <prefix>
```

Where `<prefix>` is some block of addresses that are routed to the machine running protomask.

For more information, run `protomask --help`. Configuration may also be supplied via a JSON file. See the [example config](./config/protomask.json) for more information.


### CLAT

To start up a CLAT server on the Well-Known Prefix (WKP), run:

```bash
protomask-clat --customer-prefix <prefix>
```

Where `<prefix>` is some block of addresses that are routed to the machine running protomask. This would generally be the address range of a home network when run on CPE. It may also be an individual client address if run on a client device instead of a router.

For more information, run `protomask-clat --help`. Configuration may also be supplied via a JSON file. See the [example config](./config/protomask-clat.json) for more information.
