ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

FROM ${BASE_IMAGE} as build

# Install thoth
COPY --chown=rust:rust Cargo.toml Cargo.lock ./
COPY --chown=rust:rust ./src ./src
COPY --chown=rust:rust ./migrations ./migrations
RUN cargo build --release

# Switch to debian for run time
FROM scratch

# Get thoth and diesel binaries
COPY --from=build \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/thoth \
    /usr/local/bin/

EXPOSE 8080

CMD ["/usr/local/bin/thoth", "init"]
