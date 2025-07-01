ARG MUSL_IMAGE=ghcr.io/blackdex/rust-musl:x86_64-musl-stable-1.88.0

FROM ${MUSL_IMAGE} as build

ARG THOTH_GRAPHQL_API=https://api.thoth.pub
ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_GRAPHQL_API=${THOTH_GRAPHQL_API}
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

# Get source
COPY . .

# Disable SIMD
ENV RUSTFLAGS="-C target-cpu=x86-64"
ENV CFLAGS="-march=x86-64"

# Build Thoth for release
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get thoth and diesel binaries
COPY --from=build \
    /volume/target/x86_64-unknown-linux-musl/release/thoth /

# Get CA certificates
COPY --from=build \
    /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Expose thoth's default ports
EXPOSE 8080
EXPOSE 8000
EXPOSE 8181

# Make thoth our default binary
ENTRYPOINT ["/thoth"]

# By default run `thoth init` (runs migrations and starts the server on port 8080)
CMD ["init"]
