# ADR-0005 - Terminal merge evidence and non-recursive closeout

Status: APPROVED
Date: 2026-08-07
Decision owner: CTO
Approval basis: CTO-approved policy direction recorded in
[issue #786](https://github.com/thoth-pub/thoth/issues/786), 2026-08-07
Programmes affected: Shared Engineering Control; all programmes using the AI-led delivery controls (Publisher Services, Thoth Metrics, Shared Repository Controls)
Repositories affected: `thoth-pub/thoth`; the shared control model it defines applies to every repository that adopts these delivery controls
Supersedes: None
Superseded by: None

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch
(`develop`). Live review, authorization and merge evidence for the pull request
that introduced it is the GitHub pull-request record.

## 1. Context

The AI-led delivery controls in `docs/engineering/ai-delivery/` require an
approved specification, bounded tasks, independent substantive review bound to
an exact pull-request head, CI evidence, and explicit CTO merge authorization
for high-risk or explicitly gated work. Those controls are working and are not
in question.

A separate convention grew up alongside them. Committed control documents came
to carry transient workflow status such as:

```text
PENDING MERGE
IMPLEMENTATION NOT AUTHORIZED
requires fresh review
becomes repository-authoritative on merge
merge authorization pending
```

Once independent review and CTO authorization were obtained, those identifiers
were copied into the committed files. That produced a distinct class of commit
whose only content was lifecycle metadata: review IDs, approval IDs, merge
authorization IDs, and status transitions.

The repository history shows the resulting shape directly. `ADR-0004` was
approved at content head `44e6f821` under review `4881233664`, then a separate
"approval-state head" `82874c2b` existed only to record that approval, and
required its own independent review `4881832108` and its own CTO merge
authorization `4881847699`. After PR
[#783](https://github.com/thoth-pub/thoth/pull/783) merged, PR
[#785](https://github.com/thoth-pub/thoth/pull/785) existed to record that it
had merged. The same pattern produced PRs
[#767](https://github.com/thoth-pub/thoth/pull/767),
[#773](https://github.com/thoth-pub/thoth/pull/773),
[#776](https://github.com/thoth-pub/thoth/pull/776) and
[#782](https://github.com/thoth-pub/thoth/pull/782).

## 2. Decision drivers

- Independent substantive review is a scarce, valuable control. It should be
  spent on substantive change, not on verifying that an identifier was
  transcribed correctly.
- Copying an approval into a tracked file changes the commit SHA, which
  invalidates the exact-head review that justified the copy. The control is
  self-defeating when applied to its own metadata.
- After merge, committed "pending merge" prose is false. Correcting it requires
  another PR, which itself merges, which makes its own status prose stale.
- GitHub already holds immutable, timestamped, attributable records of the exact
  head, the review, the reviewer, the CTO authorization, CI, the merge event and
  the merge commit. A Markdown transcription of those records is a strictly less
  reliable second copy.
- The repository's own authority order (`operating-model.md` section 3, root
  `AGENTS.md` section 2) already ranks GitHub pull requests, review threads and
  CI evidence as an authoritative source. The recursive convention treated them
  as insufficient until transcribed.

## 3. Options considered

### Option A - Retain the current convention

Description: continue recording each review, authorization and merge event by
changing repository files, and independently re-review each metadata update.

Advantages: every lifecycle fact is readable from a Git checkout alone, without
network access to GitHub.

Disadvantages: each recording commit invalidates the exact-head review that
justified it and requires a fresh one; each closeout PR merges and creates its
own stale status, which the convention says must be closed out. The work is
unbounded and adds no substantive assurance. Reviewer attention is consumed by
transcription checking.

Operational implications: sustained growth in administrative PRs; slower
delivery; a real risk that reviewers habituate to rubber-stamping metadata
diffs and carry that habit into substantive ones.

### Option B - Terminal merge evidence (chosen)

Description: treat the GitHub review, authorization, CI and merge record as
terminal lifecycle evidence. Write committed documents in durable language so
that merging does not falsify them. Require a new PR only when repository
content is materially wrong.

Advantages: bounded control work; no self-invalidating commits; review effort
concentrated on substantive change; committed documents stay true before and
after merge.

Disadvantages: reviewers and auditors must consult the GitHub PR record for
lifecycle evidence rather than reading it from a checkout; authors must write
durable rather than transient status prose.

Operational implications: lifecycle auditing depends on GitHub availability and
retention. This is already true for review threads and CI, which the authority
order already relies on.

### Option C - Automate the metadata commits

Description: keep the convention but generate approval-state commits
mechanically.

Advantages: less manual effort per cycle.

Disadvantages: requires CI workflow and repository automation changes, which are
out of scope for this decision; automation that pushes to a reviewed branch
still moves the head and still invalidates exact-head review; it entrenches the
recursion rather than removing it.

Operational implications: rejected without prejudice. If the CTO later wants
generated evidence artefacts, that is a separate bounded task with its own
workflow-change authorization.

## 4. Decision

Adopt the **terminal merge evidence rule**.

### 4.1 The rule

1. An implementation reaches one final reviewable head.
2. Independent substantive review is bound to that exact head.
3. CTO merge authorization, where required, is recorded on the same pull request
   and bound to that exact head.
4. Controlled merge uses an expected-head guard, so the merge fails if the
   reviewed head has moved.
5. The successful GitHub merge event and the resulting merge commit are terminal
   evidence that the task merged.
6. No new repository commit or pull request is required solely to record any of:
   - independent review identifiers;
   - CTO approval identifiers;
   - CTO merge-authorization identifiers;
   - the fact that the pull request merged;
   - the merge commit SHA;
   - the merged timestamp;
   - a transition from "pending merge" to "merged";
   - a transition from "implementation authorized" to "implementation complete";

   when those facts are already authoritatively present in the GitHub pull-request
   record.
7. An optional post-merge evidence comment may be added to the already merged
   pull request.
8. Such a comment is evidence only. It requires no new branch, commit or pull
   request, no further review and no further merge authorization.
9. A new post-merge task and pull request is required only when a material
   repository change is genuinely necessary.
10. **Approval-state-only commits are prohibited** when their sole purpose is
    copying existing GitHub review or approval metadata into repository files.

### 4.2 Ownership and boundary

This decision governs *lifecycle evidence* only: how a task's review,
authorization and merge are recorded. It does not govern what is reviewed, who
may review it, or what must be true before merge.

### 4.3 Compatibility

Historical records remain valid as written. This decision is not retroactive and
does not require any existing document, merged pull request, review or
implementation report to be rewritten to match it. See section 9.

### 4.4 Migration

None. No schema, data or runtime state is affected.

### 4.5 Rollout

The rule becomes authoritative when the pull request introducing this ADR merges
into `develop`. Tasks in flight at that point may complete under either form;
tasks starting afterwards use this one.

### 4.6 Explicit exclusions

This decision does not change branch protection, repository settings, CI
workflow code, auto-merge configuration, or any programme's architecture.

## 5. Authority model for lifecycle evidence

For task lifecycle evidence, authority runs in this order:

1. committed specifications and ADRs define what is authorized and required;
2. the exact pull-request diff and head define what is proposed for merge;
3. GitHub review records define independent review decisions;
4. GitHub CTO authorization records define merge authorization;
5. CI records define automated validation evidence;
6. the GitHub merge event and merge commit define whether the task actually
   merged.

A lower-level live GitHub event must not be duplicated into a new repository
commit merely so that a Markdown file repeats it. Repository documents may
reference the pull request; they need not copy every review identifier or merge
timestamp.

This refines, and does not replace, the general authority order in
`operating-model.md` section 3 and root `AGENTS.md` section 2.

## 6. Durable and transient state

Committed files must distinguish durable repository state from transient
workflow state.

A committed document must not be required to assert:

```text
PENDING MERGE
AWAITING REVIEW
AWAITING CTO MERGE AUTHORIZATION
MERGE NOT YET COMPLETE
```

when GitHub is the live authority for that question.

Prefer durable form:

```text
Decision:
<the actual durable decision>

Authority condition:
This record is repository-authoritative when this exact content is reachable
from the repository's authoritative integration branch.

Live review, authorization and merge evidence:
GitHub pull-request history.
```

Materially equivalent wording consistent with repository conventions is
acceptable. The test is that the wording remains truthful before review, after
review, before merge and after merge — so that the act of reviewing or merging
does not itself require another commit to correct status prose.

The `PROPOSED`/`APPROVED`/`SUPERSEDED`/`REJECTED` status vocabulary in
`docs/engineering/decisions/README.md` is retained. It records a durable decision
state owned by the CTO, not a transient pull-request state, and committing a
`PROPOSED` ADR still does not approve it.

## 7. Consequences

### Positive

- fewer administrative commits;
- fewer redundant pull requests;
- fewer reviews invalidated by their own recording commit;
- faster delivery of substantive work;
- a clear distinction between durable architecture and transient workflow state;
- reviewer attention concentrated on change that can actually be wrong.

### Negative

- reviewers and auditors must inspect the GitHub pull-request record for
  lifecycle evidence instead of expecting it in Markdown;
- authors must write durable status prose, which takes more care than stating a
  current status;
- lifecycle auditing depends on GitHub availability and retention.

### Risks

- **Drift risk:** a committed tracker could disagree with GitHub. Mitigation:
  trackers reference the pull request rather than restating its lifecycle facts,
  so there is less to drift.
- **Under-recording risk:** a genuine material post-merge correction could be
  dismissed as "just metadata". Mitigation: section 8 states the test
  explicitly, and the retained controls in section 10 are unchanged.
- **Erosion risk:** this decision could be misread as general permission to skip
  documentation PRs. Mitigation: section 4.2 bounds it to lifecycle evidence.

## 8. When a post-merge pull request is still required

A separate post-merge task is legitimate when the merge reveals or creates a
material state that GitHub evidence alone cannot carry.

Required:

```text
- a committed tracker contains materially incorrect programme state;
- an ADR contains a substantive contradiction;
- a migration requires a follow-up correction;
- runtime verification discovers a defect;
- authorization or security behaviour differs from the approved design;
- a production rollout requires a separately reviewed repository change;
- a substantive documentation error;
- a state transition that could not reasonably have been represented before merge.
```

Not required:

```text
- "the PR is now merged";
- "the merge SHA is X";
- "review ID was Y";
- "CTO authorized merge";
- "the task is complete";
- "the PR is no longer draft";
```

when those facts are already conclusively represented by GitHub.

## 9. Historical records

Existing pull requests, reviews, evidence comments, implementation reports and
control records remain historical evidence and are preserved as written. They are
not rewritten to conform to this decision retrospectively, and no merged
pull-request description or comment is retroactively modified.

Stale state discovered in historical records is recorded as separate control debt
and corrected, if at all, under its own bounded task. It is not repaired
opportunistically under this decision.

## 10. Invariants created by this decision

1. One bounded task per branch and pull request.
2. An approved written specification is required before implementation.
3. Independent substantive review remains mandatory where it is currently
   required, and inspects the actual diff, tests, CI, migrations and
   authorization rather than a narrative report.
4. An implementing agent may not approve or merge its own work.
5. Independent review is bound to an exact pull-request head.
6. If the reviewed head changes for any repository commit, substantive or not,
   the previous exact-head review does not carry forward and fresh review is
   required. A commit must not be created solely to copy the previous review or
   approval into the repository.
7. Pull-request body edits and GitHub comments do not change the Git commit and
   therefore may record metadata without invalidating the reviewed head.
8. Controlled merge is guarded by the expected reviewed head.
9. CTO merge authorization remains mandatory for HIGH and CRITICAL risk work and
   anywhere a specification or control explicitly requires it, including
   explicitly gated LOW and MEDIUM work.
10. Merge is not production activation. Production activation, deployment,
    migration execution and release remain separately controlled and separately
    authorized.
11. A material post-merge correction requires its own bounded task and pull
    request.
12. Approval-state-only commits are prohibited when their sole purpose is
    copying existing GitHub review or approval metadata into repository files.
13. Missing evidence is missing work.

## 11. Implementation impact

Affected tasks: `CTRL-MERGE-01`.

Required sequencing: none. This decision has no dependencies.

Required migrations: none.

Required client changes: none.

Required operational changes: agents and reviewers follow the lifecycle in
`operating-model.md` section 5 and the merge-ready gate in `release-gates.md`
section 1 as updated by `CTRL-MERGE-01`.

## 12. Validation

Evidence that this decision works:

- a task completes review, authorization and merge without any
  approval-state-only commit;
- no post-merge pull request is opened solely to record that the previous pull
  request merged;
- the committed documents produced by that task remain truthful after merge
  without editing;
- exact-head review, implementer/reviewer separation, guarded merge, HIGH-risk
  CTO merge authorization and production controls are all still observably
  applied.

## 13. Rejected alternative

Rejected: continuing to record every review, authorization and merge event by
changing the repository and then independently re-reviewing the metadata update.

Rejected because it creates unbounded recursive control work — each recording
commit invalidates the review that justified it, and each closeout merge creates
the stale state that demands the next closeout — while adding no substantive
assurance beyond what the GitHub record already provides immutably.

## 14. Approval

Approved by: CTO
Approval date: 2026-08-07
Approval basis: the CTO-approved policy direction recorded in
[issue #786](https://github.com/thoth-pub/thoth/issues/786).

Notes: this is the durable decision approval. It is the decision owner's
adoption of the terminal merge evidence rule, and it is complete.

It is distinct from the merge controls that apply to the pull request carrying
this content. Independent exact-head review and explicit CTO merge authorization
remain required before that pull request may merge, and they govern whether this
exact content reaches `develop` — not whether the decision itself is adopted.

Per the authority condition above, this record is repository-authoritative when
this exact approved content is reachable from `develop`. The policy is not
effective from an unmerged branch. Live independent-review, merge-authorization,
CI and merge evidence is the GitHub pull-request record and is not copied here.
