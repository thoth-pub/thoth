# AGENTS.md

These instructions apply to the complete `thoth-pub/thoth` repository.

A more deeply nested `AGENTS.md` adds or narrows instructions for its directory. Read this file and every applicable nested file before editing.

## 1. Required task identity

Before changing anything, record:

```text
Programme:
Owning GitHub issue:
Repository: thoth-pub/thoth
Task ID:
Approved specification:
Risk: LOW | MEDIUM | HIGH | CRITICAL
Base branch and exact base commit:
PR target:
Task branch:
Dependencies:
Authorized write paths (existing files):
Authorized new-file paths:
Prohibited paths:
Action authorization: see section 6
Cross-repository impact: see section 6.1
Implementing agent/model:
Independent reviewer/model:
```

Do not implement without an approved written specification. A GitHub issue is sufficient only when it contains the information required by `docs/engineering/ai-delivery/task-specification-template.md`, including an explicit write budget and action-authorization matrix.

If any item is unknown, treat it as missing work.

GitHub is the live task ledger for this repository: the owning issue, its linked pull request, review threads and CI hold current lifecycle state. Committed documentation records durable architecture and doctrine, not the day-to-day status of an individual task. See `docs/engineering/AGENTS.md` section 1.1 and `ADR-0005`.

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

Branch naming is workflow-specific. `STANDARD` tasks and `PROGRAMME_INTEGRATION`
slices do not share a naming form. See
[`ADR-0009`](docs/engineering/decisions/ADR-0009-programme-integration-branch-namespace.md).

`STANDARD` work:

```text
develop -> feature/<area>/<task> -> develop
```

Approved large programme work (`PROGRAMME_INTEGRATION`):

```text
develop
  -> feature/<programme>
  -> feature/<programme>--<slice>
  -> feature/<programme>
  -> develop
```

A programme slice branch is a **sibling** of its integration branch, not a
descendant of it. `feature/<programme>/<slice>` is not usable while
`feature/<programme>` exists as a branch, because Git cannot hold a ref and a
ref namespace at the same path.

`--` is the reserved programme/slice separator. Governed `<programme>`,
`<area>`, `<slice>` and `<task>` identifiers must each be non-empty, must each
be a single Git path segment, and must not themselves contain `--`.

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

### 5.1 Fail-closed namespace preflight

Before creating any new governed ref, verify against live remote refs, not
against assumption. The check is symmetric.

Before creating a new governed **flat** ref such as `feature/<programme>`:

- the exact ref does not already exist;
- no descendant ref beneath that path already exists, because such a descendant
  requires the flat ref's location to be a ref namespace.

Before creating or using a governed **descendant** ref such as
`feature/<area>/<task>`:

- no flat parent ref `feature/<area>` already occupies that location.

Before creating a programme slice `feature/<programme>--<slice>`:

- the exact ref is absent;
- the identifiers satisfy the reserved-token rule above;
- no incompatible descendant occupancy exists beneath that prospective flat ref.

If preflight fails, HOLD. Never resolve a namespace collision by deleting,
renaming or moving another branch. An active task whose authorized branch name
is invalid or superseded must HOLD and receive a task-specific specification
amendment plus fresh review and authorization before proceeding.

## 6. Granular action authorization

Authorization is granted action-by-action and is **not transitive**. Authorization
for one action never implies authorization for another. A task's specification or
implementation-handoff prompt (see
`docs/engineering/ai-delivery/implementation-handoff-template.md`) must state
exactly which of the following actions are authorized for that task. Any action
not explicitly authorized is denied by default.

Distinct actions:

- repository/GitHub read inspection;
- source/worktree modification within the approved write budget;
- creation of new files at explicitly authorized paths;
- deletion, move or rename of files;
- branch creation;
- commit;
- push;
- pull-request creation/update;
- issue/comment mutation;
- manual CI dispatch or rerun;
- provider/runtime read;
- provider/runtime write;
- migration execution;
- release/tag/publication;
- merge;
- deployment;
- production activation.

Without limiting the list above:

