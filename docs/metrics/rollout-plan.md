# Thoth Metrics Rollout Plan

Status: PROPOSED CONTROLLED SEQUENCE  
Owner: CTO  
Production activation: explicit CTO approval required

## 1. Principles

Add schema before behavior, keep accounts/schedules/serving/export disabled initially, version mappings, compare before cutover, never dual-write canonical stores, reconcile all layers, retain rollback and never turn missing coverage into zero.

## 2. Stage 0 - Control/readiness

Require PR #764, approved ADRs, master issue, Sphinx bootstrap, branch decisions, Diesel procedure and role decision before WP5.

## 3. Stage 1 - Canonical schema

Deliver WP1 slices additively. Registries/accounts/routes/export stay disabled. Prove migrations, constraints, seeds, rollback and query baseline.

## 4. Stage 2 - Ingestion

Deliver WP2 against fixtures and disposable environments. Use bounded batches, central resolution, transactional identity/overlap handling, provenance and no production source credentials.

## 5. Stage 3 - Rollups/protected queries

Deliver WP4/WP5 behind inactive flags. Prove correctness, p95, coverage, authorization, capability checks and rebuild.

## 6. Stage 4 - Sphinx core

Deliver WP6 after bootstrap. No local durable DB, no production credentials, no-op driver first, overlapping/restart tests and pinned API contract.

## 7. Stage 5 - CloudFront pilot

Use one source account and bounded day with adjacent context. Dry-run, compare golden results, ingest non-production, verify coverage/rollup, recompute a small window, review legacy divergence, then enable collection without serving/export. Stop on personal-data leakage or unexplained differences.

## 8. Stage 6 - Publisher import pilot

Use one entitled publisher and approved platform. Verify private upload, ownership, approval, partial errors, duplicate retry, changed-value conflict and error download. Keep OPERAS export off.

## 9. Stage 7 - Additional sources

Enable one source account at a time only after its fixture/mapping gate. COUNTER support must not approximate unsupported metrics.

## 10. Stage 8 - OPERAS

Pilot one mapping/publisher/date scope. Inbound remains disabled without complete discovery or explicitly unverified. Historical export is never an upgrade side effect.

## 11. Stage 9 - Client comparison/cutover

Compare selected publishers/periods/measures/dimensions/coverage and known legacy divergence. A read fallback may exist temporarily; never write to two canonical stores.

## 12. Stage 10 - Migration/operations

Use the migration inventory, versioned normalizers, CloudFront recomputation, no accidental export, restore readiness, monitoring, runbooks and reconciliation. Production requires CTO approval.

## 13. Observation/cleanup

Observe freshness, errors, unresolved identifiers, rollup lag, export backlog, reconciliation, latency, coverage and late changes. Cleanup of legacy paths requires MET-E2E-01, stable observation, independent review and CTO approval.
