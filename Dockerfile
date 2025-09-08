FROM clux/muslrust:1.88.0-stable

ARG THOTH_GRAPHQL_API=https://api.thoth.pub
ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_GRAPHQL_API=${THOTH_GRAPHQL_API}
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

# Install OpenSSL development packages and pkg-config
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Get source
COPY . .

# Set environment variables for OpenSSL
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

# Build Thoth for release from source
RUN cargo build --release

# Move the binary to root for easier access
RUN mv /volume/target/x86_64-unknown-linux-musl/release/thoth /thoth

# Expose thoth's default ports
EXPOSE 8080
EXPOSE 8000
EXPOSE 8181

# Make thoth our default binary
ENTRYPOINT ["/thoth"]

# By default run `thoth init` (runs migrations and starts the server on port 8080)
CMD ["init"]