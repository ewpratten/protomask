# protomask
[![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask)
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/Ewpratten/protomask/actions/workflows/build.yml)
[![Audit](https://github.com/ewpratten/protomask/actions/workflows/audit.yml/badge.svg)](https://github.com/ewpratten/protomask/actions/workflows/audit.yml)

**A user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation.**

*This section is WIP*

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
