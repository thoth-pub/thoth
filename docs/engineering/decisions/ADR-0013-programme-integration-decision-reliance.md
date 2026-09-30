# ADR-0013 - Programme-integration reliance on approved decisions

Status: PROPOSED
Date: 2026-09-30
Decision owner: CTO
Programmes affected: Shared Engineering Control; THOTH-ASYNC-01 as the first intended opt-in consumer; any future programme that explicitly adopts this mechanism
Repositories affected: `thoth-pub/thoth` for the shared control model; any other repository only through a separately authorized repository-local adoption task
Parent programme: [THOTH-ASYNC-01 #957](https://github.com/thoth-pub/thoth/issues/957)
Authoring task: [THOTH-ASYNC-01-CTRL-01 #961](https://github.com/thoth-pub/thoth/issues/961)
Superseded by: None
Supersedes: None

Decision: preserve repository authority as the state in which an approved
decision is reachable from the repository's authoritative development branch,
while adding a narrower, explicitly opt-in **programme-integration reliance**
state. Programme-integration reliance allows separately authorized work inside
one designated `feature/<programme>` integration line to rely on one exact
approved and independently reviewed decision version after that exact version
has been merged into that programme integration branch. It does not make the
decision repository-authoritative, does not bind unrelated programmes or normal
development-branch work, and authorizes no production or provider action.

Authority condition: this ADR changes shared engineering-control doctrine only
when this exact ADR-0013 content is `APPROVED`, independently reviewed at its
exact source head, and reachable from `develop`. Until then, the existing
default rule remains unchanged: implementation may rely on an ADR only after the
approved version is merged into `develop`.

This ADR is control-plane doctrine only. It creates no programme integration
branch, task branch, runtime implementation, schema, migration, API, generated
contract, IAM/provider configuration, deployment, release or production
activation.

---

## 1. Context

Thoth already supports a programme-integration workflow for work that should be
assembled and validated coherently before entering the repository development
branch:

```text
develop
  -> feature/<programme>
  -> feature/<programme>--<slice>
  -> feature/<programme>
  -> develop
```

That workflow solves source-integration and validation concerns, but the current
decision-authority rule has one important consequence: an implementation task
may not rely on an approved ADR until that ADR is merged into `develop`.

For most work this is the correct default. It prevents an unmerged branch from
silently becoming repository policy.

A programme-integration workflow can nevertheless need a stronger isolation
boundary. A programme may deliberately keep a coherent architecture and its
implementation slices off `develop` while another programme has delivery
priority, or because intermediate states should not enter the repository's
normal integration line. In that case, forcing the architecture ADR into
`develop` merely so its own programme branch may implement against it defeats
part of the reason for using the programme integration branch.

The concrete first intended consumer is `THOTH-ASYNC-01`. Programme Workflow
Amendment 1 on #957 selects `feature/worker` as its integration line and holds
PR #960 from entering `develop`. ADR-0012 is approved and independently
reviewed on its separately controlled candidate, but it remains
non-repository-authoritative until it reaches `develop`. Under current doctrine
that also means no worker-programme implementation may rely on it while it is
only present on `feature/worker`.

This ADR addresses the general control problem. It does not approve ADR-0012,
create `feature/worker`, retarget PR #960 or authorize any worker implementation.

## 2. Decision drivers

- preserve one clear repository-authority boundary at `develop`;
- support genuinely isolated programme-integration delivery without using
  `develop` as a staging branch;
- prohibit implementation against proposed, unapproved or unreviewed decisions;
- bind programme-local reliance to exact decision bytes and exact branch
  ancestry rather than to a title or mutable branch name alone;
- make programme adoption explicit, opt-in and non-retroactive;
- preserve one bounded task, branch, PR and independent review per slice;
- prevent unrelated programmes or repositories from treating a programme-local
  decision as repository policy;
- prevent downstream repositories from guessing an unmerged contract;
- keep merge, deployment, migration execution and activation separate;
- fail closed when a relied-upon decision changes.

## 3. Options considered

### Option A - Require every relied-upon ADR to merge to develop first

Description: retain the current universal rule with no programme exception.

Advantages:

- one simple authority state;
- every implementation task reads decisions from the normal development branch.

Disadvantages:

- a programme cannot remain genuinely isolated while still implementing its own
  approved architecture;
- architecture-only changes must enter `develop` earlier than the coherent
  programme even when the CTO intentionally wants the programme held back;
- `develop` becomes an incremental staging area for a programme that selected
  integration-branch delivery specifically to avoid that.

**Rejected as the only available rule. It remains the default.**

### Option B - Treat every approved ADR branch as implementation-authoritative

Description: once the CTO approves an ADR, any implementation task may depend on
the approved branch even before merge.

Advantages:

- minimal process overhead.

Disadvantages:

- branch-local policy leaks repository-wide;
- unrelated work could depend on unmerged decisions;
- mutable branch names become de facto policy references;
- the repository authority boundary becomes ambiguous;
- no programme or exact-version containment exists.

**Rejected.**

### Option C - Programme-integration reliance on an exact approved decision

Description: preserve repository authority at `develop`, but allow an
explicitly opted-in programme to rely on one exact approved, independently
reviewed decision version after that version is merged into the programme's own
integration branch.

Advantages:

- preserves the repository-wide authority boundary;
- keeps incomplete programmes off `develop`;
- exact-version and same-integration-line requirements make the exception
  bounded and reconstructable;
- adoption is explicit and non-retroactive;
- downstream repository dependencies remain pinned and separately controlled.

Disadvantages:

- introduces a second, narrower reliance state that must not be confused with
  repository authority;
- programme controls must record exact decision and branch evidence;
- a material decision change can force dependent programme work back to HOLD.

**SELECTED.**

## 4. Decision

### 4.1 Two distinct states

Repository authority and programme-integration reliance are deliberately
different.

**Repository-authoritative decision**

An approved decision is repository-authoritative when the repository's normal
authority condition is satisfied, including reachability from the authoritative
development branch. For `thoth-pub/thoth`, that branch is `develop`.

Repository authority may be relied on by work whose own specification permits
it, regardless of programme branch topology.

**Programme-integration reliance**

Programme-integration reliance is a narrower exception available only under
section 4.2. It means one programme integration line may rely on one exact
decision version before that decision becomes repository-authoritative.

Programme-integration reliance:

- does not make the ADR repository-authoritative;
- does not amend the development branch;
- does not bind unrelated programmes;
- does not authorize normal `develop` work to depend on the ADR;
- does not authorize another repository to treat the ADR as merged policy;
- does not authorize deployment or activation.

### 4.2 Eligibility gate

A programme may use programme-integration reliance for a decision only when all
of the following are true:

1. ADR-0013 itself is `APPROVED`, independently reviewed at its exact source
   head, and repository-authoritative.
2. The relied-upon ADR is already `APPROVED`.
3. The relied-upon ADR records its approver and approval date.
4. The exact relied-upon ADR content has received independent exact-head review.
5. The CTO/programme control explicitly designates one
   `feature/<programme>` integration branch and explicitly opts that programme
   into programme-integration reliance.
6. The programme control records the relied-upon ADR identity and exact reviewed
   source SHA.
7. The exact approved and reviewed ADR version is merged into the designated
   programme integration branch.
8. The programme ledger records the merge evidence that makes that exact version
   reachable from the programme integration branch.
9. Each relying implementation task has its own approved specification,
   dependencies, exact authorized base, bounded branch/PR, write budget, tests,
   rollout/rollback effects and explicit implementation authorization.
10. Each relying repository-local task branches from and targets that same
    programme integration line, or consumes an explicitly authorized pinned
    cross-repository preview under section 4.6.

If any condition is missing, implementation relying on the unmerged decision
must HOLD.

Decision approval alone is not programme-integration reliance. Merging an ADR
into a programme branch alone is not programme-integration reliance. A task
still needs its own implementation authorization.

### 4.3 Exact-version pin and decision changes

Programme-integration reliance is bound to the exact approved and reviewed
decision version recorded by the programme control.

A material decision change:

- invalidates programme-integration reliance on the changed version until the
  decision completes the required ADR decision and independent-review controls;
- prevents new dependent implementation work from proceeding against the
  changed decision;
- requires an impact assessment of open and already-merged programme slices
  that relied on the prior version;
- requires fresh review or remediation of affected slices where the changed
  decision alters their correctness or acceptance basis.

No task may silently follow a moving ADR branch.

A factual clarification that does not alter the decision remains governed by
the repository ADR amendment rules. If it changes the source commit relied on by
programme work, the programme ledger must explicitly reconcile the exact-version
pin before new dependent work proceeds.

### 4.4 Programme branch containment

Programme-integration reliance is contained to the designated integration line.

A relying task must:

- use the repository's governed programme-slice branch grammar;
- record the integration-branch base and exact SHA from which it branches;
- target the designated integration branch, not the normal development branch;
- preserve compatibility with already merged programme slices;
- receive independent review against its exact task head before slice merge.

The programme integration branch is not a substitute for task specifications,
review or merge authorization.

A programme integration branch carrying an approved but
non-repository-authoritative ADR must not be described as making that ADR
repository policy.

### 4.5 No implicit or retroactive adoption

Existing programmes do not adopt ADR-0013 merely because this decision becomes
repository-authoritative.

Adoption requires a durable programme-control record created after the programme
has been assessed for the mechanism. That record must identify:

- the programme;
- the repository-local integration branch;
- the relied-upon ADR;
- the exact approved/reviewed ADR source SHA;
- the permitted scope of programme-local reliance;
- affected repositories and dependency ordering;
- the first eligible dependent task or the HOLD that precedes it.

Historical work is not retroactively reclassified as having used
programme-integration reliance.

### 4.6 Cross-repository consumers

Programme-integration reliance never permits a downstream repository to guess an
unmerged upstream contract.

When a downstream repository must work before the upstream programme reaches its
normal development branch, its own repository-local task must record one of:

- an explicitly pinned preview environment;
- a versioned generated contract from the upstream programme integration line;
- an exact upstream branch/commit reference that the downstream repository's
  approved controls permit as a preview dependency.

The downstream task must record compatibility, merge order and deployment order.
It receives its own branch, PR, independent review and authorization.

Programme-integration reliance in one repository does not automatically create
the same state in another repository.

### 4.7 Production and external-effect boundary

Programme-integration reliance authorizes no external effect.

It does not authorize:

- production or environment migration execution;
- provider reads or writes;
- IAM changes;
- credential provisioning;
- deployment;
- release or publication;
- production activation;
- external writes;
- removal of compatibility paths.

Those actions retain their normal separate authorization gates.

A programme branch may contain migration source or runtime source only through
separately authorized implementation tasks. Merely merging that source into the
programme integration branch does not authorize executing or deploying it.

### 4.8 Final programme integration

The final `feature/<programme> -> <development-branch>` transition remains the
normal programme integration gate.

Before final integration, the programme must satisfy the existing integrated
review requirements, including as applicable:

- accepted slices present in the intended order;
- current development-branch reconciliation;
- integrated CI;
- migration ordering;
- generated-contract consistency;
- cross-repository compatibility;
- programme-wide acceptance;
- rollout and rollback;
- fresh independent review of the complete integration diff;
- explicit CTO merge authorization where required.

When the final merge makes an approved relied-upon ADR reachable from
`develop`, that ADR may become repository-authoritative under its own authority
condition. Programme-integration reliance does not bypass that final transition.

## 5. Initial intended consumer: THOTH-ASYNC-01

`THOTH-ASYNC-01` is the first intended consumer of this mechanism, but this ADR
does not itself opt that programme in.

The programme has selected the intended integration topology:

```text
develop
  -> feature/worker
  -> feature/worker--<slice>
  -> feature/worker
  -> develop
```

At the time of this proposal:

- `feature/worker` has not been created;
- PR #960 remains separately controlled and unmerged;
- ADR-0012 is not repository-authoritative;
- worker/shared-async implementation remains HOLD.

After ADR-0013 becomes repository-authoritative, `THOTH-ASYNC-01` requires a
separate durable opt-in record and the normal branch/PR gates before
programme-integration reliance on ADR-0012 can become effective.

## 6. Invariants

1. Repository authority remains distinct from programme-integration reliance.
2. `develop` remains the repository-authority boundary for
   `thoth-pub/thoth`.
3. Programme-integration reliance is opt-in, exact-version-bound and
   non-retroactive.
4. Only an `APPROVED`, independently reviewed decision may be relied on.
5. The exact relied-upon version must be merged into the designated programme
   integration branch before dependent implementation may rely on it.
6. Every dependent implementation task still requires its own approved
   specification and explicit authorization.
7. A material decision change invalidates reliance until the changed decision
   completes the required decision/review controls and dependent impact is
   assessed.
8. Unrelated programmes and normal development-branch work cannot rely on a
   programme-local decision merely because another programme can.
9. Cross-repository consumers use explicitly pinned previews and never guess an
   unmerged contract.
10. Programme-integration reliance authorizes no migration execution, provider
    action, deployment, release or production activation.
11. Final programme integration into the repository development branch remains
    subject to integrated review and merge authorization.
12. Existing programmes do not adopt this mechanism implicitly.

## 7. Doctrine changes

When this decision becomes repository-authoritative:

- `decision-register.md` retains merge to `develop` as the default reliance
  gate and records the controlled ADR-0013 exception;
- `branching-and-release-workflow.md` records how opted-in programme branches
  may carry exact approved decision dependencies without becoming repository
  authority;
- `operating-model.md` requires task specifications to pin the exact
  programme-local decision dependency and fail closed when the eligibility gate
  is not satisfied.

No ADR other than ADR-0013 is amended by this task.

## 8. Cross-repository impact

This decision changes engineering-control doctrine, not a runtime or data
contract.

Known effects:

- `thoth-pub/thoth`: owns this shared control record and is the first intended
  repository in which `THOTH-ASYNC-01` may later opt in.
- Other Thoth repositories adopting the shared engineering controls: remain
  compatible without change because adoption is explicit and repository-local.
  No repository gains programme-integration reliance automatically.
- `thoth-pub/thoth-dissemination`, `thoth-pub/thoth-app` and
  `thoth-pub/infrastructure`: no source or runtime change is required by this
  ADR. Any future THOTH-ASYNC cross-repository dependency receives its own
  bounded task and pinned preview/merge-order evidence.

No shared API, GraphQL schema, database model, generated client, configuration
contract, event payload or deployment contract changes in this task.

## 9. Rollout

Rollout is control-only:

1. independently review the exact ADR-0013 candidate;
2. obtain explicit CTO decision approval;
3. reconcile durable decision status through the normal controlled process;
4. merge the exact approved content into `develop`;
5. only then may a programme create an explicit opt-in record;
6. only after its exact relied-upon ADR version is merged into the designated
   programme branch may separately authorized dependent implementation begin.

No programme branch is created by this rollout.

## 10. Rollback

Before ADR-0013 becomes repository-authoritative, rollback is simply rejection
or replacement of the proposal.

After it becomes repository-authoritative, a material rollback requires a new
ADR that supersedes this one under the normal ADR amendment process.

Stopping a particular programme's use of the mechanism does not require
superseding ADR-0013. The programme control may place its reliance state on HOLD
and return to the default rule, provided dependent work is reconciled and no
branch history is rewritten to conceal prior reliance.

Rollback never authorizes deleting or rewriting historical task, review or merge
evidence.

## 11. Validation

Independent review must verify at minimum:

- repository authority remains tied to `develop`;
- the exception cannot apply to a `PROPOSED`, unapproved or unreviewed ADR;
- the exception cannot activate merely because an ADR is present on a branch;
- programme opt-in is explicit and non-retroactive;
- exact-version pinning prevents silent decision drift;
- every relying task remains separately specified, authorized and reviewed;
- unrelated programmes cannot inherit the exception;
- cross-repository consumers cannot guess unmerged contracts;
- no production/provider action follows from programme-local reliance;
- final programme integration remains independently reviewed and separately
  merge-authorized;
- doctrine wording is durable before and after ADR-0013 approval/merge and does
  not copy transient lifecycle metadata into committed source.

## 12. Approval

Decision owner: CTO

Current decision state: **PROPOSED**.

Approval of this decision requires explicit CTO decision approval after
independent CRITICAL exact-head review. Approval is not implementation
authorization for any adopting programme, is not authorization to create a
programme integration branch, and is not merge, deployment or production
activation authorization.

Live review, authorization, CI and merge evidence belongs in GitHub and is not
duplicated into this ADR as transient lifecycle metadata.
