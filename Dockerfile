FROM rust:1.40.0

WORKDIR /usr/src/thoth

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN cargo install --path .

COPY diesel.toml ./
COPY ./migrations ./migrations

RUN cargo install diesel_cli

COPY entrypoint.sh ./

ENTRYPOINT ["./entrypoint.sh"]
CMD ["thoth"]
