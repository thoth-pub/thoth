# ADR-0009 - Programme-integration branch namespace

Status: APPROVED
Date: 2026-08-26
Approved by: Javi, CTO
Approval date: 2026-08-26
Decision owner: CTO
Programmes affected: Shared Engineering Control (owning programme); Thoth Metrics; any current or future programme that adopts the shared programme-integration workflow under these delivery controls
Repositories affected: `thoth-pub/thoth` implements this shared doctrine. The naming grammar governs any repository that adopts these shared delivery controls; each repository's own base, development and release branch names remain owned by that repository.
Supersedes: None
Superseded by: None

Decision: programme integration branches remain `feature/<programme>`;
`PROGRAMME_INTEGRATION` slice branches are **siblings** of the integration
branch, spelled `feature/<programme>--<slice>`; `--` is the reserved
programme/slice separator and must not appear inside a governed identifier;
`STANDARD` task branches remain `feature/<area>/<task>`; and a fail-closed,
symmetric live namespace preflight is required before any governed ref is
created.

Authority condition: this record is repository-authoritative when this exact
approved content is reachable from the repository's authoritative integration
branch (`develop`). A branch carrying `APPROVED` is **not** repository-authoritative
before it merges. `APPROVED` here is the durable decision state owned by the
decision owner; it is not a claim that this shared doctrine is yet in force. No
task may rely on this decision until the exact approved content has received
independent exact-head source review and has been merged into `develop`. That
exact-head-review and merge mechanism is an existing repository control under
[`ADR-0005`](ADR-0005-terminal-merge-evidence.md) and the engineering controls,
not a clause approved on 2026-08-26. Live independent-review, merge-authorization,
CI and merge evidence is the GitHub pull-request record.

