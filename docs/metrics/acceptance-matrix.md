# Thoth Metrics Acceptance Matrix

Status: ACTIVE CONTROL MATRIX
Owner: CTO

| Requirement | Work | Evidence | Gate |
|---|---|---|---|
| Thoth sole canonical store | WP1/WP2 | schema/API review; no direct Sphinx DB writes | ingestion |
| Stable registries | WP1 | migration/seed tests | configuration |
| Work/publication/ROR resolution | WP2 | valid/unknown/ambiguous tests | ingestion |
| Value rules | WP1/WP2 | measure-specific tests | ingestion |
| Deterministic duplicates/revisions/conflicts | WP2 | concurrency/transaction tests | ingestion |
| No overlapping double count | WP1/WP2 | exclusion/lock race tests | ingestion |
| Publisher finality | WP2/WP3 | changed-value conflict tests | imports |
| Every row auditable | WP2 | provenance/error parity | ingestion |
| Zero vs unknown coverage | WP1/WP4 | coverage fixtures | client cutover |
| Rollups rebuildable | WP1/WP4 | delta/revision/rebuild tests | serving |
| Dashboard p95 <1s | WP4 | 600-work benchmark | dashboard |
| Useful view <2s | WP10 | browser/server timing | dashboard |
| Least-privilege service auth | WP5 | role matrix | serving |
| No browser secret | WP5/WP10 | bundle/source inspection | clients |
| Entitlements enforced | WP5/WP9 | capability tests | serve/export |
| CloudFront sessions correct | WP7 | golden adjacent-day fixtures | driver |
| No IP/user-agent persisted | WP7/WP11 | artifact/schema/log inspection | driver |
| Restartable/idempotent runs | WP6-WP9 | lease/retry/overlap tests | schedules |
| OPERAS outbound idempotent | WP9 | retry/hash tests | export |
| No inbound loops | WP9 | direct/exported tests | inbound |
| Honest inbound completeness | WP9 | cursor/snapshot or unverified tests | inbound |
| Old/new comparison | WP10/WP11 | publisher/period reports | cutover |
| Migration reconciled | WP11 | source/canonical/rollup/OPERAS report | shutdown |
| Operational support | WP11 | alerts/runbooks/restore/rebuild exercise | production |

Programme completion additionally requires integrated acceptance, migration reconciliation, performance, monitoring, rollback, observation and separately approved cleanup.
