# Model and Reasoning Selection

Model names evolve. Use this document as a selection policy, and record the exact model/version used in each task report.

## 1. Architecture and scoping

Use the strongest available general reasoning model.

Recommended reasoning:

- Medium: low-risk documentation or mechanical task decomposition.
- High: normal backend/API design and multi-file task scoping.
- Maximum: canonical data semantics, concurrency, migrations, authorization, cross-programme architecture or production cutover.

The scoping model should not implement the task it later approves.

## 2. Implementation

Default:

- strongest available Codex coding model;
- high reasoning.

Use maximum practical reasoning for:

- database migrations;
- transaction boundaries;
- concurrency, claims and leases;
- authorization;
- idempotency and deduplication;
- metrics identity, revisions and rollups;
- production migration utilities;
- external deposit or synchronization logic;
- cross-repository schema contracts;
- unexplained production defects.

Medium reasoning is acceptable only for contained low-risk changes with strong existing patterns and tests.

## 3. Independent review

Prefer a different model family from the implementer.

Example policy:

- Codex implements -> strongest available Claude model reviews.
- Claude implements -> strongest available Codex model reviews.

Use high or maximum reasoning for high-risk tasks.

The independent reviewer should initially receive:

- the approved task specification;
- the PR diff;
- relevant design/ADRs;
- test and CI evidence;
- migration and rollout information.

Do not bias the first review with the implementer's narrative justification unless required to understand an explicit deviation.

## 4. Dual review

Require two independent reviews for critical tasks involving:

- destructive migration;
- canonical historical recomputation;
- mass redistribution;
- broad authorization changes;
- source-of-truth cutover;
- production secrets or identity-provider configuration.

At least one reviewer must be from a different model family than the implementer.

## 5. Record keeping

Every task specification and implementation report records:

- model name;
- model version where visible;
- reasoning level;
- role: scope, implementation or review;
- material limitations encountered.

Do not infer quality from model choice alone. Approval depends on evidence.
