# All sources used to build the protomask binary
SRC = Cargo.toml $(shell find src/ -type f -name '*.rs') $(shell find protomask-tun/src/ -type f -name '*.rs')

# Used to auto-version things
GIT_HASH ?= $(shell git log --format="%h" -n 1)

# Release binary for x64
target/x86_64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target x86_64-unknown-linux-musl --release

# Release binary for aarch64
target/aarch64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target aarch64-unknown-linux-musl --release

# All tars
tars: tars/protomask-$(GIT_HASH)-x86_64.tar.gz tars/protomask-$(GIT_HASH)-aarch64.tar.gz

# TAR file for x64
tars/protomask-$(GIT_HASH)-x86_64-linux-musl.tar.gz: target/x86_64-unknown-linux-musl/release/protomask protomask.toml
	mkdir -p tars
	cp protomask.toml target/x86_64-unknown-linux-musl/release/
	tar -czf $@ -C target/x86_64-unknown-linux-musl/release/ protomask protomask.toml

# TAR file for aarch64
tars/protomask-$(GIT_HASH)-aarch64-linux-musl.tar.gz: target/aarch64-unknown-linux-musl/release/protomask protomask.toml
	mkdir -p tars
	cp protomask.toml target/aarch64-unknown-linux-musl/release/
	tar -czf $@ -C target/aarch64-unknown-linux-musl/release/ protomask protomask.toml