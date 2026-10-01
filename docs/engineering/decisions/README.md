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

#### Exact-final-blob approval

A material correction uses this approval sequence:

1. Prepare the exact final ADR bytes, including the corrected architecture, `Status: APPROVED`, the fresh approver, the fresh approval date, and any durable approval/authority wording required by the ADR.
2. Calculate the exact Git blob with `git hash-object --no-filters` or an equivalently exact Git-object calculation before commit.
3. The CTO explicitly approves that exact blob in durable GitHub evidence before it is committed.
4. The approval record identifies at minimum the ADR blob SHA, ADR path, owning issue, the approval date carried by the blob, and the inspection basis: either the full exact candidate text or a diff against the prior approved blob.
5. The approval date carried by the blob must match the durable approval record; do not back-date approval metadata. If the date changes, prepare a new blob and obtain fresh exact-blob approval.
6. Only after that approval may the implementing agent commit the ADR, and the committed ADR blob must equal the CTO-approved blob exactly.
7. Any later ADR byte change that is not recorded as a factual clarification under the rules above invalidates the exact-content approval and requires a new exact-blob approval.
8. The final source head, including the approved ADR blob and all companion documentation, then requires fresh independent exact-head review before merge or reliance.
9. CTO merge authorization remains a separate gate.

No earlier approver or approval date may be retained as though it approved corrected decision bytes. Earlier approvals and reviews remain historical evidence bound to their actual exact versions and heads.

An ADR-local amendment or authority clause that merely restates this repository amendment process is governed by this section for an ADR eligible for the material pre-authority exception. A CTO-authorized correction may, and where necessary for consistency must, align that local clause as part of the new exact ADR version. This does not permit overriding an ADR-specific substantive architecture constraint that is not merely a restatement of the general amendment process.

If any approved version of an ADR number has ever become repository-authoritative, a later material architectural change may not use this exception. It requires a new ADR that supersedes the authoritative ADR.

Every update follows normal review and changelog requirements. Nothing in this amendment process authorizes runtime implementation, migration or schema execution, provider/IAM action, deployment, release or production activation.
