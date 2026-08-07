# AGENTS.md - Engineering documentation

This file extends the repository-root `AGENTS.md`.

It applies to engineering control, architecture, ADR, task, repository-map, rollout and review documents.

## 1. Documentation authority

Do not describe a proposal as deployed or approved.

Use explicit status terms:

- `DRAFT`
- `PROPOSED`
- `APPROVED`
- `ACTIVE`
- `SUPERSEDED`
- `BLOCKED`

Separate:

- observed repository/runtime state;
- approved target policy;
- planned future architecture;
- unresolved decisions.

### 1.1 Durable and transient state

Committed files record durable repository state. GitHub records transient
workflow state.

Do not write into a committed file a statement that merging will falsify, such
as:

```text
PENDING MERGE
AWAITING REVIEW
AWAITING CTO MERGE AUTHORIZATION
MERGE NOT YET COMPLETE
```

GitHub is the live authority for those questions. Prefer a durable form:

```text
Decision:
<the actual durable decision>

Authority condition:
This record is repository-authoritative when this exact content is reachable
from the repository's authoritative integration branch.

Live review, authorization and merge evidence:
GitHub pull-request history.
```

Materially equivalent wording is acceptable. The test is that the wording stays
truthful before review, after review, before merge and after merge, so that
reviewing or merging does not itself require another commit to correct status
prose.

Under `ADR-0005`, do not create a commit whose only purpose is copying an
existing GitHub review or approval identifier into a repository file, and do not
open a PR merely to record that a previous PR merged. See
`ai-delivery/operating-model.md` section 5.1.

The decision statuses in section 1 above, and the ADR statuses in
`decisions/README.md`, are retained. They record durable decision state owned by
the CTO, not transient pull-request state.

## 2. No silent duplication

Prefer links and indexes over independent copies of authoritative designs.

When a Project source or document is superseded:

- replace it;
- mark it superseded; or
- remove it from the active source set.

Do not leave two apparently current control indexes or designs.

## 3. Required document content

A task specification must include:

- programme;
- repository;
- task ID;
- risk;
- approved base and PR target;
- dependencies;
- objective;
- scope;
- non-goals;
- invariants;
- acceptance criteria;
- required tests;
- migration effect;
- rollout;
- rollback;
- stop conditions;
- approval.

An ADR must identify all affected programmes and repositories.

A repository map must distinguish verified facts from gaps.

## 4. Completion claims

Do not mark a task complete because:

- documents were written;
- code exists;
- an agent says tests passed;
- CI is green.

Completion requires the evidence and independent decision defined by the operating model.

## 5. Terminology

Use canonical repository and component names.

Current canonical metrics orchestrator:

```text
thoth-pub/thoth-sphinx
Sphinx
```

Avoid obsolete or variant spellings even when explaining historical errors.

## 6. Verification

For documentation changes:

```bash
git diff --check
```

Also check:

- relative paths;
- headings and file indexes;
- stale status statements;
- duplicate definitions;
- repository names;
- branch names;
- references to superseded sources;
- changelog entry.
