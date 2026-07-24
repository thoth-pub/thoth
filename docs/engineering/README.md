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

## 4. Repository and environment orientation

- [Repository map](./repository-map/README.md)
- [Branch topology](./repository-map/branch-topology.md)
- [Environment boundaries](./repository-map/environments.md)
- [Control gaps](./repository-map/control-gaps.md)
- [Agent-instruction rollout](./agent-instructions/rollout-plan.md)

Agents must use observed repository state until an approved normalization task changes it.

## 5. Cross-programme decisions

- [Decision process](./decisions/README.md)
- [Decision register](./decisions/decision-register.md)
- [ADR-0001: publisher package capabilities](./decisions/ADR-0001-publisher-package-capability-model.md) - `PROPOSED`
- [ADR-0002: platform domain boundaries](./decisions/ADR-0002-platform-domain-boundaries.md) - `PROPOSED`

A committed proposed ADR is not approved. Dependent implementation remains blocked until the CTO decision is recorded and independently reviewed.

## 6. Active programmes

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

## 7. Canonical names

```text
thoth-pub/thoth-sphinx
Sphinx
```

## 8. Current foundation gate

- [ ] complete final repository consistency checks;
- [ ] obtain independent review of the complete PR diff and CI;
- [ ] resolve blocking findings;
- [ ] obtain CTO merge approval;
- [ ] merge PR #764 into `develop`;
- [ ] replace stale Project sources with merged repository versions.

The implementing conversation cannot approve its own work.
