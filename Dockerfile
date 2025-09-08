FROM rust:1.88.0-alpine

ARG THOTH_GRAPHQL_API=https://api.thoth.pub
ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_GRAPHQL_API=${THOTH_GRAPHQL_API}
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

# Install musl build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    gcc \
    && rustup target add x86_64-unknown-linux-musl

# Get source
COPY . .

# Set environment variables for musl build
ENV OPENSSL_STATIC=1
ENV RUSTFLAGS="-C target-feature=-crt-static"

# Build Thoth for release from source targeting musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# Move the binary to root for easier access
RUN mv target/x86_64-unknown-linux-musl/release/thoth /thoth

# Expose thoth's default ports
EXPOSE 8080
EXPOSE 8000
EXPOSE 8181

# Make thoth our default binary
ENTRYPOINT ["/thoth"]

# By default run `thoth init` (runs migrations and starts the server on port 8080)
CMD ["init"]