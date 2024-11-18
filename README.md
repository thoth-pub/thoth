<div align="center">
  <img src="https://cdn.thoth.pub/thoth_logo.png" height="400" />

  <h1>Thoth</h1>

  <p>
    <strong>Open bibliographic metadata management and dissemination system</strong>
  </p>

  <p>
    <a href="https://github.com/thoth-pub/thoth/actions"><img alt="GitHub Workflow" src="https://img.shields.io/github/actions/workflow/status/thoth-pub/thoth/build_test_and_check.yml?branch=master"></a>
    <a href="https://github.com/thoth-pub/thoth/releases"><img alt="Thoth Releases" src="https://img.shields.io/github/release/thoth-pub/thoth.svg?colorB=58839b&maxAge=86400"/></a>
    <a href="https://github.com/thoth-pub/thoth/blob/master/LICENSE"><img alt="License Info" src="https://img.shields.io/github/license/thoth-pub/thoth.svg?colorB=blue"/></a>
  </p>
</div>

## About

**Thoth** (/toʊt, θoʊθ/, Greek Θώθ < Coptic Ⲑⲱⲟⲩⲧ < Egyptian *ḏḥwtj*) is an Open Dissemination System for Open Access books. Written purely in rust, it consists of:

* A [GraphQL API](https://api.thoth.pub), implementing a data model specifically designed for OA books
* A [REST API](https://export.thoth.pub) to export metadata in formats like ONIX, MARC, etc.
* A [WebAssembly GUI](https://thoth.pub) to manage metadata records.

For more information about Thoth, its data and metadata formats, and more, see the repo's [wiki](https://github.com/thoth-pub/thoth/wiki). You can also use GraphiQL to [explore the GraphQL API](https://api.thoth.pub/graphiql) (click on "Docs" at the top right), or RapiDoc to [inspect the REST API](https://export.thoth.pub).

## Getting Started

### Requirements

- [Rustup](https://rustup.rs/)
- Stable Toolchain: `rustup default stable`
- [wasm-pack](https://rustwasm.github.io/docs/wasm-pack/introduction.html)
- [rollup](https://www.npmjs.com/package/rollup)
- A PostgreSQL database (included in docker-compose.yml if ran using docker)
- `libssl-dev`

### Running with docker (development)

#### Config
```sh
git clone https://github.com/thoth-pub/thoth.git
cd thoth
cp .env.example .env  # Edit the credentials in .env
```
#### Run
```sh
make docker-dev
```
or
```sh
docker-compose -f docker-compose.dev.yml build
docker-compose -f docker-compose.dev.yml up
```

### Running with docker (production)

```sh
git clone https://github.com/thoth-pub/thoth.git
cd thoth
cp .env.example .env  # Edit the credentials in .env
docker-compose up
```

### Running with rust (cargo)

#### Config

```sh
git clone https://github.com/thoth-pub/thoth.git
cd thoth
cp .env.example .env  # Edit the credentials in .env
```

#### Creating Postgres DB and User

```
psql
psql -U postgres
CREATE ROLE thoth SUPERUSER LOGIN PASSWORD 'thoth';
CREATE DATABASE thoth WITH OWNER thoth;
```

Exit the psql command line with:

```\q```

An example of a .env file:

```
THOTH_GRAPHQL_API=http://localhost:8000
# THOTH_EXPORT_API is used at compile time, must be a public facing URL
THOTH_EXPORT_API=http://localhost:8181
# Authentication cookie domain
THOTH_DOMAIN=localhost
# Full postgres URL (With the role and db we created in the orevious step, it will look like this)
DATABASE_URL=postgres://thoth:thoth@localhost/thoth
# Authentication cookie secret key (can be any string really)
SECRET_KEY=we_like_s%_books_255
# Logging level
RUST_LOG=info
```

#### GraphQL API

```sh
cargo run init
```

#### Export API

```sh
cargo run start export-api
```

#### GUI

```sh
cargo run start app
```

### Building with docker

The wasm APP needs to know the endpoint the API will be running at compile time, we must provide `THOTH_API` as a build argument to the docker daemon upon build:

```
docker build \
    --build-arg THOTH_GRAPHQL_API=https://api.thoth.pub \
    --build-arg THOTH_EXPORT_API=https://export.thoth.pub \
    . -t openbookpublishers/thoth
```

## Acknowledgements

Thoth is being developed as part of the [COPIM](https://www.copim.ac.uk) project, an international effort to build community-owned, open systems and infrastructures to enable Open Access book publishing to flourish. COPIM is funded by the [Research England Development (RED) Fund](https://re.ukri.org/funding/our-funds-overview/research-england-development-red-fund/), and [Arcadia](https://www.arcadiafund.org.uk/), a charitable fund of Lisbet Rausing and Peter Baldwin.
