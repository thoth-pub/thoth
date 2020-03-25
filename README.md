<img src="https://www.openbookpublishers.com/shopimages/thoth.png" alt="Thoth" height="500" />

> GraphQL API for bibliographic data

[![Build Status](https://travis-ci.org/openbookpublishers/thoth.svg?branch=master)](https://travis-ci.org/openbookpublishers/thoth)
[![Release](https://img.shields.io/github/release/openbookpublishers/thoth.svg?colorB=58839b&maxAge=86400)](https://github.com/openbookpublishers/thoth/releases)
[![Crates.io](https://img.shields.io/crates/v/thoth.svg?maxAge=86400)](https://crates.io/crates/thoth)
[![License](https://img.shields.io/github/license/openbookpublishers/thoth.svg?colorB=blue)](https://github.com/openbookpublishers/thoth/blob/master/LICENSE)

---

## Requirements

- [Rustup](https://rustup.rs/)
- Stable Toolchain: `rustup default stable`
- A PostgreSQL database (included in docker-compose.yml if ran using docker)

## Getting Started

### With docker


```sh
git clone https://github.com/OpenBookPublishers/thoth.git
cd thoth
cp .env.example .env  # Edit the credentials in .env
docker-compose up
```

### With rust


```sh
git clone https://github.com/OpenBookPublishers/thoth.git
cd thoth
cp .env.example .env  # Edit the credentials in .env
cargo run init
```
