# All sources used to build the protomask binary
PROTOMASK_SRC = protomask/Cargo.toml $(shell find protomask/src/ -type f -name '*.rs') 
PROTOMASK_EBPF_SRC = protomask-ebpf/Cargo.toml $(shell find protomask-ebpf/src/ -type f -name '*.rs')

# Used to auto-version things
CRATE_VERSION = $(shell sed -n -r "s/^version = \"([0-9\.]+)\"/\1/p" protomask/Cargo.toml)
TOOLCHAIN_CHANNEL = $(shell sed -n -r "s/^channel = \"(.+)\"/\1/p" rust-toolchain.toml)

.PHONY: clean

# x64 Protomask binary (Debug)
target/x86_64-unknown-linux-musl/debug/protomask: $(PROTOMASK_SRC) target/bpfel-unknown-none/debug/protomask-ebpf
	cross build --target x86_64-unknown-linux-musl --bin protomask

# x64 Protomask binary (Release)
target/x86_64-unknown-linux-musl/release/protomask: $(PROTOMASK_SRC) target/bpfel-unknown-none/release/protomask-ebpf
	cross build --target x86_64-unknown-linux-musl --bin protomask --release

# Docker image used for building bpfel and bpfeb images
.cargo/cross-images/bpfex-unknown-none.dockerfile.timestamp: .cargo/cross-images/bpfex-unknown-none.dockerfile
	docker build -t protomask/bpfex-unknown-none -f $< --build-arg TOOLCHAIN_CHANNEL=$(TOOLCHAIN_CHANNEL) .cargo/cross-images
	touch .cargo/cross-images/bpfex-unknown-none.dockerfile.timestamp

# Little-Endian BPF (Debug)
target/bpfel-unknown-none/debug/protomask-ebpf: $(PROTOMASK_EBPF_SRC) .cargo/cross-images/bpfex-unknown-none.dockerfile.timestamp
	cross build -Z build-std=core --target bpfel-unknown-none --bin protomask-ebpf

# Little-Endian BPF (Release)
target/bpfel-unknown-none/release/protomask-ebpf: $(PROTOMASK_EBPF_SRC) .cargo/cross-images/bpfex-unknown-none.dockerfile.timestamp
	cargo build -Z build-std=core --target bpfel-unknown-none --bin protomask-ebpf --release

# Big-Endian BPF (Debug)
target/bpfeb-unknown-none/debug/protomask-ebpf: $(PROTOMASK_EBPF_SRC) .cargo/cross-images/bpfex-unknown-none.dockerfile.timestamp
	cross build -Z build-std=core --target bpfeb-unknown-none --bin protomask-ebpf

# Big-Endian BPF (Release)
target/bpfeb-unknown-none/release/protomask-ebpf: $(PROTOMASK_EBPF_SRC) .cargo/cross-images/bpfex-unknown-none.dockerfile.timestamp
	cargo build -Z build-std=core --target bpfeb-unknown-none --bin protomask-ebpf --release

# Cleanup task
clean:
	cargo clean

# target/x86_64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_amd64.deb: target/x86_64-unknown-linux-musl/release/protomask
# 	cargo deb --target x86_64-unknown-linux-musl --no-build

# target/aarch64-unknown-linux-musl/debian/protomask_${CRATE_VERSION}_arm64.deb: target/aarch64-unknown-linux-musl/release/protomask
# 	cargo deb --target aarch64-unknown-linux-musl --no-build
