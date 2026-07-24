# AGENTS.md - `thoth-api-server`

This file extends the repository-root `AGENTS.md`.

It applies to the Actix HTTP server that exposes Thoth's GraphQL API.

## 1. Ownership boundary

This crate owns transport concerns:

- Actix server setup;
- GraphQL endpoint serving;
- request context construction;
- authentication/introspection integration;
- CORS and HTTP middleware;
- logging and transport-level error handling.

Canonical domain rules, validation and authorization policies belong in `thoth-api`.

Do not duplicate a domain rule in the HTTP layer.

## 2. Authentication

Treat authentication and ZITADEL introspection failures as deny-by-default.

Do not:

- convert an authentication backend failure into anonymous elevated access;
- trust browser-provided publisher IDs or roles;
- use CORS or origin checks as authorization;
- log bearer tokens or introspection payloads;
- collapse distinct machine roles into a broad superuser path without an approved ADR.

Preserve the public behaviour of unrelated GraphQL queries when adding protected service operations.

## 3. Request context

Any new context field must have:

- a clear owner and lifetime;
- bounded resource use;
- safe construction failure;
- tests for missing/invalid state;
- no production secret exposure.

Database pools, authenticated users and request metadata must be passed through existing context patterns.

## 4. HTTP compatibility

For endpoint or middleware changes, assess:

- URL and method compatibility;
- content types;
- status/error mapping;
- CORS;
- proxy behaviour;
- request size and timeout limits;
- observability;
- health checks.

Do not make a transport change that silently changes GraphQL domain semantics.

## 5. Required checks

```bash
cargo test -p thoth-api-server
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

For changes that affect shared context or API behaviour, also run the full workspace tests.
