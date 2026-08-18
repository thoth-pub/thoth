# Engineering Task Risk Classification

Classify every task before selecting an implementation model or approving execution.

Use the highest applicable risk level.

## Low risk

Typical characteristics:

- documentation-only changes;
- test-only additions;
- local refactors with no behaviour change;
- generated-code refresh from an unchanged contract;
- isolated UI copy or styling;
- no authorization, migration, data or operational effect.

Required controls:

- approved task scope;
- focused branch and PR;
- standard tests and CI;
- independent review may be lightweight.

## Medium risk

Typical characteristics:

- bounded API additions;
- non-sensitive UI behaviour;
- backwards-compatible schema additions with no backfill;
- internal tooling that cannot affect production automatically;
- feature-flagged behaviour with limited data effect.

Required controls:

- complete task specification;
- unit and integration tests;
- compatibility assessment;
- independent review;
- rollout note if behaviour will later be activated.

## High risk

Any of:

- database migration on populated tables;
- authorization or role enforcement;
- machine-to-machine credentials or service roles;
- job claiming, retries, leases or concurrency;
- idempotency or deduplication;
- backfill or bulk mutation;
- external deposits or write-back;
- package entitlements;
- changes to canonical data semantics;
- production feature activation;
- cross-repository API contract change;
- user uploads or untrusted file parsing;
- personal-data handling;
- changes capable of broadening processing scope.

Required controls:

- approved design and task specification;
- high or maximum implementation reasoning;
- independent cross-model review;
- migration tests on empty and populated databases where applicable;
- failure-path and authorization tests;
- rollout and rollback plan;
- feature flag, comparison mode or controlled pilot where possible;
- explicit CTO merge or activation approval.

## Critical risk

Any of:

- destructive or irreversible production migration;
- canonical data rewrite at scale;
- change capable of mass redistribution or external publication;
- security boundary affecting all publishers;
- secrets, credential rotation or identity-provider reconfiguration;
- historical metrics recomputation replacing existing canonical totals;
- cutover from one source of truth to another without immediate rollback;
- operation with material legal, privacy or contractual consequences.

Required controls:

- explicit CTO-approved specification;
- architecture review;
- implementation by the strongest appropriate coding model at maximum practical reasoning;
- at least one independent cross-model review;
- rehearsal in a production-like environment;
- verified backup/restore or recovery path;
- dry run and comparison report;
- named pilot;
- monitoring and kill switch;
- explicit production change window;
- observation period;
- no simultaneous unrelated critical cutover.

## Escalation rules

Raise the risk level when:

- evidence about the current system is incomplete;
- the affected production data volume is unknown;
- rollback is uncertain;
- multiple repositories or external systems must change atomically;
- tests cannot reproduce the operational path;
- an external API has weak idempotency or correction support.

Uncertainty increases risk. It does not justify reducing controls.
