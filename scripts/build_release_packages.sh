#! /bin/bash
# Builds everything needed for a new release
set -ex

# Build RPM
cargo rpm build

# Build Docker image
cross build --release --target x86_64-unknown-linux-musl
docker build -t ewpratten/protomask:latest .
