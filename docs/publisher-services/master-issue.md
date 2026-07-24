# Master GitHub Issue - Publisher Services and Distribution Configuration

Use this as the body of the programme's master issue in `thoth-pub/thoth`.

Suggested title:

```text
Publisher Services: packages, distribution configuration and controlled rollout
```

Suggested labels:

```text
programme
publisher-services
engineering-control
```

---

## Objective

Implement the approved Publisher Services and Distribution Configuration design across Thoth, thoth-app, thoth-dissemination and cc-license with additive schema, explicit authorization, audited migration, comparison-mode cutover, bounded pilots, monitoring and rollback.

## Authoritative control

- [Technical design](./README.md)
- [Decision summary](./decisions.md)
- [Task tracker](./task-status.md)
- [Platform inventory](./platform-inventory.md)
- [Acceptance matrix](./acceptance-matrix.md)
- [Rollout plan](./rollout-plan.md)
- [Engineering operating model](../engineering/ai-delivery/operating-model.md)
- [Decision register](../engineering/decisions/decision-register.md)

## Current gate

- [ ] P0-01 approved and merged
- [ ] ADR-0001 approved
- [ ] ADR-0002 approved
- [ ] ADR-01 platform inventory approved
- [ ] branch-readiness dependencies recorded
- [ ] programme integration branches approved

No production implementation begins before the applicable gate passes.

## Tasks

### Foundation

- [ ] P0-01 - Project control documents and tracker
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

## Issue maintenance

For every task, add:

```text
Repository:
Risk:
Specification:
Base:
Branch:
PR target:
PR:
Implementation model:
Independent reviewer:
Status:
Acceptance evidence:
Rollout/activation:
```

Do not close a checkbox at PR creation or CI success.

Close it only after:

- independent `APPROVED` decision;
- merge;
- required rollout/observation;
- tracker update.

## Production gates

- [ ] package/platform schema deployed without behaviour change
- [ ] licence audit complete before strict enforcement
- [ ] backfill dry run approved
- [ ] production backfill produced zero jobs
- [ ] comparison mode clean
- [ ] API cutover rollback exercised
- [ ] one bounded worker pilot approved
- [ ] monitoring and runbooks operational
- [ ] E2E-01 passed
- [ ] observation complete
- [ ] CTO approved cleanup
