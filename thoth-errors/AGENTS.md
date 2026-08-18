# AGENTS.md - `thoth-errors`

This file extends the repository-root `AGENTS.md`.

It applies to shared error types and their GraphQL/HTTP representations.

## 1. Error contract

Treat externally observable errors as part of the API contract.

New errors must have:

- one clear semantic meaning;
- stable classification when clients depend on it;
- safe user-facing text;
- no secrets, tokens, SQL statements or unbounded upstream bodies;
- appropriate GraphQL/HTTP mapping;
- tests for serialization/mapping where applicable.

Do not replace a specific actionable error with a generic internal error unless disclosure would be unsafe.

Do not expose internal implementation details merely to make debugging easier.

## 2. Compatibility

Assess all consumers before renaming, merging or removing an error variant.

Import, reconciliation and background-job errors may need a stable machine-readable code separate from the human-readable message.

Sanitize and bound errors before storing them in durable job/import tables.

## 3. Required checks

```bash
cargo test -p thoth-errors
cargo test --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```
