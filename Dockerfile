FROM rust:slim

RUN rustup component add rustfmt
RUN apt-get update && apt-get install -y clang libclang-dev libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src
COPY Cargo.toml .
COPY Cargo.lock .

RUN mkdir src && echo // > src/lib.rs && cargo check
