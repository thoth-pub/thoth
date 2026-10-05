# Engineering Decisions

Status: ACTIVE decision process
Owner: CTO

This directory contains cross-programme Architecture Decision Records and their normative appendices.

## Authority

An ADR status records durable decision state. `APPROVED` means the named decision owner has approved the exact decision content; `APPROVED` alone does not make an ADR repository-authoritative.

For `thoth-pub/thoth`, an exact ADR version is repository-authoritative only when:

1. its status is `APPROVED`;
2. the exact final source head has received the required independent exact-head review; and
3. the approved and reviewed version is reachable from `develop`.

Normal implementation may rely on a decision only when it is repository-authoritative, unless the work is using the narrower exact-version programme-integration reliance mechanism governed by repository-authoritative `ADR-0013`.

Statuses:

- `PROPOSED` - complete recommendation awaiting the named decision owner;
- `APPROVED` - the named decision owner has approved the exact decision content; repository authority and implementation reliance still require the applicable authority condition above;
- `SUPERSEDED` - replaced by another approved ADR;
- `REJECTED` - considered and not adopted.

Committing a proposed ADR does not approve it. Approval by itself does not make an ADR repository-authoritative.

## Current decisions

See `decision-register.md`.

## Numbering

Use:

```text
ADR-NNNN-short-title.md
```

Numbers are repository-wide and never reused.

## Required content

Each ADR must identify:

- decision owner;
- affected programmes and repositories;
- context and decision drivers;
- options considered;
- exact decision;
- invariants;
- implementation impact;
- migration, rollout and rollback effects;
- validation evidence;
- approval state.

## Cross-programme rule

A programme conversation may propose a cross-programme decision, but only the CTO control process may approve it.

Implementation must stop when it depends on a decision that is not repository-authoritative, except where repository-authoritative `ADR-0013` explicitly permits exact-version programme-integration reliance and every condition of that mechanism is satisfied.

## Amendments

Before implementation starts, a proposed ADR may be edited in place.

After approval, the CTO records whether a proposed update is a `FACTUAL CLARIFICATION` or a `MATERIAL ARCHITECTURAL CORRECTION`. An independent reviewer may challenge that classification; an unresolved challenge blocks merge. If the classification is uncertain, treat the change as material.

### Factual clarification

A factual clarification may update an approved ADR in place only when it does not alter the selected architecture decision.

The CTO classification record must identify the exact prior approved ADR blob being clarified. Because the selected decision is unchanged, the existing architecture approval may remain, but:

- the new source head requires the normal independent exact-head review;
- any source/head-bound merge authorization becomes stale when the source head changes;
- any `ADR-0013` exact-version programme-local reliance pin must be explicitly reconciled before new dependent work proceeds; and
- historical approval and review evidence must remain truthful.

A byte change not explicitly recorded by the CTO as a factual clarification is governed by the material-correction rules below.

### Material correction before repository authority

A material architectural correction may update an approved ADR in place only when all of the following are true:

1. the ADR is already `APPROVED`;
2. the CTO explicitly authorizes the material in-place correction; and
3. no approved version of that ADR number has ever been repository-authoritative.

This exception is keyed to the ADR number, not to a file path or one branch-local version.

For this eligibility test, any commit that was at any time reachable from `develop` and contained that ADR number with `Status: APPROVED` counts as prior repository authority, irrespective of the completeness of historical review evidence. ADRs already reachable from `develop` as `APPROVED` when this rule becomes repository-authoritative remain treated as repository-authoritative under the controls in force at their merge. A later revert, deletion, removal from the active set or supersession does not restore eligibility. Reachability only from a `feature/<programme>` integration branch under `ADR-0013` programme-local reliance does not count as repository authority.

For these controls, an ADR version is the exact Git blob of the ADR file. A material correction creates a new exact ADR version. Where applicable, the correction makes stale:

