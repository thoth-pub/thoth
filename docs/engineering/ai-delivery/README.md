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
