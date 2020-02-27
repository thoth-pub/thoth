FROM rust:1.40.0 AS build

# Install thoth
WORKDIR /usr/src/thoth
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
COPY ./migrations ./migrations
RUN cargo install --path .

# Switch to debian for run time
FROM debian:buster-slim

# Use postgres repo to get postgresql-client-12 (not yet distributed in buster)
RUN apt-get update && apt-get install -y \
  gnupg2 \
  wget
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc \
  | apt-key add -
RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ buster-pgdg main" \
  | tee /etc/apt/sources.list.d/pgdg.list
# Install dependencies
RUN apt-get update && apt-get install -y postgresql-client-12

# Get thoth and diesel binaries
COPY --from=build /usr/local/cargo/bin/thoth /usr/local/bin/thoth
COPY --from=build /usr/local/cargo/bin/thoth_db /usr/local/bin/thoth_db

COPY entrypoint.sh ./

ENTRYPOINT ["./entrypoint.sh"]
CMD ["thoth"]
