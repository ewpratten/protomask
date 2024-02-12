# RFC6052 for Rust
[![Crates.io](https://img.shields.io/crates/v/rfc6052)](https://crates.io/crates/rfc6052)
[![Docs.rs](https://docs.rs/rfc6052/badge.svg)](https://docs.rs/rfc6052)

[RFC6052](https://datatracker.ietf.org/doc/html/rfc6052) defines *"the algorithmic translation of an IPv6 address to a corresponding IPv4 address, and vice versa, using only statically configured information"*. In simpler terms, this means *embedding IPv4 address into IPv6 addresses*. The primary use case of which being NAT64 translators.

The RFC defines the following scheme for embedding IPv4 addresses into IPv6 addresses:

```text
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|PL| 0-------------32--40--48--56--64--72--80--88--96--104---------|
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|32|     prefix    |v4(32)         | u | suffix                    |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|40|     prefix        |v4(24)     | u |(8)| suffix                |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|48|     prefix            |v4(16) | u | (16)  | suffix            |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|56|     prefix                |(8)| u |  v4(24)   | suffix        |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|64|     prefix                    | u |   v4(32)      | suffix    |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|96|     prefix                                    |    v4(32)     |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
```

- `PL` is the prefix length
- `u` is a reserved byte that **must** be set to `0`

## Safe vs. Unsafe

This library provides both a "regular" and "unchecked" version of the functions for embedding and extracting IPv4 addresses from IPv6 addresses.

The "regular" functions enforce the restricted set of IPv6 prefix lengths allowed by the RFC (32, 40, 48, 56, 64, and 96 bits long). The "unchecked" functions do not enforce this restriction, and will happily accept any prefix length at the cost of non-compliance with the RFC.

