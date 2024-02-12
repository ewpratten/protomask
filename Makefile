# All sources used to build the protomask binary
SRC = Cargo.toml $(shell find src/ -type f -name '*.rs')

# Used to auto-version things
CRATE_VERSION = $(shell sed -n -r "s/^version = \"([0-9\.]+)\"/\1/p" Cargo.toml)

all: target/x86_64-unknown-linux-musl/release/protomask target/aarch64-unknown-linux-musl/release/protomask target/x86_64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_amd64.deb target/aarch64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_arm64.deb

target/x86_64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target x86_64-unknown-linux-musl --release

target/aarch64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target aarch64-unknown-linux-musl --release

target/x86_64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_amd64.deb: target/x86_64-unknown-linux-musl/release/protomask
	cargo deb --target x86_64-unknown-linux-musl --no-build

target/aarch64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_arm64.deb: target/aarch64-unknown-linux-musl/release/protomask
	cargo deb --target aarch64-unknown-linux-musl --no-build