# Implementation Handoff Template

Copy this file's structure into the bounded prompt given to a coding agent.
Replace every bracketed field. An implementation handoff is derived from an
already-approved `task-specification-template.md` record; it does not
introduce new scope, and every field here must be consistent with that
approved specification.

A handoff may not be issued with unresolved required fields. If a required
field cannot be filled from approved, verified sources, the task is not ready
for implementation.

---

## 1. Task identity

```text
Programme:
Stage:
Owning GitHub issue: [owner/repository#NNN]
Parent programme issue: [owner/repository#NNN or None]
Repository: [owner/repository]
Task ID:
Risk: LOW | MEDIUM | HIGH | CRITICAL
Reasoning: [level]
Workflow: STANDARD | PROGRAMME_INTEGRATION
Production/runtime effect: [NONE intended, or explain]
```

## 2. Authority

Approved specification: [link/path]
Relevant ADRs: [list or `None`]

Distinguish two separate authority sources for the target repository named in
section 1, and do not conflate them:

a) **the target repository's own root and nested `AGENTS.md` files and local
   controls.** These govern the target repository directly, are read from
   that repository (not assumed to be `thoth-pub/thoth`), and are
   authoritative for its own conventions, stack, branch topology and
   prohibited assumptions. If the target repository is not `thoth-pub/thoth`,
   do not assume it has a `docs/engineering/` directory or any of the other
   canonical-doctrine paths below; verify what that repository actually has.

b) **canonical shared doctrine**, maintained in `thoth-pub/thoth` and
   consulted whenever this task requires the shared cross-repository control
   model (action authorization, cross-repository impact analysis, lifecycle
   evidence):

   - `thoth-pub/thoth`'s `docs/engineering/ai-delivery/operating-model.md`;
   - `thoth-pub/thoth`'s
     `docs/engineering/ai-delivery/branching-and-release-workflow.md` (scoped
     to `thoth-pub/thoth`'s own branch topology; not authoritative for any
     other repository's branch topology);
   - `thoth-pub/thoth`'s `docs/engineering/repository-map/README.md`, the
     relevant repository-map entries and `contracts.md`;
   - [any additional programme-specific documents or ADRs, and their owning
     repository].

Use live repository evidence for repository facts. Do not rely on this
handoff for a repository fact if live repository evidence contradicts it. If
an authoritative conflict exists, return `HOLD`/`BLOCKED` rather than
resolving it unilaterally.

## 3. Exact base, branch and PR target

```text
Authorized base branch: [the target repository's verified repository-local
  base branch — for thoth-pub/thoth this is normally `develop`; for any other
  repository, verify it live rather than assuming `develop`]
Exact authorized base commit: [full 40-character SHA]
PR target: [the same verified repository-local base branch, or an approved
  `feature/<programme>` integration branch]
Authorized task/slice branch: [use the form matching the `Workflow` field in
  section 1; do not mix them]

  STANDARD:               feature/<area>/<task>
  PROGRAMME_INTEGRATION:  feature/<programme>--<slice>
```

Under `PROGRAMME_INTEGRATION` the slice branch is a **sibling** of the
`feature/<programme>` integration branch, not a descendant of it. Never
authorize `feature/<programme>/<slice>` while `feature/<programme>` exists as a
branch: Git cannot hold a ref and a ref namespace at the same path, so branch
creation fails. `--` is the reserved programme/slice separator, and governed
`<programme>`, `<area>`, `<slice>` and `<task>` identifiers must each be
non-empty, must each be a single Git path segment, and must not themselves
contain `--`. See
[`ADR-0009`](../decisions/ADR-0009-programme-integration-branch-namespace.md).

## 4. Mandatory preflight

Before creating a branch or editing any file:

1. Fetch the remote and verify the authorized base branch exists.
2. Verify the base branch is exactly at the authorized base commit above.
3. Verify the owning issue is still open and has not materially changed in a
   way that conflicts with this handoff.
4. Verify the working tree is clean.
5. Verify there is no existing conflicting branch or active PR for this task.
6. Run the fail-closed namespace preflight in `AGENTS.md` section 5.1 against
   live refs: the exact authorized ref is absent; no descendant namespace
   already occupies a prospective flat ref; and no flat parent ref already
   occupies a prospective descendant namespace. If it fails, HOLD. Never
   delete, rename or move another branch to make room.
7. Read the applicable `AGENTS.md` files.
8. Inspect every file in the authorized write budget before editing it.

If the base has moved from the authorized commit: **stop**. Do not silently
rebase the authorization onto the new head. Return `HOLD - AUTHORIZED BASE
MOVED` with the authorized SHA, the current SHA, the intervening commits and
whether they appear relevant to this task. Do not implement until the base is
explicitly reconciled.

If unrelated local changes exist and cannot be isolated safely, stop and
report rather than discarding them.

## 5. Authorized write budget

Existing files this task may modify:

- [path]
- [...]

New files this task may create:

- [path]
- [...]

No other file writes are authorized. Unless explicitly listed above, the
implementing agent is **not** authorized to: delete files; rename files; move
files; modify application/runtime source outside the listed paths; modify
tests unrelated to this task; modify GitHub workflow files; modify branch
protection/repository settings; modify provider/runtime configuration.

If another path appears necessary, stop and return `HOLD` with the proposed
path and reason. Do not broaden the write budget unilaterally.

## 6. Granular action authorization

Authorization is granted action-by-action and is **not transitive**: see root
`AGENTS.md` section 6. Unlisted mutations are denied.

| Action | Authorized |
|---|---|
| repository/GitHub read inspection | |
| documentation/source edits within the write budget | |
| creation of the specifically authorized new files | |
| deletion/move/rename of files | NO |
| create the authorized task branch from the exact authorized base | |
| local validation | |
| commit bounded changes | |
| push the task branch | |
| open/update a draft PR targeting the authorized PR target | |
| update/comment on the owning issue for task-state/evidence | |
| update/comment on the parent programme issue where coordination requires it | |
| manual CI dispatch/rerun | NO |
| provider/runtime reads | NO |
| provider/runtime writes | NO |
| migration execution | NO |
| release/tag/publication | NO |
| merge | NO |
| deployment | NO |
| production activation | NO |
| production secrets access | NO |
| branch protection/settings changes | NO |

## 7. Expected automatic side effects

State every automatic effect that an authorized action is expected to
trigger — for example, opening or updating a pull request triggers repository
CI, and identify any CI path capable of an external write (container/package
registry publication, deployment trigger). State the expected behaviour for
this task's diff (for example, a documentation-only classifier expected to
skip a build/push step) and the risk if that classification fails. Do not
authorize manual dispatch of any such workflow; report exactly what occurred
after the PR is created.

