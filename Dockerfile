ARG RUST_IMAGE=rust:1.64.0
ARG MUSL_IMAGE=clux/muslrust:1.63.0-stable

FROM ${RUST_IMAGE} as wasm

ENV NPM_VERSION=8.15.1
ENV N_VERSION=9.0.0
ENV NODE_VERSION=16.16.0
ENV ROLLUP_VERSION=2.77.2
ENV WASM_PACK_VERSION=0.10.3

ARG THOTH_GRAPHQL_API=https://api.thoth.pub
ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_GRAPHQL_API=${THOTH_GRAPHQL_API}
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

WORKDIR /wasm

# Install build dependencies
RUN apt-get update && apt-get -y install pkg-config npm
RUN npm install -g npm@${NPM_VERSION}
RUN npm install -g n@${N_VERSION}
RUN n ${NODE_VERSION}
RUN npm install -g rollup@${ROLLUP_VERSION}
RUN cargo install wasm-pack --version ${WASM_PACK_VERSION}

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

COPY --from=wasm --chown=rust:rust /wasm/ /volume/
# Build Thoth for release
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get thoth and diesel binaries
COPY --from=build \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/thoth /

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
