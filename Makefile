SRC=$(wildcard src/*.rs) $(wildcard src/**/*.rs) $(wildcard src/**/**/*.rs) Cargo.toml

target/debug/protomask: $(SRC)
	cargo build
	sudo setcap cap_net_admin=eip $@