FROM rust:1.40.0 AS build

# Install thoth
WORKDIR /usr/src/thoth
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN cargo install --path .

# Install diesel_cli so we can run our migrations with entrypoint
RUN cargo install diesel_cli --no-default-features --features postgres

# Switch to debian for run time
FROM debian:buster-slim

# Use postgres repo to get postgresql-client-12 (not yet distributed in buster)
RUN apt-get update && apt-get install -y gnupg2 wget
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc \
  | apt-key add -
RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ buster-pgdg main" \
  | tee /etc/apt/sources.list.d/pgdg.list
# Install dependencies
RUN apt-get update && apt-get install -y postgresql-client-12

# Get thoth and diesel binaries
COPY --from=build /usr/local/cargo/bin/thoth /usr/local/bin/thoth
COPY --from=build /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# Get thoth migrations
COPY ./migrations ./migrations

COPY entrypoint.sh ./

ENTRYPOINT ["./entrypoint.sh"]
CMD ["thoth"]
