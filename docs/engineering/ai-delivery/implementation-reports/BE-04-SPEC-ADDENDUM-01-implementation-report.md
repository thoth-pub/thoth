# BE-04-SPEC-ADDENDUM-01 Implementation Report

## 1. Repository state

```text
Programme:                Publisher Services and Distribution Configuration
Repository:               thoth-pub/thoth
Task ID:                  BE-04-SPEC-ADDENDUM-01
Approved specification:   docs/engineering/ai-delivery/tasks/BE-04.md (the document
                          this addendum corrects), read in full before any edit
Risk:                     HIGH
Base branch and commit:   develop at ed32712766c8f5a1951bb53ec3192e18f067c7d2
PR target:                develop
Task branch:              feature/publisher-services/be-04-spec-addendum-01
Dependencies:             ADR-0003, ADR-0005, ADR-0007, ADR-0008 (all APPROVED and
                          repository-authoritative); BE-02, BE-03 (CLOSED)
Implementing agent/model: Claude Opus 5, extra-high reasoning
Independent reviewer:     required and absent - a fresh independent strong model and
                          context that did not author or materially assist with BE-04
                          or with this addendum
```

Preflight, performed before any edit and re-verified against the live remote:

| Check | Required | Observed |
|---|---|---|
| `origin/develop` | `ed32712766c8f5a1951bb53ec3192e18f067c7d2` | `ed32712766c8f5a1951bb53ec3192e18f067c7d2` |
| PR [#816](https://github.com/thoth-pub/thoth/pull/816) state | `OPEN` | `OPEN` |
| PR #816 draft | `true` | `true` |
| PR #816 merged | never | `mergedAt: null` |
| PR #816 head | `6356ac1c7fc8d53001a93b378d92dc0368a77405` | `6356ac1c7fc8d53001a93b378d92dc0368a77405` |
| `origin/feature/publisher-services/be-04` | equal to the PR head, no later commit | `6356ac1c7fc8d53001a93b378d92dc0368a77405` |
| PR #816 commit list | six commits ending at `6356ac1c` | six commits ending at `6356ac1c` |
| Merge base of #816 and develop | `ed32712766c8f5a1951bb53ec3192e18f067c7d2` | identical, so #816 branches directly from the authorized base |
| Working tree | clean | clean |

The branch was created from the exact develop SHA, **not** from PR #816.

Documents read in full before any edit: [`AGENTS.md`](../../../../AGENTS.md),
[`thoth-api/AGENTS.md`](../../../../thoth-api/AGENTS.md),
[`docs/engineering/AGENTS.md`](../../AGENTS.md),
[`ADR-0007`](../../decisions/ADR-0007-conventional-request-scoped-graphql-dataloader.md),
[`ADR-0008`](../../decisions/ADR-0008-machine-roles-and-durable-job-primitives.md)
and [`tasks/BE-04.md`](../tasks/BE-04.md) (4,574 lines at the base).

PR #816's code, migration, loaders and implementation report were inspected as
**evidence only**. Neither that branch nor its pull request was modified,
rebased, pushed to, commented on or closed. PR
[#799](https://github.com/thoth-pub/thoth/pull/799) was not touched.

## 2. Scope confirmation

In scope and delivered: correction of three specification defects, the addendum
record, the programme tracker and the changelog.

Out of scope and not done: any change under `src/`, `thoth-api/`,
`thoth-api-server/`, `thoth-client/`, `thoth-errors/`, `migrations/`,
`.github/`, `Cargo.toml`, `Cargo.lock` or `schema.rs`; any change to PR #816 or
its branch; any repair of the `thoth-client` dependency/feature architecture; any
ADR creation or amendment; any approval or authorization.

## 3. Commits

Ordinary additive documentation commits on
`feature/publisher-services/be-04-spec-addendum-01`. No amend, no rebase, no
squash, no force push. Exact SHAs are the GitHub pull-request record.

## 4. Files changed

| File | Change |
|---|---|
| `docs/engineering/ai-delivery/tasks/BE-04.md` | the three corrections and the new section 34 addendum record |
| `docs/publisher-services/task-status.md` | BE-04 state, dependencies, PR column and narrative items 14-15 |
| `CHANGELOG.md` | one `### Changed` entry under `## [Unreleased]` |
| `docs/engineering/ai-delivery/implementation-reports/BE-04-SPEC-ADDENDUM-01-implementation-report.md` | this report |

Nothing else changed. `git status` shows no other modified, added or deleted
path.

## 5. Implementation decisions

### 5.1 Finding A - the attempt-error `CHECK` was not NULL-safe

The specified and implemented expression was:

```sql
CHECK ((error_code IS NULL AND error_detail IS NULL) OR result = 'FAILED')
```

PostgreSQL rejects a row only when a `CHECK` evaluates to `FALSE`; an `UNKNOWN`
result is accepted. On an open attempt — `finished_at IS NULL`, and therefore
`result IS NULL` under `distribution_job_attempt_closure_check` — the first arm
is `FALSE` and `result = 'FAILED'` is `NULL`, so the whole expression is `NULL`
and the row is admitted with error fields set.

**Demonstrated, not asserted.** Against a disposable PostgreSQL 17.10 the whole
specified attempt-table constraint set was created (every named `CHECK`, the two
unique constraints; only the foreign key was omitted, there being no parent table
in the fixture) and this row was inserted successfully:

```text
attempt_number | finished_at | result | error_code        | error_detail
             1 |             |        | TRANSPORT_FAILURE | remote host refused
```

The same fixture evaluated both expressions directly. With `result = NULL` and
`error_code` set: arm 1 is `FALSE`, `result = 'FAILED'` is `NULL`, the current
expression is `NULL` (accept), and `result IS NOT NULL AND result = 'FAILED'` is
`FALSE`, making the corrected expression `FALSE` (reject). Row-by-row insert
tests against two tables carrying the two expressions produced:

| Case | current | corrected |
|---|---|---|
| open attempt, `error_code` only | **accepted** | rejected |
| open attempt, both error fields | **accepted** | rejected |
| `SUCCEEDED` + error fields | rejected | rejected |
| `ABANDONED` + error fields | rejected | rejected |
| `CANCELLED` + error fields | rejected | rejected |
| open attempt, both error fields NULL | accepted | accepted |
| `FAILED` + valid error fields | accepted | accepted |
| `FAILED`, no error fields | accepted | accepted |

The correction adds `result IS NOT NULL AND` to the second arm, which converts
exactly the `NULL` rows to `FALSE` and changes no row the constraint already
decided. The state machine of section 11.2 is untouched. An equivalent expression
(`result = 'FAILED' IS TRUE`, `COALESCE(result = 'FAILED', FALSE)`) is permitted
only with the same truth table demonstrated against the target PostgreSQL
version.

Sections changed: 7.4 (DDL plus the truth table), 25.1 (acceptance criterion),
25.4 (required rejections and acceptances on `INSERT` and `UPDATE`), 18.2
(wording), 26 item 2 (catalog expression and observed truth-table evidence).

### 5.2 Finding B - the exact report statement-count contract was unsatisfiable

The specification required three DataLoaders — latest job keyed by
`publisher_id`, targets keyed by `distribution_job_id`, attempts keyed by
`distribution_job_id` — and a fixed five/six statements at every supported page
size. PR #816 measured 5 / 7 / 11 (job-only) and 6 / 8 / 6 (full report) at page
sizes 1 / 25 / 200, with the two second-level loaders dispatching in four chunks
each at page size 200 in one run and one chunk each in another.

**Route B (keep the nested structure, prove a hard bound) is rejected**, on
source evidence rather than on the measurement:

- `dataloader` 0.18.0 `non_cached::Loader::try_load` inserts its key under the
  state lock, then either dispatches immediately when
  `pending.len() >= max_batch_size` or awaits the yield budget and **drains the
  entire pending set**. A dispatch therefore carries whatever is pending when the
  first waiter's budget expires, and nothing in the crate bounds the number of
  dispatch opportunities.
- Juniper 0.16.2 resolves list items (`types/containers.rs`) and object selection
  sets (`types/async_await.rs`) through `FuturesOrdered` inside one task, so a
  cohort whose keys are available at resolver entry registers together — which is
  why the first-level latest-job loader dispatched in exactly one chunk at every
  measured page size, including 200.
- A second-level cohort has no such property. Its key exists only after the
  upstream loader resolves, and the upstream batch function holds the loader
  state lock across its database round trip, so parents resume through a
  serialized hand-off and child keys arrive spread over scheduler time. The
  observed leading chunk of eleven is the signature of a ten-yield budget
  elapsing while parents resume one at a time.

For a dependent-arrival cohort the only bound provable from the pinned library is
`ceil(N / max_batch_size) <= dispatches <= N`. The upper bound is linear in N, so
no hard N-independent constant exists. ADR-0007 reaches the same conclusion for
the general case in sections 4.5, 4.6 and 10.3 and in decision driver 11: the
specification was demanding a guarantee the approved shared architecture
explicitly declines to make.

**Route A is selected and specified normatively.** `latestBackCatalogueJob` is
backed by one field-specific, first-level, request-local loader keyed by
`publisher_id` whose value is the complete field value — the latest job with its
targets and attempts — so the whole field becomes one loader-first cohort. Its
batch function issues, inside one `spawn_blocking` boundary on one connection
acquired and dropped inside the closure:

| # | Statement |
|---|---|
| L1 | `SELECT DISTINCT ON (publisher_id) ... FROM distribution_job WHERE publisher_id = ANY($1) AND kind = 'PUBLISHER_BACK_CATALOGUE' ORDER BY publisher_id, created_at DESC, distribution_job_id DESC` |
| L2 | `SELECT ... FROM distribution_job_target WHERE distribution_job_id = ANY($2) ORDER BY distribution_job_id, platform` |
| L3 | `SELECT ... FROM distribution_job_attempt WHERE distribution_job_id = ANY($2) ORDER BY distribution_job_id, attempt_number DESC` |

with `$2` the job ids L1 returned; L2 and L3 are skipped only when L1 returns
none. The deterministic arithmetic is:

```text
statements = 2
           + (3 if the chunk resolves a job else 1) * C_job
           + 1 * C_assign   (only when enabledDistributionPlatforms is selected)
```

which evaluates to **five** (job-only) and **six** (full report) at `C = 1`.

The distinction the corrected section draws, and which the previous revision did
not, is between what BE-04 owns and what it consumes. Structural and provable:
two root statements; three statements per composite-loader chunk resolving a job,
one otherwise; one per assignment-loader chunk; every statement set-based; exact
selection dependence; and **zero** dispatches of the target and attempt loaders
on the report path. Not BE-04's property: the value of `C`, which is the shared
ADR-0007 chunking property with ADR-0007 section 4.6's stated bound
`ceil(N / 200)` and its section 10.2 evidence. Section 25.12 therefore measures
`C` per loader and asserts the **derived** total, and a `C_job > 1` at a page
size at or below 200 is escalated as an ADR-0007 finding (new stop condition 23)
rather than absorbed by relaxing the expectation — because the same deviation
would apply equally to BE-02's already-merged assignment loader.

Explicitly not done, each being outside this task's authority: no change to
ADR-0007's `200`/`10`; no look-ahead-driven load shaping, which would reintroduce
the ADR-0006 mechanism ADR-0007 retired; no request-scoped result store; no merge
of unrelated loaders to make a count fit. The composite loader is not such a
merge: targets and attempts of the latest job are sub-structure of the one value
the field returns, which is ADR-0007 section 4.4's "one reviewed logical
field/query family". Its accepted cost — materializing targets and attempts even
when the query selects neither — is stated in section 17.4.2 and recorded as a
known limitation.

The two second-level loaders may be retained **only** for the single-job mutation
payloads, where the cohort is one job; the worker claim path is unchanged and
uses no loaders.

Sections changed: 3 item 9, 7.5, 12.3 (wording), 17.4 (rewritten as 17.4.1-17.4.4),
25.1, 25.12, 26 item 16, 27 item 23.

### 5.3 Finding C - a mandatory command that fails at the authorized base

Verified independently at the authorized base, on a branch identical to
`develop`, rather than taken from PR #816's report:

```text
$ cargo test -p thoth-client
error[E0433]: cannot find `graphql` in `crate`   (x20)
error[E0432]: unresolved import `crate::schema`
error[E0433]: cannot find `db` in `crate`
error: use of unresolved module or unlinked crate `diesel`   (x2)
error: could not compile `thoth-api` (lib) due to 26 previous errors; 1 warning emitted
```

Every error is annotated by the compiler as "the item is gated behind the
`backend` feature". The merged cause:

- `thoth-client/Cargo.toml` depends on `thoth-api` **without** features under
  `[dependencies]` and **with** `features = ["backend"]` under
  `[build-dependencies]`;
- `thoth-api/src/lib.rs` gates `graphql`, `db`, `schema`, `storage`, `redis` and
  `policy` behind `backend`, and `thoth-api` does not compile with it off;
- `thoth-client/build.rs` imports `thoth_api::graphql::create_schema`, which is
  why the build-dependency edge enables `backend`;
- under Cargo's v2 feature resolution a single-package build resolves features
  for that package alone, so the normal-dependency copy is built without
  `backend` and the build fails inside `thoth-api` before `thoth-client`'s own
  code is reached; a workspace build unifies those features with the root
  package's `features = ["backend"]` edge and succeeds.

The repository's own control is the workspace run:
`.github/workflows/build_test_and_check.yml` runs `cargo build` and
`cargo test --workspace`, and requires no single-package `thoth-client`
execution. No repository control is therefore waived by removing the standalone
command, and stop condition 24's "identify the prerequisite repair task" branch
is not reached.

The command is removed from section 25.13 with the reason stated; section 20.2
item 5 records the merged evidence and requires the workspace path plus an
explicit demonstration that `thoth-client`'s tests **executed** within each
workspace run — a count of executed tests, not a green workspace summary;
sections 25.12 and 26 item 15 follow; section 26 item 19 records the standalone
failure as pre-existing repository packaging/test-mode debt for a separate task.
It is not described as `PASS`, and the unrelated dependency architecture is not
repaired here (non-goal 22).

### 5.4 Statements corrected because the repository disproves them

Two statements elsewhere in BE-04.md were falsified by observed repository state
and were corrected rather than left standing, as
[`docs/engineering/AGENTS.md`](../../AGENTS.md) section 1 requires: section 31's
"`feature/publisher-services/be-04` does not exist and must not exist", and the
tracker's equivalent sentence. Both now record the branch and draft pull request
as observed state that is explicitly not an approval, an authorization or a
delivery.

## 6. Database and migration effects

None. This task creates, alters and drops nothing: no migration directory, no
table, no enum, no constraint, no index and no row. `thoth-api/src/schema.rs` is
untouched. The corrected DDL in section 7.4 is **specification text**; no
migration implementing it exists on `develop`, and PR #816's unmerged migration
was not edited. The disposable PostgreSQL 17.10 used for the Finding A
demonstration ran in a throwaway database created and dropped for that purpose;
it was never pointed at production or at any shared service, and no thoth
migration was applied.

## 7. API and compatibility effects

None. No GraphQL schema, SDL, resolver, input, enum or generated client artefact
changes. `thoth-client/assets/queries.graphql` is unchanged and no SDL was
regenerated by this task.

## 8. Authorization and security

None. No change to `thoth-api/src/policy.rs`, to any role, to any authorization
matrix or to any identity-provider configuration. No role was created, granted or
revoked, and no credential was read, created or rotated. No secret, token or
production URL appears in any changed file.

## 9. Tests and checks

Documentation-only change; the repository requires `git diff --check` plus the
link, terminology, stale-source and changelog verifications of
[`AGENTS.md`](../../../../AGENTS.md) section 8.

| Check | Result |
|---|---|
| `git diff --check` | pass, no whitespace errors |
| relative Markdown links in changed files resolve | pass, verified by resolving each link target against the working tree |
| consistency search over the required terms | pass, see below |
| `CHANGELOG.md` entry under `## [Unreleased]` `### Changed` | present, no duplicate heading created |
| no source, migration, manifest, workflow or generated file touched | pass, verified by `git status` and `git diff --name-only` |

Evidence gathered rather than checks of this change:

| Command | Purpose | Result |
|---|---|---|
| `cargo test -p thoth-client` at `ed32712` | Finding C verification at the authorized base | fails, 26 errors in `thoth-api` (lib) |
| `psql` truth-table and constraint-set fixtures on disposable PostgreSQL 17.10 | Finding A demonstration | loophole reproduced; corrected expression rejects it |

No workspace test run was performed, and none is claimed: this change compiles
nothing. The full workspace gate remains BE-04 **implementation**'s obligation
under section 25.13, unchanged except as Finding C corrects it.

Consistency search, over the terms the addendum task fixed:

| Term | Finding |
|---|---|
| `error_result_check` | 7 occurrences, all consistent with the NULL-safe form |
| `result = 'FAILED'` | remaining occurrences are the state-machine transitions T3/T4 and the corrected DDL/truth table |
| "error fields on a non-`FAILED`" | the section 25.4 bullet is replaced by the explicit reject/accept lists; the section 11.2 reference remains and is now true |
| "five statements" / "six statements" | only inside the withdrawn-contract narrative of 17.4.1 and section 34.3, both explicitly historical |
| `200` | ADR-0007's configured batch size and the report page size, never restated as a BE-04 guarantee |
| "yield" | the loader yield budget in 17.4.1 and pre-existing unrelated uses |
| "upstream loader" | 17.4.1 only, describing the rejected shape |
| `cargo test -p thoth-client` | 4 occurrences, every one stating it is **not** a gate and does not pass |
| "IMPLEMENTATION DELIVERED" | absent from the repository |
| "READY" | no occurrence describes BE-04 as ready |
| "BLOCKED" | BE-04 and its candidate are `BLOCKED` in the tracker, the spec and this report |

## 10. Manual verification

- PR #816's migration was read and confirmed to carry the defective `CHECK`
  verbatim, so Finding A is a real property of the candidate and not only of the
  specification text.
- PR #816's `dataloader.rs`, `model.rs` resolvers and implementation report were
  read; the report's own section 11.1 records the divergence and refers it to the
  reviewer and the CTO, which this addendum answers.
- `DistributionJobPayload` already carries `preloaded_targets`/`preloaded_attempts`,
  used by the worker claim path, so the corrected design has an existing seam and
  does not require a new payload concept.
- The pinned `dataloader` 0.18.0 and `juniper` 0.16.2 sources were read from the
  local registry at the versions `Cargo.lock` pins.

## 11. CI

Repository CI runs on the pull request. This change touches only Markdown, so the
build, test and clippy jobs are unaffected by its content; the changelog check
requires the `## [Unreleased]` entry, which is present.

## 12. Rollout and rollback

No rollout. The addendum changes requirements, not behaviour: nothing is
deployed, migrated, activated or exposed. Rollback is the ordinary revert of a
documentation change, which would restore the three defective requirements and is
therefore not recommended without a replacement correction.

## 13. Known limitations and deferred work

1. **The corrected batching contract is unimplemented.** It specifies a design;
   no code implements it, and PR #816's three-loader implementation does not
   satisfy it.
2. **No library-guaranteed constant exists, and none is claimed.** Even a
   loader-first cohort's chunk count is a property of the shared ADR-0007
   foundation rather than of the pinned crate's contract. If the CTO requires a
   count guaranteed by the library rather than by cohort shape, that is an
   ADR-0007 amendment — for example a different dispatch trigger through
   `with_custom_wait_for_work`, or an explicit per-request cohort barrier — and it
   is outside this task's authority. It is named here as an available decision,
   not requested, and the selected route does not need it.
3. **The `thoth-client` packaging gap is unrepaired**, deliberately, and is
   recorded as residual debt for a separate task.
4. **PR #816 is not reconciled.** Bringing it into line with the corrected
   specification is later, separately authorized work.
5. **The specification remains unapproved.** This addendum corrects a candidate;
   it does not approve one.

## 14. Unresolved issues

1. Whether the CTO accepts Route A's cost — the composite loader materializing
   targets and attempts even when the query selects neither — in exchange for a
   contract that is true. The alternative (selection-driven load shaping) was
   rejected here as an ADR-0006 reintroduction, and that judgement is the
   reviewer's and the CTO's to confirm.
2. Whether the residual ADR-0007 chunking property should be strengthened
   repository-wide, per limitation 2. This addendum takes no position beyond
   naming it.
3. Whether `BE-04.md` should also bound the report's `limit` argument. A caller
   may currently request a page larger than the configured maximum batch size, in
   which case `C > 1` legitimately and the arithmetic scales as stated; clamping
   would change BE-03's merged behaviour and is therefore outside this task.

## 15. Agent self-assessment

The three findings are each evidenced against merged repository state rather than
against narrative: Finding A was reproduced against the full specified constraint
set on PostgreSQL 17.10, Finding C was reproduced by running the command at the
authorized base, and Finding B's mechanism was derived from the pinned
`dataloader` and `juniper` sources and corroborated by the candidate's own
measurements.

Suggested review focus, in order:

1. **Finding B's route selection.** Is the composite loader genuinely ADR-0007
   section 4.4's "one logical field/query family", or is it the loader merge the
   addendum task forbids? The report's position is that it is the former, because
   the merged data is the one field's own sub-structure and no unrelated family
   is involved — but this is the judgement most worth challenging.
2. **Whether the corrected arithmetic is a control or a description.** Section
   17.4.3 separates structural properties from the shared ADR-0007 property; a
   reviewer should test whether that separation is honest or whether it relocates
   an unprovable claim rather than removing it.
3. **Finding A's truth table**, row by row, including whether any other `CHECK`
   in section 7 has the same three-valued exposure. The addendum inspected the
   others and found none, since each compares a `NOT NULL` column or is already
   guarded, but that inspection deserves independent repetition.
4. **The Finding C resolution**, specifically whether removing a mandatory
   command is a waiver. The report's position is that the repository's own
   control is `cargo test --workspace` and that the removed command was never one
   of the repository's controls.

No approval decision is issued or implied by this report.
