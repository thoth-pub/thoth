# Thoth Engineering Control

Status: ACTIVE AFTER MERGE
Owner: CTO
Foundation task: `CTRL-FOUNDATION-01`
Foundation pull request: [#764](https://github.com/thoth-pub/thoth/pull/764)

## 1. Purpose

This is the repository entry point for Thoth's AI-led engineering delivery controls. It is an index; linked documents, approved designs, ADRs, task specifications, repository state, pull requests and CI remain authoritative.

## 2. Authority order

1. merged code, migrations and generated contracts;
2. approved ADRs and technical designs;
3. approved task specifications;
4. GitHub issues, PRs, review threads and CI;
5. programme-control and rollout documents;
6. agent reports and conversations.

Stop when authoritative sources conflict. Record cross-programme resolutions in an ADR or approved programme decision.

## 3. Mandatory delivery controls

- [AI-led engineering delivery](./ai-delivery/README.md)
- [Operating model](./ai-delivery/operating-model.md)
- [Branching and release workflow](./ai-delivery/branching-and-release-workflow.md)
- [Risk classification](./ai-delivery/risk-classification.md)
- [Model selection](./ai-delivery/model-selection.md)
- [Release gates](./ai-delivery/release-gates.md)
- [Task specification template](./ai-delivery/task-specification-template.md)
- [Implementation report template](./ai-delivery/implementation-report-template.md)
- [Independent review template](./ai-delivery/independent-review-template.md)

## 4. Approved design references

- [Private approved-design references](./design-references.md)

Reviewers must use the exact Google Drive file and revision metadata recorded in `design-references.md` and must have authorized access to those private revisions. The repository intentionally does not contain the design bodies.

## 5. Repository and environment orientation

- [Repository map](./repository-map/README.md)
- [Branch topology](./repository-map/branch-topology.md)
- [Environment boundaries](./repository-map/environments.md)
- [Control gaps](./repository-map/control-gaps.md)
- [Agent-instruction rollout](./agent-instructions/rollout-plan.md)

Agents must use observed repository state until an approved normalization task changes it.

## 6. Cross-programme decisions

- [Decision process](./decisions/README.md)
- [Decision register](./decisions/decision-register.md)
- [ADR-0001: publisher package capabilities](./decisions/ADR-0001-publisher-package-capability-model.md) - `APPROVED AND MERGED` (approved by Javi, CTO, on 2026-07-28; approval PR [#772](https://github.com/thoth-pub/thoth/pull/772) merged on 2026-07-29 as `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`)
- [ADR-0002: platform domain boundaries](./decisions/ADR-0002-platform-domain-boundaries.md) - `APPROVED` (CTO, 2026-07-27, PR [#769](https://github.com/thoth-pub/thoth/pull/769))

ADR-0001 and ADR-0002 are approved and their approval records are merged.
ADR-0001 was approved by Javi, CTO, on 2026-07-28. Its independently reviewed
approval PR [#772](https://github.com/thoth-pub/thoth/pull/772) merged on
2026-07-29 as `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`. The shared ADR
approval-and-merge dependency is satisfied, but no implementation task is ready.
Publisher Services and Metrics implementation remain blocked by their genuine
programme, task-specification and repository-readiness prerequisites. PR #772
changed engineering-control documentation only and implemented or activated no
runtime behaviour.

## 7. Active programmes

### Publisher Services

- [Programme controls](../publisher-services/README.md)
- [Task tracker](../publisher-services/task-status.md)
- [Master issue #765](https://github.com/thoth-pub/thoth/issues/765)

Current implementation decision: `BLOCKED`.

### Thoth Metrics

- [Programme controls](../metrics/README.md)
- [Task tracker](../metrics/task-status.md)
- [Master issue #766](https://github.com/thoth-pub/thoth/issues/766)

Current implementation decision: `BLOCKED`.

## 8. Canonical names

```text
thoth-pub/thoth-sphinx
Sphinx
```

## 9. Current foundation closeout gate

Achieved:

- [x] PR #764 merged into `develop` as
  `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`;
- [x] PR #764 final-head CI passed;
- [x] the retrospective independent review's P1 findings were resolved;
- [x] closeout PR [#767](https://github.com/thoth-pub/thoth/pull/767) (reviewed
  content head `d72137893ddea512c0d05c81d310eb59d045cd2b`) received an
  independent `APPROVED` review;
- [x] PR #767 merged into `develop` as
  `bac598e32abbd0d7e69ff467c82945ee00df02ba` on 2026-07-27, closing P0-01;
- [x] the repository programme trackers record the completed closeout;
- [x] issue #765 was synchronized on 2026-07-27 as an external mirror of the
  completed repository closeout; issue #765 remains open.

Outstanding:

- None for the foundation closeout gate. Dependent Publisher Services and Metrics
  implementation remains blocked by Publisher Services ADR-01 and its final
  distribution-platform inventory, task-specific approved specifications, and
  applicable branch-readiness controls. For Metrics, the previously recorded
  Diesel/schema-control blocker is resolved (ADR-0003 is
  repository-authoritative through merged PR
  [#778](https://github.com/thoth-pub/thoth/pull/778)) and the `MET-CTRL-01`
  programme-control dependency is satisfied through merged PR
  [#833](https://github.com/thoth-pub/thoth/pull/833), so
  Thoth WP1 entry now waits only on separately authorized `feature/metrics`
  creation and one approved bounded WP1 child specification; Sphinx, client
  and source-specific readiness gates remain
  attached to their owning later work packages rather than blocking WP1 (see
  [CG-08](./repository-map/control-gaps.md#cg-08---metrics-readiness-open)).
  ADR-0001 and ADR-0002 approval remove shared decision dependencies without
  making either programme implementation-ready.

The implementing conversation cannot approve its own work.
