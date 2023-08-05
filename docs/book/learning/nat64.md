# What is NAT64?

IPv4 and IPv6 are two different versions of the Internet Protocol that, while being similar in many ways, are not directly compatible (largely due to their differing header structure).


## Addressing

IPv4 addresses are 32-bit numbers (represented as `xxx.xxx.xxx.xxx`), while IPv6 addresses are 128-bit numbers (represented as `xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx`).

When an IPv4 packet is sent from one host to another, the sender embeds both the sending and receiving address into the packet header (just like a destination and return address on physical mail). This means that a packet traveling from `192.0.2.1` to `192.0.2.2` would be marked as such in the packet header:

