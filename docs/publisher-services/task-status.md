# Publisher Services Task Status

Status: ACTIVE TRACKER  
Programme owner: CTO  
Master issue: [#765](https://github.com/thoth-pub/thoth/issues/765)  
Last updated: 2026-07-24

## 1. Control rule

No task moves to `READY` without an approved written specification, approved architecture dependencies, verified repository/base, branch-readiness completion or explicit CTO exception, named implementation and independent-review models, and explicit test, migration, rollout and rollback requirements.

`TBD` is missing work, not implicit approval.

## 2. Task tracker

| Task | Repository | Risk | Current status | Required base / PR target | Blocking dependencies | Issue | PR | Acceptance |
|---|---|---:|---|---|---|---|---|---|
| P0-01 Control documents and tracker | `thoth` | LOW | IN PROGRESS | `develop` / `develop` | Independent review; PR #764 merge | [#765](https://github.com/thoth-pub/thoth/issues/765) | [#764](https://github.com/thoth-pub/thoth/pull/764) | PENDING REVIEW |
| ADR-01 Platform inventory and final architecture | `thoth` | MEDIUM | BLOCKED | `develop` / `develop` | P0-01; ADR-0002 approval | #765 | TBD | NOT STARTED |
| LIC-01 Expand `cc-license` | `cc-license` | MEDIUM | BLOCKED | actual `develop`; target after branch-readiness decision | P0-01; BR-LIC-01; approved specification | #765 | TBD | NOT STARTED |
| LIC-02 Enforce supported licences | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | LIC-01 release; production licence audit plan | #765 | TBD | NOT STARTED |
| BE-01 Publisher package model | `thoth` | MEDIUM | BLOCKED | Publisher Services integration branch | P0-01; ADR-0001 approval | #765 | TBD | NOT STARTED |
| BE-02 Distribution platform model | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | ADR-01; ADR-0002 approval | #765 | TBD | NOT STARTED |
| BE-03 Protected service configuration | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | BE-01; BE-02 | #765 | TBD | NOT STARTED |
| BE-04 Durable distribution jobs | `thoth` | HIGH | BLOCKED | Publisher Services integration branch | BE-02; BE-03 | #765 | TBD | NOT STARTED |
| MIG-01 Audit and production backfill | `thoth` plus operations | CRITICAL | BLOCKED | dedicated migration branch/PR and separate production approval | BE-01; BE-02; BE-03; licence audit; dry run | #765 | TBD | NOT STARTED |
| APP-01 Service configuration UI | `thoth-app` | MEDIUM | BLOCKED | actual `dev`; target after BR-APP-01 | BE-03; generated API contract | #765 | TBD | NOT STARTED |
| APP-02 Staff subscription report | `thoth-app` | MEDIUM | BLOCKED | actual `dev`; target after BR-APP-01 | BE-03; BE-04; APP-01 | #765 | TBD | NOT STARTED |
| APP-03 API-backed licence options | `thoth-app` | MEDIUM | BLOCKED | actual `dev`; target after BR-APP-01 | LIC-02 | #765 | TBD | NOT STARTED |
| DIS-01 API discovery and comparison mode | `thoth-dissemination` | HIGH | BLOCKED | actual `develop`; target after BR-DIS-01 | BE-02; MIG-01; branch readiness | #765 | TBD | NOT STARTED |
| DIS-02 Back-catalogue job worker | `thoth-dissemination` | CRITICAL | BLOCKED | Publisher Services integration branch | BE-04; DIS-01 clean comparison; production controls | #765 | TBD | NOT STARTED |
| EXP-01 OCLC KBART feed index | `thoth` | MEDIUM | BLOCKED | Publisher Services integration branch | BE-02; ADR-01 OCLC decision | #765 | TBD | NOT STARTED |
| OAI-01 Package and licence gating | `thoth` | HIGH | BLOCKED | fresh branch after divergence assessment | BE-01; LIC-02; ADR-0001; branch assessment | #765 | TBD | NOT STARTED |
| OPS-01 Monitoring, runbooks and cleanup | multiple | HIGH | BLOCKED | repository-local tasks | implemented components; runtime ownership map | #765 | TBD | NOT STARTED |
| E2E-01 Full workflow verification | multiple | HIGH | BLOCKED | controlled acceptance run | all activated components; pilot data; rollback | #765 | TBD | NOT STARTED |

## 3. Branch preparation

- `thoth`: current `develop -> master` topology conforms. Create `feature/publisher-services` only after P0-01 and required ADRs.
- `thoth-app`: current `dev -> main`; complete BR-APP-01 or approve an exception.
- `thoth-dissemination`: current `develop -> main`; complete BR-DIS-01 or approve the programme-branch strategy.
- `cc-license`: current `develop -> main`; complete BR-LIC-01 or approve the programme-branch strategy.

## 4. Next status actions

1. Independently review PR #764.
2. Resolve review findings.
3. Approve or amend ADR-0001 and ADR-0002 through the CTO decision gate.
4. Merge the engineering-control foundation.
5. Scope ADR-01 as the first bounded Publisher Services implementation-readiness task.
