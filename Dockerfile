ARG BASE_IMAGE=ekidd/rust-musl-builder:1.44.1

FROM ${BASE_IMAGE} as build

# Install build dependencies
RUN sudo apt-get install -y
RUN sudo apt-get update && sudo apt-get -y install pkg-config npm
RUN sudo npm install -g npm@6.14.8
RUN sudo npm install -g n@6.7.0
RUN sudo n 12.19.0
RUN sudo npm install -g rollup@2.28.2
RUN cargo install wasm-pack

# Compile thoth for release
COPY --chown=rust:rust Cargo.toml Cargo.lock ./
COPY --chown=rust:rust ./src ./src
COPY --chown=rust:rust ./thoth-api ./thoth-api
COPY --chown=rust:rust ./thoth-client ./thoth-client
COPY --chown=rust:rust ./thoth-app ./thoth-app
RUN wasm-pack build thoth-app/ \
  --target web \
  --release
RUN rollup thoth-app/main.js \
  --format iife \
  --file thoth-app/pkg/thoth_app.js
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get thoth and diesel binaries
COPY --from=build \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/thoth /

# Expose thoth's default ports
EXPOSE 8080
EXPOSE 8000

# Make thoth our default binary
ENTRYPOINT ["/thoth"]

# By default run `thoth init` (runs migrations and starts the server on port 8080)
CMD ["init"]
