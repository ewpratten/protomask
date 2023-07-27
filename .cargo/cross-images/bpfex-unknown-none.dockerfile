# Cross image for building bpfel and bpfeb targets

FROM rust:latest
ARG TOOLCHAIN_CHANNEL

# Install the needed toolcahin components
RUN rustup install ${TOOLCHAIN_CHANNEL}
RUN rustup component add rust-src --toolchain ${TOOLCHAIN_CHANNEL}

# Install the BFP linker
RUN cargo install bpf-linker
