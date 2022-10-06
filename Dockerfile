FROM rust:1.64.0 as build

# WASM dependencies
ENV NPM_VERSION=8.15.1
ENV N_VERSION=9.0.0
ENV NODE_VERSION=16.16.0
ENV ROLLUP_VERSION=2.77.2
ENV WASM_PACK_VERSION=0.10.3

# static dependencies
ENV TARGET=x86_64
ENV BUILD_TARGET=${TARGET}-unknown-linux-musl
ENV POSTGRES_VERSION=12.11
ENV OPENSSL_VERSION=1.1.1o
ENV ZLIB_VERSION=1.2.12
ENV MUSL_CROSS_VERSION=fe915821b652a7fa37b34a596f47d8e20bc72338

ARG THOTH_GRAPHQL_API=https://api.thoth.pub
ARG THOTH_EXPORT_API=https://export.thoth.pub
ENV THOTH_GRAPHQL_API=${THOTH_GRAPHQL_API}
ENV THOTH_EXPORT_API=${THOTH_EXPORT_API}

ENV STATIC_DIR=/static

# Build musl toolchain
WORKDIR $STATIC_DIR
RUN curl -fLsS "https://github.com/richfelker/musl-cross-make/archive/${MUSL_CROSS_VERSION}.tar.gz" | tar xz
WORKDIR "${STATIC_DIR}/musl-cross-make-${MUSL_CROSS_VERSION}"
RUN make install TARGET=${TARGET}-linux-musl OUTPUT=/usr/local/musl > /dev/null
ENV CC_x86_64_unknown_linux_musl=/usr/local/musl/bin/x86_64-linux-musl-gcc

# Build zlib
WORKDIR $STATIC_DIR
RUN curl -fLsS "https://zlib.net/zlib-${ZLIB_VERSION}.tar.gz" | tar xz
WORKDIR "${STATIC_DIR}/zlib-${ZLIB_VERSION}"
RUN env CC=/usr/local/musl/bin/${TARGET}-linux-musl-gcc ./configure --static --prefix=/usr/local/musl
RUN make
RUN make install

# Build OpenSSL
WORKDIR $STATIC_DIR
RUN curl -fLsS "https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz" | tar xz
WORKDIR "${STATIC_DIR}/openssl-${OPENSSL_VERSION}"
RUN env CC=/usr/local/musl/bin/${TARGET}-linux-musl-gcc ./Configure no-shared --prefix=/usr/local/musl linux-${TARGET}
RUN make install_sw
ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_DIR=/usr/local/musl
ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=1

# Build libpq
WORKDIR $STATIC_DIR
RUN curl -fLsS "https://ftp.postgresql.org/pub/source/v${POSTGRES_VERSION}/postgresql-${POSTGRES_VERSION}.tar.gz" | tar xz
ENV POSTGRES_DIR="${STATIC_DIR}/postgresql-${POSTGRES_VERSION}"
WORKDIR $POSTGRES_DIR
RUN ./configure --host=$(uname -m)-linux --prefix=/usr/local/musl \
        CC=/usr/local/musl/bin/${TARGET}-linux-musl-gcc \
        CPPFLAGS=-I/usr/local/musl/include \
        LDFLAGS=-L/usr/local/musl/lib \
        --with-openssl --without-readline
ENV POSTGRES_SRC="${POSTGRES_DIR}/src"
WORKDIR "${POSTGRES_SRC}/interfaces/libpq"
RUN make all-static-lib
RUN make install-lib-static
WORKDIR "${POSTGRES_SRC}/bin/pg_config"
RUN make
RUN make install

# Install build dependencies (all required for WASM, except clang that is required by proc-macros)
RUN apt-get update && apt-get -y install pkg-config npm clang
RUN npm install -g npm@${NPM_VERSION}
RUN npm install -g n@${N_VERSION}
RUN n ${NODE_VERSION}
RUN npm install -g rollup@${ROLLUP_VERSION}
RUN cargo install wasm-pack --version ${WASM_PACK_VERSION}
RUN rustup target add ${BUILD_TARGET}

WORKDIR /thoth

# Get source
COPY . .

# Compile WASM for release
RUN wasm-pack build thoth-app/ \
  --target web \
  --release
RUN rollup thoth-app/main.js \
  --format iife \
  --file thoth-app/pkg/thoth_app.js

# Build Thoth for release
ENV CARGO_BUILD_TARGET=${BUILD_TARGET}
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=/usr/local/musl/bin/x86_64-linux-musl-gcc
RUN cargo build --release

# Switch to minimal image for run time
FROM scratch

# Get thoth and diesel binaries
COPY --from=build \
    /thoth/target/x86_64-unknown-linux-musl/release/thoth /

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
