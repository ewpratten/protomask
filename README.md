# protomask
[![Crates.io](https://img.shields.io/crates/v/protomask)](https://crates.io/crates/protomask) 
[![Docs.rs](https://docs.rs/protomask/badge.svg)](https://docs.rs/protomask) 
[![Build](https://github.com/Ewpratten/protomask/actions/workflows/build.yml/badge.svg)](https://github.com/Ewpratten/protomask/actions/workflows/build.yml)

**A user space NAT64 implementation.**

*Protomask* started as a challenge to create a NAT64 implementation in a weekend. The goal of this implementation is to *keep things simple*. There aren't many knobs to tweak, so if you want to do stateful NAT or source address filtering, put something like `iptables` in front of it.


