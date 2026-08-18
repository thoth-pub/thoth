# Repository and Environment Map

Status: Verified 2026-07-24; repository entries and contract relationships for
`thoth-client` (standalone), `thoth-pyramid`, `thoth-strapi` and the
`thoth-sphinx` branch state independently verified 2026-08-15; `baboon` added
and verified 2026-08-16 (see `branch-topology.md` and `contracts.md` for
evidence dates by row)
Owner: CTO

This directory records the repository, branch, build, CI, release and deployment boundaries required for AI-led delivery.

## Files

- `branch-topology.md` - observed branch state, desired standard and normalization gates.
- `contracts.md` - verified repository ownership and contract/consumer
  relationships, used by the cross-repository impact-analysis gate; also
  disambiguates the standalone Python `thoth-pub/thoth-client` from the
  internal Rust `thoth-client` workspace member in `thoth-pub/thoth`.
- `environments.md` - verified runtime, preview and release boundaries.
- `control-gaps.md` - missing controls and required follow-up tasks.
- `graphql-mutation-guard-runtime-operations.md` - the evidenced operational
  control record for the GraphQL mutation-guard runtime mode
  (`THOTH_GRAPHQL_MUTATION_GUARD_MODE`), delivered by `THOTH-GQL-OPS-01`. It
  establishes ownership, configuration authority, restart/redeploy semantics,
  propagation, partial-fleet handling, rollback and audit; proves the two
  capability gaps that block mode control; and records CG-13 disposition
  `C - BLOCKED` with the `ADR-0006` runtime-operations gate `NOT SATISFIED`.
  It authorizes nothing.
- `graphql-mutation-guard-mode-transition-runbook.md` - the `ADR-0006` section
  8.3.5 mode-transition runbook. **PROVISIONAL and NOT EXECUTABLE** until
  `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` are implemented, independently
  reviewed and merged.
- `repositories/thoth.md`
- `repositories/thoth-app.md`
- `repositories/thoth-dissemination.md`
- `repositories/thoth-sphinx.md`
- `repositories/thoth-client.md` - standalone Python API client (distinct from
  the internal Rust `thoth-client` workspace member documented in
  `repositories/thoth.md`).
- `repositories/thoth-pyramid.md`
- `repositories/thoth-strapi.md`
- `repositories/metrics-dashboard.md`
- `repositories/metrics-widget.md`
- `repositories/cc-license.md`
- `repositories/baboon.md` - library-oriented MARC exchange service; consumes
  the Thoth GraphQL and export APIs, and has a HIGH-risk pull-request-triggered
  production SFTP scratch write.

## Usage rule

At the start of every task:

1. read the relevant repository file;
2. verify the current branch and head commit directly;
3. verify commands against the current repository;
4. report any difference from this map;
5. update this map through a reviewed documentation task when durable repository behaviour changes.

This map is not a substitute for live GitHub state.
