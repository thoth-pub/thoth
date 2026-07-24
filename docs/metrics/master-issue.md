# Master GitHub Issue - Thoth Metrics

Suggested title:

```text
Thoth Metrics: canonical ingestion, Sphinx orchestration and client cutover
```

Suggested labels:

```text
programme
metrics
engineering-control
```

## Objective

Make Thoth the canonical datastore and API for usage and sales metrics, with restartable Sphinx collection, publisher imports, coverage-aware rollups, protected dashboard/widget queries, OPERAS synchronization and reconciled historical migration.

## Authoritative control

- [README](./README.md)
- [Decisions](./decisions.md)
- [Task tracker](./task-status.md)
- [Source inventory](./source-inventory.md)
- [Contract register](./contract-register.md)
- [Migration inventory](./migration-inventory.md)
- [Acceptance](./acceptance-matrix.md)
- [Rollout](./rollout-plan.md)

## Current gate

- [ ] MET-CTRL-01 merged
- [ ] ADR-0001 approved
- [ ] ADR-0002 approved
- [ ] SPHINX-BOOT-01 complete
- [ ] THOTH-DB-CTRL-01 complete
- [ ] branch decisions recorded
- [ ] service-role codes approved before WP5

## Work

- [ ] WP1 Domain/database
- [ ] WP2 Ingestion
- [ ] WP3 Imports/UI
- [ ] WP4 Rollups/GraphQL
- [ ] WP5 Auth/entitlements
- [ ] WP6 Sphinx core
- [ ] WP7 CloudFront
- [ ] WP8 Other drivers/COUNTER
- [ ] WP9 OPERAS/reconciliation
- [ ] WP10 Clients
- [ ] WP11 Deployment/migration
- [ ] MET-E2E-01

## Dependencies

- [ ] representative source files
- [ ] regenerated reports
- [ ] COUNTER examples/selection
- [ ] CloudFront resources/decision
- [ ] OPERAS cursor/snapshot/replication
- [ ] URI mappings
- [ ] source retention/finalization

For every task record repository, risk, specification, base/integration/slice branches, PR, contracts, fixtures, implementation/reviewer, evidence, migration, rollout and rollback.

Do not close on code or CI alone. Close after independent approval, merge, required rollout, reconciliation and tracker update.
