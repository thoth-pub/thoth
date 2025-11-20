FROM rust:1.90.0

ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Get source
COPY . .

# Build Thoth for release from source
RUN cargo build --release

# Move the binary to root for easier access
RUN mv target/release/thoth /thoth

# Expose thoth's default ports
EXPOSE 8080
EXPOSE 8000
EXPOSE 8181

# Make thoth our default binary
ENTRYPOINT ["/thoth"]

# By default run `thoth init` (runs migrations and starts the server on port 8080)
CMD ["init"]
