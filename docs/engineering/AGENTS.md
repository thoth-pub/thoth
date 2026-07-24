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