Provenance: owning issue
[#837](https://github.com/thoth-pub/thoth/issues/837). Independent review round 1
returned `CHANGES REQUIRED` in issue comment `5426849829`; the fresh review of
the amended specification returned `APPROVED` in comment `5426855358`; CTO
decision approval is comment `5426997804`; bounded implementation authorization
is comment `5427107258`. Blocked consumer:
[#836](https://github.com/thoth-pub/thoth/issues/836) (`MET-WP1-01`), which
remains HOLD.

Verification base: `develop` at `e555b25217b0cdaeae40aa7b84ea6c15363a8282`. The
defect evidence, the live branch inventory and the cross-repository doctrine
search recorded below were produced against this exact commit.

---

## 1. Context

The shared delivery controls define two workflows: `STANDARD`, where a bounded
task branches from and targets the repository's development branch, and
`PROGRAMME_INTEGRATION`, where an approved large programme keeps a long-lived
integration branch and merges bounded slices into it before one final programme
pull request enters the development branch.

Merged doctrine expressed the slice branch as a **descendant** of the
integration branch:

```text
feature/<programme>/<slice>
```

That form cannot be created while the integration branch exists. Git stores a
branch as a loose ref file or a packed ref at `refs/heads/<name>`. A branch
`feature/metrics` occupies `refs/heads/feature/metrics`; a branch
`feature/metrics/wp1-registry-foundation` requires `refs/heads/feature/metrics`
to be a **directory** holding `wp1-registry-foundation`. One path cannot be both
a ref and a ref namespace, so the second branch cannot be created while the
first exists, and the reverse is equally true.

This was not theoretical. At the baseline above, `develop` and `feature/metrics`
were both at `e555b25217b0cdaeae40aa7b84ea6c15363a8282`. Issue #836
(`MET-WP1-01`) received bounded implementation authorization in comment
`5425222882` for the child branch `feature/metrics/wp1-registry-foundation`.
Branch creation returned HTTP 422 `Reference update failed`. A post-failure
search confirmed the child branch was not created and no source implementation
had started. #836 was placed on HOLD in comment `5425836766` and remains HOLD.

The doctrine also conflated the two workflows, prescribing a single
`feature/<programme-or-area>/<task-id-or-short-name>` form for both. A programme
that runs an integration branch therefore cannot use the area-shaped descendant
form for its slices, and doctrine that offers one form for both regenerates the
defect.

The first independent review established that the originally proposed
replacement, a single-hyphen sibling `feature/<programme>-<slice>`, is not a
safe general grammar. `-` is already legal inside programme, area, slice and
task identifiers, so it cannot be a deterministic separator; and the live
repository already contains both `feature/metrics` and the `STANDARD` namespace
`feature/metrics-control/...`, so a programme/slice pair `metrics` + `control`
would produce the flat ref `feature/metrics-control` and recreate the same
prefix collision class against an existing area namespace.

At the baseline, no live branch or tag in `thoth-pub/thoth` used `--`, and no
live flat branch ref was a path prefix of any other live branch ref.

The approved private Metrics design remains authoritative for Metrics
architecture. It requires a repository-local `feature/metrics` integration
branch, with focused child branches created from it and merged back into it. Its
child examples are design-level shorthand such as `metrics/db-foundation` and
`metrics/record-schema`; it does not establish an explicit repository-wide Git
ref grammar and does not require the invalid spelling `feature/metrics/<slice>`.

## 2. Decision drivers

- valid Git ref namespaces while a programme integration branch remains live;
- deterministic parsing of programme and slice identifiers by a human or a tool
  reading a branch name;
- compatibility with the existing `feature/*` convention and with existing PR
  targeting;
- no rename or deletion of valid existing branches solely for naming
  consistency;
- deny-by-default, exact-name authorization with live preflight before branch
  creation;
- minimal migration cost, and no workflow, classifier, ruleset or repository
  settings change unless live evidence proves one necessary;
- repository-local implementation boundaries when shared doctrine is adopted.

## 3. Options considered

### Option A - descendant slice ref

Description: `feature/<programme>/<slice>`, the previously documented form.

Advantages: reads as a hierarchy; matches the `STANDARD` area shape.

Disadvantages: **not creatable** while `feature/<programme>` exists as a branch.
A live `feature/<programme>` integration ref prevents that descendant ref from
coexisting, because Git would require the flat ref's location to be a
directory.

Operational implications: this is the exact failure that blocked #836. It would
be usable only by deleting or renaming the integration branch, which the
programme workflow requires to stay live.

**Rejected.**

### Option B - single-hyphen sibling

Description: `feature/<programme>-<slice>`.

Advantages: a sibling ref, so it avoids the Option A namespace failure; stays
inside `feature/*`.

Disadvantages: `-` is not a reserved separator. It already appears inside
programme, area, slice and task identifiers, so `feature/metrics-control` cannot
be distinguished from programme `metrics` + slice `control`. It can also
recreate ambiguous programme/area namespace collisions: the live repository
holds both `feature/metrics` and `feature/metrics-control/...`, so that exact
pair would collide with an existing `STANDARD` area namespace.

Operational implications: parsing is not deterministic, and the collision class
that blocked #836 returns in a different position.

**Rejected.**

### Option C - reserved-double-hyphen sibling

Description: `feature/<programme>--<slice>`, with `--` reserved as the
programme/slice separator and prohibited inside governed identifiers.

Advantages: a sibling of the integration branch, so it is always creatable
alongside it; stays inside the existing `feature/*` family; deterministically
parseable by splitting at the first reserved `--`; no live branch or tag used
`--` at the baseline, so adoption requires no migration.

Disadvantages: the reserved token must be stated and enforced by doctrine rather
than by tooling; `--` is visually close to `-` and must be written carefully.

Operational implications: no workflow, classifier, branch-protection, ruleset or
script change is required. Existing branches remain valid and untouched.

**SELECTED.**

### Option D - separate top-level slice namespace

Description: `slice/<programme>/<slice>`.

Advantages: structurally valid; hierarchical; no collision with `feature/*`.

Disadvantages: introduces a second top-level branch family, with the additional
doctrine and migration cost of teaching, documenting and protecting it.

Operational implications: no demonstrated control benefit over Option C on
current evidence.

**Rejected.**

### Option E - literal Metrics-design shorthand as Git refs

Description: adopt the Metrics design's child examples literally, for example
`metrics/db-foundation` or `metrics/record-schema`.

Advantages: matches the design document's wording directly.

Disadvantages: those examples are design-level child shorthand establishing
focused child/base/target semantics, not an explicit repository-wide Git-ref
grammar. Adopting them literally moves programme slices outside the `feature/*`
family and still supplies no general `STANDARD`-versus-`PROGRAMME_INTEGRATION`
naming rule.

Operational implications: this decision preserves the design's substantive
child-of-`feature/metrics` intent through the selected sibling convention
instead.

**Rejected as shared repository policy.**

## 4. Decision

### 4.1 Branch grammar

```text
programme integration:       feature/<programme>
PROGRAMME_INTEGRATION slice: feature/<programme>--<slice>
STANDARD task:               feature/<area>/<task>
```

1. Programme integration branches remain `feature/<programme>`.
2. Bounded slice branches under a `PROGRAMME_INTEGRATION` workflow use
   `feature/<programme>--<slice>` and are **sibling** refs of the integration
   branch, not descendants of it.
3. `--` is reserved as the programme/slice separator. Split at the first
   reserved `--` to recover the programme and the slice.
4. `STANDARD` task branch naming remains `feature/<area>/<task>`, available
   where no conflicting flat parent ref occupies that location.

### 4.2 Governed identifiers

Governed `<programme>`, `<area>`, `<slice>` and `<task>` identifiers:

- must be non-empty;
- must each be a single Git path segment;
- must not themselves contain `--`.

No broader leading- or trailing-hyphen prohibition is created by this decision.
Ordinary Git ref-format rules continue to apply and are not restated here.

### 4.3 Pull-request targeting

Slice pull requests continue to target `feature/<programme>`. Final programme
integration continues as `feature/<programme> -> <repository development
branch>`. For `thoth-pub/thoth` the development branch is `develop`.

### 4.4 Fail-closed namespace preflight

Before creating any new governed ref, verify against live remote refs rather
than assumption. The rule is symmetric.

Before creating a new governed **flat** ref, including a programme integration
branch `feature/<programme>`:

- the exact ref does not already exist;
- no descendant ref beneath that flat ref already exists, since such a
  descendant requires the flat ref's location to be a ref namespace. For a
  programme integration branch this means no `feature/<programme>/*` ref may
  already occupy its descendant namespace.

Before creating or using a governed **descendant** namespace such as
`feature/<area>/<task>`:

- no flat parent ref `feature/<area>` already occupies that ref location.

Before creating a programme slice `feature/<programme>--<slice>`:

- the exact ref is absent;
- the identifiers are well-formed under section 4.2;
- no incompatible descendant occupancy exists beneath that prospective flat ref.

On failure: HOLD. No collision may be worked around by deleting, renaming or
moving another branch.

### 4.5 Compatibility and migration

Existing valid branches are not renamed, deleted or moved for cosmetic
consistency. `feature/metrics` is unchanged. Existing `STANDARD` branches,
including the `feature/metrics-control/...` namespace, remain valid. Historical
lifecycle evidence — issues, task specifications, implementation reports, closed
pull requests and review records — is preserved as written and is not rewritten
to the new spelling.

An active task whose authorized branch name used an invalid or superseded form
must HOLD and receive a task-specific specification amendment plus fresh
required review and authorization before proceeding.

No task may work around a namespace collision by deleting, renaming or moving an
integration or `STANDARD` namespace branch unless a separate reviewed
branch-topology or migration task explicitly authorizes that operation.

### 4.6 Explicit exclusions

- Repository-local base, development and release branch names remain owned by
  each repository. This decision standardizes only the shared programme/slice
  naming grammar where these controls are adopted; it is not a universal
  `develop`/`master` mandate.
- This decision does not amend the substantive Thoth Metrics architecture. It
  standardizes the repository ref spelling of programme slices only. The Metrics
  design invariant is preserved: each affected repository owns its own
  `feature/metrics` integration branch, focused children are created from it,
  those children merge back into it, and they do not target `develop` directly.
- No data model, API boundary, authorization rule, runtime behaviour, schema or
  migration is created, changed or activated by this decision.
- No branch is created by this decision. In particular, the future Metrics WP1
  replacement slice `feature/metrics--wp1-registry-foundation` is named here as
  the approved spelling only; creating it is not authorized by this record.

### 4.7 Rollout and rollback

Rollout is doctrine-only:

- no existing branch is renamed, deleted or moved;
- future or resumed programme-slice tasks use this grammar only after this
  decision is repository-authoritative;
- active tasks already authorized with an invalid or superseded branch name
  remain HOLD until individually amended and re-authorized;
- repository-local follow-up issues are created only where live evidence finds
  separately owned local doctrine requiring changes.

Rollback is by a later explicitly approved superseding ADR. Do not roll back by
renaming or deleting live integration branches, or by rewriting historical
lifecycle evidence.

## 5. Consequences

### Positive

- The blocking Git namespace failure that stopped #836 cannot recur for a
  correctly named slice.
- Programme and slice are recoverable from a branch name deterministically.
- `STANDARD` and `PROGRAMME_INTEGRATION` naming are distinguishable at a glance,
  so a specification or handoff cannot silently emit the wrong form.
- Adoption requires no migration: no live branch or tag used `--` at the
  verification base.
- No workflow, classifier, branch-protection, ruleset or script change is
  required.

### Negative

- `--` must be written carefully; it is visually close to `-`.
- The reserved token is enforced by doctrine and review, not by tooling.
- Slice branches no longer sort under their integration branch in a
  hierarchical branch listing.

### Risks

- A future programme or area identifier could be proposed containing `--`. The
  section 4.2 rule and the section 4.4 preflight are the mitigation; such a
  proposal must be renamed, not accommodated.
- A future repository could create `feature/<programme>/*` refs before its
  integration branch exists, blocking the flat ref later. The symmetric
  preflight in section 4.4 is the mitigation.

## 6. Invariants created by this decision

1. `feature/<programme>` remains the programme integration branch where an
   approved programme uses that workflow.
2. `PROGRAMME_INTEGRATION` slices use `feature/<programme>--<slice>`; `--` is
   reserved and may not appear inside a governed identifier component.
3. A branch must never require a ref whose path has an existing branch ref as a
   prefix, and a new flat integration or area ref must never be created where
   descendant refs already occupy that namespace.
4. Slice pull requests target the integration branch, not the repository
   development branch directly, when `PROGRAMME_INTEGRATION` is selected.
5. `STANDARD` task branches remain `feature/<area>/<task>`, subject to the
   bidirectional live namespace preflight.
6. No branch deletion or rename is implied by this decision.
7. Existing historical lifecycle evidence is not rewritten for naming cosmetics.
8. Repository-local branch topology remains authoritative for each target
   repository; this decision supplies a collision-safe naming grammar, not a
   universal base or release branch name.
9. The Metrics design invariant that focused children branch from and merge back
   into repository-local `feature/metrics` is preserved; this decision
   standardizes the child ref spelling only.

## 7. Implementation impact

Affected tasks: `CTRL-BRANCH-NAMESPACE-01` (#837) implements this doctrine
correction in `thoth-pub/thoth`. `MET-WP1-01` (#836) is the blocked consumer.

Required sequencing: this decision becomes repository-authoritative only after
the exact approved implementation receives independent exact-head source review
and is merged into `develop`. #836 does not resume from that merge alone: it
requires its own task-specific amendment changing its authorized child branch to
`feature/metrics--wp1-registry-foundation`, fresh HOLD-sensitive verification,
fresh required review and fresh bounded implementation authorization.

Required migrations: none. No database migration, and no branch migration.

Required client changes: none.

Required operational changes: none. No GitHub Actions workflow, CI classifier,
branch protection rule, ruleset or repository setting changes. At the
verification base, no workflow or script parsed branch names in a way this
grammar affects; the only branch filters present were literal `master` and
`develop` push filters, and the live rulesets targeted tags only.

## 8. Validation

Evidence required to prove the decision works:

- `git check-ref-format` accepts the representative refs
  `refs/heads/feature/metrics`,
  `refs/heads/feature/metrics--wp1-registry-foundation` and
  `refs/heads/feature/example-area/example-task`.
- A live branch inventory shows no existing branch or tag uses `--`, and no flat
  branch ref is a path prefix of another branch ref.
- The impossible descendant relationship in Option A is demonstrated by the
  recorded HTTP 422 `Reference update failed` from #836 and by the Git ref
  storage rule, **not** by deleting or rearranging live refs to reproduce it.
- Active normative doctrine consistently describes `feature/<programme>`,
  `feature/<programme>--<slice>` and `feature/<area>/<task>` with correct PR
  targets, and any remaining occurrence of a rejected form is either explicit
  contextual discussion of that rejected form or preserved historical lifecycle
  evidence.

## 9. Approval

Approved by: Javi, CTO
Approval date: 2026-08-26
Notes: decision approval recorded in issue #837 comment `5426997804`, following
independent review `APPROVED` in comment `5426855358`. This is decision
approval. It is not merge authorization, and it does not make this record
repository-authoritative; see the authority condition above. Per the decision
record template and `ADR-0005`, no later commit is added to this file solely to
copy the independent review identifier, the CTO merge authorization or the merge
commit; those are terminal GitHub evidence.
