# Independent Review Brief - CTRL-FOUNDATION-01

Review status: READY AFTER FINAL CONSOLIDATION COMMIT
Task: `CTRL-FOUNDATION-01`
Repository: `thoth-pub/thoth`
Pull request: [#764](https://github.com/thoth-pub/thoth/pull/764)
Base: `develop` at `652a499dfdfbaa7594537e0865c41ec617f52dc2`
Risk: LOW, documentation-only
Production effect: None

## Reviewer independence

The reviewer must not be the ChatGPT conversation that authored these documents. Use a separate Codex or Claude instance at high reasoning.

## Required inputs

PR #764 final head/diff/CI; CTRL-FOUNDATION-01; approved Publisher Services and Metrics designs; repository map; ADR-0001/0002; issues #765/#766.

## Review objectives

Verify authority, task/review/release controls, proposed-decision status, programme separation, repository accuracy, canonical naming, agent safety, no production effects, complete trackers/issues and explicit blockers.

## Required commands

```bash
git diff --check
grep -Rni "sphynx" AGENTS.md .github thoth-* docs || true
grep -Rni "NOT YET CREATED" docs || true
git diff --name-only 652a499dfdfbaa7594537e0865c41ec617f52dc2...HEAD
git diff --stat 652a499dfdfbaa7594537e0865c41ec617f52dc2...HEAD
```

Confirm only CHANGELOG, AGENTS documentation and docs changed; no workflow behavior, Rust, SQL, migration, package or deployment files changed; CI green; master issues exist; ADRs remain proposed; acceptance checkboxes match evidence.

## Red-team questions

1. Could any document be read as permission to merge/deploy/access secrets?
2. Do task/PR rules conflict with programme branches?
3. Are missing normalized branches assumed?
4. Are package business decisions silently settled?
5. Is OPERAS inbound completeness overstated?
6. Is the Publisher Services inventory presented as approved?
7. Are package and platform assignments independent?
8. Are distribution and metrics platforms separate?
9. Are missing CI/deployment/schema-generation controls explicit?
10. Are links valid after merge?

## Decision format

Return exactly one: `APPROVED`, `CHANGES REQUIRED`, or `BLOCKED`.

For each finding provide severity, file/line, requirement, evidence and required change.

Approval requires no unresolved P0/P1, complete evidence, green final CI and no unreviewed head movement. The reviewer must not merge.