[...]

## 8. Cross-repository impact

Carried forward from the approved specification's cross-repository impact
section. Restate here so the implementing agent does not need to re-derive it:

- affected contracts: [...]
- owning repository: [...]
- known consumers and their status (`REQUIRES CHANGE` / tracked task, or
  `REMAINS COMPATIBLE` with reason): [...]
- downstream repository-local tasks this handoff must not start: [list, if
  applicable]
- dependency/merge/deployment order: [...]

An implementing agent under this handoff must not modify another repository
and must not give itself or another agent unrestricted write access beyond
section 5's write budget.

## 9. Implementation requirements

[Restate the approved specification's explicit scope, non-goals and
invariants as concrete, bounded instructions. Do not add scope beyond the
approved specification.]

## 10. Acceptance criteria

- [ ] [...]
- [ ] [...]
- [ ] [...]

## 11. Validation

At minimum:

```bash
[repository-appropriate minimum check, e.g. `git diff --check`]
```

Also perform/document, as applicable:

- [link/path/reference-existence review];
- [naming/terminology consistency review];
- [stale-state or duplicate-definition review];
- `git diff --stat`;
- `git status --short`.

Do not run production- or provider-connected validation. If a
repository-supported check exists beyond the minimum above, run it and record
the exact command and result.

## 12. Rollout / runtime impact

State explicitly whether this task has any runtime, schema, migration,
authorization-implementation, CI-workflow, branch-setting or deployment
effect. The expected answer for a bounded documentation/control task is
`NONE`; any deviation must be explicit and justified against the approved
specification.

## 13. HOLD/STOP conditions

Use `HOLD` for a temporary dependency/evidence/authorization/environment
blocker. Use `BLOCKED` where the approved task cannot proceed safely as
specified. Do not reinterpret authorization to get around either.

Stop and report rather than improvise if:

- the authorized base is not at the exact commit above;
- the owning issue materially conflicts with this handoff;
- approved architecture would need to change;
- another repository needs source modification to complete this task;
- another file path is required outside the write budget in section 5;
- a file deletion/move appears necessary;
- CI workflow modification appears necessary;
- provider/runtime access is required;
- branch protection/settings changes are required;
- production credentials are required;
- repository ownership/consumer relationships would need to be guessed;
- unrelated working-tree changes cannot be safely isolated;
- the task cannot be completed without materially broadening scope.

## 14. Required completion report

Provide a final report using
`docs/engineering/ai-delivery/implementation-report-template.md`, including
at minimum:

- task, repository, owning issue, parent issue;
- authorized base vs actual base; task branch; final head; commits; PR;
- files modified/created/deleted (deleted/moved: expected `NONE`);
- write-budget compliance: PASS/FAIL with explanation;
- authorized actions actually used vs the matrix in section 6;
- unauthorized actions performed: `NONE`, or list explicitly and stop;
- cross-repository impact actually realized vs section 8;
- runtime/migration/auth/CI-workflow effect: `NONE` or explain deviation;
- automatic external effects (CI runs, registry publication, other);
- validation performed with exact commands and results;
- CI state;
- deviations: `NONE` or exact deviation plus authorization;
- limitations;
- remaining gates (independent review at exact final head, any further
  authorization, merge, downstream tasks, integration review).

An implementing agent may never approve its own work.
