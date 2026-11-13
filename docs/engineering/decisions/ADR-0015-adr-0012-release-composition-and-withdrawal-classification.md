# ADR-0015 - ADR-0012 release composition and withdrawal classification

Status: PROPOSED
Date: 2026-11-13
Decision owner: CTO
Programmes affected: Shared Backend Architecture (owning programme, THOTH-ASYNC-01); Publisher Services and Distribution Configuration; Thoth Hosting; future Thoth programmes that need asynchronous work
Repositories affected: `thoth-pub/thoth` (the shared engine, in two releases); `thoth-pub/thoth-dissemination` and `thoth-pub/thoth-app` (the Thoth release each consumer migrates onto); `thoth-pub/infrastructure` (rollout order only)
Parent programme: [THOTH-ASYNC-01 #957](https://github.com/thoth-pub/thoth/issues/957)
Authoring task: [THOTH-ASYNC-01-ENGINE-01 #1026](https://github.com/thoth-pub/thoth/issues/1026)
Supersedes in part:
- `ADR-0012` release composition: section 4.1 (the Release N list) and section 8 item 1 (one release for the whole shared engine, and its consumer contracts); the precondition of section 4.3 and the Phase A to Phase B rule of section 9; section 9's "Rollout should prove in order" list (items 1 to 11) and its migration test matrix, which are partitioned; the route-epoch rollback rules of section 10, which bind from Release R; and section 11's single approval gate, which is partitioned.
- `ADR-0012` text narrowed by Property P: section 3.2's withdrawal-first order of anomaly creation against withdrawal; invariant 43's withdrawal-first sentence; section 11's withdrawal-first race item and withdrawal-first detection item; the detector clause of `A13-T15`; the withdrawal-first race of `A13-T17`; the serializable-failure-and-retry schedule of `A13-T03`; and section 3.2's first-observer provenance, which admits a third observer.
Superseded by: None

Decision: The shared engine of `ADR-0012` is delivered in two releases. Release N carries the job engine. Release R carries events, logical routes and coalescing, and must be deployed before the first route is registered or the first coalescing kind activates. Release R must also meet a new strand-safety requirement, Property P: when a revision withdrawal or a route retirement decides strand safety, every committed event that could be an in-epoch contract mismatch for the route has already been classified. This ADR also records that the staff `ACCEPT_REGRESSION` resolution of the Release N engine is an "explicitly authorized historical replay/rollback operation" in the sense of `ADR-0012` section 3.3. Every other clause of `ADR-0012` is unchanged.

Authority condition: this ADR becomes repository-authoritative only when this exact approved version has received independent exact-head review and is reachable from `develop`. It decides architecture only. It authorizes no implementation, schema change, migration, release, deployment or activation. Release N and Release R are each specified, reviewed and authorized separately.

---

Line numbers in this ADR are lines of `ADR-0012` blob `896195c8e95f0590fae2d26237b6265c5bf9d1c9`, the version that was repository-authoritative when this ADR was written. This ADR's change to the `ADR-0012` header adds one line, so in later versions every line after the header is one line lower.

## 1. Context

`ADR-0012` has been repository-authoritative since PR [#1006](https://github.com/thoth-pub/thoth/pull/1006). The programme THOTH-ASYNC-01 ([#957](https://github.com/thoth-pub/thoth/issues/957)) delivers it in steps: the shared engine in `thoth` (step 1), the worker process (step 2), its infrastructure (step 3), consumer migration (step 4), BE-04 retirement (step 5) and Hosting (step 6).

### 1.1 Release composition

`ADR-0012` section 8 item 1 (lines 3478-3512) puts the whole shared engine into one "Release N": the job engine, and also events, logical routes, activation epochs, route-contract revisions, mismatch anomalies, historical enrollment, revision withdrawal, coalescing admission and successor generations. Section 4.1 lists "event/route/job/attempt primitives" in Release N (line 2766).

None of programme steps 2 to 5 needs event routing. The work of `thoth-dissemination` and the job reads of `thoth-app` are directly created jobs. `ADR-0012` allows these: section 3.2 says jobs "may also be created without an event for scheduled work, reconciliation, backfills or explicit operator actions" (lines 225-227), and section 4.4 lets a Publisher Services activation commit its "generic event/job intent" atomically (lines 2843-2845). Step 6 (Hosting) is the exception to watch, because the ADR's own example event is `HOSTING_ENABLED`.

Routing and coalescing are the largest and most concurrency-heavy part of item 1. They account for roughly two thirds of the validation list in section 11.

### 1.2 The carried review finding

A Codex review of the `ADR-0012` pull request (PR [#960](https://github.com/thoth-pub/thoth/pull/960), review thread `PRRT_kwDODkn0bc60f8FH`, "Include undetected mismatches in withdrawal safety") found this window:

1. A mismatched event commits.
2. Before an asynchronous detector records its `IN_EPOCH_CONTRACT_MISMATCH` anomaly, a revision withdrawal linearizes. It sees no anomaly, passes its strand-safety check and may withdraw the only compatible revision.
3. The detector then records an anomaly that has no selectable resolution path.

`ADR-0012` treats this as the lawful withdrawal-first order: the anomaly is visible and attention-raising, and it is resolved only by an approved reconciliation or divergence procedure (lines 1145-1157). The thread proposed either serializing classification with withdrawal or making undetected mismatches strand-relevant.

On 2026-11-06 the CTO ruled the finding NOT BLOCKING for `ADR-0012` (PR #960 `discussion_r4492908637`; [#958](https://github.com/thoth-pub/thoth/issues/958) comment 6686086846, item 5). The ruling says that making withdrawal account for latent mismatches "would be a new strand-safety requirement", and that such a requirement needs CTO approval in the implementation specification. The finding was carried to that specification.

### 1.3 Governance route

`docs/engineering/decisions/README.md` ("Amendments") says that once any approved version of an ADR has been repository-authoritative, a later material architectural change "requires a new ADR that supersedes the authoritative ADR". An uncertain classification is treated as material. Splitting the release composition changes decision text in `ADR-0012` sections 4.1 and 8, so it is material. Property P is a new strand-safety requirement, so it is material too. Precedents for a partial supersession: `ADR-0012` supersedes `ADR-0008` and `ADR-0010` in part, and `ADR-0014` supersedes `ADR-0001` sections 4.4 and 4.5 in part.

On 2026-11-13 the CTO decided both questions, on stage issue [#1026](https://github.com/thoth-pub/thoth/issues/1026). This ADR writes those decisions down. It proposes no in-place change to the decision text of `ADR-0012`.

## 2. Decision drivers

- Steps 2 to 5 can be delivered on directly created jobs. They should not wait for routing.
- Routing and coalescing carry most of the concurrency risk and most of the validation work. They deserve their own specification, review and release.
- `develop` must never carry code whose release composition contradicts or outruns an authoritative ADR.
- A withdrawal or retirement must not decide strand safety while a committed mismatch is still undetected.
- Classification must stay off the business write path: a classifier fault must not fail a metadata edit.
- The absent-anomaly races that `ADR-0012` section 11 requires as tests must stay reachable in the shipped design, without a harness that removes the race.
- The routing protocol must never abort a business transaction through a deadlock.
- The Release R specification must rest on repository-authoritative text, not on a review thread.

## 3. Options considered

### 3.1 Release composition

**Option A - everything in one release (not chosen).** Step 1 also builds events, routing and coalescing inside the same integration, as `ADR-0012` section 8 item 1 is written. No release-composition change is needed. Property P would still be a new strand-safety requirement, so an ADR would still have to record it, and the `ACCEPT_REGRESSION` reading with it. Step 1 would then include the largest and most concurrency-heavy part of item 1, which steps 2 to 5 do not need. The job engine itself is the same under either option.

**Option B - jobs first, routing later (chosen).** Step 1 delivers the job engine only (Release N). Events, routes and coalescing follow in Release R under their own specification. Section 4 gives the exact partition.

### 3.2 Withdrawal classification

The review thread proposed two options.

**Thread option 1 - serialize classification with withdrawal.** Property P keeps this, at the control operation only. A withdrawal or retirement classifies every committed, unclassified event itself, under exclusive locks that order it against event emission. Classification is not moved into business writes, and detection otherwise stays asynchronous.

**Thread option 2 - make undetected mismatches strand-relevant.** Property P keeps the aim: no committed mismatch escapes the strand-safety decision. It does not keep the form. The strand-safety predicate of `ADR-0012` is unchanged, and an undetected mismatch does not become a new strand-relevant state. Instead, no undetected committed mismatch exists when the decision is made: each one has been classified first, and it is then an ordinary `OPEN` anomaly that the existing predicate already counts. Property P changes when classification must have happened, not the predicate.

Three concrete designs were weighed against the chosen one.

**C1 - classify in the event's own transaction (not chosen).** A trigger writes the anomaly. This also closes the window. But it puts classification on every business write, so a classifier fault fails a metadata edit. The production design itself would also remove the absent-anomaly race. `ADR-0012` section 11 accepts deterministic barriers and fault injection as test drivers, but rejects "a mock or harness that ... removes the absent-anomaly race" (lines 4312-4313). Under C1 the required absent-anomaly cases (`A13-T02`, `A13-T03` and the others named in rule 8) could run only against a configuration that never ships, so the A13 list itself would have to change.

**C2 - keep asynchronous detection and `ADR-0012` as written (not chosen).** The window stays. The outcome is visible and fail-safe, and resolution needs reconciliation or divergence.

**C3 - asynchronous detection plus a commit-order-safe detection watermark (not chosen).** This uses `pg_snapshot`/`xid8` machinery. It is also a new strand-safety requirement, and it is the most complex option.

**Chosen - classify before deciding (Property P).** Section 5 gives the exact requirement and its rules.

## 4. Decision: release composition

Step 1 of the programme delivers the job engine only. The clauses below supersede `ADR-0012` in the places named, and in no other.

### 4.1 Two releases (supersedes section 4.1's Release N list and section 8 item 1's single-release composition)

- **Release N** carries the job and attempt schema, the engine, the kind-scoped executor API, the operator controls and the generic operational read contract.
- **Release R** carries events, routes, epochs, revisions, mismatch anomalies, historical enrollment, withdrawal, route dispositions and coalescing. It also carries section 8 item 1's mixed-version and out-of-order-commit tests for routing.
- Release R must be deployed before the first route is registered or the first coalescing kind activates.
- A consumer that runs on Release N alone uses directly created jobs. Each consumer specification must show that its creation path falls within the categories that `ADR-0012` section 3.2 and section 4.4 allow (section 1.1). Step 6 (Hosting) either justifies direct job creation in its own specification or waits for Release R.

### 4.2 Consumer contracts and the migration precondition (supersedes section 8 item 1's consumer contracts, section 4.3's precondition and section 9's Phase A to Phase B rule)

- The Publisher Services generic creation, cancellation and activation-guard contract, and the `thoth-app` read fields, land in Release N or in a later Thoth release.
- Section 4.3's "After Release N is the deployed/rollback-safe Thoth version" (line 2809) becomes: after the Thoth release that carries the contract the consumer migrates onto is the deployed and rollback-safe version.
- Section 9's rule that Release N "must be fully deployed and become a valid rollback target before any downstream consumer migration starts" (lines 3613-3614) is extended the same way. Phase A is repeated, unchanged, for Release R and for any such later release.
- Section 4.7 (development and fresh-database path) is unchanged in effect. Release N's migration is the "additive generic schema migration" it names (line 2934). Release R's migrations follow in the same ordered history.

### 4.3 Rollout order (replaces section 9's "Rollout should prove in order", items 1 to 11)

The list at lines 3707-3734 is replaced by this partition, in this order. Item numbers are those of that list.

1. Release N proves item 1's job part (schema and job concurrency without external writes), item 4, item 5's job part (effect-scoped idempotency and revision ordering) and item 6.
2. After Release N is deployed, and without waiting for Release R, items 7, 8 and 9 may be proven for directly created jobs (non-routed and non-coalescing), each under its own activation grant. Item 10 may be proven in the same way, and it also needs the untrusted-content pool that programme steps 2 and 3 deliver.
3. Release R proves item 1's event-route part, items 2 and 3, and item 5's coalescing and routed-job part. A routed or coalescing kind is activated only after these.
4. Item 11 (broader handler activation) comes last for each handler family, after every item that applies to that family.

The order of `ADR-0012` therefore changes in one respect: for non-routed, non-coalescing handler families, the job and handler items (4 to 11) may be proven before the routing parts of items 1 to 3.

### 4.4 Migration test matrix (partitions section 9's matrix, lines 3676-3695)

- These matrix items gate Release R: event-route activation and every reactivation or revision-transition epoch; producer deployments and rollbacks that keep the emission set within the accepted-version set; closing the current epoch before a rollback below an emission floor or to an uncovered producer version; and current-state reconciliation or backfill of an authorized emission gap.
- Every other matrix item gates Release N or the later programme step that carries it. The production-like schema with v1.7.0 applied, the pre-v1.7 upgrade, the full empty-database chain and the mixed-version rolling deploy during Phase A gate Release N. The retirement rollback window, the proof of zero runtime legacy-table access and the two contraction items gate BE-04 retirement and contraction (programme step 5 and the later storage contraction).

### 4.5 Rollback (section 10)

- The route-epoch rules bind from Release R: closing an epoch before rolling below an emission floor or to an uncovered producer version, the route-activation condition on a previous-image rollback, and recovery by a new epoch.
- Release N rollback is governed by the remaining rules of section 10, unchanged. In short: a previous image is a rollback target only while the schema and API it needs are present; after generic async data exists, a rollback stops job creation and claiming and preserves the durable records; that data is never dropped or rewritten.

### 4.6 Approval gate (partitions section 11's "Before shared implementation can be approved")

The single gate at line 3789 is partitioned. The table lists every job-engine item of section 11. An item, or the part of it, in the "Release N" column gates Release N. Every other item, and every part in the last column, gates Release R or the later step named there. No item changes in content.

| `ADR-0012` section 11 item | Release N | Later, and where |
| --- | --- | --- |
| real PostgreSQL concurrent claim tests with multiple workers | yes | |
| effect-scoped idempotency across newer source revisions and schedule slots | yes | |
| `CURRENT_STATE` fingerprint covers every canonical input | engine part: a no-op records the observed fingerprint when it differs and changes nothing when it is equal | step 4: the fingerprint content of each kind |
| `REVISION_BOUND` older retried revision cannot overwrite a newer one | yes | authorized replay: the stage that introduces historical replay |
| cross-kind writers share owning domain, serialization and evidence | yes | |
| stale-claim and normal-versus-reconciler races; evidence only under the current claim and the target boundary | yes | |
| provider without a revision identifier | yes: the local order blocks regression | |
| target-identity change | engine primitives only: immutable targets and operator cancel | the first kind whose identity can change (step 4) |
| multi-target all-or-nothing acquisition with no wait while holding a strict subset | yes | |
| lease renewal, stale-token refusal, `WAITING`, resume, budgets, deadlines, holds through `WAITING` and `RECONCILIATION_REQUIRED`, the blocked signal, the effect window, crash before and after `EFFECT_STARTED`, `EFFECT_CONFIRMED` recovery, cancel before and after effect start, `INDETERMINATE`, reconciliation races, priority, `available_at` and fairness, backoff and poison jobs, unsupported version, worker filtering | yes | |
| `EFFECT_CONFIRMED` not conflated with `ADR-0010` observed state | yes | |
| credentials rejected; personal and snapshot data absent | engine guard and test kinds | real kinds: step 4 |
| diagnostics and provider-evidence retention and erasure | redaction | retention policy: its own specification |
| current-state execution reads canonical state | engine part: the read of applied target state | kind handlers: step 4 |
| graceful shutdown and abrupt restart for every pool | engine part: a graceful release of a claim and recovery of an expired lease | step 2 |
| job-kind and version retirement keeps work interpretable | yes | |
| domain executors cannot mutate another domain's jobs | yes | |
| protected and audited operator cancel, retry and reconcile | yes | |
| queue depth, oldest ready, expired lease, `RECONCILIATION_REQUIRED`, failure, blocked and worker-liveness signals | yes | blocked-successor signal: Release R |
| logs and durable diagnostics contain no configured credentials | engine guard and a no-logging test | worker logs: step 2 |
| static AWS keys, untrusted pools, isolation, staging, network denial, promotion, TOCTOU, follow-on jobs, database access by untrusted pools | none | steps 2 and 3 |
| Phase-A migration retaining BE-04 storage and API | yes | |
| `thoth-dissemination` and `thoth-app` compatibility, `AUTOMATIC_PUSH` fail-closed, released error contract | the BE-04 regression suite stays unchanged | step 4 |
| generated SDL and client contract checks | yes | |
| Phase-C and contraction tests | none | step 5 |
| rolling deployment and rollback validity | previous-image compatibility and recovery of jobs of a kind the serving binary does not know | step 2: worker tasks |
| section 8 item 1's mixed-version and out-of-order-commit tests | mixed-version: as the row above; out-of-order commit does not apply to jobs, which have no event ordering | Release R: routing |
| full empty-database migration chain | yes | |
| per-kind retention, coalescing and admission before high-volume activation | none | the first high-volume kind |
| more than one compatible worker task | engine part: claims over several connections | steps 2 and 3 |
| every routing, epoch, revision, anomaly, enrollment, withdrawal, disposition and coalescing item, and `A13-T01` to `A13-T31` | none | Release R |

### 4.7 The legal transition table of Release N

- The legal transition table of the Release N engine, as its approved specification on #1026 defines it, is the contract for non-routed jobs.
- Release R may add preconditions to existing transitions for routed or coalescing kinds (`ADR-0012` section 3.4, obligation accounting; section 3.5, the attachment boundary). A new legal status pair needs its own reviewed specification.
- Release N's kind policy type has no field that declares event consumption or coalescing. No routed or coalescing kind can therefore exist before Release R.

### 4.8 The `ACCEPT_REGRESSION` reading

`ADR-0012` section 3.3 says of applied-revision evidence: "It is monotonic and must not move backwards except through an explicitly authorized historical replay/rollback operation" (lines 1433-1435).

The Release N engine has a protected staff command that resolves a job in `RECONCILIATION_REQUIRED`. One of its outcomes is `ACCEPT_REGRESSION`. It serves one case. A `REVISION_BOUND` job has written its revision at the provider, although the stored evidence for the target is newer (a regression) or records the same order with other content (a conflict). A forward `SUCCEEDED` cannot record that write, and `FAILED` or `CANCELLED` would deny an effect that happened. The command, with kind-validated evidence that the provider holds the job's own revision, sets the stored evidence to that revision and ends the job `SUCCEEDED`.

This ADR records that this resolution is an "explicitly authorized historical replay/rollback operation" in the sense of section 3.3, in its narrowest form. It records a regression or conflict that already happened at the provider. It performs none. An executor reconciler can never accept a regression.

This reading does not depend on the release composition: this ADR records it under either option of section 3.1. The Release N engine rests on `ADR-0012` lines 1433-1435 themselves.

## 5. Decision: withdrawal classification (Property P)

Property P and the rules of this section are a strand-safety requirement on Release R. They bind the Release R specification. Release N builds none of this.

### 5.1 The finding

A mismatched event can commit before an asynchronous detector records its `IN_EPOCH_CONTRACT_MISMATCH` anomaly. A revision withdrawal that linearizes in that window sees no anomaly, passes its strand-safety check, and may withdraw the only compatible revision. The detector then records an anomaly with no selectable resolution path. `ADR-0012` treats this as the lawful withdrawal-first order (section 1.2).

### 5.2 Property P

When a revision withdrawal or a route retirement decides strand safety, every event of the route's consumed kinds (section 5.3) that committed before the decision has been classified for that route.

A committed but unclassified event can therefore never be part of a withdrawal or retirement decision. Events committed after the decision are the producer violations that `ADR-0012` already accepts (withdrawal-first).

Property P changes when classification must have happened. It does not change the strand-safety predicate.

### 5.3 Definitions

**Consumed kinds of a route, `K(route)`.** The union of the event kinds consumed by:

- every revision that is or was the revision of any epoch of the route;
- the revision being withdrawn; and
- for an epoch opening, the revision the new epoch will reference.

Withdrawal, retirement, and epoch opening and closing take the keys of this set (rule 1), so an event of a kind consumed only by an older epoch's revision is covered.

The set is read before the keys are held. A control operation therefore re-reads `K(route)` in a statement issued after its last lock (its keys and the route boundary) is granted. If the set differs from the keys it holds, it rolls back and retries within its bound. Because an epoch opening also holds the route boundary, the re-read sees every opening that committed before the control's boundary was granted. That includes the route's first epoch, when the control held no key.

**Classified.** Only an event that can be an in-epoch mismatch for the route needs classifying. That is an event whose captured routing generation lies inside one of the route's epochs and whose kind is the consumed kind of that epoch's revision. This is the precondition of `ADR-0012` for an in-epoch contract mismatch (lines 598-600).

- An event of a kind in `K(route)` that its own epoch's revision does not consume can never be a mismatch for the route. It needs no marker.
- An event that needs classifying is classified when it has a durable marker for the (event, route) pair. The classifier writes the marker in the same transaction as the canonical `OPEN` anomaly it creates for a mismatch. It writes the marker alone when the pair is compatible, or when the pair's canonical anomaly already exists (for example one created born-terminal by an enrollment that won the race, `A13-T02`).
- "Unclassified" means such an event without a marker. Events outside every epoch of the route need none.
- The backlog of unclassified events is found through an index bounded by each epoch's generation range and its revision's consumed kind.
- The Release R specification may replace the marker only with an equivalent that it proves exact under the exclusive locks of rule 1.

**Control operation.** A revision withdrawal, a route retirement, an epoch opening or an epoch closing.

**Route boundary.** The route's lifecycle serialization boundary of `ADR-0012` section 3.2 (the per-`route_key` boundary, lines 1217-1222).

**Pair gate.** The single claim that every writer takes for an (event, route) pair before it writes a marker, an anomaly or a disposition for that pair (rule 2).

**Routing protocol and its members.** The members of the routing protocol are:

- asynchronous detector runs;
- control operations;
- enrollments;
- divergences;
- asynchronous first-disposition writers (routed materialization in its own transaction, called the asynchronous materializer below); and
- the staff terminal non-materialized disposition writer (`ADR-0012` lines 737-739).

Every other transaction is **outside the routing protocol**. That includes every business transaction that emits an event or writes a routed job inline, because it may already hold business row locks.

**Inline writer.** A business transaction that, in its event's own transaction, creates a routed job, attaches an obligation to a job whose coalescing admission is open, or creates the post-closure successor (`ADR-0012` lines 729-731 and section 3.5). An inline writer is a first-disposition writer, and it is outside the routing protocol.

**First lock of a member.** The first lock of any kind that the member's transaction takes: a row lock, an advisory key, or an uncommitted unique-key or foreign-key claim.

### 5.4 Rules

The mechanism is: classify before deciding.

#### Rule 1 - Routing locks, per consumed event kind

The routing key of an event kind is `(NS_ROUTING, hashtext(event_kind))` in the two-key advisory lock space. `NS_ROUTING = 1095981393` is reserved for this use.

**1(a) Event-emitting transactions.**

- An event-emitting transaction takes the key of an event's kind in shared mode on every event insert, and holds it to commit.
- The call is re-entrant and cheap. A per-transaction "already locked" cache is not allowed, because a transaction-level advisory lock taken inside a savepoint is released when that savepoint rolls back.
- The event trigger function is `VOLATILE`. It reads the routing generation in a query issued after the lock call returns, so the stored generation reflects every routing change committed before the grant.

**1(b) Control operations.**

- A control operation takes the keys of `K(route)` in exclusive mode, ascending by lock key, as the first locks of its transaction.
- It sets `lock_timeout` to at most 2 s for its whole transaction (`SET LOCAL`, before its first lock).
- It waits only for its first lock. That is its first key, or the route boundary when `K(route)` is empty.
- It takes every further key, and then the route boundary, with the non-waiting form. If one is not granted, it rolls back its transaction and retries with jittered backoff, within a retry bound stated in the Release R specification. It refuses with a retryable code when the bound is exhausted.
- It therefore never waits for a routing key or a route boundary while it holds either.
- After its first lock it never locks a business row.
- The only wait left to it is, while classifying, for a pair gate that another member holds: an uncommitted marker, anomaly, canonical claim or first disposition of the same pair. That wait is bounded by its `lock_timeout`. The member that holds the gate never waits outside the protocol after its first lock (rule 1(e)), so the wait ends.

**1(c) Detectors.**

- A detector never waits for a business transaction: it takes no routing key and locks no business row.
- It waits only on a pair gate that another member holds: another classifier's marker or anomaly, an enrollment's or divergence's uncommitted canonical claim (`A13-T03`), or a first-disposition writer's disposition.
- It waits only before its own first write for that pair, and it commits after each pair (rule 2).
- A transaction that waits on a detector therefore waits on one that will commit without waiting again. No wait cycle can pass through a detector.

**1(d) One canonical order.** Every writer of a canonical identity or a pair gate (detector, control classify step, enrollment, divergence and first-disposition writer) takes its claims in the one ascending canonical order.

**1(e) A member waits only for its first lock and inside the protocol.**

- Every member takes its first lock while it holds nothing that a transaction outside the protocol can wait on, or else takes it in the non-waiting form.
- After its first lock a member never waits, by any mechanism (row lock, advisory key, unique index, foreign key or trigger), on a transaction outside the protocol.
- After its first lock it takes each of these in the non-waiting form, and rolls back its whole transaction and retries when one is not granted:
  - every further routing key;
  - every route boundary;
  - the row lock of any job it attaches to; and
  - a `KEY SHARE` lock on every row that its writes reference and that a transaction outside the protocol can lock `FOR UPDATE`, delete or key-update (for example a predecessor job that a Release N worker write holds).
- Its only other waits are for a pair gate or canonical claim that another member holds, never one that a transaction outside the protocol holds. A detector's such wait comes only before its first write for the pair.
- The Release R specification shows this for every lock and every write a member makes after its first lock, using rules 1(f) and 1(g).

**1(f) The route boundary: order, modes and foreign keys.**

- Enrollment, divergence, the asynchronous materializer and the staff terminal disposition writer, under the universal admission guard, take every routing key they need before the boundary and none after it. They wait on no transaction outside the routing protocol while holding it.
- An inline writer may hold business row locks when it reaches the boundary, so the bullet above cannot bind it. Rule 1(g) orders it against the members. The Release R specification states its full lock order and proves the guarantee of rule 7 for it.
- The Release R specification lists every holder of the boundary and shows their order.
- **Admission mode.** First-disposition writers take the boundary in a mode they share with each other: an advisory lock in shared mode, or `FOR SHARE`. Two admissions therefore never wait on each other at the boundary. For this rule the first-disposition writers are the ones `ADR-0012` lists (lines 726-739): the asynchronous materializer, inline writers, both kinds of authorized historical enrollment and the staff terminal disposition writer. Enrollment is named separately in other rules of this ADR only because it has rules of its own; at the boundary it is an admission and takes the admission mode.
- **Control mode.** Control operations take the boundary in a mode that excludes the admission mode and is no stronger than `FOR NO KEY UPDATE` (or the exclusive advisory mode).
- **Divergence.** A divergence creates no disposition (`ADR-0012` lines 884-886), so it is not a first-disposition writer, and this ADR does not require it to take the boundary. Where the Release R specification has a divergence take the boundary, the divergence takes it in the admission mode, never in the control mode, and the specification's list of boundary holders says so.
- **Boundary before gate.** Every first-disposition writer takes the route boundary before the gate of the pair it materializes. This includes the staff terminal disposition writer, which has no job write. A divergence that takes the boundary takes it before its gate too.
- **Foreign keys.** The boundary lock is never `FOR UPDATE`, and its holder neither deletes the boundary row nor changes its key columns. A foreign-key check takes `KEY SHARE`, which waits on `FOR UPDATE` and on both of those writes. The Release R specification shows that no foreign-key check that a member makes after its first lock can wait.
- The no-deadlock argument of rule 7 does not rely on any relation between `lock_timeout` and `deadlock_timeout`.

**1(g) Routed-job writes.**

- Every routed-job writer, member or not, makes its job write after its routing keys and before the route boundary and the gate of the pair being materialized.
- The job write is the job insert. For an attachment it is the lock of the job's row and any write to the job row (its job-wide input).
- That pair's disposition follows its gate (rule 2). For an attachment, the attached pair's membership and input facts also follow the gate, with the disposition, in the same transaction. Only the job-row lock and job-wide input precede the gate.
- A job write after another pair's gate or claim is allowed. It falls under rule 1(e).

**1(h) Routed-job identity.**

- This ADR does not fix how a member avoids waiting, at a routed job's unique identity (idempotency key or coalescing identity), on a transaction outside the protocol. Every Release N idempotency key begins with its job kind, so only writers of the same routed kind can share an identity.
- The Release R specification states that mechanism (for example a per-identity lock that members take without waiting), its lock order and its collision handling. It proves two things for it: no member waits on a transaction outside the protocol; and the mechanism adds no deadlock between two transactions outside the protocol that their own writes would not already create.
- **Scope of "their own writes would not already create".** The phrase is a baseline for that second proof and for its tests: it separates what the mechanism adds from what was there without it. It is not a licence. A deadlock that two transactions outside the protocol can form through their own routed-job writes, at a shared job identity or at an attached job's row, stays forbidden by the `ADR-0012` clauses named in the next bullet. For an inline writer it is covered by the lock-order proof of rule 1(f), which includes the inline writer's own waits at a shared identity and at an attached job's row.
- **No refusal on a collision.** A collision here means two different identities that share one key under the mechanism. The mechanism must not refuse, fail or abort a transaction outside the protocol because of a collision. Such a transaction may wait, within the stall bound of rule 7. A true conflict on one identity is not a collision in this sense: the rules of `ADR-0012` decide it (for example the atomic refusal of a stale attachment, `A14-T02`).
- No residual is allowed. The no-deadlock clauses of `ADR-0012` for successor selection and creation, coalescing attachment and job-key-collision races stay binding: "deadlock-safe lock order" (line 1786), invariant 46 (lines 3366-3370), the uniqueness and rollback race tests (lines 4156-4161) and `A14-T16`. A mechanism that cannot meet them needs an ADR that supersedes those clauses, not a task specification.

**1(i) Retry bounds and exhaustion outcomes.** Every member has a retry bound stated in the Release R specification. When it is exhausted:

- an enrollment, a divergence and the staff terminal disposition writer end with a retryable refusal whose attempt and reason are durably audited (`ADR-0012` lines 857-858);
- a control operation ends with the retryable refusal of rules 1(b) and 4; where it has rolled back without classifying, this ADR does not require that refusal to be durably recorded;
- the asynchronous materializer leaves the obligation owed and observed (`A14-T02`);
- a detector run leaves the pair unclassified, with no marker and no anomaly from that run; a later detector run, or a control's classify step, classifies it.

None loops silently.

**1(j) Timeouts, deadlines and starvation.**

- Every member that can hold a route boundary, a job row or an identity lock sets a `lock_timeout` for its waits on other members and has a total deadline, as a control operation does.
- The Release R specification states the boundary's lock modes. It shows, by mode compatibility or by measurement, that transactions outside the protocol cannot starve a member at the boundary, at a job row, at a referenced row's `KEY SHARE` lock or at an identity lock.
- It shows in the same way that members holding the boundary in the admission mode cannot starve a control operation at the boundary. This covers withdrawal, retirement, and the epoch close that `ADR-0012` section 10 requires before a rollback, for example while a materialization backlog keeps several admissions on one route.

**1(k) Enrollment and divergence keys.**

- First historical enrollment and divergence take the shared keys of the events they act on as the first locks of their transaction, before any canonical-identity claim, anomaly row lock or route row lock.
- With more than one key they follow the control rule: wait for the first, try the rest.
- They read revision availability and anomaly state only in statements issued after the grant.

**1(l) A coarser key.** The Release R specification may replace the per-kind keys with one routing key for all kinds (strictly coarser) if its measurements show that the try rule starves control operations. It may not use a finer key.

#### Rule 2 - Classify, then decide

- Holding its exclusive keys, a withdrawal or retirement first runs the ordinary classifier (the detector's code path and canonical anomaly identity) over every unclassified committed event of `K(route)`. It then runs its strand-safety scan. Each runs in statements issued after the grant.
- Detection otherwise stays asynchronous, exactly as `ADR-0012` describes.
- An asynchronous detector run commits after each (event, route) pair. Within a pair it takes or waits for the pair's gate before its first write, so it never waits while it holds an uncommitted marker or anomaly.
- A control's classify step writes its markers and anomalies in ascending canonical order.
- **The pair gate.** Every classifier, enrollment, divergence and first-disposition writer (the asynchronous materializer and the staff terminal disposition writer included) takes the same single per-pair gate before any marker, anomaly or disposition write for that pair. A marker and an anomaly of one pair are therefore never claimed in different orders by two writers. The gate is one unique key or lock that the Release R specification defines for the (event, route) pair.
- An inline writer takes the gate of its own event's pairs. That event is uncommitted, so no other transaction can contend for an exact gate of that pair.
- The gate is exact for the pair: a unique key on it, or a lock that two pairs cannot share. A hashed gate is allowed only under the same Release R proof duty as the routed-job identity mechanism (rule 1(h)).
- The gate is the pair's canonical claim. Writers take gates in the one ascending canonical order of rule 1(d). Every wait that this ADR describes on a marker or anomaly write is a wait for that pair's gate.
- A detector run and a control's classify step therefore cannot wait on each other in a cycle, and no cycle can pass through a detector.
- Epoch opening and closing take their keys and the route boundary only, to order themselves against event emission, withdrawal and enrollment. They never classify and never refuse on backlog. The epoch close that `ADR-0012` section 10 requires before a rollback (lines 3747-3749 and 3763-3766) is therefore never blocked by a classification backlog. Rule 1(j) covers a materialization backlog.

#### Rule 3 - Provenance

- The classify step is a real detector run inside the control's transaction.
- An anomaly it creates records that run, and the control operation that invoked it, as its first observer (`ADR-0012` section 3.2, first-observer provenance, lines 612-616).
- It never records a detector run that did not happen (section 11, "provenance fidelity", line 4318; `A13-T04`).

#### Rule 4 - Bounded work and refusals

- If the unclassified backlog of `K(route)` exceeds a bound stated in the Release R specification, the control operation refuses with a retryable code before classifying. The exclusive section therefore stays short.
- The backlog count and the scan are index-bounded. There is no full event scan.
- A control operation that classifies and then refuses for strand safety commits its markers, its anomalies and its refusal record together (a committed refusal), so a retry does not repeat the work.
- Each classify batch and the strand-safety scan run under a savepoint. A `lock_timeout` or `statement_timeout` therefore rolls back only that statement's batch, and the control still commits its earlier batches with a retryable committed refusal (rule 7).

#### Rule 5 - Isolation

- Event-emitting transactions and every member of the routing protocol (detector runs, controls, enrollments, divergences, the asynchronous materializer and the staff terminal disposition writer) run READ COMMITTED.
- The event trigger and every member check `transaction_isolation` and fail closed otherwise.
- The routing generation column on the event row is NOT NULL. That catches only an insert that bypasses the trigger and omits the generation. An insert that bypasses the trigger and supplies a value is outside the guarantee, like any trigger-disabling path.

#### Rule 6 - Enrollment and divergence

- Because enrollment and divergence take their shared keys first and read after the grant (rule 1(k)), they serialize against withdrawal, retirement and epoch transitions on the same keys. This gives invariant 41 of `ADR-0012` its single withdrawal boundary, without classifying first.
- Their races with detection keep the canonical decision order of `ADR-0012`.

#### Rule 7 - Costs and the no-deadlock guarantee

**Stall.** While a control operation waits for its first key or holds its keys, new transactions emitting those kinds queue behind it (PostgreSQL queues shared requests behind a waiting exclusive one). The stall is bounded by the control's bounded work plus its waits, each bounded by its `lock_timeout`: its first lock, and every pair gate it meets while classifying.

**No deadlock.** The protocol never aborts a business transaction through a deadlock. The reasons are:

- A control operation never waits for a routing key or a route boundary while it holds either. It never locks a business row after its first lock. After its first lock it waits only for pair gates that other members hold, and those members never wait outside the protocol after their first lock.
- Every other boundary holder that rule 1 binds takes every routing key it needs before the boundary. After its first lock it waits on no transaction outside the routing protocol.
- After its first lock no member waits on a transaction outside the protocol (rule 1(e)).
- For an inline writer, and for any wait that a routed-job mechanism adds between two transactions outside the protocol, the guarantee is a proof obligation of the Release R specification (rules 1(f) and 1(h)).

So a business transaction that waits on a lock a member holds is behind a member that will finish or roll back within its deadline, without waiting outside the protocol. A business transaction that waits on a routing key is in one of three positions:

- behind a control that holds nothing yet;
- behind a control that holds the key and waits at most for pair gates that other members hold, and those members never wait outside the protocol after their first lock; or
- in a queue-order (soft) wait, which PostgreSQL's deadlock check resolves after `deadlock_timeout` by reordering the wait queue, without an error.

**Timeouts of business transactions.** A business transaction with its own `lock_timeout` or `statement_timeout` can still time out during the stall. The repository has such paths, for example `SET LOCAL lock_timeout = '1s'` in `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs`. Therefore:

- The Release R specification bounds the stall in time. The bound includes waits behind a member's identity lock, job row or boundary, and the `deadlock_timeout` that passes before a queue-order wait is reordered.
- Because `lock_timeout` bounds each wait separately, a control also has a total deadline for its transaction, stated in the Release R specification and checked before each classify batch. The stall is then bounded by that deadline plus one batch, its strand-safety scan and its commit, rather than by a sum of waits. The Release R specification states that sum, or runs each statement after the check with `statement_timeout` set to the time left, under the savepoint of rule 4.
- A control that reaches its deadline commits the markers and anomalies it has written together with a retryable committed refusal, as rule 4 does.
- Every event-emitting path states its timeouts against that bound.

**Markers.** Markers cost one row per in-epoch event per route that consumes its kind. The Release R specification states their expected volume and retention.

**Business writes.** Business writes take only shared locks and capture the generation. Classification never runs in them, so a classifier fault cannot fail a metadata edit.

#### Rule 8 - Required tests

**Reachability of the A13 cases.**

- Because detection stays asynchronous, every A13 absent-anomaly order stays reachable with real concurrent transactions and no harness that removes the race (`ADR-0012` section 11): `A13-T02`, `A13-T03`, the absent-anomaly cases of `A13-T08`, `A13-T10` and `A13-T11`, detection versus divergence, and `A13-T27` to `A13-T30`.
- `A13-T15` and `A13-T17` remain reachable for events committed after the withdrawal.
- For an event committed before the withdrawal, the anomaly's first observer is now the withdrawal's own detector run (rule 3). The clause of `A13-T15`, "a detector run after the withdrawal records the mismatch", therefore describes the post-withdrawal case.
- The latent-mismatch variant becomes impossible. Release R proves that with the asynchronous detector paused.

**The stale-scan proof.** The proof item at lines 3946-3948 is met by construction and shown by a test:

- A withdrawal is paused by a barrier between its strand-safety scan and its commit. Meanwhile every anomaly-creating path is driven against the route: the detector on an event committed before the withdrawal, a new event of `K(route)`, enrollment and divergence.
- The detector waits on the withdrawal's uncommitted marker for that pair. The withdrawal's markers are not yet visible to it.
- A new event of `K(route)`, and an enrollment or divergence, wait behind the exclusive keys.
- So no anomaly that the scan missed linearizes before the withdrawal commits. After the commit the detector finds the pair classified.

The stale-scan rule is unchanged. It stays binding on every anomaly-creating path, including any that the Release R specification adds.

**Further required tests.** Release R also tests the following, with real sessions and an unchanged `pg_stat_database.deadlocks`. A test that names an inline writer applies where the Release R design has inline writers of that kind.

1. A business transaction emitting two consumed kinds in descending key order against a concurrent withdrawal: no deadlock, and the business write is never aborted.
2. An enrollment racing a withdrawal on one canonical identity.
3. An event of a kind consumed only by an older epoch's revision.
4. An epoch opening with a new kind racing a retirement: the retirement re-reads `K(route)` and retries.
5. An epoch close while a classification backlog exists: it is not refused.
6. Two multi-event enrollments, and an enrollment and a divergence, on overlapping identities requested in opposite orders.
7. A multi-event enrollment against concurrent single-pair detector runs.
8. An inline writer that creates a routed job, and one that attaches to a coalescing-admission-open job, while an enrollment or an asynchronous materializer writes a job with the same identity, in both orders and with two identities taken in opposite orders: no cycle and no aborted business transaction, because after its first lock the member never waits outside the protocol (rule 1(e)).
9. An enrollment that has written its job and needs a route boundary held by an inline writer, which is itself waiting for a business row whose holder then writes a job with the same identity. Both the enrollment and the inline writer are admissions, so the shared admission mode grants the enrollment's boundary request at once: no wait, no retry and no deadlock, and the business transactions finish.
10. Two transactions outside the protocol that both write routed jobs A then B (ascending idempotency-key order, which Release N requires of a caller that creates more than one job in one transaction), while one of them learns both identities together, so that a mechanism that orders by its own key would take them in the other order; and two whose identities share a key under the mechanism (a forced collision). Neither pair deadlocks, and no transaction is refused because of the collision.
11. A forced collision of two pairs' gates, if the gate is hashed.
12. For each member type (detector, control operation, enrollment, divergence, asynchronous materializer and staff terminal disposition writer), a run that exhausts its retry bound and ends in the outcome that rule 1(i) states for that type.
13. An asynchronous materializer about to attach to a coalescing job, which needs that job's row, while an authorized current-state supersession holds that row (it is cancelling the job) and then creates its superseding job with the same identity, in both lock orders: no aborted business transaction (`A14-T16`).
14. A materializer inserting a successor that references its predecessor, while a Release N worker write holds the predecessor `FOR UPDATE` and then attaches inline to the successor's identity: the member's non-waiting `KEY SHARE` request fails and it retries.
15. `A14-T06` run with an inline attacher, which waits for the cancel, and with a member attacher whose row lock is not its first lock, so that its non-waiting row-lock request fails and it retries. The case is not narrowed.
16. A staff terminal disposition writer that has locked a row a business transaction can lock, and then needs a route boundary that an inline writer holds while waiting for that row. The boundary is not the staff writer's first lock, so its request is non-waiting. Both are admissions, so the shared admission mode grants the request at once: no wait, no retry and no deadlock, and both transactions finish.
17. Each routing-protocol member and an event-emitting transaction, run under REPEATABLE READ and under SERIALIZABLE: each fails closed. This test replaces the serializable schedule of `A13-T03` (section 5.6).
18. A detector writing a marker that references a route row while a control holds that route's boundary: the foreign-key check does not wait (rule 1(f)).
19. `A13-T02` (an enrollment-first born-terminal anomaly) followed by a withdrawal: the pair is classified once and never counts as backlog again.
20. The schedule control -> detector's uncommitted marker -> enrollment's canonical claim -> business row -> routing key held by the control. It cannot form: the detector waits only before its first write, and the enrollment waits on no transaction outside the routing protocol while it holds a claim.
21. A retirement of a route with no epoch racing the route's first epoch opening, in both orders: the retirement re-reads `K(route)` after the boundary and retries.
22. An event of a kind in `K(route)` that its own epoch's revision does not consume: it never needs a marker and never counts as backlog.
23. An event insert inside a savepoint that rolls back, followed by an insert of the same kind: the key is taken again.
24. A detector and a withdrawal classifying the same events, including a detector that is mid-transaction on an event when the withdrawal reaches it: the withdrawal waits within its `lock_timeout` under the savepoint of rule 4, then classifies or commits a retryable refusal, and no deadlock occurs.

### 5.5 Why this closes the window

Under READ COMMITTED, an event transaction that holds a shared key commits, with its generation, before the exclusive key is granted. PostgreSQL makes a commit visible before it releases the committing transaction's locks. The control's classify step is a later statement, so it sees the event and classifies it before the scan.

An event transaction that has not yet taken its shared key waits, and commits after the decision. That is the withdrawal-first order, which `ADR-0012` accepts.

Every kind the decision depends on is in `K(route)`. No schedule leaves a committed, unclassified event of a relevant kind at the decision point.

### 5.6 `ADR-0012` text that Property P narrows, and what stays binding

Property P and its rules narrow `ADR-0012` text in three ways. This is the complete list.

**First, the withdrawal-first order.** The withdrawal-first order, and the rules and proofs written for it, apply only to events committed after the withdrawal or retirement decides. An event committed before the decision is classified first. The narrowed text is:

- section 3.2's withdrawal-first order of anomaly creation against withdrawal (lines 1145-1157);
- invariant 43's withdrawal-first sentence (lines 3278-3284);
- section 11's withdrawal-first race item (line 3945) and withdrawal-first detection item (lines 3957-3962);
- the detector clause of `A13-T15` only: "a detector run after the withdrawal records the mismatch" (lines 4405-4407). Its first clause stays binding for every event: when the withdrawal linearizes first, "no terminal `ENROLLED` outcome, disposition, job or enrollment-created anomaly results" (lines 4404-4405);
- the withdrawal-first race of `A13-T17` only. Its anomaly-first race, its rule that no stale strand scan is permitted and its rule that anomaly creation is never suppressed stay binding (lines 4412-4415).

The stale-scan rule ("A withdrawal transaction whose scan preceded the anomaly's creation may not commit from that stale scan", lines 1140-1142) and its proof item (lines 3946-3948) are not narrowed. They stay binding as written, and Property P meets them by construction (rule 8).

**Second, first-observer provenance.** Section 3.2 names two first observers: "the detector that recorded it `OPEN`, or the enrollment that created it born-terminal" (lines 615-616). Property P admits a third: a detector run invoked by a control operation. It counts as a detector observation (rule 3).

**Third, the serializable schedule of `A13-T03`.** `A13-T03` requires "at least one explicit serializable-failure-and-retry schedule" (lines 4343-4344). That schedule cannot occur, because every routing-protocol member runs READ COMMITTED and fails closed otherwise (rule 5). It is replaced by the isolation fail-closed test (rule 8, test 17) and by the remaining row-lock and unique-key schedules of `A13-T03`.

Every other A13 case, every other part of `A13-T03` and `A13-T17`, and every decision-order rule is unchanged.

Every other `ADR-0012` clause, invariant, rule and validation item stays in force unchanged, and every routing and coalescing rule binds Release R exactly as written.

## 6. Invariants

These restate the decision. They add nothing to sections 4 and 5.

1. No route is registered and no coalescing kind is activated before Release R is deployed. No routed or coalescing kind can exist in Release N.
2. Release N keeps everything `ADR-0012` section 4.1 says about legacy BE-04 storage and its released contract.
3. A consumer migrates only after the Thoth release that carries the contract it migrates onto is deployed and rollback-safe.
4. Property P: a withdrawal or retirement never decides strand safety while a committed event that needs classifying for the route is unclassified.
5. The strand-safety predicate of `ADR-0012` and its stale-scan rule are unchanged.
6. Detection stays asynchronous outside control operations. Classification never runs in a business transaction.
7. The routing protocol never aborts a business transaction through a deadlock. No residual is allowed.
8. After its first lock, a member of the routing protocol never waits on a transaction outside it.
9. Event-emitting transactions and every member run READ COMMITTED and fail closed otherwise.
10. Anomaly provenance is truthful: a classify step records its own detector run and the control that invoked it, and never a run that did not happen.
11. Epoch opening and closing never classify and never refuse on backlog.
12. `ACCEPT_REGRESSION` records a regression or conflict that already happened and performs none. An executor reconciler never accepts a regression.
13. Every `ADR-0012` invariant not named in section 5.6 is unchanged.

## 7. Implementation impact

This ADR's own change is documentation only (THOTH-ASYNC-01-ENGINE-01, #1026):

- this file and its `decision-register.md` row; the `ADR-0012` register row notes the partial supersession;
- the supersession header of `ADR-0012` (`Superseded by: None in full` and a new `Superseded in part by` line naming this ADR). The CTO records this as a `FACTUAL CLARIFICATION` naming the prior approved blob `896195c8e95f0590fae2d26237b6265c5bf9d1c9`. No other byte of `ADR-0012` changes;
- one `CHANGELOG.md` line.

Release N (programme step 1, separately specified and authorized on #1026):

- builds the job engine of section 4.1 and nothing of Release R;
- is the same under either option of section 3.1, and relies only on repository-authoritative `ADR-0012`. Its `ACCEPT_REGRESSION` resolution rests on `ADR-0012` lines 1433-1435, which section 4.8 reads. The README rule that implementation stops when it depends on a decision that is not repository-authoritative is therefore not engaged by it, and it may be built on the programme integration branch `feature/worker`, which never deploys, before this ADR is authoritative;
- is integrated into `develop` only once this ADR and the `ADR-0012` version with the clarified header are both repository-authoritative, so that `develop` never carries code whose release composition contradicts or outruns an authoritative ADR. After this ADR lands, the later gates of step 1 bind to both blobs;
- if this ADR is not approved, may still land on `feature/worker`, and the programme returns to the CTO for option A of section 3.1 or a revised ADR.

Release R (its own later specification):

- is bound by section 5 and by every routing and coalescing rule of `ADR-0012`;
- states and proves what section 5 leaves to it: every retry bound, the backlog bound, total deadlines and the stall bound in time; the pair gate; any marker equivalent; the boundary's lock modes, holders and order; the inline writer's full lock order; the routed-job identity mechanism with its two proofs; the absence of waits outside the protocol for every lock and write a member makes after its first lock; the absence of waiting foreign-key checks; both starvation showings of rule 1(j); marker volume and retention; and the timeouts of every event-emitting path.

Consumers: each consumer specification shows that its job creation path is a direct creation that `ADR-0012` allows, or waits for Release R (section 4.1).

## 8. Migration, rollout and rollback

Migration: none from this ADR. Release N's migration is the additive generic schema migration of `ADR-0012` section 4.7. Release R's migrations follow in the same ordered history.

Rollout: documentation only, with no runtime effect. The releases roll out in the order of section 4.3. Phase A of `ADR-0012` section 9 is repeated for Release R and for any later release that carries a consumer contract (section 4.2).

Rollback: Release N follows the rules of `ADR-0012` section 10 other than the route-epoch rules, which bind from Release R (section 4.5). Rollback of this decision requires a further superseding ADR.

## 9. Validation evidence

Release N obligations:

- the Release N parts of the rollout order (section 4.3, step 1);
- the matrix items that gate Release N (section 4.4);
- every item and part in the "Release N" column of section 4.6.

Release R obligations:

- the Release R parts of the rollout order (section 4.3, step 3), before any routed or coalescing kind is activated;
- the matrix items that gate Release R (section 4.4);
- every `ADR-0012` section 11 item for routing, epochs, revisions, anomalies, enrollment, withdrawal, dispositions and coalescing, and `A13-T01` to `A13-T31`, as narrowed only by section 5.6;
- every test of rule 8, including the proof with the asynchronous detector paused, the stale-scan test and tests 1 to 24.

Evidence for this decision:

- the Release N engine specification that carried these decisions was reviewed in successive revisions by two independent CRITICAL reviewers, and both approved its final revision; the record is on #1026;
- this ADR's own independent exact-head review, on its pull request.

## 10. Approval

Proposed. Not approved.

On 2026-11-13 the CTO decided the release composition, Property P and the `ACCEPT_REGRESSION` reading, recorded on #1026. Those decisions do not approve these bytes.

This ADR follows the PR-first staged sequence of `docs/engineering/decisions/README.md` (Route B). The CTO's approval of this exact content, and then of the exact final blob, are each recorded on #1026. Repository authority follows only from independent exact-head review and merge into `develop`. Until then no implementation may rely on this ADR.
