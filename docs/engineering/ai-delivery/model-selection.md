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

- the model the section 6 table assigns to the task's risk;
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

The reviewer is always a fresh instance that did not implement the change. HIGH and CRITICAL work gets two independent reviews on two different models (section 6). Neither reviewer sees the other's ruling before giving its own.

No gate depends on an automatic third-party reviewer such as Codex. Where one is installed its findings are advisory: answer a P0 or P1 before merging, but never wait for a result.

Use high or maximum reasoning for high-risk tasks.

The independent reviewer should initially receive:

- the approved task specification;
- the PR diff;
- relevant design/ADRs;
- test and CI evidence;
- migration and rollout information.

Do not bias the first review with the implementer's narrative justification unless required to understand an explicit deviation.

## 4. Dual review

Two independent reviews are standard for every HIGH and CRITICAL task (section 6). They are always required for tasks involving:

- destructive migration;
- canonical historical recomputation;
- mass redistribution;
- broad authorization changes;
- source-of-truth cutover;
- production secrets or identity-provider configuration.

The two reviews run on different models (section 6), and neither reviewer implemented the change.

## 5. Record keeping

Every task specification and implementation report records:

- model name;
- model version where visible;
- reasoning level;
- role: scope, implementation or review;
- material limitations encountered.

Do not infer quality from model choice alone. Approval depends on evidence.

## 6. Task-type table

Model names evolve. The table names the current assignment for each kind of task. Where it names a model, it takes precedence over the defaults in sections 1 to 3. Sections 4 and 5 still apply in full.

| Task type | Model and effort |
| --- | --- |
| Cross-programme architecture, ADRs, CRITICAL design, large audits | Fable 5.1 (named only) |
| Specifications and stage decomposition | Opus 5.5, high |
| Stage orchestrator; LOW and MEDIUM implementation | Sonnet 5.5, medium |
| HIGH implementation | Opus 5.5, high |
| CRITICAL implementation | Opus 5.5, max |
| LOW and MEDIUM review | Fresh Sonnet 5.5, high |
| HIGH review | Fresh Opus 5.5, high, and fresh Fable 5.1, independent |
| CRITICAL review | Fresh Opus 5.5, max, and fresh Fable 5.1, independent |
| Land-check runs, refresh re-checks, landings, CI watching, state block and board edits, thread handoffs | Haiku 5.5, low |

"Fresh" means a new instance that did not implement the task. "Named only" means the model is used when the CTO names it for the task.

Record the model and effort in every `REVIEW` and `LANDED` comment (`operating-model.md` section 9.1), as well as in the task report.
