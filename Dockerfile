FROM alpine:latest

# Copy the binary from the builder container
COPY ./target/x86_64-unknown-linux-musl/release/protomask /usr/local/bin/protomask

# NOTE: We expect the config file to be mounted at /etc/protomask.toml
ENTRYPOINT ["/usr/local/bin/protomask", "/etc/protomask.toml"]