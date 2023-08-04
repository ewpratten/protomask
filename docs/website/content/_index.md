---
title: Protomask
---

## What is protomask?

Protomask is a user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation geared towards networks that need fast and reliable inter-protocol packet translation. Behind the scenes, protomask uses the [Universal TUN/TAP Device Driver](https://docs.kernel.org/networking/tuntap.html) to translate incoming packets from IPv4 to IPv6 and vice-versa.

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

## Want to learn more?

Check out the [documentation](/book) for a deeper dive into protomask, or head over to the [GitHub repository](https://github.com/ewpratten/protomask) to see the source code.
