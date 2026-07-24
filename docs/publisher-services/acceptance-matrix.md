# Publisher Services Acceptance Matrix

Status: ACTIVE CONTROL MATRIX
Owner: CTO

This matrix records the minimum evidence required for programme acceptance. Task specifications may add stricter criteria.

## 1. Programme-level requirements

| Requirement | Owning task(s) | Required evidence | Activation gate |
|---|---|---|---|
| Every publisher has one non-null package | BE-01, MIG-01 | migration tests on empty/populated DB; production dry-run counts; post-backfill query | schema/backfill approval |
| OASIS is the default | BE-01 | existing/new publisher tests | backend merge |
| Package does not imply platforms | BE-01, BE-03 | unit/integration tests showing unchanged assignments/jobs | backend merge |
| Publisher reads own package/platforms only | BE-03, APP-01 | anonymous, wrong-org, correct-org, superuser tests | API/UI activation |
| Only superusers change configuration | BE-03, APP-01 | backend authorization tests; UI control absence | API/UI activation |
| Linked OAPEN/DOAB normalized server-side | ADR-01, BE-02, BE-03 | one-sided state impossible; one logical activation identity | backend merge |
| One linked activation creates one logical job | BE-04 | transactional job/target tests | worker pilot |
| Pull/manual destinations create no upload job | BE-02, BE-04 | exhaustive descriptor/job tests | backend merge |
| Configuration changes are audited atomically | BE-03 | transaction tests including failure rollback | backend merge |
| Claims are exclusive and leased | BE-04 | concurrency and stale-token tests | worker deployment |
| Legacy assignments backfill without jobs | MIG-01 | reviewed dry run; repeat run no-op; job count unchanged | production backfill |
| Dissemination API mode fails closed | DIS-01 | API failure and empty-list tests | comparison activation |
| Comparison mode is clean before cutover | DIS-01 | publisher/platform diff report reviewed and signed off | API source cutover |
| Back-catalogue worker is at-least-once safe | DIS-02 | retry/idempotency/concurrency tests; controlled pilot | general job creation |
| OCLC index reflects enabled publishers | EXP-01 | JSON/text equivalence, deterministic order, cache tests | public endpoint |
| Licence options come from `cc-license` | LIC-01, LIC-02, APP-03 | canonicalization/spoof tests; all write paths; generated client | strict enforcement |
| OAI filtering occurs before pagination | OAI-01 | GetRecord/List*/count/token matrix | OAI activation |
| OASIS/non-open works never leak to OAI | OAI-01 | complete package/licence/lifecycle matrix | OAI activation |
| Monitoring and rollback are operational | OPS-01 | dashboards, alerts, runbooks, named owners, exercise evidence | production enablement |
| Full flow works end to end | E2E-01 | configuration -> job -> adapter -> result/report evidence | cleanup |
| Legacy configuration retained through observation | DIS-01, DIS-02, OPS-01 | observation record and explicit cleanup approval | cleanup |

## 2. Cross-cutting technical evidence

Every applicable task must provide:

### Authorization

- anonymous;
- authenticated without permission;
- another publisher's scope;
- correct publisher scope;
- superuser;
- worker/service role;
- stale or invalid credentials.

### Migrations

- forward migration;
- revert or approved forward repair;
- empty database;
- representative populated database;
- defaults and nullability;
- constraints and indexes;
- locking/downtime assessment;
- generated Diesel schema;
- deployment order.

### Pagination and reports

- deterministic ordering;
- bounded page size;
- filters applied before pagination;
- counts match filtered results;
- CSV respects the same filters;
- no protected package leakage.

### Jobs and workflows

- duplicate activation;
- retry;
- stale lease;
- overlapping workers;
- partial adapter failure;
- empty scope;
- unknown platform;
- pull/manual platform;
- linked multi-target platform;
- sanitized bounded error storage.

### Compatibility

- GraphQL schema;
- internal Rust client;
- `thoth-app` generated client;
- dissemination mapping;
- export formats;
- OAI tokens and counts;
- current publisher configuration.

## 3. Definition of programme done

Publisher Services is not done until:

1. all required tasks are independently approved and merged;
2. production backfill is reconciled;
3. comparison mode is clean;
4. one controlled automatic-push pilot succeeds;
5. monitoring and rollback are verified;
6. E2E-01 passes;
7. the observation period closes without unresolved P0/P1 findings;
8. cleanup receives explicit CTO approval;
9. task tracker and repository status are updated.
