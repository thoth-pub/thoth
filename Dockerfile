ARG RUST_IMAGE=rust:1.51.0
ARG MUSL_IMAGE=ekidd/rust-musl-builder:1.51.0

FROM ${RUST_IMAGE} as wasm

ARG THOTH_GRAPHQL_API=https://api.thoth.pub
ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_GRAPHQL_API=${THOTH_GRAPHQL_API}
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

WORKDIR /wasm

# Install build dependencies
RUN apt-get update && apt-get -y install pkg-config npm
RUN npm install -g npm@6.14.8
RUN npm install -g n@6.7.0
RUN n 12.19.0
RUN npm install -g rollup@2.28.2
RUN cargo install wasm-pack

# Get source
COPY . .

# Compile WASM for release
RUN wasm-pack build thoth-app/ \
  --target web \
  --release
RUN rollup thoth-app/main.js \
  --format iife \
  --file thoth-app/pkg/thoth_app.js

# Switch to musl for static compiling
FROM ${MUSL_IMAGE} as build

# "An ARG instruction goes out of scope at the end of the build stage where it was defined.
# To use an arg in multiple stages, each stage must include the ARG instruction."
# https://docs.docker.com/engine/reference/builder/#scope
ARG THOTH_GRAPHQL_API=https://api.thoth.pub
ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_GRAPHQL_API=${THOTH_GRAPHQL_API}
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

COPY --from=wasm --chown=rust:rust /wasm/ /home/rust/src/
# Build Thoth for release
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get thoth and diesel binaries
COPY --from=build \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/thoth /

# Expose thoth's default ports
EXPOSE 8080
EXPOSE 8000
EXPOSE 8181

# Make thoth our default binary
ENTRYPOINT ["/thoth"]

# By default run `thoth init` (runs migrations and starts the server on port 8080)
CMD ["init"]
