# Publisher Services Task Status

Status: ACTIVE TRACKER  
Programme owner: CTO  
Master issue: NOT YET CREATED  
Last updated: 2026-07-24

## 1. Control rule

No task moves to `READY` without:

- approved written specification;
- approved architecture dependencies;
- verified repository and base;
- branch-readiness dependency complete or explicit CTO exception;
- named implementation and independent-review models;
- test, migration, rollout and rollback requirements.

`TBD` is missing work, not an implicit approval.

## 2. Task tracker

| Task | Repository | Risk | Current status | Required base / PR target | Blocking dependencies | Issue | PR | Acceptance |
|---|---|---:|---|---|---|---|---|---|
| P0-01 Control documents and tracker | `thoth` | LOW | IN PROGRESS | `develop` / `develop` | Independent review; master issue; PR #764 merge | TBD | #764 | PENDING |
| ADR-01 Platform inventory and final architecture | `thoth` | MEDIUM | BLOCKED | `develop` / `develop` | P0-01; ADR-0002 approval | TBD | TBD | NOT STARTED |
| LIC-01 Expand `cc-license` | `cc-license` | MEDIUM | BLOCKED | actual `develop`; programme target after branch-readiness decision | P0-01; branch readiness `BR-LIC-01`; approved LIC-01 spec | TBD | TBD | NOT STARTED |
| LIC-02 Enforce supported licences | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | LIC-01 release; production licence audit plan | TBD | TBD | NOT STARTED |
| BE-01 Publisher package model | `thoth` | MEDIUM | BLOCKED | Publisher Services integration branch | P0-01; ADR-0001 approval | TBD | TBD | NOT STARTED |
| BE-02 Distribution platform model | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | ADR-01; ADR-0002 approval | TBD | TBD | NOT STARTED |
| BE-03 Protected service configuration | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | BE-01; BE-02 | TBD | TBD | NOT STARTED |
| BE-04 Durable distribution jobs | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | BE-02; BE-03 | TBD | TBD | NOT STARTED |
| MIG-01 Audit and production backfill | `thoth` plus operations | CRITICAL | BLOCKED | dedicated migration branch/PR and separately approved production run | BE-01; BE-02; BE-03; licence audit; dry-run tooling | TBD | TBD | NOT STARTED |
| APP-01 Service configuration UI | `thoth-app` | MEDIUM | BLOCKED | actual `dev`; programme target after `BR-APP-01` | BE-03; branch readiness; generated API contract | TBD | TBD | NOT STARTED |
| APP-02 Staff subscription report | `thoth-app` | MEDIUM | BLOCKED | actual `dev`; programme target after `BR-APP-01` | BE-03; BE-04; APP-01 foundation | TBD | TBD | NOT STARTED |
| APP-03 API-backed licence options | `thoth-app` | MEDIUM | BLOCKED | actual `dev`; programme target after `BR-APP-01` | LIC-02; branch readiness | TBD | TBD | NOT STARTED |
| DIS-01 API discovery and comparison mode | `thoth-dissemination` | HIGH | BLOCKED | actual `develop`; programme target after `BR-DIS-01` | BE-02; MIG-01 approved backfill; branch readiness | TBD | TBD | NOT STARTED |
| DIS-02 Back-catalogue job worker | `thoth-dissemination` | CRITICAL | BLOCKED | Publisher Services integration branch | BE-04; DIS-01 clean comparison; production controls | TBD | TBD | NOT STARTED |
| EXP-01 OCLC KBART feed index | `thoth` | MEDIUM | BLOCKED | Publisher Services integration branch | BE-02; ADR-01 OCLC decision | TBD | TBD | NOT STARTED |
| OAI-01 Package and licence gating | `thoth` | HIGH | BLOCKED | fresh branch after divergence assessment | BE-01; LIC-02; ADR-0001; branch assessment | TBD | TBD | NOT STARTED |
| OPS-01 Monitoring, runbooks and cleanup | multiple | HIGH | BLOCKED | repository-local tasks | Implemented components; production ownership map | TBD | TBD | NOT STARTED |
| E2E-01 Full workflow verification | multiple | HIGH | BLOCKED | no implementation branch; controlled acceptance run | all activated components; pilot data; rollback | TBD | TBD | NOT STARTED |

## 3. Branch preparation

Before programme implementation:

### `thoth`

Current topology conforms:

```text
develop -> feature/publisher-services -> develop -> master
```

Create the integration branch only after P0-01 and required ADRs are approved.

### `thoth-app`

Current topology is `dev -> main`.

Complete `BR-APP-01` or obtain an explicit CTO exception before creating a long-lived Publisher Services integration branch.

### `thoth-dissemination`

Current topology is `develop -> main`.

Complete `BR-DIS-01` or record the approved temporary programme-branch strategy.

### `cc-license`

Current topology is `develop -> main`.

Complete `BR-LIC-01` or record the approved temporary programme-branch strategy.

## 4. Recommended implementation models

| Risk | Implementation | Independent review |
|---|---|---|
| LOW | Codex medium reasoning | separate Codex/Claude medium |
| MEDIUM | Codex medium/high | separate Claude high |
| HIGH | Codex high | separate Claude high plus control review |
| CRITICAL | Codex high with tightly bounded permissions | separate Claude high, control review and explicit CTO activation approval |

The implementing agent never approves its own work.

## 5. Next status actions

1. Create the master GitHub issue from `master-issue.md`.
2. Record its number here.
3. Complete PR #764.
4. Approve or amend ADR-0001 and ADR-0002.
5. Merge the engineering-control foundation.
6. Scope and begin ADR-01 only.
