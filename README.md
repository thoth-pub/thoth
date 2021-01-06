<div align="center">
  <img src="https://www.openbookpublishers.com/shopimages/thoth-logo.png" height="400" />

  <h1>Thoth</h1>

  <p>
    <strong>Open bibliographic metadata management and dissemination system</strong>
  </p>

  <p>
    <a href="https://github.com/thoth-pub/thoth/actions"><img alt="GitHub Workflow" src="https://img.shields.io/github/workflow/status/thoth-pub/thoth/build-and-test/master"></a>
    <a href="https://github.com/thoth-pub/thoth/releases"><img alt="Thoth Releases" src="https://img.shields.io/github/release/thoth-pub/thoth.svg?colorB=58839b&maxAge=86400"/></a>
    <a href="https://crates.io/crates/thoth"><img alt="Crate Info" src="https://img.shields.io/crates/v/thoth.svg?maxAge=86400"/></a>
    <a href="https://github.com/thoth-pub/thoth/blob/master/LICENSE"><img alt="License Info" src="https://img.shields.io/github/license/thoth-pub/thoth.svg?colorB=blue"/></a>
  </p>
</div>

## About

**Thoth** (/θoʊθ, toʊt/, Greek Θώθ < Coptic Ⲑⲱⲟⲩⲧ < Egyptian *ḏḥwtj*) is an Open Dissemination System for Open Access books. Written purely in rust, it consists of:

* A GraphQL API, implementing a data model specifically designed for OA books
* An actions API to export metadata in formats like ONIX, MARC, etc.
* A WebAssembly GUI to manage metadata records.


## Getting Started

### Requirements

- [Rustup](https://rustup.rs/)
- Stable Toolchain: `rustup default stable`
- [wasm-pack](https://rustwasm.github.io/docs/wasm-pack/introduction.html)
- [rollup](https://www.npmjs.com/package/rollup)
- A PostgreSQL database (included in docker-compose.yml if ran using docker)
- `libssl-dev`

### Running with docker


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

#### API

```sh
cargo run init
```

#### Wasm GUI

```sh
wasm-pack build thoth-app/ --target web \
  && rollup thoth-app/main.js --format iife --file thoth-app/pkg/thoth_app.js \
  && cargo run start app
```

### Building with docker

The wasm APP needs to know the endpoint the API will be running at compile time, we must provide `THOTH_API` as a build argument to the docker daemon upon build:

```
docker build --build-arg THOTH_API=https://api.thoth.openbookpublishers.com . -t openbookpublishers/thoth
```

## Acknowledgements

Thoth is being developed as part of the [COPIM](https://www.copim.ac.uk) project, an international effort to build community-owned, open systems and infrastructures to enable Open Access book publishing to flourish. COPIM is funded by the [Research England Development Fund](https://re.ukri.org/funding/our-funds-overview/research-england-development-red-fund/) (REDFund) and [Arcadia](https://www.arcadiafund.org.uk/).
