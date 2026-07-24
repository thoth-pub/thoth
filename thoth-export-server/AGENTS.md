# AGENTS.md - `thoth-export-server`

This file extends the repository-root `AGENTS.md`.

It applies to metadata export endpoints, query selection, format projections and OpenAPI/HTTP behaviour.

## 1. Ownership boundary

The export server owns:

- export routes;
- format-specific projections;
- export query selection;
- output serialization;
- content type and cache behaviour;
- API documentation derived from route definitions.

It does not own canonical metadata validation or authorization rules.

## 2. Source contract

The server consumes Thoth data through the internal client and domain types.

When the GraphQL contract changes:

- update `thoth-client` first or in the same approved slice;
- test against the exact local schema;
- do not guess fields from an unmerged downstream branch.

`THOTH_EXPORT_API` and related values may be used at build time. Record environment assumptions and never bake secrets into build output.

## 3. Format correctness

For every changed export format:

- use representative fixtures;
- test optional/missing fields;
- test escaping of XML/CSV/JSON reserved characters;
- preserve namespaces, identifiers and ordering required by the target;
- preserve existing output unless the specification explicitly changes it;
- document any external standard/version relied upon;
- assess whether caches need invalidation.

Do not reuse semantically different roles, measures or identifiers merely because their serialized shape is similar.

## 4. Pagination and eligibility

Apply eligibility and authorization filtering before pagination and counts.

Verify consistency across:

- individual record endpoints;
- list endpoints;
- counts;
- feed/index endpoints;
- cache validators;
- OpenAPI documentation.

Do not expose protected package or internal configuration data through a public export.

## 5. Licence handling

`cc-license` is the canonical Creative Commons parser and metadata authority.

Do not add a second hard-coded licence registry or parse licence URLs independently in export code.

## 6. Required checks

```bash
cargo test -p thoth-export-server
cargo build -p thoth-export-server
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Add focused fixture/golden tests for every changed serialization path.