- source-write authorization does not include commit authorization;
- commit authorization does not include push authorization;
- push authorization does not include pull-request mutation authorization;
- repository-write authorization does not include GitHub issue/comment mutation authorization;
- merge authorization does not include deployment authorization;
- deployment authorization does not include production-activation authorization;
- provider-read authorization does not include provider-write authorization.

A typical bounded documentation or implementation task authorizes: repository
inspection; source edits within the stated write budget; creation of the
specifically listed new files; branch creation from the stated exact base;
local validation; commit; push; and opening or updating a draft pull request
targeting the stated PR target. It does not authorize file deletion, move or
rename; manual CI dispatch or rerun; provider or runtime reads or writes;
migration execution; release, tag or publication actions; merge; deployment;
or production activation — unless the task specification explicitly lists
them.

An implementing agent must not:

- merge a PR;
- publish a release;
- deploy or activate production behaviour;
- access production secrets;
- run commands against production databases or services;
- dispatch write-capable production workflows;
- perform destructive production operations;
- broaden scope, write budget or action authorization without an approved
  specification update;
- change approved architecture silently;
- treat read/inspection authorization as edit authorization, or edit
  authorization as commit, push or pull-request authorization.

### 6.1 Cross-repository impact

Before substantive scope affecting a shared contract is approved — database or
domain model, GraphQL/API schema and behaviour, generated clients/types,
authorization semantics, export formats, configuration/environment contracts,
event/job payloads, dissemination or platform behaviour, UI assumptions,
CMS/site contracts, package/library interfaces, or deployment/compatibility
windows — identify the owning repository and known consumers from
`docs/engineering/repository-map/contracts.md`, and record whether each known
consumer requires a change or remains compatible and why. Do not treat a task
as single-repository merely because it originated in one repository.

Never give one implementing agent unrestricted write access to more than one
repository for the same task. Each affected repository gets its own bounded
task, branch and pull request, independently reviewed. A downstream repository
must never guess an unmerged upstream contract; it waits for the upstream
change to merge, or consumes an explicitly pinned preview.

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

## 12. Schema contract, generated and derived files

### 12.1 Repository-authoritative Diesel schema contract

Per [ADR-0003](docs/engineering/decisions/ADR-0003-repository-authoritative-schema-contract.md),
`thoth-api/src/schema.rs` is the repository-authoritative, manually maintained Rust/Diesel compile-time schema contract. It is not regenerated by a Diesel CLI workflow.

Work from the repository root. Create migrations with the supported repository procedure (`make migration`) and apply or revert them with the embedded Rust migration runner (`cargo run migrate`, `cargo run migrate --revert`).

A task that changes the Diesel-representable database contract must update, atomically in the same bounded PR:

- migration `up.sql` and `down.sql`;
- `thoth-api/src/schema.rs`, maintained directly;
- affected Rust models;
- affected query or GraphQL code where applicable;
- focused database and model tests.

When a migration has no `thoth-api/src/schema.rs` impact (for example a data-only, index-only, or check-constraint migration outside the checked-in Diesel table contract), state that explicitly as a reviewed conclusion rather than omitting it.

Do not use `diesel print-schema` as the canonical writer for `thoth-api/src/schema.rs`. External introspection tools may be used diagnostically, but their output is untrusted and must never write directly to the canonical schema. Do not introduce a Diesel CLI (`diesel_cli`) dependency, a root `diesel.toml`, or a schema-synchronization subsystem without a separately approved ADR that supersedes ADR-0003. Removing the Diesel CLI configuration does not remove the Diesel Rust crates or the embedded `diesel_migrations` runner, which remain in use.

Production migration execution and rollback remain governed by [CG-13](docs/engineering/repository-map/control-gaps.md) and separate release authorization.

### 12.2 Other generated and derived files

Do not hand-edit generated output unless the repository convention explicitly requires it.

Relevant generated/derived surfaces include:

- the GraphQL schema generated by `thoth-client/build.rs`;
- GraphQL client types generated in downstream repositories;
- API/OpenAPI output derived from export-server route definitions.

Record the generation command and resulting diff.

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
