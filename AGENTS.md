# AGENTS.md

These instructions apply to the complete `thoth-pub/thoth` repository.

A more deeply nested `AGENTS.md` adds or narrows instructions for its directory. Read this file and every applicable nested file before editing.

## 1. Required task identity

Before changing anything, record:

```text
Programme:
Repository: thoth-pub/thoth
Task ID:
Approved specification:
Risk: LOW | MEDIUM | HIGH | CRITICAL
Base branch and commit:
PR target:
Task branch:
Dependencies:
Implementing agent/model:
Independent reviewer/model:
```

Do not implement without an approved written specification. A GitHub issue is sufficient only when it contains the information required by `docs/engineering/ai-delivery/task-specification-template.md`.

If any item is unknown, treat it as missing work.

## 2. Authority

Use this order when sources conflict:

1. merged code, migrations and generated contracts;
2. approved ADRs and technical designs;
3. approved task specifications;
4. GitHub issues, pull requests, review threads and CI evidence;
5. programme-control and rollout documents;
6. agent reports and conversations.

Do not allow chat history or memory to silently override repository evidence.

Stop and escalate when authoritative sources conflict.

## 3. Mandatory control documents

Read as applicable:

- `docs/engineering/ai-delivery/operating-model.md`
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`
- `docs/engineering/ai-delivery/risk-classification.md`
- `docs/engineering/ai-delivery/release-gates.md`
- `docs/engineering/repository-map/repositories/thoth.md`
- the task specification;
- the relevant programme README, ADRs and design references.

## 4. Repository responsibilities

This repository owns:

- the canonical PostgreSQL domain;
- database migrations;
- the GraphQL API and authorization policy;
- metadata validation and write rules;
- the internal Rust GraphQL client;
- metadata export endpoints and formats;
- release container construction.

Workspace members:

- `thoth-api`
- `thoth-api-server`
- `thoth-client`
- `thoth-errors`
- `thoth-export-server`

Keep domain rules in the owning domain layer. Do not duplicate validation or authorization across transports and clients.

## 5. Branch and pull-request workflow

Normal work:

```text
develop -> feature/<area>/<task> -> develop
```

Approved large programme work:

```text
develop
  -> feature/<programme>
  -> feature/<programme>/<slice>
  -> feature/<programme>
  -> develop
```

Release:

```text
develop -> master
```

Rules:

- verify the actual base before branching;
- use one bounded task per slice branch and PR;
- do not target normal implementation directly at `master`;
- do not merge or approve your own work;
- delete a task/slice branch after merge;
- keep programme branches current at approved checkpoints;
- do not rewrite shared branch history after others depend on it.

## 6. Allowed and prohibited actions

An implementing agent may:

- inspect the repository and relevant history;
- edit within approved scope;
- add tests and documentation;
- run local checks against local or disposable services;
- commit, push and open/update a draft PR.

An implementing agent must not:

- merge a PR;
- publish a release;
- deploy or activate production behaviour;
- access production secrets;
- run commands against production databases or services;
- dispatch write-capable production workflows;
- perform destructive production operations;
- broaden scope without an approved specification update;
- change approved architecture silently.

## 7. Standard local checks

The repository Makefile is the preferred interface:

```bash
make build
make test
make check
make clippy
make check-format
make check-all
```

Equivalent full gate:

```bash
cargo test --workspace
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Record the exact commands and concise results. Do not report only "tests passed".

Use local PostgreSQL and Redis services where required. Never point test commands at production.

## 8. Change-specific evidence

### Documentation-only

At minimum:

```bash
git diff --check
```

Also verify:

- internal links and paths;
- terminology and repository names;
- no stale duplicate source;
- the required changelog entry;
- repository CI.

### Rust/domain change

Run the full workspace gate unless the approved specification explicitly narrows it.

### Database change

Also follow `thoth-api/AGENTS.md` and prove:

- forward migration;
- revert or approved forward-repair strategy;
- empty-database result;
- populated-database result;
- constraints and indexes;
- locking/downtime;
- generated schema handling;
- compatibility with existing data.

### GraphQL contract change

Also prove:

- authorization;
- negative authorization cases;
- nullability and enum compatibility;
- pagination and bounded query behaviour;
- internal client compatibility;
- downstream generated-client impact.

### Export change

Also follow `thoth-export-server/AGENTS.md` and prove output compatibility with representative fixtures.

### Workflow change

Also follow `.github/workflows/AGENTS.md`. Do not dispatch production workflows for verification.

