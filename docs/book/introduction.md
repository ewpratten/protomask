# Introduction

Protomask is a user space [NAT64](https://en.wikipedia.org/wiki/NAT64) implementation geared towards networks that need fast and reliable inter-protocol packet translation. Behind the scenes, protomask uses the [Universal TUN/TAP Device Driver](https://docs.kernel.org/networking/tuntap.html) to translate incoming packets from IPv4 to IPv6 and vice-versa.

For an overview of the project, see the [website](https://protomask.ewpratten.com) or [GitHub repository](https://github.com/ewpratten/protomask).

## Table of Contents

- [Protomask library suite documentation](./libraries.html)
- [Using `protomask`](./binaries/protomask.html)
- [Using `protomask-clat`](./binaries/protomask-clat.html)
