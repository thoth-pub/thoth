FROM ghcr.io/thoth-pub/muslrust AS build

ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

# Get source
COPY . .

# Build Thoth for release from source
RUN cargo build --release

FROM scratch

# Get thoth binary
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
