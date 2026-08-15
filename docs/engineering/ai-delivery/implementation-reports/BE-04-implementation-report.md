# BE-04 Implementation Report

Programme: Publisher Services and Distribution Configuration
Repository: `thoth-pub/thoth`
Task ID: BE-04 — Durable distribution jobs
Approved specification: [`docs/engineering/ai-delivery/tasks/BE-04.md`](../tasks/BE-04.md),
repository-authoritative in its corrected form — baseline through PR
[#814](https://github.com/thoth-pub/thoth/pull/814),
`BE-04-SPEC-ADDENDUM-01` through PR
[#817](https://github.com/thoth-pub/thoth/pull/817)
Risk: HIGH
Workflow: STANDARD
Implementing agent/model: Claude Opus 5, reasoning level Extra High
Independent reviewer/model: required, and **not** this agent

Authority condition: this report records what was implemented and measured. It
makes no approval decision. Live review, merge-authorization and merge evidence
is the GitHub pull-request record (`ADR-0005`).

This report covers **two** authorized episodes on one branch, and neither
supersedes the other as history:

1. the original bounded implementation, authorized against the **baseline**
   specification at `develop @ ed32712766…`;
2. the **implementation reconciliation** against the **corrected** specification
   at `develop @ 8c0c54bd…`, which is what sections marked *reconciliation*
   record.

Where the corrected contract changed a requirement, this report states the
corrected result. It does not rewrite the original episode out of the record.

---

## 1. Repository state

### 1.0 Reconciliation against the corrected specification

| Item | Value |
|---|---|
| Corrected authorized base | `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686` |
| What that SHA is | the merge commit of specification-addendum PR [#817](https://github.com/thoth-pub/thoth/pull/817), verified as `origin/develop` before any reconciliation edit |
| Reconciliation authorization | PR #816 comment [5301898691](https://github.com/thoth-pub/thoth/pull/816#issuecomment-5301898691), by `ja573` on 2026-08-15, explicitly bound to `develop @ 8c0c54bd7b2e58a645ffe39abd8ceeee86e47686` and to the corrected contract |
| Branch head before reconciliation | `6356ac1c7fc8d53001a93b378d92dc0368a77405` |
| Base incorporation | ordinary `git merge --no-ff` of that exact SHA; no rebase, amend, squash or force-push |
| Merge commit | `f4cb9dafa1d2112424742fdfa128c4a1e9c685db` |
| Branch | `feature/publisher-services/be-04`, unchanged |
| PR | [#816](https://github.com/thoth-pub/thoth/pull/816), unchanged, target `develop` |

Reconciliation preflight, performed before any edit:

- `git fetch origin --prune`;
- `git rev-parse origin/develop` = `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686`,
  matching the authorized corrected base exactly;
- `gh pr view 817` reports `state: MERGED`,
  `mergeCommit.oid: 8c0c54bd7b2e58a645ffe39abd8ceeee86e47686`,
  `baseRefName: develop`;
- `git rev-parse origin/feature/publisher-services/be-04` =
  `6356ac1c7fc8d53001a93b378d92dc0368a77405`, unmoved;
- `gh pr view 816` reports `state: OPEN`, `isDraft: true`, `mergedAt: null`,
  `headRefOid: 6356ac1c7fc8d53001a93b378d92dc0368a77405`,
  `baseRefName: develop`;
- the CTO reconciliation authorization exists on PR #816 as comment
  `5301898691` and names that exact base SHA;
- the working tree was clean (`git status --porcelain` empty).

After the merge, `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686` is an ancestor of the
branch head (`git merge-base --is-ancestor`) and
`docs/engineering/ai-delivery/tasks/BE-04.md` on the branch is byte-identical to
its content on `develop`. **No specification content was edited.**

One conflict arose, in `docs/publisher-services/task-status.md`. The branch side
still described BE-04 as an implementation candidate delivered against the
pre-addendum contract, which the corrected specification contradicts; the
`develop` side is repository-authoritative and was taken whole, and the tracker
is then reconciled to its durable post-reconciliation form by a later commit on
this branch. `CHANGELOG.md` merged cleanly, preserving PR #817's entry.

### 1.1 Original implementation base and authorization

| Item | Value |
|---|---|
| Authorized implementation base | `ed32712766c8f5a1951bb53ec3192e18f067c7d2` |
| What that SHA is | the merge commit of specification PR [#814](https://github.com/thoth-pub/thoth/pull/814), verified as `origin/develop` before any edit |
| Implementation authorization | PR #814 comment [5296197259](https://github.com/thoth-pub/thoth/pull/814#issuecomment-5296197259), by `ja573` on 2026-08-14, explicitly bound to `develop @ ed32712766c8f5a1951bb53ec3192e18f067c7d2` |
| Branch | `feature/publisher-services/be-04`, created from that exact SHA |
| PR target | `develop` |

That authorization remains valid history. It is **insufficient** for the
corrected contract, which is why the reconciliation above carries its own.

### 1.2 Preflight of the original episode, performed before any edit

- `git fetch origin --prune`;
- `git rev-parse origin/develop` = `ed32712766c8f5a1951bb53ec3192e18f067c7d2`, matching the
  authorized base exactly;
- that commit is the merge commit of PR #814 (`gh pr view 814` reports
  `state: MERGED`, `mergeCommit.oid: ed32712766c8f5a1951bb53ec3192e18f067c7d2`,
  `baseRefName: develop`);
- `docs/engineering/ai-delivery/tasks/BE-04.md` is reachable at that commit
  (261 802 bytes, 4 574 lines);
- the CTO implementation authorization exists on PR #814 as comment
  `5296197259` and names that exact SHA;
- `feature/publisher-services/be-04` did **not** exist locally
  (`refs/heads/...` absent) or on `origin` (`refs/remotes/origin/...` absent);
- no BE-04 implementation pull request existed. The open pull requests at
  preflight were #799, #752, #744, #742 and #668, none of them BE-04;
- the working tree was clean (`git status --porcelain` empty).

PR [#799](https://github.com/thoth-pub/thoth/pull/799) was not touched, dispatched,
rebased or referenced by any change in this branch.

---

## 2. Scope confirmation

Everything implemented is inside the approved specification. Nothing in the
non-goals list was implemented: there is no dissemination, no uploader
execution, no external platform call, no file, feed, message or deposit, no
work-level distribution choice, no work-upsert or withdrawal job, no delivery
fingerprint, no remote identifier, no observed-delivery state, no scheduled
reconciliation, no generic job framework, no generic `Job`/`Queue`/`Lease`/
`Worker` abstraction, no shared cross-programme job machinery, no Metrics job
machinery, no generic machine role, no role registry, no machine-identity
storage, no credential storage, no token issuance, no workflow change, no
mutation-guard change, no deployment, no production migration execution and no
production access. `thoth-app` is not modified and is not a member of this
workspace. DIS-02 is not implemented.

---

## 3. Commits

Original implementation episode:

| SHA | Subject |
|---|---|
| `c896c306` | `feat(publisher-services): add BE-04 durable distribution job schema` |
| `79203042` | `feat(publisher-services): create BE-04 jobs inside the BE-03 transaction` |
| `430d8d14` | `feat(publisher-services): add BE-04 worker API, role and staff-report fields` |
| `19e618f7` | `test(publisher-services): add BE-04 database, concurrency and contract evidence` |
| `7163b432` | `docs(publisher-services): record BE-04 implementation evidence` |
| `6356ac1c` | `docs(publisher-services): complete the BE-04 report's CI section` |

Reconciliation episode, all additive on top of the published history:

| SHA | Subject |
|---|---|
| `f4cb9daf` | `Merge develop into feature/publisher-services/be-04` |
| `951d8270` | `fix(publisher-services): reconcile BE-04 with the corrected contract` |
| _(this commit)_ | `docs(publisher-services): reconcile BE-04 control records` |

The exact implementation head is recorded on the pull request and is the SHA the
independent review must be taken against.

---

## 4. Files changed

New:

```text
thoth-api/migrations/20260814_v1.7.0/up.sql
thoth-api/migrations/20260814_v1.7.0/down.sql
thoth-api/src/model/distribution_job/mod.rs
thoth-api/src/model/distribution_job/crud.rs
thoth-api/src/model/distribution_job/tests.rs
thoth-api/src/graphql/distribution_job_tests.rs
docs/engineering/ai-delivery/implementation-reports/BE-04-implementation-report.md
```

Modified:

```text
CHANGELOG.md
docs/publisher-services/task-status.md
src/bin/arguments/mod.rs
src/bin/commands/mod.rs
src/bin/commands/start.rs
src/bin/commands/zitadel.rs
src/bin/thoth.rs
thoth-api-server/src/lib.rs
thoth-api/src/graphql/dataloader.rs
thoth-api/src/graphql/mod.rs
thoth-api/src/graphql/model.rs
thoth-api/src/graphql/mutation.rs
thoth-api/src/graphql/query.rs
thoth-api/src/graphql/service_configuration_tests.rs
thoth-api/src/graphql/distribution_platform_tests.rs
thoth-api/src/model/distribution_job/…
thoth-api/src/model/mod.rs
thoth-api/src/model/publisher_distribution_platform/crud.rs
thoth-api/src/model/publisher_distribution_platform/tests.rs
thoth-api/src/model/publisher_service_configuration/crud.rs
thoth-api/src/model/publisher_service_configuration/mod.rs
thoth-api/src/model/publisher_service_configuration/tests.rs
thoth-api/src/model/tests.rs
thoth-api/src/model/work_relation/tests.rs
thoth-api/src/policy.rs
thoth-api/src/schema.rs
thoth-api/tests/support/mod.rs
thoth-errors/src/lib.rs
```

Nothing unrelated changed. `thoth-client/assets/queries.graphql` is unchanged
(section 12), `thoth-client/assets/schema.graphql` remains build-generated and
gitignored and was neither hand-edited nor committed, `.gitignore` is unchanged,
no workflow file changed, and no `diesel.toml` was introduced.

Three existing test files were changed, each for a reason this task created and
none to relax an assertion:

1. `thoth-api/src/model/work_relation/tests.rs` — five call sites assumed the
   work-relation enforcement migration was the **last** migration and used
   `revert_last_migration` to mean "revert enforcement". BE-04 adds a later
   migration, so those calls would have reverted BE-04's instead, leaving
   enforcement installed and silently turning three of those tests into no-ops.
   They now revert down to and including the enforcement migration by version.
2. `thoth-api/src/model/publisher_service_configuration/tests.rs` — the
   schema-contract assertion "no job table exists anywhere" is narrowed to the
   exact three relations BE-04 adds, rather than deleted. The capability half of
   the assertion is unchanged.
3. `thoth-api/src/graphql/{service_configuration_tests,distribution_platform_tests}.rs`
   — two source-inspection loader guards counted batcher structs and inspected
   `for_request`'s body. BE-04 legitimately adds three loaders and moved
   construction into one private `build`. Both guards now name the exact
   expected inventory instead of counting, so an authorized addition is visible
   as an addition and an unauthorized one still fails.

`src/bin/thoth.rs` gained one shared test-environment lock and a
command-tree-initialisation helper; section 12.1 records why.

### 4.1 Files changed by the reconciliation

The reconciliation touched a strict subset of the files above, plus the control
records:

```text
thoth-api/migrations/20260814_v1.7.0/up.sql        Correction A
thoth-api/src/model/distribution_job/crud.rs       Correction B (composite read)
thoth-api/src/model/distribution_job/tests.rs      Correction A truth table
thoth-api/src/graphql/dataloader.rs                Correction B (composite loader)
thoth-api/src/graphql/dataloader/fixture.rs        per-chunk outcome recording
thoth-api/src/graphql/model.rs                     Correction B (resolver)
thoth-api/src/graphql/distribution_job_tests.rs    section 25.12 rewrite
docs/engineering/ai-delivery/implementation-reports/BE-04-implementation-report.md
docs/publisher-services/task-status.md
CHANGELOG.md
```

`down.sql` is unchanged: it drops the relations and types, and carries no copy of
the corrected `CHECK`. `thoth-api/src/schema.rs` is unchanged by the
reconciliation, for the reason in section 6.3. `BE-04.md` is **not** touched, and
neither is `ADR-0007`, `ADR-0008`, any workflow, any manifest, `policy.rs`, or
anything in `thoth-client`.

---

## 5. Implementation decisions

1. **`Crud` is not implemented for any of the three entities.** The only
   supported writes are the named domain functions of
   `model/distribution_job/crud.rs`.
2. **The claim statement's projection is the whole job row**, so section 12.3's
   optional merge of statement 2 into statement 1 is taken. The claim path is
   therefore recovery + claim + targets + attempts = a constant four statements.
3. **`DistributionJobPayload`** carries optionally preloaded children. The worker
   claim path preloads them with its own two set-based statements and
   deliberately does not use `RequestLoaders`; every other path leaves them lazy
   and batches through the ADR-0007 loaders.
4. **The kinds filter is bound as `text[]` and cast to the enum array**
   (`j.kind = ANY($2::text[]::public.distribution_job_kind[])`), which keeps the
   comparison on the enum type exactly as specified while avoiding a
   custom-enum array binding.
5. **The `T3`/`T4` branch and the backoff are computed inside the one `UPDATE`**
   from the row's own `attempt_count`, using exact `numeric` arithmetic, so no
   read-then-decide window exists.
6. **`DistributionJobTarget` exposes `platform`**, per the normative type
   definition in specification section 16.2. Section 17.4's illustrative query
   snippets write `targets { distributionPlatform }`, which does not match that
   type definition; this is an internal inconsistency in the specification's
   example, resolved in favour of the normative type definition and recorded
   here rather than silently. It is also consistent with BE-02's merged
   `PublisherDistributionPlatformAssignment.platform`.

---

## 6. Database and migration effects

### 6.1 Migration identity and ordering

| Item | Value |
|---|---|
| Directory | `thoth-api/migrations/20260814_v1.7.0/` |
| Created by | `make migration` (not hand-created, not pre-selected) |
| Ordering | sorts last of all migration directories under the embedded runner's lexicographic order; the previous highest was `20260813_v1.6.3` |
| Count | exactly one BE-04 migration |

The ordering is asserted by test
(`the_migration_directory_sorts_after_every_existing_one`), not assumed, because
`make migration` derives the version from `Cargo.toml` rather than from the
existing directories.

### 6.2 DDL summary

Four `CREATE TYPE`, three `CREATE TABLE`, three `CREATE INDEX`, and
`SELECT diesel_manage_updated_at('public.distribution_job')` — on
`distribution_job` alone, because the target and attempt relations are
append-only and carry no `updated_at`.

Catalog-verified enum labels and declaration order:

```text
distribution_job_kind               PUBLISHER_BACK_CATALOGUE
distribution_job_status             PENDING, RUNNING, SUCCEEDED, FAILED, CANCELLED
distribution_job_attempt_result     SUCCEEDED, FAILED, CANCELLED, ABANDONED
distribution_job_cancellation_reason ADMINISTRATIVE, ASSIGNMENT_DISABLED
```

No `OTHER`, `UNKNOWN`, `NONE`, `NOT_STARTED`, `NOT_APPLICABLE` or `DEFAULT`
label exists on any of the four, asserted by catalog query.

The 29 named constraints present in `pg_constraint`, all validated, none
`NOT VALID`:

```text
distribution_job_attempt_claim_token_key
distribution_job_attempt_claimed_by_check
distribution_job_attempt_closure_check
distribution_job_attempt_count_check
distribution_job_attempt_distribution_job_id_fkey
distribution_job_attempt_error_code_format_check
distribution_job_attempt_error_detail_length_check
distribution_job_attempt_error_pairing_check
distribution_job_attempt_error_result_check
distribution_job_attempt_interval_check
distribution_job_attempt_number_check
distribution_job_attempt_number_key
distribution_job_attempt_pkey
distribution_job_back_catalogue_work_check
distribution_job_cancellation_reason_check
distribution_job_claim_state_check
distribution_job_claimed_by_check
distribution_job_completed_at_check
distribution_job_deduplication_key_formula_check
distribution_job_deduplication_key_key
distribution_job_deduplication_key_length_check
distribution_job_last_error_check
distribution_job_last_error_code_format_check
distribution_job_last_error_detail_length_check
distribution_job_pkey
distribution_job_publisher_id_fkey
distribution_job_target_distribution_job_id_fkey
distribution_job_target_pkey
distribution_job_work_id_fkey
```

All four foreign keys are `ON DELETE CASCADE` (`confdeltype = 'c'`) and none is
weakened, deferred or `NOT VALID`.

Indexes present:

```text
distribution_job_claimable_idx            (partial, status = 'PENDING')
distribution_job_lease_idx                (partial, status = 'RUNNING')
distribution_job_publisher_latest_idx
distribution_job_deduplication_key_key    (constraint-backed; no separate index)
distribution_job_pkey
distribution_job_target_pkey
distribution_job_attempt_pkey
distribution_job_attempt_number_key
distribution_job_attempt_claim_token_key
```

Observed constraint expressions:

```text
distribution_job_attempt_count_check
  CHECK (((attempt_count >= 0) AND (attempt_count <= 5)))

distribution_job_deduplication_key_formula_check
  CHECK (((kind <> 'PUBLISHER_BACK_CATALOGUE'::distribution_job_kind)
    OR (deduplication_key = ((('PUBLISHER_BACK_CATALOGUE:'::text
      || (publisher_id)::text) || ':'::text) || (activation_id)::text))))
```

PostgreSQL 17.10 accepted the deduplication-key formula check, so stop
condition 7 does not fire.

#### 6.2.1 Correction A — the NULL-safe attempt-error `CHECK` (reconciliation)

The migration previously carried

```sql
CONSTRAINT distribution_job_attempt_error_result_check CHECK (
    (error_code IS NULL AND error_detail IS NULL)
    OR result = 'FAILED'
)
```

which does not enforce the invariant it names. PostgreSQL rejects a row only when
a `CHECK` evaluates to `FALSE` and admits it when the result is `UNKNOWN`. On an
**open** attempt — `finished_at IS NULL` and therefore `result IS NULL` under
`distribution_job_attempt_closure_check` — the first arm is `FALSE` and
`result = 'FAILED'` is `NULL`, so `FALSE OR NULL` is `NULL` and a row carrying
error fields was **accepted**. Those are exactly the states section 11.2's claim
statement creates most often.

It now reads:

```sql
CONSTRAINT distribution_job_attempt_error_result_check CHECK (
    (error_code IS NULL AND error_detail IS NULL)
    OR (
        result IS NOT NULL
        AND result = 'FAILED'
    )
)
```

The specification's own expression was implemented literally, so no equivalence
argument is owed. The `result IS NOT NULL` conjunct converts the previously
`UNKNOWN` rows to `FALSE` and changes no row the constraint already decided: the
state machine of section 11.2 is untouched.

Observed `pg_get_constraintdef` on the migrated database, PostgreSQL 17.10:

```text
CHECK ((((error_code IS NULL) AND (error_detail IS NULL))
     OR ((result IS NOT NULL) AND (result = 'FAILED'::distribution_job_attempt_result))))
```

PostgreSQL stores the conjunct rather than folding it away, so the catalog itself
is evidence the hole is closed. The test asserts the stored expression still
contains `error_code IS NULL`, `error_detail IS NULL`, `result IS NOT NULL` and
`FAILED`, so a future edit that reintroduces the three-valued hole fails on the
catalog and not only on behaviour.

Observed truth table, every case asserted on **`INSERT`** and on **`UPDATE`** of
an open attempt into the same state, with each rejection attributed to
`distribution_job_attempt_error_result_check` **by name** through the database
error's `constraint_name`, so a neighbouring constraint cannot stand in for it:

| # | `result` | Error fields | `INSERT` | `UPDATE` | Old expression |
|---:|---|---|---|---|---|
| 1 | `NULL` (open) | `error_code` only | **rejected** | **rejected** | accepted — the defect |
| 2 | `NULL` (open) | `error_code` + `error_detail` | **rejected** | **rejected** | accepted — the defect |
| 3 | `SUCCEEDED` | `error_code` only, and both | **rejected** | **rejected** | rejected |
| 4 | `ABANDONED` | `error_code` only, and both | **rejected** | **rejected** | rejected |
| 5 | `CANCELLED` | `error_code` only, and both | **rejected** | **rejected** | rejected |
| 6 | `NULL` (open) | both `NULL` | **accepted** | **accepted** | accepted |
| 7 | `FAILED` | valid `error_code` + `error_detail` | **accepted** | **accepted** | accepted |
| 8 | `FAILED` | both `NULL` | **accepted** | **accepted** | accepted |

Rows 3–5 are each exercised twice, in the `error_code`-only and the
`error_code` + `error_detail` form, giving eight distinct rejection cases in each
of the two write modes. `error_detail` alone is deliberately not a case for this
constraint: `distribution_job_attempt_error_pairing_check` refuses it first, so it
would prove nothing here. Every acceptance is proven to have **persisted** by
reading the row back, not merely by the absence of an error.

Rows 1 and 2 are the load-bearing ones — the rows the withdrawn expression
admitted. Test:
`model::distribution_job::tests::the_attempt_error_result_constraint_is_null_safe_on_insert_and_update`.

`down.sql` needs no change: it drops the relations and the types, and never
restates the constraint.

### 6.3 `schema.rs`

`thoth-api/src/schema.rs` was edited **manually** under ADR-0003 Architecture A.
`diesel print-schema` was not used as a writer, no `diesel.toml` was introduced
and no schema-synchronization subsystem was added. The additions are the four
`sql_types` structs after `PublisherServiceConfigurationSource`, three `table!`
blocks in alphabetical position, four `joinable!` entries and three
`allow_tables_to_appear_in_same_query!` entries, all in repository ordering, with
no unrelated reformatting.

Parity with the migration is asserted by test
(`schema_rs_matches_the_migration_for_all_three_relations`) against
`information_schema.columns` — name, type and nullability in ordinal position for
all three relations — and against the text of `schema.rs` for the `joinable!` and
same-query entries.

**Correction A has no `schema.rs` impact, recorded as a reviewed conclusion**
rather than omitted. Diesel's `table!` macro encodes columns, types and
nullability; it does not encode `CHECK` constraints, so changing a check
expression is outside the checked-in Diesel table contract. `schema.rs` is
therefore unchanged by the reconciliation, and migration/schema parity is
unaffected — the parity test above passes at the reconciled head.

### 6.4 Forward, revert, re-apply and populated behaviour

| Check | Result |
|---|---|
| Empty database: apply → revert → re-apply | all three succeed; after revert, zero `distribution_job%` tables and zero `distribution_job%` types remain |
| `cargo run migrate` / `cargo run migrate --revert` | both succeed against a disposable database (section 12) |
| Representative populated database | forward migration succeeds; **zero** job, target and attempt rows created |
| Existing rows | every `publisher`, `publisher_distribution_platform`, `publisher_service_configuration_history` and `work` row byte-identical before and after |
| `pg_class.relfilenode` | **unchanged** for `publisher`, `publisher_distribution_platform`, `publisher_service_configuration_history` and `work` |
| Down migration on a **populated** database | exercised with one job, two targets and one closed attempt present, plus representative desired-state and audit rows; the drop succeeds, no assignment row and no configuration audit row is touched, and the migration re-applies afterwards |

Every row above was **re-run at the reconciled head** and passed, through the
migration tests
(`the_migration_directory_sorts_after_every_existing_one`,
`the_migration_applies_reverts_and_re_applies_on_an_empty_database`,
`the_migration_changes_no_existing_row_and_rewrites_no_existing_table`,
`only_distribution_job_is_diesel_managed_and_the_indexes_are_exactly_the_specified_three`,
`every_named_constraint_of_sections_7_2_to_7_4_exists_in_the_catalog`,
`schema_rs_matches_the_migration_for_all_three_relations`,
`the_attempt_budget_constant_and_the_migration_agree`).

The CLI path was re-run at the reconciled head against a **freshly created,
disposable** database (`thoth_be04_migrate`, created for the run and dropped
after it, on a local PostgreSQL 17.10 — never production and never a shared
service):

| Step | Observed |
|---|---|
| `cargo run -- migrate --database-url <disposable>` on an empty database | exit 0; `__diesel_schema_migrations` head `20260814`; `distribution_job`, `distribution_job_target`, `distribution_job_attempt` present |
| Row counts immediately after forward | `distribution_job=0 target=0 attempt=0` |
| Corrected constraint as stored | the `pg_get_constraintdef` output quoted in section 6.2.1 |
| `cargo run -- migrate --database-url <disposable> --revert` | exit 0; zero `distribution_job%` relations and zero `distribution_job%` types remain |
| Re-apply after the revert | exit 0; the three relations and the corrected constraint return, with the nine named indexes of section 6.2 |

Measured forward duration on a representative populated disposable database:
**7.0165 ms**. Under deliberate contention with `lock_timeout = 750ms` the
migration waited **757.454083 ms** and then failed cleanly, taking no partial
effect. **Neither figure is a production prediction**: they are
disposable-environment measurements on a small local dataset, and the production
lock window depends on the production tables' size and concurrent write load.

### 6.5 Observed locking on the referenced existing tables

Captured from a **second session** by joining `pg_locks` to `pg_class` while the
migration's own `up.sql` ran inside a transaction held open by a first session,
against a representative populated database:

```text
publisher AccessShareLock
publisher ShareRowExclusiveLock
work      AccessShareLock
work      ShareRowExclusiveLock
```

The `SHARE ROW EXCLUSIVE` locks on `public.publisher` and `public.work` are
**observed, not absent**. This corrects the earlier false claim that the
migration takes no locks on existing tables. No `ACCESS EXCLUSIVE` lock is taken
on either referenced table, so concurrent readers are not blocked; concurrent
`INSERT`, `UPDATE` and `DELETE` on those two tables are.

| Measurement | Observed value |
|---|---|
| Forward migration duration, representative populated disposable database | **8.7295 ms** |
| Contention fixture: one session holding an open transaction that has `UPDATE`d a publisher row, migration started with `lock_timeout = 750ms` | the migration **waited 757.91 ms and then failed cleanly**; nothing partial was left behind |
| Same fixture with **no** `lock_timeout` | the migration **waited** for the whole time the writer held its lock (proven by a 750 ms receive timeout elapsing with the migration still blocked) and then **completed** once the writer committed; all three relations exist afterwards |

**These are disposable-environment measurements. They are not a production
duration prediction, and no production-safe duration or catalogue-size threshold
is extrapolated from them.** Whoever authorizes the production migration decides
when `public.publisher` and `public.work` may be closed to writers and for how
long a wait behind an existing writer is acceptable; this measurement is an input
to that decision, never a substitute for it.

The foreign keys were **not** weakened, deferred, made `NOT VALID` or dropped to
shorten the window.

### 6.6 Deduplication evidence

The stored key equals `PUBLISHER_BACK_CATALOGUE:<publisher_id>:<activation_id>`
for every row, proven by `distribution_job_deduplication_key_formula_check`,
which is present in the catalog (expression quoted in section 6.2) and is what
refuses a wrongly computed key — a direct insert with a non-formula key, and one
with an empty key, are both refused by the database.

The rendered key is fixed at 98 characters. Example from the domain test:

```text
PUBLISHER_BACK_CATALOGUE:11111111-1111-4111-8111-111111111111:22222222-2222-4222-8222-222222222222
```

---

## 7. Creation, atomicity and the switch

### 7.1 Lifecycle widening

`AssignmentLifecycleOutcome` is now
`Unchanged | Activated { activation_id } | Repaired { activation_id } | Disabled`,
with `changed()` returning `!matches!(self, Unchanged)`. The
`Activated`/`Repaired` distinction is decided **inside `enable_on`, from the
`member_rows` it already reads, before it writes anything**: a group with no
currently enabled member is `Activated`; any other not-fully-normalized state is
`Repaired`. There is no second read, no duplicated linked-platform algorithm and
no inference from a destination's name. BE-02's pool-level `enable`/`disable`
keep their `ThothResult<()>` signatures, their early non-assignable check and
their merged semantics; the enum is `pub(crate)`, so the widening is internal.

Observed in the widened BE-02 test: an absent row enabled ⇒ `Activated`; an
already-enabled singleton ⇒ `Unchanged`; an enabled group disabled ⇒ `Disabled`;
a group with no enabled member disabled ⇒ `Unchanged`; a disabled row re-enabled
⇒ `Activated` with a **different** activation identity; an already-normalized
linked group re-requested ⇒ `Unchanged`; a split linked pair ⇒ `Repaired`.

### 7.2 Creation matrix, row by row

| Situation | Specified | Observed |
|---|---|---|
| `Activated` group with ≥1 `AutomaticPush` member, `SUPERUSER_API`, switch `ON` | one job, one target per `AutomaticPush` member | one job created, targets as specified |
| Same, switch `OFF` | whole transaction fails and rolls back | `DistributionJobCreationDisabled`; zero committed change (section 7.3) |
| `Activated` group, all `PullFeed` (`OCLC_KB`, `EX_LIBRIS_KB`, both) | no job | 0 jobs; the activation itself committed |
| `Activated` group, all `Manual` (`SCIENCE_OPEN`) | no job | 0 jobs; the activation itself committed |
| `Repaired` group | no job | 0 jobs, under both switch positions |
| `Unchanged` group | no job | 0 jobs |
| `Disabled` group | no job | 0 jobs |
| package-only change | no job | 0 jobs, and no assignment row written |
| true no-op replacement | no job | 0 jobs, token unmoved |
| stale replacement | no job | `StalePublisherServiceConfiguration`, 0 jobs, token unmoved |
| repeated identical replacement | no second job | 1 job total |
| `source = MIGRATION_BACKFILL` | no job, and no failure | 0 jobs under **both** `OFF` and `ON`; the write commits normally and all three assignments are created |
| non-assignable `JISC_NBK` | rejected before any write | `DistributionPlatformNotAssignable("JISC_NBK")`, token unmoved, 0 jobs, 0 `JISC_NBK` targets |
| disable then re-enable | new legitimate job | 2 jobs, different `activation_id`, different `deduplication_key`, the first `CANCELLED` with `ASSIGNMENT_DISABLED` and the second `PENDING` |

### 7.3 Exact OAPEN/DOAB result

For all three request shapes — name `OAPEN`, name `DOAB`, name both — the
observed result is identical:

| Quantity | Observed |
|---|---|
| `distribution_job` rows | **1** |
| `distribution_job_target` rows | **2** |
| target order | `OAPEN`, then `DOAB` — the enum's declaration order |
| distinct `activation_id` across the two assignment rows | **1** |
| the job's `activation_id` | equal to that one shared activation |
| the job's `deduplication_key` | equal to the formula for that publisher and activation |
| job status / attempt count / claim fields / `completed_at` | `PENDING` / `0` / all null / null |

`SELECT count(*) FROM distribution_job j WHERE NOT EXISTS (SELECT 1 FROM
distribution_job_target t WHERE t.distribution_job_id = j.distribution_job_id)`
is `0` after every creation scenario in the suite.

A replacement activating a linked group **and** an independent destination
produces exactly two jobs with two distinct keys and target sets
`[OAPEN, DOAB]` and `[ZENODO]`.

### 7.4 `OFF` fail-closed evidence

The refused transaction returns `ThothError::DistributionJobCreationDisabled`,
mapped to `DISTRIBUTION_JOB_CREATION_DISABLED`, and the mutation returns **no**
configuration payload — `data.replacePublisherServiceConfiguration` is `null`.
There is no silent success, no partial success and no warning.

Before/after snapshots taken around the refused call are **equal** for every one
of:

- every `publisher_distribution_platform` row for that publisher, including
  `enabled`, `activation_id`, `enabled_at`, `disabled_at` and `updated_at`;
- `publisher.service_configuration_updated_at`;
- `publisher.subscription_package` and the publisher row generally;
- the `publisher_service_configuration_history` row count;
- the `distribution_job` and `distribution_job_target` row counts (both `0`);
- every `work.updated_at_with_relations` value for that publisher, proving the
  `AFTER UPDATE` trigger's effect did not survive.

No new activation is committed: no `OAPEN` assignment row exists afterwards.

**`OFF` → `ON` retry**: the identical desired activation replayed with
`job_creation = On` succeeds and creates **exactly one** job with its full
`AutomaticPush` target set (`OAPEN`, `DOAB`), and exactly **one** audit row —
proving the earlier refusal lost nothing and left no partial state that would
deduplicate the retry away.

**Permitted under `OFF`**: a `PullFeed`-only activation, a `Manual`-only
activation, a package-only change, a true no-op, a disable and a linked-state
repair each commit normally, create no job and raise no error.

**No retroactive sweep**: with three publishers holding pre-existing enabled
`AutomaticPush` assignments and no jobs, turning the switch `ON` creates zero
jobs; a subsequent unrelated no-op replacement still creates zero; only a genuine
disable/re-enable cycle creates one, and only for the publisher that performed
it.

**The switch is the only difference** between the `OFF` failure and the `ON`
success: both are driven through `ServiceConfigurationWriteContext.job_creation`,
and **no test mutates an environment variable** to produce either.

### 7.5 Transaction integration

Observed statement order for a job-creating change, captured with the
connection-level SQL probe:

```text
1  SELECT publisher_id ... FOR UPDATE        (the publisher row lock, first)
2  reads under that lock
3  INSERT INTO "publisher_distribution_platform" ...   (lifecycle writes)
4  INSERT INTO "distribution_job" ...                  (job write)
5  INSERT INTO "distribution_job_target" ...           (target write)
6  UPDATE "publisher" ...                              (exactly one)
7  INSERT INTO "publisher_service_configuration_history" ...  (exactly one)
```

- `lock < lifecycle < job insert < target insert < publisher UPDATE < audit` is
  asserted, so the job writes provably precede the publisher `UPDATE` that fires
  the work-freshness cascade;
- exactly **one** `UPDATE "publisher"` statement is issued, so the cascade runs
  once per committed change;
- the configuration path takes **no** `distribution_job` row lock at all, so no
  path locks a job row before its publisher row;
- the audit row's `before_state` and `after_state` key sets are exactly
  `configurationVersion`, `enabledDistributionPlatforms`, `subscriptionPackage`
  after a job-creating change — the key set is **not** widened;
- a job never exists without a token bump and exactly one audit row, asserted
  directly.

The catalogue-scale measurement BE-03 established continues to pass unchanged
for a job-creating change; it is disposable-environment evidence only, and no
production extrapolation or "safe catalogue size" is derived from it. Stop
condition 9 is not triggered: the measured behaviour showed no material
operational problem, and no shared publisher trigger, storage location or
configuration token was altered.

---

## 8. State machine, concurrency and cancellation

### 8.1 Transitions exercised

| Transition | Evidence |
|---|---|
| `T0` | creation matrix, section 7.2 |
| `T1` | claim returns the job `RUNNING` with `attempt_count = 1`, a fresh token, the worker identity, a lease, and exactly one open attempt numbered 1 |
| `T2` | complete sets `SUCCEEDED` and `completed_at`, clears every claim field, **clears `last_error_*`**, and closes the open attempt `SUCCEEDED` |
| `T3` | `fail(retryable = true)` at attempts 1–4 returns the job to `PENDING`, leaves `completed_at` null, sets `last_error_*`, closes the attempt `FAILED`, and schedules `available_at` at the computed backoff |
| `T4` | `fail(retryable = false)`, and `fail(retryable = true)` at attempt 5, both terminalize to `FAILED` with `completed_at` set |
| `T5a` | an expired lease at attempts 1–4 closes the attempt `ABANDONED`, clears the claim fields, returns the job to `PENDING`, leaves `completed_at` null and leaves `attempt_count` **unchanged**; the job is claimable again immediately with no backoff |
| `T5b` | an expired lease at attempt 5 transitions **directly to `FAILED`** with `completed_at` set, `attempt_count` unchanged, the attempt closed `ABANDONED`, and the job is never `PENDING` and never claimable again on any later call |
| `T6` | cancelling a `PENDING` job sets `CANCELLED`, `completed_at` and reason `ADMINISTRATIVE`; no open attempt exists to close |
| `T7` | cancelling a `RUNNING` job, live **or** expired lease, clears every claim field and closes the open attempt `CANCELLED` |
| `T8` | disabling an assignment cancels its `PENDING` jobs with reason `ASSIGNMENT_DISABLED`, in the same transaction as the disable |

`T5a` is observed **in isolation** from the increment the next claim would
perform, by suspending the job's target eligibility so recovery runs and
selection finds nothing. That is what makes "`attempt_count` neither incremented
nor decremented" a direct observation rather than an inference.

### 8.2 Rejections of section 11.6

| Attempt | Observed |
|---|---|
| complete/fail/cancel a job that does not exist | `EntityNotFound` |
| complete/fail a `SUCCEEDED`/`FAILED`/`CANCELLED` job | `DistributionJobAlreadyTerminal(<status>)` with the exact status code |
| cancel a terminal job, including an already-`CANCELLED` one | `DistributionJobAlreadyTerminal(<status>)`, nothing changed |
| complete/fail a `PENDING` job | `StaleDistributionJobClaim` |
| complete/fail a `RUNNING` job with a non-current token | `StaleDistributionJobClaim` |
| claim a `RUNNING` job with a live lease | not an error; simply not selected |
| claim a job whose targets are no longer enabled under its activation | not an error; not selected |
| claim a `PENDING` job at the attempt budget | not an error; not selected, and its count never moves |
| `failDistributionJob` with a malformed or over-length `errorCode` | `InvalidDistributionJobErrorCode` before any transition; no state change |

`PENDING` and "held by another worker" produce the **same** error, deliberately.

### 8.3 Attempt budget

- maximum `attempt_number` observed after **every** exhaustion route —
  four patterns mixing reported failures and lease expiries in every combination
  — is **5**, with exactly 5 attempt rows and the job `FAILED`;
- an exhausted `PENDING` row written directly with `attempt_count = 5` is never
  selected by any claim call, and its count never moves;
- the database refuses `attempt_count` of `-1`, `6`, `7` and `100` on `UPDATE`,
  and `-1` and `6` on `INSERT`; `5` is accepted;
- `DISTRIBUTION_JOB_MAX_ATTEMPTS == 5` and the catalog's
  `distribution_job_attempt_count_check` expression carries the same upper bound,
  asserted together so neither can move without the other.

### 8.4 Retry and backoff

`fail(retryable = true)` at attempt _n_ sets `available_at` to
`CURRENT_TIMESTAMP + min(300 · 2^(n−1), 21 600)` seconds, verified in the
database within a ±30 s window for _n_ = 1…4, giving 5, 10, 20 and 40 minutes.
Attempt 5 is terminal, so the 21 600 s cap is not reached by the current budget.
Each retry adds exactly one closed `FAILED` attempt row. There is no separate
retry mutation.

### 8.5 Concurrency, with real connections

All of the following used **multiple real connections and real transactions**;
none is a mocked sequential substitute.

| Property | Observed |
|---|---|
| zero claims | zero rows returned, zero attempt rows inserted, no error, no state change, and **no payload statement issued at all** |
| one claim | exactly one row, carrying the job id, a freshly minted token, `claimed_by`, `claimed_at`, `lease_expires_at` and `attemptNumber = 1`; exactly one attempt row |
| many claims | M = 7 due jobs and a batch of 7 → exactly 7 rows, 7 **distinct** tokens, 7 attempt rows, one per claimed job |
| concurrent workers | 4 real worker threads on 4 connections against 24 due jobs: every job claimed **at most once**, the union a partition of all 24, every token distinct, each worker's rows carrying only its own identity, exactly 24 open attempts, and the whole run completing well inside the non-blocking bound |
| deterministic order | with staggered `available_at`, a single worker's returned rows are in `(available_at ASC, distribution_job_id ASC)` order, including for two jobs sharing an `available_at` |
| concurrent recovery | 4 threads racing recovery of one expired job produce **exactly one** closed `ABANDONED` attempt, at most one claim, no duplicate attempt row and unique attempt ordinals |
| concurrent creation | 2 threads creating a job for the **same** activation, bypassing the publisher row lock entirely, produce exactly one job row; the loser observes the `ON CONFLICT DO NOTHING` no-op and raises no error |

### 8.6 Claim statement shape and payload cost

The claim is **one atomic statement**: `SELECT ... FOR UPDATE OF j SKIP LOCKED`
feeding an `UPDATE ... RETURNING`, an `INSERT ... SELECT FROM claimed RETURNING`,
and a final ordered projection. Exactly one statement containing
`FOR UPDATE OF j SKIP LOCKED` is issued per claim, and there is no second claim
query and no read-back of recently claimed rows.

| Batch size | Observed statements |
|---:|---:|
| 1 | **4** |
| 10 | **4** |
| 50 | **4** |

The four are: one bounded lease-recovery statement, the atomic claim, one
set-based target read and one set-based attempt read. The count is constant in
the batch size, so there is no N+1 path over jobs, targets or attempts. The
request-local `RequestLoaders` are **not** used on this path, asserted by
observing zero dispatches on all three job loaders during a claim.

Clamping: `limit` above 50 clamps to 50; `limit <= 0` claims nothing and issues
no database work; `leaseSeconds` below 60 clamps up and above 3600 clamps down,
verified against the stored `lease_expires_at`.

### 8.7 Stale-token evidence

Each of the seven prohibited effects of section 12.6 was attempted and observed
to be refused:

| Prohibited effect | Observed |
|---|---|
| complete a newer attempt | `StaleDistributionJobClaim`; the newer attempt untouched |
| fail a newer attempt | `StaleDistributionJobClaim` |
| retry a newer attempt (`retryable = true`) | `StaleDistributionJobClaim` |
| overwrite worker identity | the job row is byte-identical afterwards; `claimed_by` still the newer worker |
| overwrite the current lease | `lease_expires_at` unchanged |
| mutate a terminal result | `DistributionJobAlreadyTerminal`; nothing changed |
| close someone else's attempt | the full attempt list is byte-identical afterwards |

The token of an attempt closed `ABANDONED` is stale in **both** recovery
branches: in the recovered-`PENDING` case it yields `StaleDistributionJobClaim`,
and in the terminal-`FAILED` case `DistributionJobAlreadyTerminal("FAILED")`.

Every row of the section 13.3 repeat-call table was exercised, and no sequence
produced contradictory state, a second attempt row or a second terminal result.

### 8.8 `lastError` evidence

| Case | Observed job-level values |
|---|---|
| `T5a` after a previous worker-reported failure | **unchanged** at the earlier reported values; the newest attempt is `ABANDONED` with its own error fields null |
| `T5a` with no previous reported failure | both **null** |
| `T5b` after a previous reported failure | retained at the earlier reported values (`TRANSPORT_FAILURE` / "reported at attempt 4"), with the newest attempt `ABANDONED` — so the retained value is demonstrably **not** the cause of terminalization |
| `T5b` with no previous reported failure | both **null** on a `FAILED` job, and nothing synthesizes a placeholder, an `UNKNOWN` or an abandonment pseudo-error. **This is correct, not missing data.** |
| success (`T2`) | both cleared to null |
| cancellation (`T6`, `T7`, `T8`) | neither set nor cleared, asserted both for a job with a previous reported failure and for one without |

For `T3` and `T4` the job's values equal the closing attempt row's values.

### 8.9 Cancellation evidence

Every row of the section 14.2 table was exercised, with the job effect, the
attempt effect and the error where rejected. In addition:

- cancelling a `RUNNING` job invalidates the live token immediately: the
  holder's subsequent `complete` and `fail` are both rejected;
- a cancelled job cannot be reopened, retried or re-claimed;
- target rows and attempt history are unchanged by cancellation, and **no job,
  target or attempt row is deleted** — the row counts are asserted afterwards;
- assignment disable with a `PENDING` job cancels it with `ASSIGNMENT_DISABLED`
  in the same transaction; a `PENDING` job for a **different** group of the same
  publisher is untouched, and another publisher's jobs are untouched;
- assignment disable with a `RUNNING` job leaves it `RUNNING`; the claiming
  worker can still terminalize it; after `T5a` recovery it is **not** claimable
  and remains visible for explicit cancellation; if its budget was already
  exhausted, `T5b` terminalizes it to `FAILED` instead.

---

## 9. Authorization and security

### 9.1 The role

| Property | Value |
|---|---|
| Role code | `DISSEMINATION_WORKER` |
| Scope | unscoped ZITADEL project role, checked by key presence in `project_roles` |
| Guard helpers | `UserAccess::is_dissemination_worker()`, `PolicyContext::require_dissemination_worker()` |
| Shared implementation | a **module-private** `UnscopedRoleAccess::has_unscoped_role`, not exposed as any general service-role API |
| `is_superuser()` | re-expressed in terms of it with byte-identical behaviour; the existing `policy.rs` tests pass unchanged |

### 9.2 The complete section 15.2 matrix, as observed

| Caller | claim | complete | fail | cancel | staff report |
|---|---|---|---|---|---|
| anonymous | DENY | DENY | DENY | DENY | DENY |
| authenticated, no applicable role | DENY | DENY | DENY | DENY | DENY |
| `PUBLISHER_USER` for the target publisher | DENY | DENY | DENY | DENY | DENY |
| `PUBLISHER_USER` for another publisher | DENY | DENY | DENY | DENY | DENY |
| `PUBLISHER_ADMIN` without `PUBLISHER_USER` | DENY | DENY | DENY | DENY | DENY |
| `WORK_LIFECYCLE` without `PUBLISHER_USER` | DENY | DENY | DENY | DENY | DENY |
| `CDN_WRITE` without `PUBLISHER_USER` | DENY | DENY | DENY | DENY | DENY |
| `SUPERUSER` without `DISSEMINATION_WORKER` | **DENY** | **DENY** | **DENY** | **ALLOW** | ALLOW |
| `DISSEMINATION_WORKER` only | ALLOW | ALLOW | ALLOW | **DENY** | **DENY** |
| `SUPERUSER` + `DISSEMINATION_WORKER` | ALLOW | ALLOW | ALLOW | ALLOW | ALLOW |
| invalid/absent introspection | DENY | DENY | DENY | DENY | DENY |

Every denial was observed to occur **before any database access**: the job's
state is unchanged after each denied call. The report's count query is protected
identically, including with the two new filters supplied.

### 9.3 Role-boundary evidence

- a `DISSEMINATION_WORKER`-only account returns `PublisherPermissions::default()`
  for every organisation — `publisher_admin`, `work_lifecycle` and `cdn_write`
  all `false` — and `publisher_org_ids()` returns **no** organisation for it,
  even when ZITADEL carries an organisation key under the unscoped role;
- a worker-only account is denied `publisherServiceConfiguration`, the staff
  report, `replacePublisherServiceConfiguration` and `cancelDistributionJob`;
- `publisher_org_ids()` still collects scoped roles correctly when a worker role
  is also present;
- the role enum gains exactly one variant, and it is domain-specific: no
  `Service`, `Machine`, `Worker` or `ServiceAccount` variant exists;
- no generic service-role API, role registry, machine-identity table or
  credential store was introduced, asserted by source inspection;
- **no identity-provider configuration was changed and no role was granted.**
  `src/bin/commands/zitadel.rs` now *declares* `DISSEMINATION_WORKER` in its
  `setup` role list, which is an ordinary repository source change. `zitadel
  setup` was **not executed**, the role was **not** created in any identity
  provider, it was **not** granted to any account, and no credential was issued
  or rotated. The pre-existing absence of `WORK_LIFECYCLE` and `CDN_WRITE` from
  that list is **not** repaired here and remains recorded for a separate task.

### 9.4 No general composition rule

A principal holding both `SUPERUSER` and `DISSEMINATION_WORKER` may exercise
exactly the operations each role is independently allowed, because the BE-04
matrix says so. That is a BE-04-specific matrix decision. No general
role-composition, aggregation or inheritance rule exists anywhere in
`policy.rs`, and none is created here.

---

## 10. GraphQL contract and client compatibility

### 10.1 Added surface

Enums: `DistributionJobKind`, `DistributionJobStatus`,
`DistributionJobAttemptResult`, `DistributionJobCancellationReason`.

Object types: `DistributionJob`, `DistributionJobTarget`,
`DistributionJobAttempt`, `ClaimedDistributionJob`.

Input types: `ClaimDistributionJobsInput`, `CompleteDistributionJobInput`,
`FailDistributionJobInput`, `CancelDistributionJobInput`.

Mutations: `claimDistributionJobs`, `completeDistributionJob`,
`failDistributionJob`, `cancelDistributionJob`.

Field added to an existing type:
`PublisherServiceConfigurationSummary.latestBackCatalogueJob`.

Arguments added to existing queries: `jobStatuses` and
`withoutBackCatalogueJob`, on `publisherServiceConfigurations` and on
`publisherServiceConfigurationCount`.

No scalar, interface, union or query was added.

### 10.2 Exact SDL diff

Generated only through the normal `thoth-client` build path
(`cargo build --workspace`, which runs `thoth-client/build.rs`). The baseline was
produced by building the **authorized base** in a separate git worktree the same
way.

```text
base SDL   sha256 25329c1687d8b4222638c2f673bd2751a13adeda8c6f181d4ac83e869abac479
head SDL   sha256 38820a24f7c1b1bac8f6ddc5286efd55dd7ece5f0155806ca4720f228ec93140
```

The unified diff contains **144 added lines and exactly two removed lines**. The
two removals are the previous single-line renderings of
`publisherServiceConfigurations` and `publisherServiceConfigurationCount`,
replaced by renderings that carry the two new arguments. **No existing field's
type, nullability, arguments, defaults or description changed**, and no type was
removed.

The two new argument lines, quoted verbatim from the generated SDL, beside the
merged siblings they must match:

```graphql
"If set, only shows results for publishers that have every one of these distribution platforms enabled. Multiple values narrow the results rather than widening them" enabledPlatforms: [DistributionPlatform!] = []
"If set, only shows results for publishers with these subscription packages" packages: [ThothPackage!] = []
"If set, only shows results whose latest publisher back-catalogue job has one of these statuses. Multiple values widen the results" jobStatuses: [DistributionJobStatus!] = []
"If set to true, only shows publishers with no publisher back-catalogue job at all; if false, only publishers that have at least one" withoutBackCatalogueJob: Boolean
```

`jobStatuses` renders as `[DistributionJobStatus!] = []` — the merged
`Option<Vec<T>>`-plus-default convention — and **not** as a stricter non-null
list. `withoutBackCatalogueJob` is nullable with no default, so absent and
`null` both mean "no filter". The claim input follows the same convention:
`limit: Int = 10`, `leaseSeconds: Int = 900`, `kinds: [DistributionJobKind!] = []`,
and `retryable: Boolean! = true`.

### 10.3 Non-exposure

Asserted against the generated SDL, using brace-balanced block extraction:

- `DistributionJob` has **no** `claimToken` field, no `claimedBy` field, no
  `deduplicationKey` and no `activationId`;
- `claimToken` appears only on `ClaimedDistributionJob`;
- `claimedBy` appears only on `DistributionJobAttempt`, which itself exposes no
  claim token;
- `CompleteDistributionJobInput` has **no** `errorCode` and no `errorDetail`, so
  `failDistributionJob` is the only operation that can raise
  `INVALID_DISTRIBUTION_JOB_ERROR_CODE`;
- no BE-04 type exposes an adapter profile, endpoint, bucket, host or
  credential;
- `PublisherServiceConfiguration` gains **no** job field, so BE-03's
  configuration-only surface — readable by a `PUBLISHER_USER` for their own
  publisher — returns exactly what it returned before;
- no top-level job query was added.

### 10.4 Errors

Exactly four new `ThothError` variants and exactly four new `into_field_error`
arms, with no fifth. Observed at the GraphQL boundary:

| Variant | `extensions.type` | Observed message |
|---|---|---|
| `StaleDistributionJobClaim` | `STALE_DISTRIBUTION_JOB_CLAIM` | `The distribution job claim is no longer valid.` |
| `DistributionJobAlreadyTerminal(String)` | `DISTRIBUTION_JOB_TERMINAL` | `The distribution job is already in the terminal state SUCCEEDED.` |
| `DistributionJobCreationDisabled` | `DISTRIBUTION_JOB_CREATION_DISABLED` | `Automatic distribution job creation is disabled, so this platform activation cannot be saved.` |
| `InvalidDistributionJobErrorCode` | `INVALID_DISTRIBUTION_JOB_ERROR_CODE` | `The supplied distribution job error code is not a valid classification code.` |

- the stale-claim message discloses no token, no holder and not whether one
  exists;
- the terminal message discloses only the status code;
- the creation-disabled message contains no SQL, table name, column name, driver
  text, environment-variable name or value, environment name or deployment
  identifier, asserted against an explicit forbidden-substring list;
- the invalid-code message is a fixed string containing **no part** of the
  rejected value, no length and no regex, and it maps to
  `INVALID_DISTRIBUTION_JOB_ERROR_CODE` and explicitly **not** to
  `INTERNAL_ERROR`;
- for both the malformed and the over-length case, no job or attempt row was
  written or modified, the job's status, `attempt_count`, claim fields and
  `last_error_*` were unchanged, the open attempt stayed open, and the **claim
  token stayed valid** — proven by a following conforming `failDistributionJob`
  succeeding under the same token;
- no existing mapping changed: `STALE_SERVICE_CONFIGURATION`,
  `INVALID_SUBJECT_CODE` and `NO_ACCESS` are unchanged, `INTERNAL_ERROR` remains
  a single catch-all arm, and `StalePublisherServiceConfiguration` is never
  raised for a disabled feature.

### 10.5 Error storage and sanitization

- an over-length detail is truncated to exactly **2048 Unicode scalar values**
  on a character boundary, verified with a 2548-character all-`é` input and with
  a mixed `a/é/漢/🙂` input; the stored value is valid UTF-8 and is a prefix of
  the input;
- ASCII control characters other than `\n` and `\t` are removed, leading and
  trailing whitespace is trimmed, and a detail that sanitizes to nothing is
  stored as nothing;
- the attempt row carries the same sanitized value as the job row;
- `error_code` is **validated, never truncated**; the accepted and rejected sets
  are both asserted, including the boundary at 64 characters;
- no resolver logs the detail, the actor, a token or a credential.

### 10.6 Client compatibility

- `thoth-client/assets/queries.graphql` is **unchanged**. That is the expected
  outcome and is recorded as a reviewed conclusion: BE-04 adds protected
  operations the internal export client does not consume, and one nullable field
  plus two defaulted arguments on surfaces it does not query;
- `thoth-client/assets/schema.graphql` was **not** hand-edited, not committed and
  not force-added, and `.gitignore` was not modified;
- `thoth-app` is **not modified** and is not a member of this workspace. The
  change is assessed as **additive-only** for its codegen: new types are
  unreferenced by existing selections, the one new field on an existing type is
  nullable, and the two new arguments carry defaults, so an existing document
  continues to compile and to return exactly what it returned before;
- the backend commit SHA of the reviewed BE-04 head and the SDL artifact hash
  above are the values later APP-01/APP-02 contract pinning should use.

---

## 11. Reporting

`latestBackCatalogueJob` is the `DISTINCT ON (publisher_id)` most recent
`PUBLISHER_BACK_CATALOGUE` job, ordered `created_at DESC, distribution_job_id
DESC` — a total order — and is **null** when no such job exists.

Observed no-job semantics:

- a publisher with enabled `AutomaticPush` assignments and **no** job reports
  `latestBackCatalogueJob: null`, and the serialized response contains none of
  `NOT_STARTED`, `UNKNOWN`, `NOT_APPLICABLE`, `NONE`, `delivered`, `submitted`,
  `adapterActive`, `SUCCEEDED` or `FAILED`;
- a **repaired** linked group with no job reports `null` for the same reason,
  and nothing infers delivery, adapter execution or back-catalogue presence from
  it.

Filters observed: each status individually; several statuses widening (OR within
the list); empty meaning no filter; `withoutBackCatalogueJob: true` selecting
exactly the job-free publisher and `false` the two with jobs; the documented
contradiction `withoutBackCatalogueJob: true` with a non-empty `jobStatuses`
matching **zero** publishers deterministically and without error; conjunction
with the existing filters; application before pagination; and the count query
returning exactly the number of filtered results for every case.

### 11.1 Measured statement counts (reconciliation)

Measured with the production batchers and the connection-level SQL probe. Counts
exclude pool liveness checks, transaction control statements and Diesel's
per-connection custom-type OID resolution, which is bounded by the number of new
enum types and never by the key count (observed: 1 in every run below).

Every expectation is **derived** from the measured per-chunk classification
through section 17.4.3's arithmetic and then compared with the observed total.
Nothing in the table is a hard-coded target:

```text
statements = 2 + 3 * C_job_nonempty + 1 * C_job_empty + 1 * C_assign
```

| Fixture | Selection | Page | Composite chunks | Classification | `C_job_nonempty` | `C_job_empty` | `C_assign` | Target dispatches | Attempt dispatches | Derived | **Observed** |
|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|
| page with a job | job-only | 1 | `[1]` | non-empty | 1 | 0 | 0 | 0 | 0 | 5 | **5** |
| page with a job | job-only | 25 | `[25]` | non-empty | 1 | 0 | 0 | 0 | 0 | 5 | **5** |
| page with a job | job-only | 200 | `[200]` | non-empty | 1 | 0 | 0 | 0 | 0 | 5 | **5** |
| page with a job | full report | 1 | `[1]` | non-empty | 1 | 0 | 1 | 0 | 0 | 6 | **6** |
| page with a job | full report | 25 | `[25]` | non-empty | 1 | 0 | 1 | 0 | 0 | 6 | **6** |
| page with a job | full report | 200 | `[200]` | non-empty | 1 | 0 | 1 | 0 | 0 | 6 | **6** |
| page with no job | job-only | 1 | `[1]` | empty | 0 | 1 | 0 | 0 | 0 | 3 | **3** |
| page with no job | job-only | 25 | `[25]` | empty | 0 | 1 | 0 | 0 | 0 | 3 | **3** |
| page with no job | job-only | 200 | `[200]` | empty | 0 | 1 | 0 | 0 | 0 | 3 | **3** |
| page with no job | full report | 1 | `[1]` | empty | 0 | 1 | 1 | 0 | 0 | 4 | **4** |
| page with no job | full report | 25 | `[25]` | empty | 0 | 1 | 1 | 0 | 0 | 4 | **4** |
| page with no job | full report | 200 | `[200]` | empty | 0 | 1 | 1 | 0 | 0 | 4 | **4** |

Derived equals observed in all twelve cases, so the four named outcomes of
section 17.4.3 — **5** and **6** on a page containing at least one job, **3** and
**4** on a page whose publishers have none — are consequences of the arithmetic
rather than assertions. `C_assign` is `0` for the job-only selection because it
does not select `enabledDistributionPlatforms`; the assignment chunks observed
for the full report selection were `[1]`, `[25]` and `[200]` respectively.

The whole matrix was re-run **five** times and was identical in every cell,
including the chunk vectors. The withdrawn nested-loader shape was not
reproducible run to run; this one is, which is the point of removing the
dependent-arrival cohort rather than tuning around it.

The assertions the test makes, beyond the derived total:

1. exactly **two** root statements — one filtered/ordered/paginated page query
   and one latest-change query — at every page size;
2. `C_job = 1` at page sizes 1, 25 and 200, which is `ADR-0007` section 4.6's
   `ceil(N / 200)` for a loader-first cohort. This is the shared foundation's
   property, consumed and not restated as a BE-04 guarantee. `C_job > 1` at
   `N <= 200` would be `BLOCKED` under stop condition 23, not a count to relax;
3. **zero** dispatches of the target and attempt loaders under both report
   selections — the scheduler-independent assertion that proves the report path
   carries no dependent-arrival cohort;
4. exact selection dependence in **both** directions: the assignment loader
   dispatches when and only when `enabledDistributionPlatforms` is selected, and
   a selection reaching neither job field dispatches the composite loader zero
   times and issues no `distribution_job%` statement at all (test
   `an_unselected_composite_loader_dispatches_nothing`, observed total `3`);
5. the per-chunk branch in both directions, each **measured**: L2 and L3 occur
   exactly once each for a non-empty chunk, and are **absent from the captured
   SQL entirely** for an empty one, measured on the no-job page rather than
   inferred. L1 occurs exactly once in every case;
6. every dispatch chunk **partitions** the requested key set, so no key is loaded
   twice and none is missed;
7. every statement is set-based (`= ANY(...)` or the paginated page query);
8. the fixture is really the fixture — the job page's first row carries a
   non-null job with one target and one attempt, and every row of the no-job page
   is null — so a low count cannot come from empty collections.

Test:
`graphql::distribution_job_tests::the_report_statement_count_equals_the_derived_per_chunk_arithmetic`.

Nothing was changed to make a number fit: `ADR-0007`'s `200`/`10` is untouched,
no look-ahead was added, no request-scoped result store was introduced, no yield
budget was raised, no sleep or retry was added, and no unrelated loaders were
merged. The composite loader is one loader for one field family, not two loaders
joined.

### 11.2 Loaders (reconciliation)

`latestBackCatalogueJob` is backed by **one** field-specific, request-local
`ADR-0007` DataLoader:

| Property | Value |
|---|---|
| Key | `publisher_id` — the key the summary already holds at resolver entry |
| Value | `Result<Option<DistributionJobPayload>, SharedBatchError>` — the **complete** field value, job plus targets plus attempts, or `None` |
| Type | `LatestBackCatalogueJobLoader`, batcher `LatestBackCatalogueJobBatcher` |
| Construction | `configured_loader`, unchanged approved `200`/`10` |

Its batch function executes inside **one** `tokio::task::spawn_blocking`
boundary, acquiring **one** pooled Diesel connection inside that closure and
dropping it there, so no connection crosses an `.await`:

| # | Statement | Issued |
|---|---|---|
| L1 | `SELECT DISTINCT ON (publisher_id) … FROM distribution_job WHERE publisher_id = ANY($1) AND kind = 'PUBLISHER_BACK_CATALOGUE' ORDER BY publisher_id, created_at DESC, distribution_job_id DESC` | always, once per chunk |
| L2 | `… FROM distribution_job_target WHERE distribution_job_id = ANY($2) ORDER BY distribution_job_id, platform` | once per chunk, **only** when L1 returned at least one job |
| L3 | `… FROM distribution_job_attempt WHERE distribution_job_id = ANY($2) ORDER BY distribution_job_id, attempt_number DESC` | once per chunk, **only** when L1 returned at least one job |

L1 reuses the existing `latest_back_catalogue_jobs` helper unchanged, so the
`DISTINCT ON` total order and its use of
`distribution_job_publisher_latest_idx` are exactly as before. L2 and L3 are
partitioned per parent in memory, preserving canonical platform order and
newest-attempt-first order, and each job becomes a
`DistributionJobPayload::preloaded(...)`.

The loader is request-local, non-cached and dropped with the request; uses
`try_load` only; registers its key at resolver entry with no unrelated awaited
work before the call; issues set-based SQL with no per-parent loop; and is
**total** over its requested keys — every key is seeded with a successful `None`
before jobs are placed, so a publisher with no job returns the absent value
rather than a missing map entry.

A failure in **any** of L1, L2 or L3 propagates out of the blocking closure and
replaces every requested key's value with the shared error, so the chunk **fails
closed** for all of them: no per-key fallback, no partially populated job and no
successful empty substitution. A batch-wide backend failure was observed to
surface at the `latestBackCatalogueJob` field path for every key with no
successful empty data and no partially populated job in the response. The
single-closure `?` chain is what makes an L2 or L3 failure indistinguishable from
an L1 failure in this respect.

`PublisherServiceConfigurationSummary.latestBackCatalogueJob` calls the composite
loader at resolver entry, does no unrelated awaited work first, and returns the
loaded `Option<DistributionJobPayload>` **directly**. It no longer wraps the job
as lazy. `DistributionJob.targets` and `DistributionJob.attempts` therefore read
already-materialized values on the report path and issue zero SQL and zero
second-level loader calls, which is item 3 of section 11.1.

**Child selection is deliberately not part of the load shape.** Targets and
attempts are materialized whenever the field is selected at all, even for
`latestBackCatalogueJob { status }`. Deciding otherwise would require Juniper
look-ahead at the resolver, the retired `ADR-0006` mechanism. The cost is stated
rather than hidden: a status-only selection issues three loader statements
instead of one, over row sets bounded by construction — at most 17 targets from
the closed inventory and at most 5 attempts under
`distribution_job_attempt_count_check`.

**The two second-level loaders are retained, and are not unused.** They no longer
appear on the report path, and back only the single-job mutation payloads of
`completeDistributionJob`, `failDistributionJob` and `cancelDistributionJob`,
which still return lazy payloads; there the cohort is one job and the
dependent-arrival question does not arise. Their `RequestLoaders` entries carry
that as a doc comment so a later reader does not mistake them for report
machinery.

The **worker claim path is unchanged**: it resolves its own targets and attempts
set-based, does not use `RequestLoaders`, and its four-statement claim payload
contract is intact (section 8.6, re-measured green at the reconciled head).

---

## 12. Tests and checks

Run from a disposable local environment. **Nothing was pointed at production or
at any shared service.** The disposable PostgreSQL is 17.10, `UTF8`-encoded to
match CI and production — the character-boundary truncation evidence is only
meaningful under a multi-byte-aware encoding.

All figures below are the **reconciled head's**, re-run in full after the
corrections; they are not carried over from the original episode.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `git diff --check` | pass |
| `cargo check --workspace` | pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass — no warnings |
| `cargo test -p thoth-api --features backend` | pass — 1176 lib + 13 integration passed, 0 failed, 8 doc-tests ignored |
| `cargo test --workspace` | pass — 1385 passed, 0 failed, 8 ignored |
| `cargo test --workspace --release` | pass — the same 1385 passed, 0 failed, 8 ignored |
| `cargo run migrate` | pass against a disposable database |
| `cargo run migrate --revert` | pass against a disposable database |

Per-target breakdown, identical in both profiles: `thoth` lib 0, `thoth` bin 24,
`thoth-api` lib 1176, `thoth-api` integration 13, `thoth-api-server` 3,
`thoth-client` 4, `thoth-errors` 11, `thoth-export-server` 144; doc-tests
`thoth-api` 8 ignored, `thoth-client` 6 passed, `thoth-export-server` 2 passed.
The count rose from 1381 to 1385 through the four tests the reconciliation added
or split, net of the one it replaced.

**`thoth-client` test execution, as section 25.13 requires.** In `cargo test
--workspace`, `thoth-client`'s test target executed and passed **4** unit tests
(`Running unittests src/lib.rs (target/debug/deps/thoth_client-…)`, `test result:
ok. 4 passed; 0 failed`) plus **6** doc-tests (`Doc-tests thoth_client`, `test
result: ok. 6 passed; 0 failed`). In `cargo test --workspace --release`, the same
target executed and passed **4** unit tests
(`target/release/deps/thoth_client-…`) and **6** doc-tests. Both runs therefore
show `thoth-client` tests actually executing, not merely a green workspace
summary.

**`cargo test -p thoth-client` is deliberately not run as a gate**, per section
25.13 and section 34.4 of the corrected specification. It does not build at the
authorized base either — `thoth-api` does not compile with its `backend` feature
off, and only the workspace and build-dependency edges enable it — and reporting
it as `PASS` would be false while repairing it is unrelated scope under non-goal
22. The repository's own CI control is `cargo test --workspace`, required here in
both profiles. The standalone failure remains pre-existing repository
packaging/test-mode debt for a separate task. No repository control changed
between the addendum and this reconciliation that would require the standalone
command instead.

The release run is required in addition to the debug run because the
configuration-argument tests are profile-dependent, exactly as
`THOTH-GQL-OPS-02` established: in pinned `clap_builder`, an unregistered
argument panics only under `cfg(debug_assertions)`.

### 12.1 Switch registration evidence

| Item | Value |
|---|---|
| Argument | `--distribution-job-creation` |
| Environment variable | `THOTH_DISTRIBUTION_JOB_CREATION` |
| Default | `OFF`, asserted as a **declared** property of the argument on both paths |
| Parser | `value_parser(["OFF", "ON"])`, plus a typed `FromStr` accepting exactly `OFF` and `ON` |
| Registration | **both** production-capable paths: `start graphql-api` and `init` |
| Plumbing | `start_server(...)` parameter → `.app_data(Data::new(...))` → `/graphql` handler → `Context` → `ServiceConfigurationWriteContext.job_creation` |
| Reload path | none; the value is read once at process start |
| Ambient lookup | none; the coordinator takes it as an explicit parameter |
| Sweep, backfill or startup scan | **none exists** |

Both profiles were exercised. An invalid value fails parsing with
`ErrorKind::InvalidValue` on the environment surface and on both command-line
surfaces, and is never coerced to `OFF`; the rendered error leaks no value bound
to `DATABASE_URL`, `PRIVATE_KEY` or `AWS_SECRET_ACCESS_KEY`; and the merged
mutation-guard control is asserted untouched.

One test-harness correction was required and is worth naming. `clap`'s
`Arg::env` captures a variable's value when the `Arg` is **constructed**, and the
command tree is a process-wide `lazy_static`. A tree first built while a test
held a deliberately invalid value kept that value for the life of the process,
making unrelated parses fail on an argument they never mentioned. The two
control-argument test modules now share one environment lock and force the
command tree to build under an ambient environment before any test mutates it,
which removes the ordering dependency. This affected only test determinism; no
production behaviour changed.

---

## 12.2 CI

Repository CI runs on the pull request carrying this implementation and covers
the classification, changelog, format, lint, build, test and migration jobs.
Its result at the exact reviewed head is **terminal GitHub evidence** under
`ADR-0005` and is deliberately not transcribed into this file, which would be
falsified by any later run. No workflow file was changed and no workflow was
manually dispatched.

---

## 13. ADR-0008 compliance

The seven approved shared conventions are enumerated as exactly seven and are
nowhere broadened: PostgreSQL durability, explicit state machines, database
uniqueness, leases, claim tokens, deterministic idempotency, and
`FOR UPDATE SKIP LOCKED` where justified. `SKIP LOCKED` is justified on this
workload's own evidence — select a bounded batch of eligible rows, exclusively,
without blocking a concurrent worker — and the non-blocking property is measured
in section 8.5, not asserted.

No BE-04-specific mechanism is presented as additional ADR-0008-approved
cross-programme architecture. Stale-token rejection, deterministic claim
ordering, database-enforced concurrency, bounded lease semantics, the
deduplication formula, the GraphQL worker protocol and the operation lists are
BE-04's own requirements or existing `AGENTS.md` obligations.

No generic framework, universal lease, shared worker/service-role convention,
generic machine role, metrics-job abstraction, universal queue or cross-programme
identity model was introduced, asserted by source inspection across the domain
model, the crud module, `policy.rs` and the loader module. Every table, type,
enum, constant, module and role code added is named for distribution jobs and is
unusable as a generic facility. No BE-04 source file references any Metrics
surface. No ADR was created in this branch.

### 13.1 Observed-delivery boundary

**No surface, field, filter, test or sentence in this implementation infers
observed delivery, adapter execution or back-catalogue presence from
desired-state rows, from a repaired group or from the absence of a job.** A
repaired group creates no job because a repair is not a new
zero-enabled-to-enabled activation, and for no other reason. A null
`latestBackCatalogueJob` means "Thoth holds no durable back-catalogue job for
this publisher", and nothing else.

---

## 14. Rollout and rollback

Merging this pull request would authorize **repository integration only**. It
would not authorize deployment, environment or production migration execution,
worker credential provisioning, worker deployment, the `OFF -> ON` activation, a
pilot, dissemination, distribution activation, any `OBSERVE`/`ENFORCE`
transition, workflow change or dispatch, or production access. Each remains
separately gated.

The merged state is inactive: `DistributionJobCreation::Off` is the default,
the migration creates zero rows, no job exists, no worker exists and no
credential exists.

Rollback before deployment is an ordinary code revert. After deployment but
before any job exists, leaving the switch `OFF` prevents creation, and the
additive tables are empty and inert. Once durable jobs exist, job, target and
attempt history must **not** be destroyed to roll back application code: it is
operational audit evidence of what was attempted against external platforms, and
dropping a populated `distribution_job*` relation requires separate explicit
authorization. The `down.sql` migration is reversibility **evidence**, not an
automatic production rollback procedure.

---

## 15. Known limitations and deferred work

1. **No heartbeat or lease-extension operation.** A worker sizes its lease
   through `leaseSeconds`; a job that outlives its lease is recovered and
   retried, which is safe because DIS-02's worker is required to be
   at-least-once safe. If DIS-02's measured behaviour later shows a heartbeat is
   genuinely needed, that is a bounded addition under DIS-02's own specification.
2. **A terminal job cannot be reopened.** Recovery from an exhausted onboarding
   is a genuine re-activation, which mints a new `activation_id` and creates a
   new job through the same audited configuration path.
3. **`EntityNotFound` still maps to `INTERNAL_ERROR`.** This is BE-03's recorded
   pre-existing limitation, inherited rather than widened.
4. **The `zitadel setup` role-list gap is unrepaired.** `WORK_LIFECYCLE` and
   `CDN_WRITE` remain absent from that list; repairing it is a separate task.
5. **`work_id`'s `ON DELETE CASCADE` is unobservable in BE-04**, because no
   BE-04 row ever sets it. A future work-level job task must revisit that choice
   explicitly rather than inherit it silently.
6. **A status-only `latestBackCatalogueJob { status }` selection still issues
   three loader statements**, because the composite loader's shape deliberately
   does not depend on the query's selection (section 11.2). This is the accepted
   cost recorded by section 17.4.2, over row sets bounded by construction, and
   the alternative would reintroduce look-ahead.
7. **`cargo test -p thoth-client` does not build in isolation**, at this head and
   at the authorized base alike (section 12). It is not a BE-04 gate under
   section 25.13, and the standalone failure remains pre-existing repository
   packaging debt for a separate task.
8. **The specification's section 17.4 example selections write
   `targets { distributionPlatform }`** while its normative type definition in
   section 16.2 declares `platform`. The implementation follows the normative
   type definition; see section 5 item 6.
9. **An L2- or L3-specific backend failure is proven by construction rather than
   by fault injection.** The observed failure evidence induces the failure at the
   connection, which the single blocking closure propagates identically for all
   three statements through one `?` chain. There is no separate fixture that
   fails only L2 or only L3.

The previous item 6 — the report statement-count divergence from section 17.4 —
is **removed rather than rewritten**, because the corrected implementation
satisfies the corrected contract exactly, measured in section 11.1 and
reproducible across five full re-runs.

---

## 16. Unresolved issues

**None.** The statement-count divergence that was the single unresolved item at
the previous head is resolved by Correction B: derived and observed totals are
equal in all twelve measured cases, `C_job = 1` at every page size at or below
200, and the second-level loaders record zero dispatches on the report path. Stop
condition 23 did not fire, so no shared-foundation classification was required
and **no claim is made about `BE-02`'s assignment loader or any other field**.

---

## 17. Explicit confirmations

- **No dissemination, upload, feed, deposit or external platform call occurs**
  anywhere in this implementation or its tests.
- **No job exists that the tests did not create.** The migration creates zero
  rows.
- **Automatic creation is `OFF` in the merged state.**
- **BE-02's and BE-03's public contracts are unchanged**, and their existing
  tests pass; the three test files that changed are named and justified in
  section 4.
- **No deployment, no production migration execution, no production rollback, no
  role grant, no credential provisioning, no worker deployment, no
  automatic-creation activation, no pilot, no workflow dispatch and no
  production access** was performed.
- **PR #799 is untouched.**
- Merge would authorize repository integration only.

Reconciliation-specific confirmations:

- **The published history was not rewritten.** The reconciliation is an ordinary
  merge commit plus additive commits: no amend, no rebase, no squash, no
  force-push, and no second branch or pull request.
- **No specification content was edited.** `BE-04.md` on this branch is
  byte-identical to its content on `develop @ 8c0c54bd…`; `ADR-0007` and
  `ADR-0008` are untouched.
- **No GitHub metadata was mutated.** The pull request was not marked ready for
  review, its body was not edited, no reviewer was requested, no review was
  submitted, no workflow was dispatched and nothing was merged. The
  reconciliation changed the branch only; live pull-request lifecycle state is
  the GitHub record.
- **The public GraphQL contract did not change.** The generated SDL at the
  reconciled head hashes to
  `38820a24f7c1b1bac8f6ddc5286efd55dd7ece5f0155806ca4720f228ec93140`, byte-identical
  to the pre-reconciliation head recorded in section 10.2.
- **All validation ran against disposable local services.** The one database
  created for CLI migration evidence was created for that run and dropped after
  it.

---

## 18. Agent self-assessment and suggested review focus

This report makes **no approval decision**, and the implementing agent does not
declare its own work approved.

Suggested review focus, in order:

1. **The composite loader and its statement arithmetic** (sections 11.1 and
   11.2). Specifically: that the value really is the complete field, that L2 and
   L3 are skipped rather than merely cheap for an empty chunk, that the resolver
   returns a preloaded payload rather than a lazy one, that the second-level
   loaders retain a genuine mutation-payload consumer, and that the test derives
   its expectation from measured classification rather than asserting a constant.
2. **The NULL-safe `CHECK` and its truth table** (section 6.2.1). Specifically:
   that the first two rejection cases — the ones the old expression admitted —
   are present on both `INSERT` and `UPDATE`, and that each rejection is
   attributed to this constraint by name.
3. **The `OFF` fail-closed rollback** (section 7.4). It is the highest-
   consequence behaviour in the task, and the snapshot comparison is the
   evidence it rests on.
3. **The claim statement** (`model/distribution_job/crud.rs`). Whether it really
   is one atomic statement that returns exactly the rows it claimed, inserts
   exactly one attempt per claim, and cannot fan out.
4. **The `T5a`/`T5b` split** and the three independent guards against a sixth
   attempt.
5. **The authorization matrix**, particularly the `SUPERUSER`-denied rows and
   the absence of any composition rule.
6. **The migration's locking evidence** (section 6.5), and whether the
   disposable-environment figures are being read as what they are.
7. The three modified pre-existing test files (section 4), to confirm that each
   change preserves the original assertion's intent rather than relaxing it.
