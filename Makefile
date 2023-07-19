# All sources used to build the protomask binary
SRC = Cargo.toml $(shell find src/ -type f -name '*.rs') $(shell find protomask-tun/src/ -type f -name '*.rs')

# Used to auto-version things
GIT_HASH ?= $(shell git log --format="%h" -n 1)

# Release binary for x64
target/x86_64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target x86_64-unknown-linux-musl --release
#	sudo setcap cap_net_admin=eip $@

# Release binary for aarch64
target/aarch64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target aarch64-unknown-linux-musl --release
#	sudo setcap cap_net_admin=eip $@