- CTO approval bound to an earlier ADR blob or source head;
- independent review bound to an earlier source head;
- `ADR-0013` programme-local reliance pinned to the earlier exact version;
- CTO merge authorization for the earlier source head;
- any equivalent SHA-, blob- or content-bound authorization; and
- any branch-local decision-register assertion that identifies the earlier version's approval state or content.

Existing `ADR-0013` impact-assessment requirements for dependent programme slices continue to apply.

#### Approval routes for a material correction

A material correction before repository authority uses exactly one of two
routes. The CTO selects and records the route in the correction's task
authorization before the implementing agent creates any correction commit:

- **Route A - pre-commit exact-final-blob approval**: the CTO approves the exact
  final ADR blob before it is committed.
- **Route B - PR-first staged approval**: the corrected candidate is pushed to a
  real task branch and pull request as `PROPOSED`, the CTO approves its exact
  content from that pull request, one bounded approval-state commit then records
  `APPROVED`, and the CTO approves the exact final ADR blob before merge.

Both routes end with the same exact-version gates: CTO exact-final-blob
approval of the final ADR blob, fresh independent review bound to the exact
final source head, and separate CTO merge authorization where the task's risk or
governing control requires it. Route B changes only *when* exact-final-blob
approval occurs, not whether it occurs. Neither route changes the eligibility
test above, the list of controls a material correction makes stale, the
`ADR-0013` exact-version reliance and drift rules, or the repository-authority
condition.

#### Route A - pre-commit exact-final-blob approval

A material correction under Route A uses this approval sequence:

1. Prepare the exact final ADR bytes, including the corrected architecture, `Status: APPROVED`, the fresh approver, the fresh approval date, and any durable approval/authority wording required by the ADR.
2. Calculate the exact Git blob with `git hash-object --no-filters` or an equivalently exact Git-object calculation before commit.
3. The CTO explicitly approves that exact blob in durable GitHub evidence before it is committed.
4. The approval record identifies at minimum the ADR blob SHA, ADR path, owning issue, the approval date carried by the blob, and the inspection basis: either the full exact candidate text or a diff against the prior approved blob.
5. The approval date carried by the blob must match the durable approval record; do not back-date approval metadata. If the date changes, prepare a new blob and obtain fresh exact-blob approval.
6. Only after that approval may the implementing agent commit the ADR, and the committed ADR blob must equal the CTO-approved blob exactly.
7. Any later ADR byte change that is not recorded as a factual clarification under the rules above invalidates the exact-content approval and requires a new exact-blob approval.
8. The final source head, including the approved ADR blob and all companion documentation, then requires fresh independent exact-head review before merge or reliance.
9. CTO merge authorization remains a separate gate.

#### Route B - PR-first staged approval

Route B lets control review a correction candidate from an actual GitHub pull
request instead of an out-of-band pre-commit candidate transport. It keeps the
candidate, content-approval, approval-state, exact-final-blob, review and merge
stages distinct:

1. **Candidate stage.** The implementing agent prepares the corrected ADR on
   the bounded task branch and opens or updates a draft pull request. The pushed
   candidate carries `Status: PROPOSED`. It must not present the earlier
   version's approver, approval date or approval statement as approval of the
   corrected bytes. Earlier approval evidence may remain in the candidate only
   where it is explicitly described as historical and bound to the earlier exact
   blob or source head. Companion durable records in the same candidate, such as
   the decision-register row, changelog entry and implementation report, must
   likewise describe the corrected version as proposed and must not assert that
   it is approved. The earlier exact version's historical `APPROVED` state is
   unaffected by the candidate's `PROPOSED` status; the two states belong to
   different exact versions.
2. **Content-approval stage.** The CTO reviews the exact correction content
   from the pull request and records architecture-content approval in durable
   GitHub evidence. That record identifies at minimum the exact candidate ADR
   blob SHA, ADR path, owning issue, the source head reviewed and the inspection
   basis: the full candidate text or a diff against the exact prior approved
   blob, which the record names. Content approval is approval of the corrected
   architecture; it is not approval of the final bytes, not independent review,
   and not merge authorization.
