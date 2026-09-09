# ADR-0011 - Preserve established repository release branch names

Status: APPROVED
Date: 2026-09-09
Approved by: Javi, CTO
Approval date: 2026-09-09
Decision owner: CTO
Programmes affected: Shared Engineering Control (owning programme); Thoth Metrics; Publisher Services and Distribution Configuration; any current or future programme that uses the shared branch-topology controls
Repositories affected: every repository governed by the shared repository map. `thoth-pub/thoth` implements this shared doctrine. No repository branch, setting, protection or provider configuration is mutated by this decision.
Supersedes: None
Superseded by: None

Decision: shared engineering doctrine standardizes the **role** of a
repository's release branch, not its spelling. That role is written
`<release-branch>` and its concrete value is the repository's own verified,
established release/default branch. A repository established on `main` remains
on `main`; a repository established on `master` remains on `master`. Neither
name may be created, renamed, moved or substituted merely because another
repository uses it.

Authority condition: this record is repository-authoritative when this exact
approved content is reachable from the repository's authoritative integration
branch (`develop`). A branch carrying `APPROVED` is **not**
repository-authoritative before it merges. `APPROVED` here is the durable
decision state owned by the decision owner; it is not a claim that this shared
doctrine is yet in force, and it is not implementation authorization for any
consumer. No task may rely on this decision until the exact approved content has
received independent exact-head source review and has been merged into
`develop`. That exact-head-review and merge mechanism is an existing repository
control under [`ADR-0005`](ADR-0005-terminal-merge-evidence.md) and the
engineering controls, not a clause approved on 2026-09-09. Live
independent-review, merge-authorization, CI and merge evidence is the GitHub
pull-request record.

