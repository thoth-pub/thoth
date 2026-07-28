# ADR-0002-APPROVE Implementation Report

## 1. Repository and delivery record

Repository: `thoth-pub/thoth`
Original task: `ADR-0002-APPROVE`
Original base: `f2e09bd9b138e8ba2ca47a791533f4aae4ffab28`
Reviewed head: `78307578f680050581f1a4e16a9668d9dfcc037a`
Original PR: [#769](https://github.com/thoth-pub/thoth/pull/769)
Merge commit: `e124221f8444bd738228f1b609c536639be8789e`
Merged: 2026-07-28T09:24:58Z

The original PR recorded the CTO approval of ADR-0002 as written. The ADR body
was not amended; only approval metadata and dependent control records changed.

## 2. CTO approval

> I approve ADR-0002 - Distribution and Metrics Platform Domain Boundaries as
> written.
> Do not amend the architectural decision.

Approved by: Javi, CTO
Approval date: 2026-07-27

## 3. Original commits and changed-file boundary

Original commits:

- `ddf635fd...` - task specification
- `7a82680c...` - approval metadata and control reconciliation
- `f4ef99c97c21f86d0dccd29081a450e3bdf4ce54` - initial evidence report
- `78307578f680050581f1a4e16a9668d9dfcc037a` - pre-review evidence correction

The cumulative original PR changed exactly 14 allowlisted documentation/control
files. No Rust, SQL, migration, GraphQL, workflow, generated-code, deployment or
repository-protection file changed. There was no runtime or production effect.

## 4. ADR integrity

ADR-0002 changed only as follows:

- `Status: PROPOSED` to `Status: APPROVED`;
- `Approved by: Javi, CTO`;
- `Approval date: 2026-07-27`;
- approval note confirming separate `DistributionPlatform` and `MetricPlatform`
  domains and no initial cross-domain mapping.

Sections 1-9 remained unchanged. ADR-0001 remained `PROPOSED`; Publisher Services
ADR-01 and the final platform inventory remained unapproved; `MET-CTRL-01`
remained `CHANGES REQUIRED`; no implementation task became `READY`.

## 5. Exact-head CI

At reviewed head `78307578f680050581f1a4e16a9668d9dfcc037a`:

```text
30342715466 - build-test-and-check
  build: success
  lint: success
  test: success
  format_check: success
30342715472 - run-migrations
  run_migrations: success
30342715538 - check-changelog
  check-changelog: success
30342715481 - publish-to-dockerhub
  build_and_push_staging_docker_image: success
```

## 6. Independent review and post-merge findings

A fresh independent reviewer returned `APPROVED` at the reviewed head with no
P0, P1 or P2 finding before merge. After the PR was marked ready, an automated
Codex review completed after the merge and raised three actionable P1 findings:

1. the issue proposals below were abbreviated rather than embedded completely;
2. the active agent-instruction rollout plan still described issue #765
   synchronization as outstanding;
3. embedded diff context contained trailing whitespace despite the report saying
   `git diff --check` was clean.

Task `ADR-0002-POST-MERGE-CORRECTION` and PR #770 correct those evidence and
control-record defects. This report has no trailing whitespace.

## 7. Live issue baselines

This task did not write either issue.

```text
Issue #765
state: OPEN
baseline updatedAt: 2026-07-27T15:50:33Z
complete-body sha256: 96c31089a3046eadf51a0fc39b12d0275ce26f4d752c64282f5dcb933f78ca15
proposed-body sha256: da12243b2a1898fd3fd574aada1dede3296ff13f38943e4fbb78a3dcb5ae1a35

Issue #766
state: OPEN
baseline updatedAt: 2026-07-24T17:17:11Z
complete-body sha256: 6b1bb092f3f0b436c01faaabbf4fb5df331268f4d687463b3c715fb4ea9d6dbc
proposed-body sha256: f4e8aa7e855b2b3c44b4cf38c60475861079698cc7f5cd95a6ac319b892cb772
```

## 8. Exact proposed body for issue #765

The complete proposed replacement body is:

```markdown
## Objective

Implement the approved Publisher Services and Distribution Configuration design across Thoth, thoth-app, thoth-dissemination and cc-license with additive schema, explicit authorization, audited migration, comparison-mode cutover, bounded pilots, monitoring and rollback.

## Immutable authority at foundation review head

- [Private approved design](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit) - Drive revision `3`
- [Private design reference metadata](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/design-references.md#publisher-services-and-distribution-configuration)
- [Programme README](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/README.md)
- [Task tracker](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/task-status.md)
- [Platform inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/platform-inventory.md)
- [Acceptance matrix](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/acceptance-matrix.md)
- [Rollout plan](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/rollout-plan.md)
- [Foundation specification](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md)
- [Foundation implementation report](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md)
- [Foundation PR #764](https://github.com/thoth-pub/thoth/pull/764)
- [P0-01 closeout PR #767](https://github.com/thoth-pub/thoth/pull/767)
- [P0-01 finalization PR #768](https://github.com/thoth-pub/thoth/pull/768)

The Publisher Services design requires one fresh task branch and one PR per task. There is no long-lived `feature/publisher-services` integration branch.

## Synchronization guard

Before applying this replacement: re-fetch the complete live issue body; re-fetch its current `updatedAt`; compare both exactly against the reviewed baseline `updatedAt: 2026-07-27T15:50:33Z` and body. If either the live body or `updatedAt` differs, do not write. Regenerate the minimal diff from the new live body, obtain fresh independent review, and obtain separate explicit CTO authorization before writing. Any later rollback must likewise re-fetch and compare the live body and `updatedAt`, preserve unrelated edits, and apply only a reviewed minimal reversal under explicit CTO authorization; it must never restore an old complete snapshot blindly.

## Current gate

- [x] P0-01 independently approved, repository-finalized and merged
- [ ] ADR-0001 approved
- [x] ADR-0002 approved
- [ ] ADR-01 platform inventory approved
- [ ] repository branch-readiness decisions recorded

No production implementation begins before the applicable gate passes.

## Tasks

### Foundation

- [x] P0-01 - Project control documents and tracker - CLOSED
- [ ] ADR-01 - Platform inventory and final architecture
- [ ] LIC-01 - Expand cc-license
- [ ] LIC-02 - Enforce supported licences in Thoth

### Backend

- [ ] BE-01 - Publisher package model
- [ ] BE-02 - Distribution platform model
- [ ] BE-03 - Protected service configuration
- [ ] BE-04 - Durable distribution jobs

### Migration and interfaces

- [ ] MIG-01 - Audit and production backfill
- [ ] APP-01 - Publisher service configuration UI
- [ ] APP-02 - Staff subscription report
- [ ] APP-03 - API-backed licence options

### Cutover and downstream services

- [ ] DIS-01 - API publisher discovery and comparison mode
- [ ] DIS-02 - Back-catalogue job worker
- [ ] EXP-01 - OCLC KBART feed index
- [ ] OAI-01 - Package and licence gating

### Stabilization

- [ ] OPS-01 - Monitoring, runbooks and cleanup
- [ ] E2E-01 - Full workflow verification

P0-01 closure records completion of the engineering-control foundation only. It does not approve an ADR, approve the final inventory, satisfy branch readiness, or make another task ready.

Do not close a task at PR creation or CI success. Close only after independent approval, merge, required rollout/observation and repository tracker update.
```

## 9. Exact proposed body for issue #766

The complete proposed replacement body is:

```markdown
## Objective

Make Thoth the canonical datastore and API for usage and sales metrics, with restartable Sphinx collection, publisher imports, coverage-aware rollups, protected dashboard/widget queries, OPERAS synchronization and reconciled historical migration.

## Immutable authority at foundation review head

- [Private approved design](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit) - Drive revision `6`
- [Private design reference metadata](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/design-references.md#thoth-metrics)
- [Programme README](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/README.md)
- [Task tracker](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/task-status.md)
- [Source inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/source-inventory.md)
- [Contract register](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/contract-register.md)
- [Migration inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/migration-inventory.md)
- [Acceptance matrix](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/acceptance-matrix.md)
- [Rollout plan](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/rollout-plan.md)
- [Foundation specification](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md)
- [Foundation implementation report](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md)
- [Foundation PR #764](https://github.com/thoth-pub/thoth/pull/764)

The Metrics design uses repository-local `feature/metrics` integration branches only after each repository's branch-readiness gate.

## Synchronization guard

Before applying this replacement: re-fetch the complete live issue body; re-fetch its current `updatedAt`; compare both exactly against the reviewed baseline `updatedAt: 2026-07-24T17:17:11Z` and body. If either the live body or `updatedAt` differs, do not write. Regenerate the minimal diff from the new live body, obtain fresh independent review, and obtain separate explicit CTO authorization before writing. Any later rollback must likewise re-fetch and compare the live body and `updatedAt`, preserve unrelated edits, and apply only a reviewed minimal reversal under explicit CTO authorization; it must never restore an old complete snapshot blindly.

## Current gate

- [ ] MET-CTRL-01 independently approved and merged
- [ ] ADR-0001 approved
- [x] ADR-0002 approved
- [ ] BR-SPHINX-01 complete
- [ ] SPHINX-BOOT-01 complete
- [ ] THOTH-DB-CTRL-01 complete
- [ ] client repository branch-readiness decisions recorded
- [ ] service-role codes approved before WP5

## Work packages

- [ ] WP1 - Metrics domain and database foundation
- [ ] WP2 - Canonical ingestion service
- [ ] WP3 - Import upload and thoth-app experience
- [ ] WP4 - Rollups and dashboard GraphQL
- [ ] WP5 - Service authentication and entitlements
- [ ] WP6 - Sphinx core
- [ ] WP7 - CloudFront driver
- [ ] WP8 - Additional platform drivers and COUNTER
- [ ] WP9 - OPERAS adapter and reconciliation
- [ ] WP10 - Dashboard and widget clients
- [ ] WP11 - Deployment, monitoring and migration
- [ ] MET-E2E-01 - Integrated acceptance and cutover

Do not close at code completion or CI success. Close only after independent approval, merge, required rollout, reconciliation and tracker update.




```

## 10. Deferred-write controls

The embedded bodies are evidence only. They do not authorize a write. Each issue
write still requires, in order:

1. immediate re-fetch of the complete live body and `updatedAt`;
2. exact comparison with the reviewed baseline;
3. stop on any mismatch;
4. regeneration and fresh independent review of any changed proposal;
5. separate explicit CTO authorization;
6. a minimal write that keeps the issue open.

Rollback must likewise re-fetch and compare, preserve unrelated edits, and apply
only a reviewed minimal reversal. A blind full-body restoration is prohibited.

## 11. Residual blockers

```text
ADR-0001: PROPOSED
Publisher Services ADR-01: unapproved
Platform inventory: VERIFIED BASELINE; FINAL ENUM NOT APPROVED
MET-CTRL-01: CHANGES REQUIRED
All Publisher Services implementation tasks: BLOCKED
All Metrics work packages: BLOCKED
Branch-readiness decisions: outstanding
```

ADR-0002 approval removes exactly one dependency. No implementation task becomes
`READY`.
