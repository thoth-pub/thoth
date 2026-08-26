# Branching and Release Workflow

Status: Proposed until merged and approved
Owner: CTO
Workflow: GitHub Flow (`ghf`)
Scope: `thoth-pub/thoth`

This document defines the branching and release workflow for the
`thoth-pub/thoth` repository. It does not impose `thoth`'s `develop`/`master`
topology on any other repository. Each other Thoth repository's branch
topology is authoritative in that repository's own entry under
`docs/engineering/repository-map/repositories/` and
`docs/engineering/repository-map/branch-topology.md`, verified from live
GitHub state, not assumed from this document. Section 6 defines how
cross-repository dependency and compatibility ordering is coordinated when a
change spans repositories with different local topologies.

## 1. Standard delivery flow

`thoth-pub/thoth` uses the following normal workflow:

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

`STANDARD` task branches are named by **area**, not by programme:

```text
feature/<area>/<task>
```

Examples:

```text
feature/publisher-services/be-01
feature/metrics-control/met-ctrl-01
feature/auth/service-role-tests
```

A normal task branch:

- branches from `develop`;
- targets `develop`;
- is deleted after merge.

This descendant form is available only where no flat `feature/<area>` branch
already occupies that ref location. A programme that runs a long-lived
`feature/<programme>` integration branch therefore cannot also use
`feature/<programme>/<task>`; its slices use the sibling form in section 3.
See [`ADR-0009`](../decisions/ADR-0009-programme-integration-branch-namespace.md)
and the fail-closed namespace preflight in `AGENTS.md` section 5.1.

## 3. Large programme integration branches

For a large, multi-slice programme that must be validated as one coherent feature before entering `develop`, use a dedicated programme integration branch.

Create:

```text
feature/<programme>
```

from the latest `develop`.

Examples:

```text
feature/metrics
feature/large-programme
```

Each independently reviewable slice then branches from the programme integration branch. The slice branch is a **sibling** of the integration branch, separated by the reserved `--` token:

```text
feature/<programme>--<slice>
```

Examples:

```text
feature/metrics--db-foundation
feature/metrics--ingestion-core
feature/large-programme--slice-01
```

The descendant form `feature/<programme>/<slice>` must not be used here. Git cannot hold both the ref `refs/heads/feature/<programme>` and the ref namespace `refs/heads/feature/<programme>/` at the same path, so while the integration branch is live, creating a descendant slice ref fails.

`--` is reserved as the programme/slice separator. Governed `<programme>`, `<area>`, `<slice>` and `<task>` identifiers must each be non-empty, must each be a single Git path segment, and must not themselves contain `--`. Split at the first reserved `--` to recover the programme and slice.

Run the fail-closed namespace preflight in `AGENTS.md` section 5.1 before creating any governed ref, and HOLD rather than deleting, renaming or moving an existing branch to make room.

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


## 5. Programme-specific workflow authority

An approved programme design may require either the standard flow or the programme-integration flow. Record the selected flow in the programme controls and every task specification.

### Publisher Services

The approved Publisher Services design requires:

```text
development branch -> feature/publisher-services/<task> -> development branch
```

Every task uses a fresh branch and one PR. Do not create a long-lived `feature/publisher-services` integration branch.

### Thoth Metrics

The Metrics design requires repository-local integration branches after branch readiness:

```text
develop -> feature/metrics -> feature/metrics--<slice> -> feature/metrics -> develop
```

Each focused Metrics child branch is created from `feature/metrics`, targets `feature/metrics`, and does not target `develop` directly. `ADR-0009` standardizes the repository ref spelling of that child branch; it does not amend the substantive Metrics architecture.

Each affected repository owns its own `feature/metrics` integration branch and final PR. No physical branch spans repositories.

## 6. Multi-repository programmes

Branches are repository-local. Each repository's branch topology — its
default branch, active development branch and target release flow — is
authoritative in that repository's own `docs/engineering/repository-map/`
entry, verified from live GitHub state. Repositories are not required to share
`thoth`'s `develop`/`master` names or flow: for example, at time of writing
`thoth-app` and `thoth-pyramid` develop on `dev` and release from `main`, while
`thoth-dissemination`, `thoth-strapi` and the standalone `thoth-client` develop
on `develop`.

A coordinated programme may use the same task/slice branch naming convention
in multiple repositories, but there is no single Git branch spanning
repositories, and the base/target of each repository-local branch follows that
repository's own topology, not `thoth`'s.

For example:

```text
thoth:              feature/metrics -> develop
thoth-sphinx:        feature/metrics -> develop   (Sphinx's own develop)
thoth-app:           feature/metrics -> dev        (thoth-app's own active branch, until BR-APP-01 normalizes it)
metrics-dashboard:   feature/metrics -> dev        (until BR-DASH-01 normalizes it)
```

Each repository has:

- its own integration or task branch from that repository's own verified base;
- its own slice branches where applicable;
- its own final pull request into that repository's own target branch.

### 6.1 Cross-repository dependency and compatibility ordering

A task specification for a change that affects a shared contract (see
`docs/engineering/repository-map/contracts.md` and operating-model.md section
4.1) must record, for every affected repository:

- the owning repository for the contract being changed;
- which repositories are consumers and whether each requires a change;
- the required merge order across repositories (which PR must merge before
  which);
- the compatibility window during which the old and new contract must both
  work, if any;
- whether deployment order matters in addition to merge order.

Final PRs are merged in the documented dependency order: an upstream
contract-owning repository's change merges before a downstream consumer's
change that depends on it, unless the task specification records an explicit
compatibility window that makes the order safe to reverse.

Where a downstream repository needs an API contract before the upstream final PR merges, use one of:

- a pinned preview environment;
- a versioned generated schema from the integration branch;
- a temporary branch reference explicitly recorded in the task specification.

Do not allow downstream code to guess an unmerged contract.

## 7. Keeping integration branches current

For programme integration branches:

- record the original `develop` base commit;
- update from `develop` at defined checkpoints;
- update before high-risk database or API slices;
- update before the final integration review;
- rerun integrated CI after every update;
- avoid arbitrary history rewriting after other agents or reviewers depend on the branch.

The task specification must state whether the repository convention uses merge or rebase for refreshing the programme branch.

## 8. Pull-request review boundaries

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

## 9. Release flow

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

## 10. Prohibited branch patterns

Do not:

- branch normal implementation work from `master`;
- target normal feature PRs directly at `master`;
- keep merged slice branches indefinitely;
- mix unrelated programmes in one integration branch;
- name a programme slice `feature/<programme>/<slice>` beneath a live `feature/<programme>` integration branch;
- use a single `-` as the programme/slice separator, or place `--` inside a governed programme, area, slice or task identifier;
- delete, rename or move an existing branch to work around a namespace collision;
- use an integration branch as a substitute for task specifications or review;
- allow an implementing agent to merge its own PR;
- activate production behaviour merely because a programme branch merged into `develop`.