Provenance: owning issue
[#897](https://github.com/thoth-pub/thoth/issues/897). The approved
specification consists of the original issue body together with Specification
Amendment 1, issue comment `5603738964`. Independent specification review
returned `APPROVED` in comment `5603985920`; CTO architecture and specification
approval is comment `5604021690`; the durable implementation preflight is
comment `5604068628`; bounded implementation authorization is comment
`5604141046`. Parent control programme:
[`CTRL-DELIVERY-01`](https://github.com/thoth-pub/thoth/issues/818). Metrics
coordination: [#766](https://github.com/thoth-pub/thoth/issues/766). Immediate
dependent task: `BR-SPHINX-01`.

Verification base: `develop` at `4546cb632428872b961ad6c17282984d298e3ade`. The
live branch evidence and the doctrine survey recorded below were produced
against this exact commit.

---

## 1. Context

The shared repository-map target policy standardized every managed repository
on one spelling:

```text
feature/<area>/<task> -> develop -> master
```

and, for programme integration:

```text
feature/<programme>--<slice> -> feature/<programme> -> develop -> master
```

That treats `master` as a universal target release/default branch, and
therefore classifies every repository legitimately established on `main` as
requiring release-branch normalization.

Live managed repositories already use both established spellings. At the
verification base the repository map records:

- `master`: `thoth`, the standalone `thoth-pub/thoth-client`, `baboon`;
- `main`: `thoth-app`, `thoth-dissemination`, `thoth-sphinx`,
  `metrics-dashboard`, `metrics-widget`, `cc-license`, `thoth-pyramid`,
  `thoth-strapi`.

Changing an otherwise valid release branch merely to standardize its name
creates cost and risk without architectural benefit. It can require changes to
GitHub default branches and protections, CI branch filters, Vercel production
routing, package publication, release automation, deployment configuration,
external references and rollback procedures. Those are real operational
mutations, several of them HIGH risk, incurred for a naming preference.

The release branch name is repository-local state. Shared engineering control
needs a consistent *role* for that branch, not a universal spelling.

The inconsistency is therefore in the shared target policy and in the active
repository-map and readiness records, not in `thoth`'s own release flow.
`thoth`'s repository-local branching workflow already states that other
repositories are not required to share its `develop`/`master` topology, and
[`ADR-0009`](ADR-0009-programme-integration-branch-namespace.md) section 4.6
already excludes a universal `develop`/`master` mandate from its own scope.

## 2. Decision drivers

- preservation of existing Git history, branch identity and release behaviour;
- avoidance of provider, deployment, publication and protection changes that
  buy nothing but naming consistency;
- a shared vocabulary that lets cross-repository doctrine name the release
  branch without asserting its spelling;
- repository-local live evidence as the owner of concrete branch names;
- separation of release-branch naming from genuinely justified development
  branch, CI, protection and readiness work;
- honest risk classification: removing a rename from a readiness task must not
  silently reclassify that task's remaining external effects;
- no consumer inferring another repository's topology by analogy.

## 3. Options considered

### Option A - universal `master`

Description: retain the current policy and normalize every managed repository to
`master`.

Advantages: one spelling across the estate; no change to already-conforming
repositories.

Disadvantages: requires creating `master`, switching the GitHub default,
re-pointing protections, CI filters, Vercel production routing, npm and crates
publication and release automation in eight repositories, purely to standardize
a name. Several of those tasks are HIGH risk because they move production
deployment routing or public package publication.

Operational implications: `BR-APP-01`, `BR-DIS-01`, `BR-DASH-01` and
`BR-WIDGET-01` each carry external production or publication effects that exist
only because of the rename.

**Rejected.**

### Option B - universal `main`

Description: normalize the estate in the opposite direction, to `main`.

Advantages: matches the current GitHub default convention and the majority of
managed repositories.

Disadvantages: identical in kind to Option A. It moves the unnecessary migration
burden onto repositories already established on `master`, including `thoth`
itself, the standalone `thoth-client` and `baboon`, whose release, tag and
deployment paths currently work.

Operational implications: would create new normalization tasks where none exist
today, including in the repository that owns the release container.

**Rejected.**

### Option C - preserve each repository's established release branch

Description: shared doctrine uses an abstract `<release-branch>` role whose
concrete value is verified independently for each repository. Established `main`
stays `main`; established `master` stays `master`.

Advantages: no branch, provider, protection, CI, publication or deployment
change is required by naming policy at all. Every repository's verified live
state is already conformant on the release-name question, so the remaining
readiness work in each repository is exactly the work that was independently
justified. Shared controls keep a single, unambiguous term for the release
branch.

Disadvantages: doctrine must be read with the substitution in mind, and every
task specification must carry the repository's verified concrete branch names
rather than inheriting them from a shared default.

Operational implications: no workflow, classifier, ruleset, protection or
provider change. Existing branches are untouched.

**SELECTED.**

## 4. Decision

### 4.1 Recorded release branch

1. Every managed repository must record its verified, established
   release/default branch in its own repository-map entry, from live GitHub
   state rather than from convention.

2. Shared engineering doctrine refers to that branch by the role name:

   ```text
   <release-branch>
   ```

### 4.2 Conceptual flows

3. The normal conceptual flow is:

   ```text
   feature/<area>/<task> -> <development-branch> -> <release-branch>
   ```

4. Where a programme integration branch is approved:

   ```text
   feature/<programme>--<slice>
     -> feature/<programme>
     -> <development-branch>
     -> <release-branch>
   ```

For `thoth-pub/thoth` those roles resolve to `develop` and `master`
respectively, so `thoth`'s own flow remains `develop -> master` and is unchanged
by this decision.

### 4.3 Preservation rules

5. A repository already established on `main` remains on `main`.

6. A repository already established on `master` remains on `master`.

7. `main` must not be created, renamed, moved or substituted merely because
   another repository uses `main`.

8. `master` must not be created, renamed, moved or substituted merely because
   another repository uses `master`.

9. Changing an established release/default branch requires a separately scoped
   repository-local decision and task, justified by a reason other than naming
   consistency. That task must assess, as applicable: GitHub settings and
   protections; CI branch filters; deployment and provider routing; release and
   publication automation; external references; compatibility; and rollback.

### 4.4 Independence of the development branch

10. Development-branch policy remains independent of this decision. A
    repository-local readiness task may still normalize an active `dev` branch
    to `develop` where that is separately justified and approved.

### 4.5 Binding of automation

11. Branch protections, CI, release automation, deployment configuration and
    publication configuration must bind to the repository's verified release
    branch rather than to a globally assumed `main` or `master`.

12. Every task specification must use the repository's live verified
    development branch and release branch. Agents must never derive either from
    shared naming convention alone.

### 4.6 Explicit exclusions

13. Historical issues, pull requests, task specifications, implementation
    reports, reviews and release records remain unchanged. They record the
    terminology and state that existed when the work occurred, and are not
    rewritten to conform.

14. [`ADR-0009`](ADR-0009-programme-integration-branch-namespace.md) is
    unaffected. Programme integration branches remain `feature/<programme>` and
    slice branches remain sibling refs `feature/<programme>--<slice>`, with the
    reserved `--` separator and the symmetric fail-closed namespace preflight
    unchanged.

15. This decision changes engineering-control architecture and documentation
    only. It authorizes no branch creation, deletion, rename or movement; no
    default-branch or protection change; no CI, workflow or provider
    configuration change; no repository-local normalization task; and no
    deployment, publication or production activation.

## 5. Invariants created by this decision

1. Existing Git history and branch identity are preserved.
2. A healthy release branch is never renamed for cosmetic consistency.
3. Repository-local live evidence owns the concrete branch name.
4. Shared doctrine owns branch roles and lifecycle controls, not spellings.
5. `main` and `master` are semantically equivalent release-branch spellings
   unless a repository's own verified controls establish otherwise.
6. Development-branch normalization remains a separate question from
   release-branch naming.
7. Existing release, deployment and publication behaviour is unchanged by this
   decision.
8. No consumer may infer another repository's topology by analogy.

## 6. Consequences

### 6.1 Effect on the active branch-readiness tasks

Once this decision is repository-authoritative, the active readiness tasks are
amended as follows. In every case the release-branch **rename** is removed and
the independently justified readiness work is retained.

- `BR-APP-01`: preserve release/default `main`. Remaining work concerns
  development-branch normalization (`dev` to `develop`), protections, CI and
  readiness, with the associated Vercel verification those changes actually
  require.
- `BR-DIS-01`: preserve release/default `main`; the requirement to create or
  switch to `master` is removed. Separately justified protection and
  release-readiness work is retained, including the repository's production
  external-write workflows.
- `BR-SPHINX-01`: preserve release/default `main`; do not create `master` and do
  not switch the default branch for naming consistency. Protect `main` and
  `develop`, preserve the existing `develop` lineage including its root
  `AGENTS.md`, and separately reconcile repository-local controls.
- `BR-DASH-01`: preserve the established Vercel-backed release/default `main`.
  Reconcile the active `dev` history against the stale `develop` separately. Do
  not move Vercel production merely to adopt `master`.
- `BR-WIDGET-01`: preserve release/default `main`. Development-branch and CI
  normalization and npm release protection remain separately assessed.
- `BR-LIC-01`: preserve release/default `main`. Publication readiness and
  protection remain separately assessed.
- `thoth-pyramid`: preserve release/default `main`. Any future
  development-branch normalization is separate.
- `thoth-strapi`: preserve release/default `main`. Any future topology or
  readiness work remains separate and must continue to account for this
  repository's publication-capable pull-request workflow.
- repositories already established on `master`, including `thoth`, the
  standalone `thoth-client` and `baboon`, continue using `master`.

Removing a release-branch rename from a readiness task does **not**
automatically lower that task's risk. Risk must be reassessed from the task's
remaining CI, provider, deployment, publishing and external-write effects, and
those effects are unchanged by this decision.

### 6.2 Positive

- No branch, GitHub setting, protection, CI filter, provider route or
  publication configuration changes for naming reasons.
- The HIGH-risk external effects previously attached to `BR-APP-01`,
  `BR-DASH-01` and `BR-WIDGET-01` solely by the rename are removed from the
  naming question, leaving only independently justified work.
- Shared doctrine can name the release branch unambiguously across repositories
  with different spellings.
- `thoth`'s own established release flow is untouched.

### 6.3 Negative

- Doctrine reads with a substitution rather than a literal branch name.
- Every task specification must carry verified concrete branch names, so a
  specification cannot be written from convention alone.
- The estate keeps two release-branch spellings indefinitely.

### 6.4 Risks

- A future task could assume a spelling by analogy with another repository. The
  section 4.5 rule and the existing repository-map usage rule requiring live
  verification are the mitigation.
- Removing the rename could be misread as closing a readiness gap. Section 6.1's
  explicit risk-reassessment rule, and the retained open control gaps, are the
  mitigation.

## 7. Rollout and rollback

Rollout:

1. Record this exact CTO-approved decision.
2. Implement it and reconcile the active shared branch-topology and control
   documents in one bounded documentation-only `thoth` pull request
   (`CTRL-BRANCH-RELEASE-01`).
3. Independently review the exact pull-request head.
4. Separately authorize merge.
5. This decision becomes repository-authoritative only once the approved,
   reviewed version is reachable from `develop`.
6. Re-specify `BR-SPHINX-01` against the resulting authoritative policy.
7. Continue the MOM-1 Sphinx lane:
   `BR-SPHINX-01 -> SPHINX-BOOT-01 -> MET-WP6-01`.

Rollback:

Before downstream tasks depend on this decision, rollback is a normal revert of
the bounded documentation and control pull request.

After repository-local tasks rely on it, do not silently revert the
architecture: a superseding ADR and impact analysis are required.

No branch, provider or runtime rollback is part of this decision, because it
performs no such mutation.

## 8. Implementation impact

Affected tasks: `CTRL-BRANCH-RELEASE-01`
([#897](https://github.com/thoth-pub/thoth/issues/897)) implements this doctrine
in `thoth-pub/thoth`. `BR-SPHINX-01` is the immediate consumer and must not be
approved against the new topology until this decision is
repository-authoritative.

Required sequencing: independent exact-head review, separate merge
authorization, and merge into `develop`.

Required migrations: none. No database migration, and no branch migration.

Required client changes: none.

Required operational changes: none. No GitHub Actions workflow, CI classifier,
branch protection rule, ruleset, repository setting or provider configuration
changes.

Cross-repository contract impact: the database and domain model, GraphQL/API
schema and behaviour, generated clients and types, authorization semantics,
export formats, configuration and environment contracts, event and job
payloads, dissemination and platform behaviour, UI assumptions, CMS and site
contracts, package and library interfaces, and deployment and compatibility
windows are all **NOT AFFECTED**. Shared engineering-governance semantics *are*
affected across managed repositories, and that governance effect is implemented
only in `thoth-pub/thoth` under `CTRL-BRANCH-RELEASE-01`. Repository-local
controls that still encode the old universal `master` target are reconciled only
by their own separately scoped, separately authorized owning tasks.

## 9. Validation

Evidence required to prove the decision works:

- the active shared branch-topology control expresses the target flow using
  `<release-branch>` rather than a universal `master`, and its observed-state
  table records each repository's verified release branch as conformant on the
  release-name question;
- no active changed control requires creating `master` in, or switching the
  default branch of, a repository established on `main`;
- no active changed control requires converting a repository established on
  `master` to `main`;
- `thoth`'s own `develop -> master` release flow is unchanged in
  `AGENTS.md`, `branching-and-release-workflow.md`, `release-gates.md` and
  `repositories/thoth.md`;
- the dashboard control no longer requires moving Vercel production from `main`
  for branch spelling, and the widget and licence controls no longer require a
  `main` to `master` conversion for naming consistency;
- development-branch normalization, protection, CI, provider and publication
  readiness remain recorded as separate, still-open work;
- `ADR-0009` is unchanged in substance;
- historical lifecycle evidence is unchanged.

## 10. Approval

Approved by: Javi, CTO
Approval date: 2026-09-09
Notes: decision approval is recorded in issue #897 comment `5604021690`,
following independent specification review `APPROVED` in comment `5603985920`
and Specification Amendment 1 in comment `5603738964`. Bounded implementation
authorization is comment `5604141046`. This is decision approval. It is not
merge authorization, and it does not make this record repository-authoritative;
see the authority condition above. Per the decision record template and
`ADR-0005`, no later commit is added to this file solely to copy the independent
review identifier, the CTO merge authorization or the merge commit; those are
terminal GitHub evidence.
