# Branching and Release Workflow

Status: Proposed until merged and approved  
Owner: CTO  
Workflow: GitHub Flow (`ghf`)

## 1. Standard delivery flow

Thoth repositories use the following normal workflow:

1. Start from the latest `develop` branch.
2. Create a short-lived branch under `feature/`.
3. Implement one bounded task.
4. Open a pull request from the feature branch into `develop`.
5. Complete independent review and required checks.
6. Merge the pull request into `develop`.
7. Delete the feature branch.
8. When `develop` contains a coherent, production-ready release, create a release that merges `develop` into `master`.
9. Deploy or activate production behaviour only under the applicable release controls.

`master` is the release branch. It is not the normal base branch for implementation work.

## 2. Standard task branch naming

Use:

```text
feature/<programme-or-area>/<task-id-or-short-name>
```

Examples:

```text
feature/publisher-services/be-01
feature/metrics/record-schema
feature/auth/service-role-tests
```

A normal task branch:

- branches from `develop`;
- targets `develop`;
- is deleted after merge.

## 3. Large programme integration branches

For a large, multi-slice programme that must be validated as one coherent feature before entering `develop`, use a dedicated programme integration branch.

Create:

```text
feature/<programme>
```

from the latest `develop`.

Examples:

```text
feature/publisher-services
feature/metrics
```

Each independently reviewable slice then branches from the programme integration branch:

```text
feature/<programme>/<task-id-or-slice>
```

Examples:

```text
feature/publisher-services/be-01
feature/publisher-services/be-02
feature/metrics/db-foundation
feature/metrics/ingestion-core
```

Each slice pull request targets the programme integration branch, not `develop`.

After a slice PR is approved and merged:

1. delete the slice branch;
2. update the programme tracker;
3. verify the integration branch remains green;
4. rebase or merge the latest `develop` into the programme integration branch at agreed checkpoints;
5. resolve integration conflicts before additional dependent slices proceed.

When the entire programme feature satisfies its integrated acceptance criteria, open one final pull request:

```text
feature/<programme> -> develop
```

That final PR is the programme integration and release-candidate gate.

After it is merged:

- delete the programme integration branch;
- retain the task specifications, PRs and review evidence;
- prepare the normal `develop` to `master` release when the release is approved.

## 4. Choosing standard versus programme integration flow

Use the standard direct-to-`develop` flow when:

- the task is independently useful or safely inert;
- it can merge without requiring incomplete sibling changes;
- compatibility can be maintained;
- the change can be hidden behind a feature flag or remain unused;
- merging early reduces branch divergence.

Use a programme integration branch when:

- several slices must be tested together before entering `develop`;
- intermediate states would break builds, generated contracts or consumers;
- multiple repositories must coordinate around a pinned feature contract;
- the feature requires a final integrated acceptance environment;
- the CTO has explicitly approved the integration-branch approach.

Do not create a programme integration branch merely because a programme is large. Prefer direct-to-`develop`, additive, inactive slices when they are independently safe. Long-lived branches increase divergence and integration risk.

## 5. Multi-repository programmes

Branches are repository-local.

A coordinated programme may use the same branch naming convention in multiple repositories, but there is no single Git branch spanning repositories.

For example:

```text
thoth:             feature/metrics
thoth-sphinx:      feature/metrics
thoth-app:         feature/metrics
metrics-dashboard: feature/metrics
```

Each repository has:

- its own integration branch from that repository's `develop`;
- its own slice branches;
- its own final `feature/<programme> -> develop` PR.

Final PRs are merged in the documented dependency order.

Where a downstream repository needs an API contract before the upstream final PR merges, use one of:

- a pinned preview environment;
- a versioned generated schema from the integration branch;
- a temporary branch reference explicitly recorded in the task specification.

Do not allow downstream code to guess an unmerged contract.

## 6. Keeping integration branches current

For programme integration branches:

- record the original `develop` base commit;
- update from `develop` at defined checkpoints;
- update before high-risk database or API slices;
- update before the final integration review;
- rerun integrated CI after every update;
- avoid arbitrary history rewriting after other agents or reviewers depend on the branch.

The task specification must state whether the repository convention uses merge or rebase for refreshing the programme branch.

## 7. Pull-request review boundaries

Every slice PR must still be independently reviewable.

A slice PR must include:

- one approved task specification;
- a bounded diff;
- task-level tests;
- migration and compatibility evidence;
- an implementation report;
- an independent review decision.

Approval of a slice PR does not approve the overall programme for production.

The final programme PR must additionally verify:

- all accepted slices are present;
- cross-slice behaviour works;
- integrated migrations apply in order;
- generated contracts are consistent;
- no temporary test scaffolding or unsafe bypass remains;
- programme-wide acceptance criteria pass;
- rollout, rollback and observation are approved.

## 8. Release flow

The normal release path is:

```text
feature branch -> develop -> master
```

For a programme integration branch:

```text
slice branch -> feature/<programme> -> develop -> master
```

A release from `develop` to `master` must:

- identify the included PRs and migrations;
- verify `develop` is green;
- include release notes;
- include deployment and rollback instructions;
- identify feature flags or post-deploy activation;
- receive required CTO approval;
- preserve traceability from release to tasks and PRs.

## 9. Prohibited branch patterns

Do not:

- branch normal implementation work from `master`;
- target normal feature PRs directly at `master`;
- keep merged slice branches indefinitely;
- mix unrelated programmes in one integration branch;
- use an integration branch as a substitute for task specifications or review;
- allow an implementing agent to merge its own PR;
- activate production behaviour merely because a programme branch merged into `develop`.
