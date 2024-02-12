# All sources used to build the protomask binary
SRC = Cargo.toml $(shell find src/ -type f -name '*.rs')
CONFIGS = $(shell find config/ -type f -name '*.json')
DEBIAN_SCRIPTS = $(shell find debian/ -type f)

# Used to auto-version things
CRATE_VERSION = $(shell sed -n -r "s/^version = \"([0-9\.]+)\"/\1/p" Cargo.toml)

.PHONY: all release clean
all: release

target/x86_64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target x86_64-unknown-linux-musl --release --bin protomask

target/x86_64-unknown-linux-musl/release/protomask-clat: $(SRC)
	cross build --target x86_64-unknown-linux-musl --release --bin protomask-clat

target/aarch64-unknown-linux-musl/release/protomask: $(SRC)
	cross build --target aarch64-unknown-linux-musl --release

target/x86_64-unknown-linux-musl/release/protomask-clat: $(SRC)
	cross build --target x86_64-unknown-linux-musl --release --bin protomask-clat

target/protomask.tar.gz:	target/x86_64-unknown-linux-musl/release/protomask \
							target/x86_64-unknown-linux-musl/release/protomask-clat \
							target/aarch64-unknown-linux-musl/release/protomask \
							target/aarch64-unknown-linux-musl/release/protomask-clat \
							$(CONFIGS)
	mkdir -p target/protomask_tar_temp/{bin,config}
	mkdir -p target/protomask_tar_temp/bin/{x86_64,aarch64}
	cp target/x86_64-unknown-linux-musl/release/protomask target/protomask_tar_temp/bin/x86_64/protomask
	cp target/x86_64-unknown-linux-musl/release/protomask-clat target/protomask_tar_temp/bin/x86_64/protomask-clat
	cp target/aarch64-unknown-linux-musl/release/protomask target/protomask_tar_temp/bin/aarch64/protomask
	cp target/aarch64-unknown-linux-musl/release/protomask-clat target/protomask_tar_temp/bin/aarch64/protomask-clat
	cp config/*.json target/protomask_tar_temp/config/
	tar -czf target/protomask.tar.gz -C target/protomask_tar_temp .
	rm -rf target/protomask_tar_temp

target/x86_64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_amd64.deb: 	target/x86_64-unknown-linux-musl/release/protomask \
																				target/x86_64-unknown-linux-musl/release/protomask-clat \
																				$(CONFIGS) \
																				$(DEBIAN_SCRIPTS)
	cargo deb --target x86_64-unknown-linux-musl --no-build

target/aarch64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_arm64.deb: 	target/aarch64-unknown-linux-musl/release/protomask \
																				target/aarch64-unknown-linux-musl/release/protomask-clat \
																				$(CONFIGS) \
																				$(DEBIAN_SCRIPTS)
	cargo deb --target aarch64-unknown-linux-musl --no-build

target/x86_64-unknown-linux-musl/generate-rpm/protomask-${CRATE_VERSION}-1.x86_64.rpm: 	target/x86_64-unknown-linux-musl/release/protomask \
																						target/x86_64-unknown-linux-musl/release/protomask-clat \
																						$(CONFIGS)
	cargo generate-rpm --target x86_64-unknown-linux-musl

target/aarch64-unknown-linux-musl/generate-rpm/protomask-${CRATE_VERSION}-1.aarch64.rpm: 	target/aarch64-unknown-linux-musl/release/protomask \
																							target/aarch64-unknown-linux-musl/release/protomask-clat \
																							$(CONFIGS)
	cargo generate-rpm --target aarch64-unknown-linux-musl

release: 	target/protomask.tar.gz \
			target/x86_64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_amd64.deb \
			target/aarch64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_arm64.deb \
			target/x86_64-unknown-linux-musl/generate-rpm/protomask-${CRATE_VERSION}-1.x86_64.rpm \
			target/aarch64-unknown-linux-musl/generate-rpm/protomask-${CRATE_VERSION}-1.aarch64.rpm
	mkdir -p release
	cp $^ release/

clean:
	rm -rf release
	cargo clean