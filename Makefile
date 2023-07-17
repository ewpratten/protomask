SRC=$(wildcard src/*.rs) $(wildcard src/**/*.rs) $(wildcard src/**/**/*.rs) Cargo.toml

target/debug/protomask: $(SRC)
	cross build --target x86_64-unknown-linux-musl
	sudo setcap cap_net_admin=eip $@

target/release/protomask: $(SRC)
	cross build --target x86_64-unknown-linux-musl --release
	sudo setcap cap_net_admin=eip $@