## 9. Authorization and security

The current policy layer is in `thoth-api/src/policy.rs`.

Do not rely on UI restrictions for authorization.

For every protected operation, test as applicable:

- anonymous caller;
- authenticated caller without the role;
- caller scoped to another publisher;
- correctly scoped publisher role;
- superuser;
- machine/service role.

Authorization failures must fail closed.

Do not log:

- tokens;
- secrets;
- raw credentials;
- sensitive object URLs;
- personal source data;
- unbounded upstream response bodies.

## 10. Database and durable-state rules

PostgreSQL is canonical.

Do not introduce local files, GitHub Actions, S3 objects, browser state or an external service as the sole durable owner of:

- jobs;
- leases;
- checkpoints;
- canonical records;
- reconciliation outcomes;
- audit history.

Prefer transactional writes, unique constraints, leases, idempotency keys and explicit state transitions.

Backfills must be dry-run capable and idempotent where repeat execution is possible.

## 11. API and compatibility

Prefer additive and backwards-compatible changes.

Do not:

- rename stable enum values or API identifiers casually;
- change field meaning while retaining the same name;
- remove an API field without an approved deprecation path;
- expose service credentials to browser clients;
- allow downstream repositories to guess an unmerged schema.

When a downstream repository needs a new contract, use the exact merged contract or an explicitly pinned preview.

## 12. Generated and derived files

Do not hand-edit generated output unless the repository convention explicitly requires it.

Relevant generated/derived surfaces include:

- the canonical Diesel schema at `thoth-api/src/schema.rs`;
- the GraphQL schema generated by `thoth-client/build.rs`;
- GraphQL client types generated in downstream repositories;
- API/OpenAPI output derived from export-server route definitions.

Record the generation command and resulting diff.

### Diesel schema generation (THOTH-DB-CTRL-01)

The Diesel schema-generation procedure is implemented and authoritative. All
commands run from the repository root, which is the only supported working
directory.

- The Diesel CLI must be exactly `2.3.10` (PostgreSQL feature), matching the
  locked Diesel crate, supplied through `DIESEL_BIN`; no global installation is
  assumed.
- Automatic Diesel output is untrusted staging only, written to
  `target/diesel-schema.rs` under the ignored `target/` directory. It is never
  the canonical contract and must never be promoted.
- `thoth-api/src/schema.rs` is the canonical, compiled contract. Direct manual
  replacement, or writing it with any direct `diesel` command, is prohibited.
- Only the validated synchronizer `.github/scripts/diesel_schema.py generate`
  may write the canonical schema, and only after every safety, exact projection,
  deterministic-repeat, focused-compile, and cleanup check passes.
- Every schema task supplies a complete version-2 expected-change manifest whose
  `expected_projection` is explicitly `change` or `none`. A clean run with no
  pending controlled change uses `none` and must leave the schema byte-identical.
- `make check-diesel-schema` is mandatory before any schema-affecting change and
  runs the same synchronizer locally and in CI. A control failure blocks the
  dependent schema work; it must not be weakened to obtain green CI.
- The `none` result certifies only the Diesel-controlled projection; it does not
  validate excluded migration effects (indexes, check constraints, data,
  comments), which remain the responsibility of migration validation.

CG-13 (Diesel runtime operations) remains a separate open control and is not
resolved by this procedure.

## 13. Changelog

Every PR must update `CHANGELOG.md` under `## [Unreleased]`.

Use the appropriate heading:

- `### Added`
- `### Changed`
- `### Fixed`
- `### Deprecated`
- `### Removed`
- `### Security`

Reference the PR number when available. Do not create duplicate headings in the same Unreleased section.

## 14. Implementation report

Before review, complete the structure in:

```text
docs/engineering/ai-delivery/implementation-report-template.md
```

Include:

- exact base and head commits;
- actual files changed;
- migrations and operational effects;
- authorization effects;
- exact test commands/results;
- CI status;
- rollout and rollback;
- known limitations and deferred work.

The implementing agent may provide a self-assessment but may not issue the approval decision.

## 15. Stop conditions

Stop and return `BLOCKED` when:

- the approved specification is absent;
- the requested base branch does not exist;
- implementation conflicts with an approved design;
- a required migration or authorization path cannot be tested;
- production credentials or destructive production access would be required;
- a downstream contract is unavailable and would need to be guessed;
- repository state differs materially from the task premise;
- a cross-programme decision is required.
