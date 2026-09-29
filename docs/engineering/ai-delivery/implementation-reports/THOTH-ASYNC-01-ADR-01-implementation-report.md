# THOTH-ASYNC-01-ADR-01 implementation report

Status: AUTHORING COMPLETE - AWAITING INDEPENDENT REVIEW

## Identity

```text
Programme: THOTH-ASYNC-01
Owning issue: thoth-pub/thoth#958
Parent programme: thoth-pub/thoth#957
Repository: thoth-pub/thoth
Task: THOTH-ASYNC-01-ADR-01
Risk: CRITICAL
Workflow: STANDARD
Authorized base: develop @ 923545d5c9028bc04c40e38efeb7de674efed3fd
Task branch: feature/async/adr-0012
PR target: develop
```

## Implemented scope

Authored the proposed shared asynchronous event/job architecture and only its
required repository records.

Content head before this report-only commit:

```text
fb8c1d4e9373153d367e2336b6a4dc349f165fc2
```

The final branch head including this report is recorded in the owning GitHub
issue and is the SHA to which independent review must bind.

## Files changed before this report

1. `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md` - new PROPOSED ADR.
2. `docs/engineering/decisions/decision-register.md` - registers ADR-0012 as PROPOSED and updates the register date.
3. `CHANGELOG.md` - records the architecture proposal under Unreleased/Added.

This report is the fourth and final path in the authorized write budget.

## Architecture recorded

The ADR proposes:

- one PostgreSQL-backed cross-programme async event/job engine;
- transactional event creation where coupled to PostgreSQL business writes;
- durable idempotent event fan-out;
- typed/versioned event and job payloads;
- shared claim/lease/claim-token/attempt/retry/reconciliation semantics;
- at-least-once execution opportunities, not exactly-once external effects;
- explicit reconciliation for indeterminate external writes;
- concurrency keys and deterministic idempotency;
- one default headless `thoth-worker` Fargate runtime for Thoth-owned handlers;
- domain-owned executors for cross-repository logic such as `thoth-dissemination`;
- one routine worker AWS task role rather than per-handler IAM roles;
- no generic catch-all ZITADEL machine role;
- external-service credentials supplied through deployment environment variables;
- no automated legacy-CDN migration;
- BE-04/DIS-02 pre-activation supersession onto the shared engine.

## BE-04 migration/data effect

No migration was created, modified or executed.

The ADR records the CTO-provided environment fact that v1.7.0 has been applied
to production and its `distribution_job*` tables have not been operationally
used, while dev has not applied that migration.

The proposal requires a future, separately authorized migration to remove the
unused legacy tables and introduce the shared async schema. Historical
`20260814_v1.7.0` remains immutable.

## Authorization/security effect

No authorization implementation changed.

ADR-0008's domain-specific machine-role and least-privilege rules remain in
force. The proposal does not create a generic ZITADEL worker/service role.

No IAM role, policy, credential, provider configuration or environment variable
was created or changed.

## External/runtime effects

```text
provider reads: 0
provider writes: 0
migration execution: 0
CI dispatch/rerun: 0
deployment: 0
release/publication: 0
production activation: 0
```

Repository/GitHub mutations were limited to the authorized issues, branch and
four-path documentation write budget.

## Validation

Live preflight established:

- `develop` exactly matched authorized base
  `923545d5c9028bc04c40e38efeb7de674efed3fd`;
- no conflicting `feature/async*` ref existed;
- ADR-0012 did not already exist;
- ADR-0011 was already allocated, therefore ADR-0012 is the next decision number.

Compare at content head `fb8c1d4e...` showed exactly three substantive paths:

```text
CHANGELOG.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

No runtime tests were required or executed because this task changes
documentation/control only.

## Deviations

The GitHub connector omitted issue numbers from the normalized create response,
causing the initial #958 body to contain `#undefined` as its parent reference.
The returned URLs established the actual issue numbers (#957 and #958), and #958
was corrected before any repository branch/source mutation.

No scope deviation remains.

## Remaining gates

1. independent exact-head review of the final branch head;
2. CTO approval of the exact ADR content;
3. approval-state reconciliation as required by repository decision process;
4. pull-request creation/update only if separately authorized;
5. independent exact-head review and CTO merge authorization;
6. merge to `develop`;
7. separately specified/authorized implementation tasks.

No implementation, migration, deployment or activation is authorized by this
report.
