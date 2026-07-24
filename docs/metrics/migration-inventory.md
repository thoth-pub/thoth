# Thoth Metrics Historical Migration Inventory

Status: TEMPLATE; DISCOVERY REQUIRED  
Owner: Thoth Metrics programme  
Production migration risk: CRITICAL

## 1. Source table

| Source account | Platform | Measure | Earliest | Latest | Grain | Dimensions | Raw retained | Generated retained | Stable IDs | Revisions | OPERAS mapping | Direct | Issues | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---:|---|---|
| TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | NOT INVENTORIED |

## 2. CloudFront

Inventory original protected logs, object metadata, DOI rule history, bot/GeoIP versions, legacy outputs and known inflation. Recompute deterministically with `cloudfront-title-session/2`. Do not import legacy generated CSV totals as canonical.

## 3. Platform reports

For each source retain raw input and checksum, report ID, source account, format/version, coverage, regenerated-report handling, row counts, errors and rerun idempotency.

## 4. OPERAS

Record complete snapshot/cursor if available, remote IDs, uploader/event/measure URIs, hashes, links to known Thoth exports, direct-collection exclusion, unresolved mappings and completeness status. Legacy submission ledgers are provenance, not proof of complete remote state.

## 5. Sequence

1. Approve registries/mappings.
2. Deploy schema inactive.
3. Import retained source reports.
4. Recompute CloudFront.
5. Import legacy ledgers as provenance.
6. Import complete OPERAS snapshot only for non-direct mappings.
7. Build rollups.
8. Reconcile all layers.
9. Compare old/new clients.
10. Activate sources.
11. Cut clients over.
12. Stop legacy submissions after observation.

## 6. Dry run

Produce input checksums, versions, expected imports/rows, accepted/duplicate/revision/conflict/invalid/overlap/unresolved counts, coverage, rollup delta, export count, reconciliation issues, no-write proof and rerun comparison.

## 7. Stop conditions

Stop on incomplete inventory, ambiguous ownership/identity, unapproved mappings, unexplained overlap, secret/personal-data exposure, unexpected export, unreconciled counts, dry-run mismatch or missing backup/restore readiness.

## 8. Rollback

Disable accounts/claims/serving/export, use audited retractions/repair, rebuild rollups and return clients to legacy reads. Do not manually delete canonical history.
