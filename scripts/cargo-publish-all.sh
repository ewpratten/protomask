#! /bin/bash
set -ex

cargo publish --package easy-tun || true
cargo publish --package fast-nat || true
cargo publish --package protomask-metrics || true
cargo publish --package interproto || true
cargo publish --package rfc6052 || true
cargo publish --package rtnl || true
cargo publish --package protomask
