# AI-led Engineering Delivery

This directory defines the mandatory delivery controls for engineering tasks implemented or substantially assisted by AI agents.

These documents apply across Thoth engineering programmes unless a stricter approved programme specification overrides them.

## Documents

- `operating-model.md` - roles, authority, lifecycle and task boundaries.
- `branching-and-release-workflow.md` - GitHub Flow, `develop`/`master`, programme integration branches and releases.
- `risk-classification.md` - task risk levels and mandatory controls.
- `model-selection.md` - implementation and review model guidance.
- `task-specification-template.md` - required specification before implementation.
- `implementation-report-template.md` - required agent completion report.
- `implementation-reports/` - completed task-specific implementation evidence.
- `independent-review-template.md` - evidence-based review format.
- `release-gates.md` - merge, staging, production and closure gates.
- `decision-record-template.md` - lightweight ADR/programme decision template.
- `tasks/CTRL-FOUNDATION-01.md` - approved foundation specification.
- `tasks/CTRL-MERGE-01.md` - terminal merge evidence and non-recursive closeout.
- `tasks/THOTH-GQL-BATCH-01.md` - request-scoped GraphQL batching foundation
  (`DRAFT`; depends on `ADR-0006` being approved and repository-authoritative).
- `tasks/THOTH-GQL-OPS-01.md` - GraphQL mutation-guard runtime operations
  (`DRAFT`; bounded feature-specific successor to CG-13 for mode control. It
  establishes how a mode change would be controlled, proves that the capability
  to make and verify one does not yet exist, and specifies the bounded
  prerequisites that must deliver it. It distinguishes the merged `develop`
  state, the deployed production release — which predates the guard and is
  recorded as **pre-guard** on combined repository and external deployment
  evidence, never on repository evidence alone — and production activation
  state. It terminates at CG-13 disposition
  `C - BLOCKED`, leaves the `ADR-0006` runtime-operations gate `NOT SATISFIED`,
  and activates nothing). Its delivered output is the
  [operational-control record](../repository-map/graphql-mutation-guard-runtime-operations.md)
  and the **provisional**
  [mode-transition runbook](../repository-map/graphql-mutation-guard-mode-transition-runbook.md).
- `tasks/THOTH-GQL-OPS-02.md` - mutation-guard mode-control path
  (**implemented, independently reviewed and merged** through PR
  [#797](https://github.com/thoth-pub/thoth/pull/797)). Makes
  `THOTH_GRAPHQL_MUTATION_GUARD_MODE` consumable on the production-applicable
  command path while preserving all existing `init` migration and startup
  semantics. **Closes capability gap 1.** Making the mode settable is not setting
  it: the default remains `OFF`, no environment was transitioned, and `OBSERVE`,
  `ENFORCE` and `BE-02` runtime all remain `NOT AUTHORIZED`.
- `tasks/THOTH-GQL-OPS-03.md` - effective-mode fleet-verification mechanism
  (`DRAFT`; implementation `NOT AUTHORIZED`; branch must not exist). Implements
  the smallest mechanism proving the effective mode of every serving instance,
  with per-instance attribution, complete-enumeration coverage that fails closed,
  `UNKNOWN` kept distinct from `OFF`, and mixed-fleet detection, without
  affecting request acceptance. Its section 3.2 information-disclosure boundary
  is resolved in the specification: the signal is
  **administrative/orchestration-plane or out-of-band only**, and a **public
  unauthenticated effective-mode surface is rejected**, with the public GraphQL
  schema unchanged. Closes capability gap 2 — the last one still open. A verifier
  is not a verified fleet.
- `tasks/THOTH-GQL-OPS-04.md` - bounded runtime-operations verification and
  closure (`DRAFT`; implementation `NOT AUTHORIZED`; branch must not exist).
  After `-02` and `-03` merge, re-establishes all external evidence, proves both
  capabilities against the real runtime, finalises the runbook, and decides CG-13
  disposition `A` or `C` on evidence. It is the earliest task that may record the
  runtime-operations gate as `SATISFIED`, and it may return `C` again. It must
  not activate `OBSERVE`.
- `reviews/CTRL-FOUNDATION-01-review-brief.md` - independent review requirements.

## Core rule

A task is not complete because code exists or CI passes.

A task is complete only when:

1. its approved scope and acceptance criteria are satisfied;
2. actual diffs and tests have been independently reviewed;
3. migration, authorization and operational effects have evidence;
4. the appropriate merge and release gates have passed;
5. repository status and follow-up work are current.

## Branching summary

Thoth uses GitHub Flow (`ghf`):

```text
feature/* -> develop -> master
```

For approved large programmes:

```text
feature/<programme>/<slice> -> feature/<programme> -> develop -> master
```

Implementation branches normally start from `develop`. `master` is the release branch.

## Authority

In descending order:

1. Merged repository state.
2. Approved ADRs and designs.
3. Approved task specifications.
4. Pull requests, review threads and CI evidence.
5. Programme control documents.
6. Conversations and agent reports.

Missing evidence is missing work.

For task lifecycle evidence specifically, `ADR-0005` treats the GitHub review,
authorization, CI and merge record as terminal evidence. Committed documents
state durable decisions and an authority condition rather than transient
pull-request status, and no commit or PR is created solely to restate a review,
authorization or merge that GitHub already records. See `operating-model.md`
section 5.1.