3. **Approval-state stage.** Only after that content approval may the
   implementing agent create exactly one bounded approval-state commit on the
   same task branch, as a direct child of the content-approved head. It may
   change only: the ADR status from `PROPOSED` to `APPROVED`; the fresh approver
   and the fresh approval date; the ADR's own approval/authority wording required
   to make the approved record truthful; and the companion durable metadata the
   task specification already authorizes, such as the decision-register row,
   changelog entry and implementation report. The architecture content approved
   at the content-approval stage may not change. Any other ADR byte change
   returns the correction to the candidate stage and requires fresh content
   approval. The approval date carried by the blob is the date of the durable
   content-approval record; it must not be back-dated and must not anticipate a
   later record.
4. **Exact-final-blob stage.** The final ADR blob is now visible in GitHub. The
   CTO records exact-final-blob approval against that blob **before merge**
   rather than before commit. The approval record carries the same fields as
   Route A: the final ADR blob SHA, ADR path, owning issue, the approval date
   carried by the blob, and the inspection basis, naming the content-approved
   candidate blob when the basis is a diff. The approval date carried by the
   blob must match the content-approval record it cites; if it does not, prepare
   a new approval-state blob through the approval-state stage again and obtain
   fresh exact-final-blob approval.
5. **Review stage.** The final source head, including the approval-state commit
   and all companion documentation, then requires fresh independent exact-head
   review before merge or reliance. Any later source commit invalidates that
   review. Any later ADR byte change that is not recorded as a factual
   clarification under the rules above also invalidates the exact-final-blob
   approval and requires a new one.
6. **Merge stage.** CTO merge authorization, where required, remains a separate
   gate bound to the exact reviewed head, and the merge itself remains guarded
   by that expected head. Merge remains separate from runtime implementation,
   migration execution, provider/IAM/runtime action, deployment, release and
   production activation.

Under Route B, no materially corrected ADR bytes may carry current `APPROVED`
status before the content-approval record exists, and an approval-state commit
created without a durable content-approval record for the exact parent head is
unauthorized. If the content-approved head moves for any reason other than the
single authorized approval-state commit, the correction returns to the candidate
stage. Route B requires no force push, amend or history rewrite; the candidate
and approval-state commits are ordinary additive commits on the task branch.

**Why the Route B approval-state commit is consistent with `ADR-0005`.**
`ADR-0005` prohibits a commit whose sole purpose is copying review identifiers,
approval identifiers, merge-authorization identifiers, merge SHAs, timestamps or
other transient lifecycle facts that GitHub already holds authoritatively. The
Route B approval-state commit is not such a commit. It records durable decision
state that only the repository can carry and that the corrected ADR needs in
order to be truthful: the `PROPOSED` to `APPROVED` status transition, the
approver and approval date that the decision statuses in this document and the
required ADR content already demand, and the companion durable metadata that
must agree with that state. It is the first commit in which the corrected
version can truthfully say `APPROVED`, so it is substantive decision content,
not a transcription of lifecycle evidence. It must not be used to copy
content-approval, exact-final-blob approval, review or merge identifiers into
repository files; those remain GitHub-owned lifecycle evidence under `ADR-0005`,
and the ADR, decision register and implementation report reference the owning
issue and pull request rather than restating them.

#### Rules common to both routes

No earlier approver or approval date may be retained as though it approved corrected decision bytes. Earlier approvals and reviews remain historical evidence bound to their actual exact versions and heads.

An ADR-local amendment or authority clause that merely restates this repository amendment process is governed by this section for an ADR eligible for the material pre-authority exception. A CTO-authorized correction may, and where necessary for consistency must, align that local clause as part of the new exact ADR version. This does not permit overriding an ADR-specific substantive architecture constraint that is not merely a restatement of the general amendment process.

If any approved version of an ADR number has ever become repository-authoritative, a later material architectural change may not use this exception. It requires a new ADR that supersedes the authoritative ADR.

Every update follows normal review and changelog requirements. Nothing in this amendment process authorizes runtime implementation, migration or schema execution, provider/IAM action, deployment, release or production activation.
