# AGENTS.md - `thoth-client`

This file extends the repository-root `AGENTS.md`.

It applies to Thoth's internal Rust GraphQL client and its query contract.

## 1. Contract boundary

`thoth-client` consumes the GraphQL schema produced by the local `thoth-api` crate.

`build.rs` creates:

```text
assets/schema.graphql
```

from `thoth_api::graphql::create_schema()` during the build.

Do not hand-edit the generated schema file.

The tracked query source is:

```text
assets/queries.graphql
```

## 2. GraphQL changes

When an API change affects internal consumers:

1. update the API schema in `thoth-api`;
2. update `assets/queries.graphql` only when the client must request new/different fields;
3. build the client so the local schema is regenerated;
4. fix generated enum/scalar conversions exhaustively;
5. run downstream export-server tests.

Do not use a remote schema for this crate. It must build against the exact local API contract.

## 3. Compatibility

GraphQL enum additions may require updates to explicit Rust conversion matches.

Do not hide a new enum variant behind a wildcard that converts it incorrectly.

Where the generated client exposes `Other(_)`, decide explicitly whether the consumer may safely ignore it or must fail.

Preserve request retry and serialization behaviour unless the task explicitly changes it.

## 4. Required checks

```bash
cargo build -p thoth-client
cargo test -p thoth-client
cargo test -p thoth-export-server
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Inspect the resulting schema/query diff and ensure it contains only the approved contract change.
