# Repository: thoth-pub/cc-license

## Responsibility

Canonical Rust parser and metadata authority for Creative Commons licence URLs retained by Thoth.

## Branches

GitHub default/release: `main`
Development: `develop`
Target: `develop -> master`

BR-LIC-01 must normalize the release branch before publication.

## Stack

- Rust 2021
- crate name `cc_license`
- regex-based parsing
- crates.io distribution documented

## Commands

```bash
cargo test --workspace --verbose
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## CI

Current CI runs tests, clippy and formatting on `main` and `develop`.

The workflow uses older checkout/actions-rs actions. Modernization should be a separate CI task from licence behaviour.

## Publisher Services requirements

LIC-01 will add or confirm:

- canonical URL output;
- enumeration of retained licences;
- descriptions and display metadata;
- legacy/jurisdiction metadata;
- public-domain tools retained by Thoth;
- rejection of invalid/spoofed URLs;
- normalization tests.

Thoth must consume a released crate version rather than copying parsing rules.

## Release gap

The exact crate publication command, credentials, approval and rollback/yank procedure are not verified. Record them before LIC-01 is marked production ready.
