ARG BASE_IMAGE=ekidd/rust-musl-builder:1.41.0

FROM ${BASE_IMAGE} as build

# Compile thoth for release
COPY --chown=rust:rust Cargo.toml Cargo.lock ./
COPY --chown=rust:rust ./src ./src
COPY --chown=rust:rust ./assets ./assets
COPY --chown=rust:rust ./thoth-api ./thoth-api
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get thoth and diesel binaries
COPY --from=build \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/thoth \
    /usr/local/bin/

# Expose thoth's default port
EXPOSE 8080

# Run `thoth init` (runs migrations and starts the server on port 8080)
CMD ["/usr/local/bin/thoth", "init"]
