# THOTH-ASYNC-01-ADR-01 Implementation Report

This report records the bounded ADR-0012 architecture task for
THOTH-ASYNC-01: the original architecture authoring, its independent-review
correction rounds, the historical approval-state reconciliation, the
programme-integration compatibility reconciliation, the route-disposition
clarification, the Route-B material activation-epoch correction, the bounded
final-head recovery authorized by Specification Amendments 4-5, the Route-B
candidate and approval-state stages of the immutable route-contract-revision
correction specified by Specification Amendments 6-7, the one-path
report-only control-record recovery of that approval-state report, the
one-path report-only transient-state correction that followed the configured
final-head automated review of that recovery head, the two-path factual
clarification of ADR-0012 and durable-report correction that followed the
configured final-head automated review of that correction head, the
Route-B candidate and approval-state stages of the revision-withdrawal/selection
ordering correction specified by Specification Amendment 8 after the configured
final-head automated review of that clarification head, and the Route-B
candidate and approval-state stages of the coalescing-attachment/first-claim
ordering correction specified by Specification Amendment 9 after the configured
final-head automated review of that Amendment-8 approval-state head. It records
durable task evidence only. Live head, CI, review, authorization and
merge state is GitHub-owned lifecycle evidence under `ADR-0005` and is not
copied here.

Sections 1-15 follow the repository implementation-report template for the
Route-B correction stages, with the Amendment-9 Route-B approval-state commit
as the current stage and every earlier candidate, approval-state, recovery,
report-only correction and clarification stage recorded as history. Appendix A
preserves the earlier authoring, review, approval-state, reconciliation and
validation record. Appendix B records the
durable control conditions.

## 1. Repository state

Owning GitHub issue: [#958](https://github.com/thoth-pub/thoth/issues/958)
Parent programme: [#957](https://github.com/thoth-pub/thoth/issues/957) (THOTH-ASYNC-01)
Repository: `thoth-pub/thoth`
Risk: CRITICAL
Workflow: PROGRAMME_INTEGRATION, retaining the approved transitional exception
that keeps the pre-topology task branch `feature/async/adr-0012` instead of a
`feature/worker--<slice>` branch (base specification, #958 comment
`5942203087`, section 1). The task originally ran as STANDARD against `develop`.
Base branch: `feature/worker`
Authorized base commit: `b44303c214498baf76c9d9a0a7cab374377f9cbe`
Actual base commit: `b44303c214498baf76c9d9a0a7cab374377f9cbe`
PR target: `feature/worker`
Programme integration branch: `feature/worker`
Task branch: `feature/async/adr-0012`
Head commit: the Amendment-9 Route-B approval-state commit that contains
this report, an ordinary single-parent additive commit whose direct parent is
the content-approved Amendment-9 candidate
`8699af5635b8f18691ceaeccd3b24707a71ef4f2`; its SHA is deliberately not
self-pinned here (section 3). That candidate is the direct child of the
historical Amendment-8 approval-state head
`e2588f7b7e63a5c71968d87a3409f0ff48635e6d`, which is the direct child of
the content-approved Amendment-8 candidate
`42ae90a5932cf24ce90679196888cc3d83d88db8`, which is the direct child of the
historical factual-clarification head
`eb3129f13a9c858b3c012ee182315801f0b8563a`, which is the direct child of
the historical report-only transient-state correction head
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`, which is the direct child of the
historical report-only control-record recovery head
`22bd94dc959f11fe49c8bef8cd2a836140aadff7`, which is the direct child of the
historical Amendment-6/7 approval-state head
`609a01427847233b7280557baa61825ac2079865`, which is the direct child of
the content-approved Amendment-6/7 candidate
`b596536e7db0d6b8ace0a42fdd26bff42cabf104`, which is the direct child of the
historical Amendment-4/5 recovery head
`9db91af7bf3d9f969556c819187420fc31d25c10`, which is the direct child of the
historical activation-epoch approval-state head
`c046e6bf6510f24ee1231f86293a3d499bceed7e`, itself the direct child of the
historical content-approved activation-epoch candidate
`95769537cdc9871e42f9f3e5443604a96beb74da`. The approval-state commit's
publication as the PR #960 head is a post-commit, GitHub-owned lifecycle fact
under `ADR-0005`.
Pull request: [#960](https://github.com/thoth-pub/thoth/pull/960), targeting
`feature/worker`; its live state, head and checks are GitHub-owned lifecycle
evidence.
Expected branch deletion after merge: YES
Final programme PR required: YES - `feature/worker -> develop` is a separate
programme-level gate.
Implementing model: activation-epoch candidate and approval-state stages -
Claude (Opus 5.5), bounded implementing agent; Amendment-4/5 recovery stage -
Claude (Fable 5.1), bounded implementing agent; Amendment-6/7 candidate,
approval-state and control-record recovery stages - Claude (Opus 5.5), bounded
implementing agent; transient-state correction, factual-clarification,
Amendment-8 candidate, Amendment-8 approval-state, Amendment-9 candidate and
Amendment-9 approval-state stages - Claude (Fable 5.1), bounded implementing
agent. Earlier rounds are recorded in Appendix A.
Reasoning level: standard

Historical and current control identities:

```text
original authorized base:
develop @ 923545d5c9028bc04c40e38efeb7de674efed3fd

historical programme integration base:
feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462

historical feature/worker refresh after CTRL-ADR-AMEND-01 (#964):
345a7a04131e7c0539f7518c6ea457fa5cb6a462
-> 8ac771c55937f7a4fe4131b3c21c466f0b4d7abc

Route-B feature/worker refresh after CTRL-ADR-PR-FIRST-01 (#966 / PR #967):
8ac771c55937f7a4fe4131b3c21c466f0b4d7abc
-> b44303c214498baf76c9d9a0a7cab374377f9cbe

repository-authoritative Route-B doctrine:
docs/engineering/decisions/README.md
blob de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7

original CTO architecture-content approval source:
631e28d1f05495f24ce88369d4557f1033f970b3
ADR-0012 blob 9eb21363aa2f60a86db1a2293d697545ec3809e9

historical approval-state source and stale earlier programme pin:
bd5223deee44b465dd860ff96b75d81ad41cbf2d
ADR-0012 blob cffee08b0c4326823f36387e6cec2abe49594273

prior ADR-0012 blob before the activation-epoch correction:
dec6665353804e42f22474954bf878308092b717

task head before the activation-epoch correction / its candidate parent 1:
b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20

refreshed feature/worker / activation-epoch candidate parent 2:
b44303c214498baf76c9d9a0a7cab374377f9cbe

unmodified Route-B merge-tree baseline:
e15607878e66d2ff12b819dcea13293a46674f13

content-approved activation-epoch candidate / approval-state parent:
95769537cdc9871e42f9f3e5443604a96beb74da
tree 5084c6951a1b57b449f6045b139aacfbc488e534
ADR-0012 blob a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c

historical Route-B approval-state head / recovery parent:
c046e6bf6510f24ee1231f86293a3d499bceed7e
tree 801e12ffc010470928da45919ede8d7239a491ce
ADR-0012 blob 4848470a252076268a9fac9da5c40f0b4ebff474 (unchanged by the recovery)
decision-register blob b19b1404dafa62a7aaebcd33708841cf44ce1831
implementation-report blob cd97eb441a38ff1885cf621e1af04519c470dffc

historical Amendment-4/5 recovery head / Amendment-6/7 candidate parent:
9db91af7bf3d9f969556c819187420fc31d25c10
tree fd9a300265c310388652ff94a4ef9f95224ce018
ADR-0012 blob 4848470a252076268a9fac9da5c40f0b4ebff474 (APPROVED activation-epoch version; historical)
decision-register blob 247d077786a9176e628baa8de64c0c3a5e9b09d3
implementation-report blob 64326e564b3f27f94aa9ced7dd7572bb30244bb8
CHANGELOG blob ff7089a4a93e6a8c2c27c75a0ebf56ce43405c27

content-approved Amendment-6/7 candidate / approval-state parent:
b596536e7db0d6b8ace0a42fdd26bff42cabf104
tree d4d7dd359c3e20d37fcb5f664721b87cd2e480a8
ADR-0012 blob cd897d576522a741c3dfa82f8de2f8d80d4288f6 (PROPOSED; content-approved)
decision-register blob 18e3708d4247a9732778b8e71e5b52b95e9de776
implementation-report blob a47c299a2f9d4a8ab4938ed958050dcb531a3455
CHANGELOG blob db70c9827cde16887c11dbb4ef19258c36251d6f

historical Amendment-6/7 approval-state head / control-record recovery parent:
609a01427847233b7280557baa61825ac2079865
tree abc024bd16ebf8198475ef196c1f829228760abe
ADR-0012 blob a7d95ca918f418d596432071c8ba78a33c2630a9 (APPROVED; unchanged by the recovery)
decision-register blob b358acd5bbedf7482cfdc7b71866815c852773af (unchanged by the recovery)
CHANGELOG blob e2947275d19d3abb351a98854268d7db213370d7 (unchanged by the recovery)
implementation-report blob 6b1a0c2e5f353c6974bf011726eb7beac804a3bd (superseded by the recovery)

historical control-record recovery head / transient-state correction parent:
22bd94dc959f11fe49c8bef8cd2a836140aadff7
tree 7af3e0b8007eb6686e94043f57db63acc4edb4ab
ADR-0012 blob a7d95ca918f418d596432071c8ba78a33c2630a9 (APPROVED; unchanged by the recovery and by the transient-state correction)
decision-register blob b358acd5bbedf7482cfdc7b71866815c852773af (unchanged by the recovery and by the transient-state correction)
CHANGELOG blob e2947275d19d3abb351a98854268d7db213370d7 (unchanged by the recovery and by the transient-state correction)
implementation-report blob 74845b91dede2ba066bd0720598651e539196eb1 (superseded by the transient-state correction)

historical transient-state correction head / factual-clarification parent:
9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a
tree 862a8e32207f1e0f61cab532b08b65034979f5ac
ADR-0012 blob a7d95ca918f418d596432071c8ba78a33c2630a9 (APPROVED; the exact prior approved blob clarified by the factual clarification)
decision-register blob b358acd5bbedf7482cfdc7b71866815c852773af (unchanged by the factual clarification)
CHANGELOG blob e2947275d19d3abb351a98854268d7db213370d7 (unchanged by the factual clarification)
implementation-report blob 81db6d33bc0d103e149dd99e1f35150f35c079e3 (superseded by the factual clarification)

historical factual-clarification head / Amendment-8 candidate parent:
eb3129f13a9c858b3c012ee182315801f0b8563a
tree 5244d6ac76578a6f78cde79671762da5b09f2fc1
ADR-0012 blob 2db08058552ce901548ef3f71af4a294f5af7e61 (APPROVED; Javi, CTO; 2026-10-06; factual clarification of a7d95ca918f418d596432071c8ba78a33c2630a9; the exact prior approved blob that the Amendment-8 correction corrects)
decision-register blob b358acd5bbedf7482cfdc7b71866815c852773af
CHANGELOG blob e2947275d19d3abb351a98854268d7db213370d7
implementation-report blob c47b874f1a48449e6c8f60a97f00ed1506d55fb9 (superseded by the Amendment-8 candidate)

content-approved Amendment-8 candidate / approval-state parent:
42ae90a5932cf24ce90679196888cc3d83d88db8
tree 45d8a0bdb05ca95b61c01cdc83dcecf3a3f60e98
ADR-0012 blob cbb31a53f6f4ed6b96faafb79b273b55664f3ae0 (PROPOSED; content-approved; material architectural correction of 2db08058552ce901548ef3f71af4a294f5af7e61)
decision-register blob 45ce935149693d0e983fac1645f5363896e23a53
implementation-report blob f85da63b89654d4774462de674eb02b4a1a0c38d
CHANGELOG blob f3c2bc7976d5fb84e1a00cbfce05748197da900b

historical Amendment-8 approval-state head / Amendment-9 candidate parent:
e2588f7b7e63a5c71968d87a3409f0ff48635e6d
tree bec3637f333a12ed1895b2aae6f1078510067f5e
ADR-0012 blob 85937a4e1ff9a27d8f5b36ca82651821711472b2 (APPROVED; Javi, CTO; 2026-10-07; architecture-equivalent to cbb31a53f6f4ed6b96faafb79b273b55664f3ae0; CTO exact-final-blob approval recorded on #958 on 2026-10-07; the exact prior approved blob that the Amendment-9 correction corrects)
decision-register blob 246bd5e981d373447fccdd457c02645338bfee4b
CHANGELOG blob d1fbe1bdc7699bae18970cdcc9c61d9cbdcefa82
implementation-report blob 307a46ed153cd1287b24d921e7286c62328e8cc1 (superseded by the Amendment-9 candidate)

content-approved Amendment-9 candidate / approval-state parent:
8699af5635b8f18691ceaeccd3b24707a71ef4f2
tree ebff7219611e7855c62856019fc051fcf41cb2b5
ADR-0012 blob 0fc33df9aca61a2c82af452426f5574bfd3493cf (PROPOSED; content-approved; material architectural correction of 85937a4e1ff9a27d8f5b36ca82651821711472b2)
decision-register blob 8b5ef6b5b6ca7f6d49ed5066aba5e49575de06be
implementation-report blob ffbb44f9f250d325da6846796dfae8051a3a839e
CHANGELOG blob 4d0e17ca0b11dea0ed4905ef9776693ee2c87bc4

Amendment-9 Route-B approval-state commit (the commit that contains this report):
ADR-0012 blob cbe8322d95fe8d5e9911d421a5f10f1ae4afd55c (APPROVED; Javi, CTO; 2026-10-08; architecture-equivalent to 0fc33df9aca61a2c82af452426f5574bfd3493cf)
decision-register blob c618d9d2eec622660ee7c1868111f747efa27629
CHANGELOG blob 2731521f363189063b9ab09068df7852889121a2

feature/worker at the recovery, the candidates, the approval-state commits, the
control-record recovery, the transient-state correction, the factual
clarification, the Amendment-8 candidate and approval-state commit and the
Amendment-9 candidate and approval-state commit (unchanged):
b44303c214498baf76c9d9a0a7cab374377f9cbe
```

Both `feature/worker` refreshes were separately authorized control actions
recorded on #958 before any correction source was prepared; the Route-B refresh
receipt and post-refresh geometry check are #958 comment `5993603036`. Neither
refresh is part of this task's source write budget.

## 2. Scope confirmation

Approved specification:

- base post-#964 reconciliation specification: #958 comment `5942203087`;
- Specification Amendment 1: #958 comment `5948725125`;
- Specification Amendment 2 / Route B: #958 comment `5958029093`;
- Specification Amendment 3: #958 comment `5958882383`, which ratifies
  Amendment 2 as the selected Route-B task authorization;
- fresh independent CRITICAL specification approval receipt: #958 comment
  `5992836778`;
- candidate-stage implementation authorization: #958 comment `5993634460`;
- replacement-candidate authorization: #958 comment `5998119577` (section 5);
- approval-state authorization: #958 comment `5999579357`;
- Specification Amendment 4 / final-head review recovery: #958 comment
  `6000543428`;
- Specification Amendment 5 / Amendment-4 review closure: #958 comment
  `6001012674`;
- fresh independent CRITICAL specification approval of the base specification
  and Amendments 1-5: #958 comment `6013265532`;
- bounded recovery implementation authorization: #958 comment `6013267369`;
- Specification Amendment 6 / immutable route-contract history: #958 comment
  `6018536323`;
- independent CRITICAL review of Amendment 6 (`CHANGES REQUIRED`, findings
  accepted as binding blocking evidence): #958 comment `6020055747`;
- Specification Amendment 7 / Amendment-6 review closure: #958 comment
  `6020074189`;
- fresh independent CRITICAL specification approval of the base specification
  and Amendments 1-7, including eight candidate-inspection clarifications:
  #958 comment `6021556329`;
- Amendment-6/7 candidate implementation authorization: #958 comment
  `6021574322`;
- CTO exact candidate-content approval of the Amendment-6/7 candidate:
  recorded on #958 on 2026-10-06 for candidate head
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104` and candidate ADR-0012 blob
  `cd897d576522a741c3dfa82f8de2f8d80d4288f6`; its GitHub comment identifier
  is GitHub-owned lifecycle evidence under `ADR-0005` and Route-B doctrine and
  is not copied here;
- Amendment-6/7 approval-state implementation authorization: #958 comment
  `6022486352`;
- report-only control-record recovery authorization: #958 comment
  `6022850916`;
- final-head review report-only transient-state correction authorization:
  #958 comment `6036961945`;
- CTO factual-clarification classification of ADR-0012 blob
  `a7d95ca918f418d596432071c8ba78a33c2630a9` and adjudication of the
  final-head review of the transient-state correction head: #958 comment
  `6041284313`;
- factual-clarification and durable-report correction authorization: #958
  comment `6041301223`;
- Specification Amendment 8 / revision-withdrawal selection linearization,
  which accepts the final-head finding `R-EB31-01` as valid and blocking and
  classifies its correction as a material architectural correction: #958
  comment `6042399237`;
- fresh independent CRITICAL specification approval of the base specification
  and Amendments 1-8, including five non-blocking candidate-inspection
  observations: #958 comment `6044203556`;
- Amendment-8 Route-B candidate implementation authorization: #958 comment
  `6044220221`;
- CTO exact candidate-content approval of the Amendment-8 candidate: recorded
  on #958 on 2026-10-07 for candidate head
  `42ae90a5932cf24ce90679196888cc3d83d88db8` and candidate ADR-0012 blob
  `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`; its GitHub comment identifier
  is GitHub-owned lifecycle evidence under `ADR-0005` and Route-B doctrine and
  is not copied here;
- Amendment-8 Route-B approval-state implementation authorization: #958
  comment `6045126538`;
- CTO exact-final-blob approval of the Amendment-8 approval-state ADR-0012
  blob `85937a4e1ff9a27d8f5b36ca82651821711472b2` at head
  `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`: recorded on #958 on 2026-10-07;
  its GitHub comment identifier is GitHub-owned lifecycle evidence under
  `ADR-0005` and Route-B doctrine and is not copied here;
- Specification Amendment 9 / coalescing attachment versus job-start
  linearization, which accepts the final-head finding `R-E258-01` as valid and
  blocking and classifies its correction as a material architectural
  correction: #958 comment `6046180884`;
- fresh independent CRITICAL specification approval of the base specification
  and Amendments 1-9, including six non-blocking candidate-inspection
  observations: #958 comment `6046871411`;
- Amendment-9 Route-B candidate implementation authorization: #958 comment
  `6046895895`;
- CTO exact candidate-content approval of the Amendment-9 candidate: recorded
  on #958 on 2026-10-08 by Javi, CTO for candidate head
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2` and candidate ADR-0012 blob
  `0fc33df9aca61a2c82af452426f5574bfd3493cf`; its GitHub comment identifier
  is GitHub-owned lifecycle evidence under `ADR-0005` and Route-B doctrine and
  is not copied here;
- Amendment-9 Route-B approval-state implementation authorization: #958
  comment `6065879000`.

Implemented objective: the Route-B material architectural correction of
ADR-0012 before repository authority. A first correction (activation epochs)
passed through candidate and approval-state stages and one bounded final-head
recovery stage, all now historical. Final-head review of the recovery head then
exposed a further architecture defect, specified by Amendments 6-7. Its
`PROPOSED` Route-B candidate received CTO exact candidate-content approval, and
the one bounded Route-B approval-state commit recorded that approval. A
one-path report-only control-record recovery then corrected how the
approval-state report recorded that content approval, and a second one-path
report-only correction, made after the configured final-head automated review
of that recovery head, removed statements that treated the live number and
resolved/unresolved state of PR #960 review threads as durable repository
truth. A two-path correction made after the configured final-head automated
review of that correction head then applied the CTO-classified factual
clarification of ADR-0012's route-disposition cardinality wording to the prior
approved ADR blob `a7d95ca918f418d596432071c8ba78a33c2630a9` and removed the
report's remaining statements about which live PR #960 lifecycle gate was
pending. The configured final-head automated review of that clarification head
exposed a further architecture defect, `R-EB31-01`, specified by Specification
Amendment 8. Its `PROPOSED` Route-B candidate, one ordinary single-parent
commit on the clarification head that changed exactly the four authorized
correction paths and requires revision withdrawal and every new durable
selection of a route-contract revision to share one database-atomic linearized
order, received independent control inspection and CTO exact
candidate-content approval on 2026-10-07. The one bounded Route-B
approval-state commit then recorded that approval as durable decision state
(ADR-0012 `PROPOSED -> APPROVED` with `Approved by: Javi, CTO` and
`Approval date: 2026-10-07`, the ADR-0012 register row and the withdrawal-order
correction changelog entry moved to the approved state), and the CTO recorded
exact-final-blob approval of that ADR blob. The configured final-head automated
review of that approval-state head exposed a further architecture defect,
`R-E258-01`, specified by Specification Amendment 9. Its `PROPOSED` Route-B
candidate, one ordinary single-parent commit on the Amendment-8 approval-state
head that changed exactly the four authorized correction paths and requires a
coalescing attachment to an existing logical job and that job's first
successful claim/start to share one database-atomic linearized order, received
independent control inspection and CTO exact candidate-content approval on
2026-10-08. The commit that contains this report is the one bounded Route-B
approval-state commit that records that approval as durable decision state:
ADR-0012 `PROPOSED -> APPROVED` with `Approved by: Javi, CTO` and
`Approval date: 2026-10-08`, the ADR-0012 register row and the coalescing-seal
correction changelog entry moved to the approved state, and this report's
approval-state evidence. No architecture content changes.

- **Activation-epoch candidate stage (historical candidate
  `95769537cdc9871e42f9f3e5443604a96beb74da`).** The candidate implements the
  CTO-selected append-only activation-epoch architecture (#958 comment
  `5935475916`) that resolves review finding `R-b467-01` - a single
  activation/deactivation pair cannot represent repeated route activation
  without rewriting historical eligibility or changing the route identity. It
  carried `Status: PROPOSED` together with a `PROPOSED` decision-register row,
  one `PROPOSED` changelog correction entry and the candidate-stage report.
- **Approval-state stage (historical head
  `c046e6bf6510f24ee1231f86293a3d499bceed7e`).** The CTO approved the exact
  candidate content - head `95769537cdc9871e42f9f3e5443604a96beb74da`,
  ADR-0012 blob `a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c`, inspected against
  prior blob `dec6665353804e42f22474954bf878308092b717` - on 2026-10-05 in a
  durable record on #958. The approval-state commit recorded that approval as
  durable decision state: ADR-0012 `PROPOSED -> APPROVED` with
  `Approved by: Javi, CTO` and `Approval date: 2026-10-05`, the ADR-0012
  register row and the correction changelog entry moved to the approved state,
  and this report's approval-state evidence. No architecture content changed.
- **Final-head recovery stage (historical recovery head
  `9db91af7bf3d9f969556c819187420fc31d25c10`).** After PR #960 was
  separately marked Ready, the configured final-head automated review of the
  approval-state head raised two findings. A route-parentage concern rested on
  a premise that did not match the authoritative PR #960 commit graph (the
  approval-state head is the direct child of the content-approved candidate)
  and required no source correction. The second finding identified that the
  active decision register still carried `Last updated: 2026-09-30` while its
  ADR-0012 row recorded an approval on 2026-10-05; that freshness finding was
  accepted as blocking and caused the bounded recovery under Specification
  Amendments 4-5. The recovery commit advanced the register header to
  `Last updated: 2026-10-05` and reconciled this report; it was one bounded
  post-final-review factual/control correction, not another candidate and not
  another approval-state commit. ADR-0012 (blob
  `4848470a252076268a9fac9da5c40f0b4ebff474`), its approval date and approver,
  the changelog and every other decision and control file were byte-identical
  to the approval-state head. Control then verified the recovery as conforming
  and the CTO confirmed the unchanged exact final ADR-0012 blob at the recovery
  head in durable records on #958.
- **Final-head review of the recovery head and Amendments 6-7 (historical
  control stage).** PR #960 was then marked Ready by control, which triggered
  the configured final-head automated review of the recovery head. It raised
  one new finding, recorded by control as `R-9DB-01` (HIGH, valid and
  blocking): ADR-0012 gave each logical route one route-level event
  kind/version contract that was not immutable and not versioned with its
  activation epochs, so a later contract change could reinterpret closed-epoch
  obligations or make outstanding obligations appear to need a different
  materializer. The CTO approved Specification Amendment 6 (immutable
  route-contract revisions bound to activation epochs). Its independent
  CRITICAL review returned `CHANGES REQUIRED` with three accepted blocking
  findings - an undefined in-epoch contract mismatch, unvalidated backfill
  revision selection and incomplete revision-failure validation - plus
  non-blocking clarifications. The CTO approved Specification Amendment 7 to
  close them, and a fresh independent CRITICAL review of the base specification
  and Amendments 1-7 returned `APPROVED`, closing all three findings and
  `R-9DB-01` at specification level and accepting eight non-blocking
  candidate-inspection clarifications. `R-9DB-01` remained open in source at
  the recovery head.
- **Amendment-6/7 candidate stage (historical content-approved candidate
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104`).**
  Authorized by #958 comment `6021574322`. PR #960 was converted Ready ->
  Draft before any candidate source was staged. The candidate is one ordinary
  single-parent commit on the recovery head that changes exactly the four
  authorized correction paths. Its ADR-0012 returned to `Status: PROPOSED`
  with no current approver or approval date and implements Amendments 6-7 as
  one coherent architecture: every activation epoch references exactly one
  immutable route-contract revision with a canonical semantic signature;
  eligibility requires both epoch membership and a match to that revision;
  continuous producer-version coverage with expand/contract transitions and
  cross-route fan-out; durable `IN_EPOCH_CONTRACT_MISMATCH` anomalies;
  field-by-field validation of historical-enrollment revision selection;
  atomic database-level revision equivalence; deterministic materializer
  compatibility; per-revision route health; and safe revision withdrawal and
  route retirement. The superseded route-level-contract wording in sections
  3.2, 3.3, 3.9, 3.15, 7, 8, 9, 10, 11 and 12 is reconciled. Its
  decision-register ADR-0012 row, one new changelog entry and candidate-stage
  report described the corrected version as `PROPOSED`; the 2026-09-30 and
  2026-10-05 CTO approvals remain historical evidence bound to the earlier
  exact versions. Control independently inspected the candidate, accepted it as
  conforming together with the five interpretations recorded in section 5
  (items 16, 19, 20, 22 and 25), and the CTO approved its exact content - head
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104`, ADR-0012 blob
  `cd897d576522a741c3dfa82f8de2f8d80d4288f6` - on 2026-10-06 in a durable
  record on #958.
- **Amendment-6/7 approval-state stage (historical approval-state head
  `609a01427847233b7280557baa61825ac2079865`).** Authorized by #958 comment
  `6022486352`. One ordinary single-parent commit directly on the
  content-approved candidate recorded that approval as durable decision state,
  changing only the four authorized paths: ADR-0012 `PROPOSED -> APPROVED` with
  `Approved by: Javi, CTO` and `Approval date: 2026-10-06` and the leading
  section-12 approval paragraphs; the ADR-0012 register row; the
  `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` changelog entry; and the
  report. No architecture content changed (section 9, "Amendment-6/7
  approval-state validation"). That authorization also contained an erroneous
  instruction requiring the report to copy the GitHub identifier of the CTO
  content-approval record into repository source, which the
  repository-authoritative Route-B rule in
  `docs/engineering/decisions/README.md` prohibits; the approval-state report
  followed that instruction.
- **Report-only control-record recovery (historical recovery head
  `22bd94dc959f11fe49c8bef8cd2a836140aadff7`).** Authorized by #958 comment
  `6022850916` after control adjudication on #958 found the approval-state
  architecture conforming and that one control-record defect nonconforming.
  Repository-authoritative Route-B doctrine takes precedence over the erroneous
  task instruction, so control required that bounded recovery. One ordinary
  single-parent commit directly on the approval-state head changed only this
  report: it removed the copied content-approval identifier, recorded the
  content approval through owning issue #958, approval date 2026-10-06,
  candidate head `b596536e7db0d6b8ace0a42fdd26bff42cabf104` and candidate
  ADR-0012 blob `cd897d576522a741c3dfa82f8de2f8d80d4288f6`, and restored
  conformity with Route-B doctrine. No ADR-0012 architecture, approval state,
  approver, approval date, decision-register state or changelog state changed;
  ADR-0012 remained blob `a7d95ca918f418d596432071c8ba78a33c2630a9`. It was
  not another candidate, not another approval-state commit and not an ADR
  amendment, and that stage authorized no PR-state or PR-metadata change.
- **Final-head review of the recovery head and report-only transient-state
  correction (historical correction head
  `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`).** Authorized by #958
  comment `6036961945`. Control separately marked PR #960 Ready at the
  recovery head, which triggered the configured final-head automated review of
  `22bd94dc959f11fe49c8bef8cd2a836140aadff7`, and returned PR #960 to Draft
  before any correction source was staged. That review raised two findings,
  both adjudicated by control on #958 (section 14). The first asserted that
  the recovery head's lineage matched a cited object whose sole parent was the
  refreshed `feature/worker` base and which changed all six PR paths; that
  object is not the PR #960 head, and its premise did not match the
  authoritative GitHub PR head/parent graph, under which the recovery head's
  single parent is the approval-state head
  `609a01427847233b7280557baa61825ac2079865` and its exact delta from that
  parent is this report alone. Control rejected it as an invalid premise and
  non-blocking; no source or lineage correction was required and the cited
  object is not adopted here. The second identified that the committed recovery
  report treated the current number and resolved/unresolved state of PR #960
  review threads as durable repository truth, contrary to
  `docs/engineering/AGENTS.md`, under which committed files record durable
  repository state while GitHub records transient workflow state. Control
  accepted it as a valid blocking control-record defect. One ordinary
  single-parent commit directly on the recovery head changed only this report:
  stage-time review-thread inspections were rephrased as historical stage
  evidence, the current-state assertion in section 14 was replaced by durable
  wording that points to PR #960 for live thread state, the final-head review
  outcome was recorded substantively without its review identifiers, and that
  correction became the then-current stage. No ADR-0012 architecture,
  approval state, approver, approval date, decision-register state or
  changelog state changed; ADR-0012 remained blob
  `a7d95ca918f418d596432071c8ba78a33c2630a9` at that head. It was not
  another candidate, not another approval-state commit and not an ADR
  amendment, and that stage authorized no PR-state or PR-metadata change.
- **Final-head review of the transient-state correction head, factual
  clarification and durable-report correction (historical clarification head
  `eb3129f13a9c858b3c012ee182315801f0b8563a`).** Authorized by #958 comment
  `6041301223` after the CTO classification and adjudication record #958
  comment `6041284313`. Control
  separately marked PR #960 Ready at the transient-state correction head,
  which triggered the configured final-head automated review of
  `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`, and returned PR #960 to Draft
  before any correction source was staged. That review raised two findings,
  both accepted by control as valid and blocking (section 14). The first found
  that ADR-0012 stated unconditionally, in section 3.2, section 3.5 and
  invariant 8, that exactly one durable disposition exists for each
  `(event_id, route_key)`, which is broader than the eligibility, backlog,
  in-epoch mismatch, historical-enrollment and reconciliation/divergence
  semantics the same ADR already selects. The CTO classified the required
  change, under the Amendments rules of
  `docs/engineering/decisions/README.md`, as a `FACTUAL CLARIFICATION` of the
  exact prior approved ADR blob `a7d95ca918f418d596432071c8ba78a33c2630a9`:
  the cardinality rule becomes at most one durable disposition per
  `(event_id, route_key)`, an eligible route obligation remains owed until it
  receives exactly one, and every existing no-disposition case and duplicate
  prevention rule is unchanged. The second found that the report still
  recorded which live PR #960 lifecycle gates were pending. One ordinary
  single-parent commit directly on the transient-state correction head changed
  only ADR-0012 and this report: the three unconditional cardinality
  sentences were replaced by the at-most-one rule with the eventual-completion
  clause (section 4; section 9, "Factual-clarification validation"), and the
  report's current-gate statements were replaced by the durable lifecycle
  model under which GitHub, through #958 and PR #960, is authoritative for
  the current gate. ADR-0012 kept `Status: APPROVED`, `Approved by: Javi,
  CTO` and `Approval date: 2026-10-06` (blob
  `2db08058552ce901548ef3f71af4a294f5af7e61`); under the factual-clarification
  rule the existing architecture approval remained applicable within the
  classified scope. No decision-register or changelog byte changed. It was not
  a Specification Amendment, not another Route-B candidate or approval-state
  commit and not a material architectural correction, and that stage
  authorized no PR-state or PR-metadata change.
- **Final-head review of the factual-clarification head, Specification
  Amendment 8 and the Amendment-8 Route-B candidate (historical
  content-approved candidate `42ae90a5932cf24ce90679196888cc3d83d88db8`).**
  Authorized by #958 comment `6044220221` after the CTO
  approved Specification Amendment 8 (#958 comment `6042399237`) and a fresh
  independent CRITICAL specification review of the base specification and
  Amendments 1-8 returned `APPROVED` (#958 comment `6044203556`). Control
  separately marked PR #960 Ready at the clarification head, which triggered
  the configured final-head automated review of
  `eb3129f13a9c858b3c012ee182315801f0b8563a`, and PR #960 had been returned to
  Draft before any candidate source was staged. That review raised one new
  finding, recorded by control as `R-EB31-01` (HIGH, valid and blocking) and
  classified as a `MATERIAL ARCHITECTURAL CORRECTION`: ADR-0012 required that
  a withdrawn revision not be selected for a new epoch or a new historical
  enrollment, and serialized epoch opening, closing and retirement per
  `route_key`, but did not require revision withdrawal to share a database
  linearization boundary with the operations that newly select a revision. An
  activation or enrollment could therefore read a revision as available, a
  withdrawal could commit, and the activation or enrollment could then commit
  a new epoch, reference or enrollment from its stale availability
  observation; the reverse order was likewise undefined. Amendment 8 selects
  one atomic availability/selection order: for one route, withdrawal and
  every operation that creates a new durable selection of or reference to a
  revision share one database-atomic serialization or constraint boundary;
  both commit orders are defined; a losing transaction re-reads durable state
  before any retry; an application-level check-then-insert is insufficient;
  the mechanism remains an implementation-specification detail that must
  compose with the existing route lifecycle serialization; and withdrawal
  semantics are otherwise unchanged. The independent review confirmed the
  finding, the classification and the sufficiency of the nine-case
  concurrency validation matrix, and recorded five non-blocking
  candidate-inspection observations. The candidate is one ordinary
  single-parent commit directly on the clarification head that changed exactly
  the four authorized correction paths: ADR-0012 returned to
  `Status: PROPOSED` with no current approver or approval date and implements
  Amendment 8 (section 4; section 5, decisions 49-60); the decision-register
  ADR-0012 row, one new changelog entry and the candidate-stage report
  described the corrected version as `PROPOSED`; and the 2026-09-30,
  2026-10-05 and 2026-10-06 CTO approvals remain historical evidence bound to
  the earlier exact versions. Control independently inspected the published
  candidate against the actual GitHub source, found no blocking
  exact-candidate finding, and the CTO approved its exact content - head
  `42ae90a5932cf24ce90679196888cc3d83d88db8`, ADR-0012 blob
  `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0` - on 2026-10-07 in a durable
  record on #958. That stage authorized no PR-state or PR-metadata change.
- **Amendment-8 approval-state stage (historical approval-state head
  `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`).** Authorized by #958 comment
  `6045126538`. One ordinary single-parent commit directly on the
  content-approved candidate recorded that approval as durable decision state,
  changing only the four authorized
  paths: ADR-0012 `PROPOSED -> APPROVED` with `Approved by: Javi, CTO` and
  `Approval date: 2026-10-07` and the leading section-12 approval paragraphs,
  which identify the approved content as the content represented by candidate
  ADR-0012 blob `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`; the ADR-0012
  register row; the `THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION`
  changelog entry; and this report. No architecture content changed (section
  9, "Amendment-8 approval-state validation"). The content approval is
  recorded here through owning issue #958, approval date 2026-10-07, the
  candidate head and the candidate ADR-0012 blob; its GitHub comment
  identifier is not copied into repository source, as the
  repository-authoritative Route-B rule in
  `docs/engineering/decisions/README.md` requires and as the control
  adjudication on #958 of the earlier Amendment-6/7 approval-state report
  established for this task (section 5, decisions 36-37 and 64). PR #960 was
  already a draft; that stage authorized no PR-state or PR-metadata change.
- **Exact-final-blob approval, final-head review of the Amendment-8
  approval-state head, Specification Amendment 9 and the Amendment-9 Route-B
  candidate (historical content-approved candidate
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2`).** Authorized by #958
  comment `6046895895` after the CTO approved Specification Amendment 9 (#958
  comment `6046180884`) and a fresh independent CRITICAL specification review
  of the base specification and Amendments 1-9 returned `APPROVED` (#958
  comment `6046871411`). After the approval-state head was published, control
  independently inspected it and the CTO recorded exact-final-blob approval of
  ADR-0012 blob `85937a4e1ff9a27d8f5b36ca82651821711472b2` at head
  `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` on 2026-10-07; control then
  separately marked PR #960 Ready, which triggered the configured final-head
  automated review of that head, and PR #960 had been returned to Draft before
  any candidate source was staged. That review raised one new finding,
  recorded by control as `R-E258-01` (HIGH, valid and blocking) and classified
  as a `MATERIAL ARCHITECTURAL CORRECTION`: ADR-0012 allowed several
  materialized route dispositions to point to one not-yet-started job under an
  explicit coalescing rule and allowed an idempotency conflict to attach a
  route to such a job, and it required database-enforced worker claims, but it
  did not require the transaction that attaches a new disposition to an
  existing coalesced job to share one database-atomic order with the
  transaction that first claims/starts that job. A router could therefore
  observe the job as not started, a worker could claim/start it and commit the
  first execution-owned transition, and the router could then commit a
  disposition recording the event as represented by a job whose inputs may
  already have been fixed or whose effect may already have completed without
  that trigger. Amendment 9 selects one coalescing-open/coalescing-sealed
  boundary per logical job, sealed permanently by the first successful
  claim/start; one database-atomic order between attachment and that
  transition, with the attachment write itself proving the job is still open;
  both commit orders; the aborted-claim rule; mechanism-neutral input/effect
  completeness; the same boundary for every attachment path the ADR already
  permits; and composition with the existing claim, idempotency, disposition
  and target-concurrency rules, with the seal earlier than and distinct from
  `EFFECT_STARTED`. The independent review confirmed the finding, the
  classification, the seal boundary and the sufficiency of the nine-case
  concurrency matrix, and recorded six non-blocking candidate-inspection
  observations. The candidate is one ordinary single-parent commit directly
  on the Amendment-8 approval-state head that changed exactly the four
  authorized correction paths: ADR-0012 returned to `Status: PROPOSED` with no
  current approver or approval date and implements Amendment 9 (section 4;
  section 5, decisions 69-80); the decision-register ADR-0012 row, one new
  changelog entry and the candidate-stage report described the corrected
  version as `PROPOSED`; and the 2026-09-30, 2026-10-05, 2026-10-06 and
  2026-10-07 CTO approvals remain historical evidence bound to the earlier
  exact versions. Control independently inspected the published candidate
  against the actual GitHub source, found no blocking exact-candidate
  finding, and Javi, CTO approved its exact content - head
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2`, ADR-0012 blob
  `0fc33df9aca61a2c82af452426f5574bfd3493cf` - on 2026-10-08 in a durable
  record on #958. That stage authorized no PR-state or PR-metadata change.
- **Amendment-9 approval-state stage (the commit that contains this
  report).** Authorized by #958 comment `6065879000`. One ordinary
  single-parent commit directly on the content-approved candidate records
  that approval as durable decision state, changing only the four authorized
  paths: ADR-0012 `PROPOSED -> APPROVED` with `Approved by: Javi, CTO` and
  `Approval date: 2026-10-08` and the leading section-12 approval paragraphs,
  which identify the approved content as the content represented by candidate
  ADR-0012 blob `0fc33df9aca61a2c82af452426f5574bfd3493cf`; the ADR-0012
  register row and the register's `Last updated` header; the
  `THOTH-ASYNC-01-ADR-01-COALESCING-SEAL-CORRECTION` changelog entry; and this
  report. No architecture content changes (section 9, "Amendment-9
  approval-state validation"). The content approval is recorded here through
  owning issue #958, approver Javi, CTO, approval date 2026-10-08, the
  candidate head and the candidate ADR-0012 blob; its GitHub comment
  identifier is not copied into repository source, as the
  repository-authoritative Route-B rule in
  `docs/engineering/decisions/README.md` and the approval-state authorization
  itself require (section 5, decision 84). PR #960 was already a draft; this
  stage authorizes no PR-state or PR-metadata change.

Out-of-scope changes made: NONE

### 2.1 Classification, route and eligibility

```text
classification:
MATERIAL ARCHITECTURAL CORRECTION (base specification section 3)

selected approval route:
Route B - PR-first staged approval (Amendment 2 B2, ratified by Amendment 3)

ADR-0012 change in the commit that contains this report:
Route-B approval-state recording (PROPOSED -> APPROVED; Javi, CTO; 2026-10-08)
of the content-approved Amendment-9 candidate blob 0fc33df9aca61a2c82af452426f5574bfd3493cf;
no architecture change

historical Amendment-9 candidate at 8699af5635b8f18691ceaeccd3b24707a71ef4f2:
MATERIAL ARCHITECTURAL CORRECTION of prior approved blob 85937a4e1ff9a27d8f5b36ca82651821711472b2
(Specification Amendment 9; Route-B candidate stage; candidate status PROPOSED)

historical Amendment-8 approval-state at e2588f7b7e63a5c71968d87a3409f0ff48635e6d:
Route-B approval-state recording (PROPOSED -> APPROVED; Javi, CTO; 2026-10-07)
of the content-approved Amendment-8 candidate blob cbb31a53f6f4ed6b96faafb79b273b55664f3ae0;
no architecture change; CTO exact-final-blob approval of 85937a4e1ff9a27d8f5b36ca82651821711472b2
recorded on #958 on 2026-10-07

historical Amendment-8 candidate at 42ae90a5932cf24ce90679196888cc3d83d88db8:
MATERIAL ARCHITECTURAL CORRECTION of prior approved blob 2db08058552ce901548ef3f71af4a294f5af7e61
(Specification Amendment 8; Route-B candidate stage; candidate status PROPOSED)

historical ADR-0012 change at eb3129f13a9c858b3c012ee182315801f0b8563a:
FACTUAL CLARIFICATION of prior approved blob a7d95ca918f418d596432071c8ba78a33c2630a9
(docs/engineering/decisions/README.md, Amendments; CTO classification on #958;
no Specification Amendment and no new Route-B cycle)
```

Eligibility for the material pre-authority correction is keyed to the ADR
number, not to one pathname (Amendment 1 A1;
`docs/engineering/decisions/README.md`). At activation-epoch candidate
validation time `b44303c214498baf76c9d9a0a7cab374377f9cbe` was simultaneously
`develop` and the refreshed `feature/worker`, and `master` was
`4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff`. Those refs were unchanged at the
Amendment-6/7 candidate stage, at the Amendment-8 candidate stage, at the
Amendment-8 approval-state stage, at the Amendment-9 candidate stage and at
the Amendment-9 approval-state stage, and the same literal commands were run
again at each stage against both commits with identical results:

```text
git log --format=%h <ref> -- ':(glob)**/*ADR-0012*'
result for both refs: no commits

git log --format=%h -G'^# ADR-0012' <ref>
result for both refs: no commits

git log --format=%h -G'^\| `ADR-0012`' <ref> -- docs/engineering/decisions/decision-register.md
result for both refs: no commits

git grep -l ADR-0012 <ref>
result for b44303c214498baf76c9d9a0a7cab374377f9cbe:
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-CTRL-01-implementation-report.md
docs/engineering/decisions/ADR-0013-programme-integration-decision-reliance.md
result for 4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff: no files
```

No path matching `*ADR-0012*`, no `# ADR-0012` decision heading and no
decision-register ADR-0012 row exists anywhere in the current history of
`develop`, `master` or `feature/worker`. The only ADR-0012 decision path,
`docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`,
and every decision-register blob carrying an ADR-0012 row are reachable only from
`feature/async/adr-0012`. The two `develop` files that mention the identifier are
ADR-0013 and the THOTH-ASYNC-01-CTRL-01 control report; neither makes ADR-0012
approved or repository-authoritative. No evidence was found of any approved
ADR-0012 version ever being reachable from `develop`. Git objects removed from
every reachable ref cannot be proven from current history; no evidence indicates
that such a removed repository-authoritative ADR-0012 version existed.

### 2.2 Stale controls

Under the repository material-correction rules, the material correction makes
stale, for every corrected version:

- the CTO architecture-content approval bound to
  `631e28d1f05495f24ce88369d4557f1033f970b3` / ADR blob
  `9eb21363aa2f60a86db1a2293d697545ec3809e9`, and the approval-state record
  carried by ADR blob `cffee08b0c4326823f36387e6cec2abe49594273`;
- every independent review bound to an earlier source head, including the
  approval-state review of `bd5223deee44b465dd860ff96b75d81ad41cbf2d`;
- the earlier #957 `ADR-0013` programme-integration pin to
  `bd5223deee44b465dd860ff96b75d81ad41cbf2d` / ADR blob
  `cffee08b0c4326823f36387e6cec2abe49594273`;
- any merge authorization bound to an earlier PR #960 head;
- the earlier branch-local decision-register assertion that ADR-0012 is
  `APPROVED`.

Those records remain historical evidence for their exact versions. They are
not approval of the corrected version. Under Route B the activation-epoch
corrected version received CTO content approval on 2026-10-05, recorded as
`APPROVED` by the historical approval-state commit
`c046e6bf6510f24ee1231f86293a3d499bceed7e` in ADR-0012 blob
`4848470a252076268a9fac9da5c40f0b4ebff474`, and that unchanged blob later
received a fresh head-bound exact-final-blob confirmation at the recovery head
`9db91af7bf3d9f969556c819187420fc31d25c10`.

The Amendment-6/7 candidate creates a new exact ADR-0012 version and so makes
stale, for that version, every one of those activation-epoch-version records
as well:

- the CTO content approval of candidate
  `95769537cdc9871e42f9f3e5443604a96beb74da` / ADR blob
  `a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c`;
- the 2026-10-05 approval state carried by ADR blob
  `4848470a252076268a9fac9da5c40f0b4ebff474`, its exact-final-blob approval
  and its head-bound confirmation at the recovery head;
- every automated or independent review bound to an earlier source head,
  including the final-head automated review of the recovery head;
- the recovery-head decision-register row and the
  `THOTH-ASYNC-01-ADR-01-CORRECTION` changelog entry that record the
  activation-epoch version as `APPROVED`; the changelog entry is preserved
  and remains true only of that historical version, while the register row now
  records the candidate as `PROPOSED`.

They remain historical evidence bound to their exact versions and heads. The
new version received its own independent control inspection and CTO exact
candidate-content approval on 2026-10-06, the historical approval-state
commit `609a01427847233b7280557baa61825ac2079865` recorded it, and neither
the historical report-only control-record recovery
`22bd94dc959f11fe49c8bef8cd2a836140aadff7` nor the historical report-only
transient-state correction `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` changed
ADR-0012; both left the approved blob
`a7d95ca918f418d596432071c8ba78a33c2630a9`. The historical factual
clarification `eb3129f13a9c858b3c012ee182315801f0b8563a` changed that blob to
`2db08058552ce901548ef3f71af4a294f5af7e61` within the CTO-classified
clarification scope only; under the factual-clarification rule the selected
architecture and its approval were unchanged, while any source-head-bound
review or merge authorization and any ADR-0013 exact-version programme pin
bound to an earlier blob or head became stale for the clarified version.

The Amendment-8 candidate creates a new exact ADR-0012 version and so makes
stale, for that version, every record bound to the route-contract-revision
version as well:

- the CTO content approval of candidate
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104` / ADR blob
  `cd897d576522a741c3dfa82f8de2f8d80d4288f6`;
- the 2026-10-06 approval state carried by ADR blobs
  `a7d95ca918f418d596432071c8ba78a33c2630a9` and
  `2db08058552ce901548ef3f71af4a294f5af7e61`, and any exact-final-blob
  approval or head-bound confirmation of either;
- every automated or independent review bound to an earlier source head,
  including the final-head automated review of the clarification head;
- the clarification-head decision-register row and the
  `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` changelog entry that
  record the route-contract-revision version as `APPROVED`; the changelog
  entry is preserved and remains true only of that historical version, while
  the register row recorded the Amendment-8 candidate as `PROPOSED` and then
  the approved version.

They remain historical evidence bound to their exact versions and heads. The
Amendment-8 candidate version received its own independent control inspection
and CTO exact candidate-content approval on 2026-10-07, the historical
approval-state commit `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` recorded it in
ADR-0012 blob `85937a4e1ff9a27d8f5b36ca82651821711472b2`, and that blob received
CTO exact-final-blob approval at that head on 2026-10-07.

The Amendment-9 candidate creates a new exact ADR-0012 version and so makes
stale, for that version, every record bound to the withdrawal/selection
ordering version as well:

- the CTO content approval of candidate
  `42ae90a5932cf24ce90679196888cc3d83d88db8` / ADR blob
  `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`;
- the 2026-10-07 approval state carried by ADR blob
  `85937a4e1ff9a27d8f5b36ca82651821711472b2` and its exact-final-blob approval at
  head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`;
- every automated or independent review bound to an earlier source head,
  including the final-head automated review of the approval-state head;
- the approval-state-head decision-register row and the
  `THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION` changelog entry that
  record the withdrawal/selection ordering version as `APPROVED`; the
  changelog entry is preserved and remains true only of that historical
  version, while the register row recorded the Amendment-9 candidate as
  `PROPOSED` and now records the approved version.

They remain historical evidence bound to their exact versions and heads. The
Amendment-9 candidate version received its own independent control inspection
and CTO exact candidate-content approval on 2026-10-08, and the approval-state
commit that contains this report records it. For this version,
exact-final-blob approval, independent exact-head review, CTO merge
authorization, guarded merge, programme-local reliance and later migration,
deployment, activation and observation are distinct controlled lifecycle
categories; GitHub, through #958 and PR #960, is authoritative for which of
them is pending, satisfied or blocked for any head.

## 3. Commits

Amendment-9 Route-B approval-state commit (the commit that contains this
report):

- exactly one ordinary single-parent additive commit on
  `feature/async/adr-0012` -
  `THOTH-ASYNC-01-ADR-01: record coalescing-seal correction approval`
  - direct parent: the content-approved Amendment-9 candidate
    `8699af5635b8f18691ceaeccd3b24707a71ef4f2`
  - changes only the four authorized paths, and only to record the approval
    state (section 4); the approval-state ADR-0012 blob is
    `cbe8322d95fe8d5e9911d421a5f10f1ae4afd55c`, the approval-state
    decision-register blob `c618d9d2eec622660ee7c1868111f747efa27629` and the
    approval-state changelog blob `2731521f363189063b9ab09068df7852889121a2`

Its SHA, its tree and this report's final blob are deliberately not embedded
here because this report is part of that tree; the approval-state ADR-0012,
decision-register and changelog blobs do not depend on this report and are
recorded above; every other file keeps its candidate blob (section 9). The
commit identities are verified against the remote task-branch head after push
and recorded in the post-publication handoff; once published, the PR #960
head and head tree are GitHub-owned lifecycle evidence under `ADR-0005`. The
authorized topology is a non-force fast-forward of the task branch from the
content-approved candidate to the approval-state commit; no merge commit,
rebase, squash, amend, force update or second approval-state commit is
authorized, and no further source commit exists implicitly after it.

Amendment-9 Route-B candidate commit (content-approved; historical):

- `8699af5635b8f18691ceaeccd3b24707a71ef4f2` -
  `THOTH-ASYNC-01-ADR-01: propose coalescing-seal ordering correction`
  - ordinary single-parent additive commit; direct parent: the historical
    Amendment-8 approval-state head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`
  - tree `ebff7219611e7855c62856019fc051fcf41cb2b5`
  - changed only the four authorized correction paths (section 4); the
    candidate ADR-0012 blob is `0fc33df9aca61a2c82af452426f5574bfd3493cf`
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    Amendment-8 approval-state head; the remote head, tree, parent and
    ADR-0012, decision-register, changelog and report blobs matched the
    locally validated object

Amendment-8 Route-B approval-state commit (historical):

- `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` -
  `THOTH-ASYNC-01-ADR-01: record withdrawal-selection correction approval`
  - ordinary single-parent additive commit; direct parent: the content-approved
    Amendment-8 candidate `42ae90a5932cf24ce90679196888cc3d83d88db8`
  - tree `bec3637f333a12ed1895b2aae6f1078510067f5e`
  - changed only the four authorized paths, and only to record the approval
    state (section 4); the approval-state ADR-0012 blob is
    `85937a4e1ff9a27d8f5b36ca82651821711472b2`, which received CTO
    exact-final-blob approval at that head on 2026-10-07
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    candidate; the remote head, tree, parent, ADR-0012, decision-register,
    changelog and report blobs and the ADR approval state matched the locally
    validated object

Amendment-8 Route-B candidate commit (content-approved; historical):

- `42ae90a5932cf24ce90679196888cc3d83d88db8` -
  `THOTH-ASYNC-01-ADR-01: propose withdrawal-selection ordering correction`
  - ordinary single-parent additive commit; direct parent: the historical
    factual-clarification head `eb3129f13a9c858b3c012ee182315801f0b8563a`
  - tree `45d8a0bdb05ca95b61c01cdc83dcecf3a3f60e98`
  - changed only the four authorized correction paths (section 4); the
    candidate ADR-0012 blob is `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    clarification head; the remote head, tree, parent and ADR-0012,
    decision-register, changelog and report blobs matched the locally
    validated object

Factual-clarification commit (historical):

- `eb3129f13a9c858b3c012ee182315801f0b8563a` -
  `THOTH-ASYNC-01-ADR-01: clarify route-disposition cardinality`
  - ordinary single-parent additive commit; direct parent: the historical
    report-only transient-state correction head
    `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`
  - tree `5244d6ac76578a6f78cde79671762da5b09f2fc1`
  - changed only ADR-0012 and this report (section 4); the clarified ADR-0012
    blob is `2db08058552ce901548ef3f71af4a294f5af7e61`
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    transient-state correction head; the remote head, tree, parent and
    ADR-0012 blob matched the locally validated object

Report-only transient-state correction commit (historical):

- `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` -
  `THOTH-ASYNC-01-ADR-01: make report review-thread record durable`
  - ordinary single-parent additive commit; direct parent: the historical
    report-only control-record recovery head
    `22bd94dc959f11fe49c8bef8cd2a836140aadff7`
  - tree `862a8e32207f1e0f61cab532b08b65034979f5ac`
  - changed only this report (section 4)
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    recovery head; the remote head, tree, parent and ADR-0012 blob matched the
    locally validated object

Report-only control-record recovery commit (historical):

- `22bd94dc959f11fe49c8bef8cd2a836140aadff7` -
  `THOTH-ASYNC-01-ADR-01: correct approval-state control record`
  - ordinary single-parent additive commit; direct parent: the historical
    Amendment-6/7 approval-state head
    `609a01427847233b7280557baa61825ac2079865`
  - tree `7af3e0b8007eb6686e94043f57db63acc4edb4ab`
  - changed only this report (section 4)
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    approval-state head; the remote head, tree, parent and ADR-0012 blob
    matched the locally validated object

Amendment-6/7 Route-B approval-state commit (historical):

- `609a01427847233b7280557baa61825ac2079865` -
  `THOTH-ASYNC-01-ADR-01: record route-contract correction approval`
  - ordinary single-parent additive commit; direct parent: the content-approved
    Amendment-6/7 candidate `b596536e7db0d6b8ace0a42fdd26bff42cabf104`
  - tree `abc024bd16ebf8198475ef196c1f829228760abe`
  - changed only the four authorized paths, and only to record the approval
    state (section 4)
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    candidate; the remote head, tree, parent and ADR-0012 blob matched the
    locally validated object

Amendment-6/7 Route-B candidate commit (content-approved; historical):

- `b596536e7db0d6b8ace0a42fdd26bff42cabf104` -
  `THOTH-ASYNC-01-ADR-01: propose route-contract revision correction`
  - ordinary single-parent additive commit; direct parent: the historical
    Amendment-4/5 recovery head `9db91af7bf3d9f969556c819187420fc31d25c10`
  - tree `d4d7dd359c3e20d37fcb5f664721b87cd2e480a8`
  - changed only the four authorized correction paths (section 4)
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    recovery head; the remote head, tree, parent and ADR-0012 blob matched the
    locally validated object

Amendment-4/5 recovery commit (historical):

- `9db91af7bf3d9f969556c819187420fc31d25c10` -
  `THOTH-ASYNC-01-ADR-01: reconcile final-head review metadata`
  - ordinary single-parent additive commit; direct parent: the historical
    Route-B approval-state head `c046e6bf6510f24ee1231f86293a3d499bceed7e`
  - tree `fd9a300265c310388652ff94a4ef9f95224ce018`
  - changed only the two authorized recovery paths (section 4): the
    decision-register `Last updated` header line and this report
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    approval-state head; the remote head, tree and parent matched the locally
    validated object

Route-B approval-state commit (historical):

- `c046e6bf6510f24ee1231f86293a3d499bceed7e` -
  `THOTH-ASYNC-01-ADR-01: record ADR-0012 approval state`
  - ordinary single-parent commit; direct parent: the content-approved
    candidate `95769537cdc9871e42f9f3e5443604a96beb74da`
  - tree `801e12ffc010470928da45919ede8d7239a491ce`
  - changed only the four authorized correction paths, and only their approval
    state (section 4)
  - published by a non-force fast-forward of `feature/async/adr-0012` from the
    candidate; the remote head, tree and parent matched the locally validated
    object

Route-B activation-epoch candidate commit (content-approved; historical):

- `95769537cdc9871e42f9f3e5443604a96beb74da` -
  `THOTH-ASYNC-01-ADR-01: propose Route-B activation-epoch correction`
  - genuine two-parent merge commit; parent 1
    `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`, parent 2
    `b44303c214498baf76c9d9a0a7cab374377f9cbe`
  - tree `5084c6951a1b57b449f6045b139aacfbc488e534`: the unmodified merge-tree
    `e15607878e66d2ff12b819dcea13293a46674f13` overlaid only by the four
    correction paths in section 4
  - published by a non-force fast-forward of `feature/async/adr-0012` from
    parent 1 before the approval-state commit was created

Earlier task-branch history from `923545d5c9028bc04c40e38efeb7de674efed3fd`,
first-parent order, newest first:

- `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` - THOTH-ASYNC-01-ADR-01: correct validation provenance
- `227057cae8a4c10f7f7795e3afe68271e56681b9` - THOTH-ASYNC-01-ADR-01: clarify route dispositions
- `7f7e62824eb40308688b46351ffea182e9e52942` - THOTH-ASYNC-01-ADR-01: reconcile programme integration base (two parents: `bd5223deee44b465dd860ff96b75d81ad41cbf2d`, `345a7a04131e7c0539f7518c6ea457fa5cb6a462`)
- `bd5223deee44b465dd860ff96b75d81ad41cbf2d` - THOTH-ASYNC-01: align approved ADR amendment rule
- `c43aa750ea4d971fe4f803cf761868876f0783bd` - THOTH-ASYNC-01: clean approval metadata formatting
- `42d79d4617a73696efd328943c1cc547e807d39d` - THOTH-ASYNC-01: reconcile ADR-0012 approval state
- `631e28d1f05495f24ce88369d4557f1033f970b3` - THOTH-ASYNC-01: bind trusted promotion to validated bytes
- `d25bb1ecd1665cc4e635b16dbd5fd09582a5889b` - THOTH-ASYNC-01: address round-4 architecture review
- `ced4cb19224d7acc1824863a85d83b2387617449` - THOTH-ASYNC-01: record round-4 architecture corrections
- `ca31d792aa3f5212a9715278464e9d5ed928dd0c` - THOTH-ASYNC-01: update round-4 proposal summary
- `88fbe6f88a864fceb713b7055acf77860c2d6e43` - THOTH-ASYNC-01: update round-4 ADR proposal record
- `359c3b7c33cc1a6794dca48b46818da039c1d627` - THOTH-ASYNC-01: address round-3 review findings
- `2d5eb1ae1e25bc222c4ca94a4f6dc5e81ecceba8` - THOTH-ASYNC-01: finalize round-3 correction report
- `6af54e45478f3ecb9422c395a4f7c070700de240` - THOTH-ASYNC-01: make route activation concurrency-safe
- `482e2df9b719a230488936079edaf334df02a3ee` - THOTH-ASYNC-01: tighten round-3 trust and ADR boundaries
- `907ea34f8dbcf88ea1c822492fe28aae3b5de993` - THOTH-ASYNC-01: record round-3 architecture corrections
- `3402308e8ed75953bf262dd436788cefecc94633` - THOTH-ASYNC-01: update round-3 proposal summary
- `5e9af9fee5a8f1b499ee74ba78091ed9f398fb8e` - THOTH-ASYNC-01: update ADR-0012 round-3 proposal record
- `28658c6672181a41c03e7b0d974ec4c0ebe41ac0` - THOTH-ASYNC-01: address round-2 architecture findings
- `9d329e063b2731b691c62a412555b470f2a9fe39` - THOTH-ASYNC-01: record ADR-0012 correction round
- `f06ba7ffdf351e8f0829a6eb0e55dd8256bccc2b` - THOTH-ASYNC-01: update ADR-0012 proposal summary
- `aa08cd38531f17ce98c1a44fdcfe0b1b258872e1` - THOTH-ASYNC-01: reconcile ADR-0012 proposal record
- `4808d42364a621fc5a86e47cea65d86354df8af8` - THOTH-ASYNC-01: address ADR-0012 review findings
- `a69921682cd02e060110304f8f9868d0c39da65e` - THOTH-ASYNC-01: add ADR-0012 authoring report
- `fb8c1d4e9373153d367e2336b6a4dc349f165fc2` - THOTH-ASYNC-01: document ADR-0012 proposal
- `30efbc63945c474d924fb233adf4ce4f4aaefb20` - THOTH-ASYNC-01: register ADR-0012 proposal
- `83f3ec29db091724b6d084385050bc7cb239d5a6` - THOTH-ASYNC-01: propose shared async architecture

## 4. Files changed

Amendment-9 Route-B approval-state write budget for the commit that contains
this report (authorization #958 comment `6065879000`), identical to the
Amendment-9 candidate write budget (authorization #958 comment `6046895895`;
the Amendment 6 G10 / Amendment 7 H12 budget, unchanged by Amendments 8-9);
exactly four existing paths, no new file, no deletion, and every other path
byte-identical to the content-approved candidate
`8699af5635b8f18691ceaeccd3b24707a71ef4f2`:

- `CHANGELOG.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/decisions/decision-register.md`

Historical Amendment-9 Route-B candidate write budget (authorization #958
comment `6046895895`; the Amendment 6 G10 / Amendment 7 H12 budget, unchanged
by Amendments 8-9); exactly four existing paths, no new file, no deletion, and
every other path byte-identical to the historical Amendment-8 approval-state
head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`:

- `CHANGELOG.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/decisions/decision-register.md`

Historical Amendment-8 Route-B approval-state write budget (authorization #958
comment `6045126538`), identical to the Amendment-8 candidate write budget
(authorization #958 comment `6044220221`; the Amendment 6 G10 / Amendment 7
H12 budget, unchanged by Amendment 8); exactly four existing paths, no new
file, no deletion, and every other path byte-identical to the content-approved
candidate `42ae90a5932cf24ce90679196888cc3d83d88db8`:

- `CHANGELOG.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/decisions/decision-register.md`

Historical factual-clarification write budget (authorization #958 comment
`6041301223`); exactly two existing paths, no new file, no deletion, and every
other path byte-identical to the transient-state correction head
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`:

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`

Historical report-only transient-state correction write budget
(authorization #958 comment `6036961945`): exactly the report path, no new
file, no deletion, and every other path byte-identical to the recovery head
`22bd94dc959f11fe49c8bef8cd2a836140aadff7`.

Historical report-only control-record recovery write budget (authorization
#958 comment `6022850916`): the same single existing path, no new file, no
deletion, and every other path byte-identical to the approval-state head.

Historical Amendment-6/7 approval-state write budget (authorization #958
comment `6022486352`), identical to the Amendment-6/7 candidate write budget
(Amendment 6 G10, Amendment 7 H12, authorization #958 comment `6021574322`);
exactly four existing paths, no new file, no deletion:

- `CHANGELOG.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/decisions/decision-register.md`

Historical Route-B activation-epoch material-correction and approval-state
write budget (base specification section 5, unchanged by Amendments 1-3; the
same four correction paths).

Historical Amendment-4/5 final-head recovery write budget (Amendment 4 E5,
Amendment 5 F3.4; exactly two paths):

- `docs/engineering/decisions/decision-register.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`

Authorized cumulative PR footprint against refreshed `feature/worker` (exactly
six paths at every stage):

- `CHANGELOG.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
- `docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md`
- `docs/engineering/decisions/ADR-0010-staff-operations-console.md`
- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/decisions/decision-register.md`

Authorized new-file paths: NONE at the approval-state, recovery,
Amendment-6/7 candidate, Amendment-6/7 approval-state, control-record
recovery, transient-state correction, factual-clarification, Amendment-8
candidate, Amendment-8 approval-state, Amendment-9 candidate and Amendment-9
approval-state stages; all four correction paths already existed.

Amendment-9 approval-state changes in the commit that contains this report,
relative to the content-approved candidate
`8699af5635b8f18691ceaeccd3b24707a71ef4f2` (exactly the four authorized paths,
approval state only):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: record the CTO exact candidate-content approval of 2026-10-08.
  - behavioural effect: the header's `Status: PROPOSED` becomes
    `Status: APPROVED` and gains `Approved by: Javi, CTO` and
    `Approval date: 2026-10-08`; section 12's two leading candidate-state
    paragraphs ("Current decision state: **PROPOSED**." and the statement that
    the corrected content has not been approved) are replaced by
    "Current decision state: **APPROVED**.", the same approver and date, and
    a paragraph stating that the CTO approved this exact corrected content on
    2026-10-08 as the architecture content represented by candidate ADR-0012
    blob `0fc33df9aca61a2c82af452426f5574bfd3493cf`, that this version differs
    from that candidate only in its approval-state representation, that the
    approval does not by itself make the ADR repository-authoritative and
    that any ADR-0013 programme-local reliance on this exact version is a
    separate state not effective until its own conditions are satisfied.
    Every other line, including all architecture, the Amendment-9 coalescing
    boundary, the Amendment-8 ordering rule, invariants 41 and 42, the
    validation requirements, the historical-approval paragraph, the authority
    condition and the architecture-versus-implementation separation, is
    byte-identical to the candidate (section 9). Approval-state blob:
    `cbe8322d95fe8d5e9911d421a5f10f1ae4afd55c`.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: record the approval state of the ADR-0012 row.
  - behavioural effect: the ADR-0012 row - `PROPOSED` becomes `APPROVED`,
    `Pending` becomes `Satisfied`, and "its corrected content awaits CTO
    approval" becomes "Javi, CTO approved its exact corrected content on
    2026-10-08"; and the header `Last updated` line advances from
    `2026-10-07` to `2026-10-08`, the date of this row change, as the
    approval-state authorization requires (section 5, decision 85). Every
    other row is unchanged. Approval-state blob:
    `c618d9d2eec622660ee7c1868111f747efa27629`.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: record the approval state of the coalescing-seal correction entry.
  - behavioural effect: the
    `THOTH-ASYNC-01-ADR-01-COALESCING-SEAL-CORRECTION` entry only - "propose"
    becomes "record a CTO-approved", "`PROPOSED`: the corrected content is not
    approved" becomes "`APPROVED`: Javi, CTO approved the exact corrected
    content on 2026-10-08", and "proposal only" becomes "decision only". Its
    statements that the earlier entries and the 2026-09-30, 2026-10-05,
    2026-10-06 and 2026-10-07 approvals apply only to historical exact
    versions are unchanged, and every other entry is byte-identical.
    Approval-state blob: `2731521f363189063b9ab09068df7852889121a2`.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: record the Amendment-9 approval-state stage truthfully.
  - behavioural effect: documentation only. Makes the approval-state commit
    the current stage; records the content-approved candidate, the CTO
    candidate-content approval and the approval-state authorization; records
    the approval-state validation and no-drift proof (section 9); and reframes
    statements that would otherwise describe the candidate as the commit
    containing this report, recording the candidate's identities as
    historical facts. All still-true earlier evidence is preserved.
  - within authorized write budget: YES

Byte-identical to the content-approved candidate through the approval-state
commit: every path other than the four authorized paths, including ADR-0008
`622060ad90eec41c792f110c373efffbb11a4b56`, ADR-0010
`aca2142a3387785e80db908b3a5c1b0afd82ab51` and ADR-0013
`d928f957bdf775d03e99fc73888ec8e2dec5f80b`.

Historical Amendment-9 candidate changes in the content-approved candidate
`8699af5635b8f18691ceaeccd3b24707a71ef4f2`, relative to the Amendment-8
approval-state head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` (exactly the four
authorized paths):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: implement Specification Amendment 9 and record the corrected
    version as `PROPOSED`.
  - behavioural effect: architecture proposal only. The header carried
    `Status: PROPOSED` and no current approver or approval date. Section 3.5
    states that a job is not yet started, and so coalescing-open, only until
    its first successful execution claim/start commits; binds the
    idempotency-conflict attachment to the database-atomic coalescing
    boundary; and gains the unnumbered subsection "Coalescing-open and
    coalescing-sealed jobs": one durable boundary per logical job of a
    coalescing kind, sealed permanently by the first successful committed
    claim/start, the seal belonging to the logical job and never reopened by
    retry, lease reclaim, WAITING/resume, reconciliation, cancellation,
    recovery or later claims, with the persisted encoding left to the
    implementation specification; one database-atomic serialization, lock or
    compare-and-set order between attachment and first claim/start, the
    attachment write itself proving the job is still open and an
    application-level pre-check insufficient, with the SQL, lock or
    constraint left to the implementation specification; both commit orders;
    the aborted-claim rule; mechanism-neutral input/effect completeness
    (`CURRENT_STATE` through current-state/effect-fingerprint semantics, other
    kinds through a retained input set or equivalent, no handler enumeration
    of dispositions); the same boundary for every attachment path the ADR
    already permits, without deciding whether any path may coalesce; and the
    seal's distinction from `EFFECT_STARTED` with the composition list.
    Section 3.15 states that a surviving job accepts attachments only while
    open and represents every pre-seal attachment. Invariant 8 names the
    coalescing-open condition and invariant 42 is appended; section 8 item 1
    and section 9 rollout proof 5 name the boundary; section 11 adds the nine
    Amendment-9 concurrency validation cases, with explicit `CURRENT_STATE`,
    retained-input, idempotency-conflict, multi-instance and crash/restart
    evidence, after the existing coalescing tests. Section 12 records the
    `PROPOSED` state, adds the boundary to the description of this material
    correction and binds the 2026-09-30, 2026-10-05, 2026-10-06 and
    2026-10-07 approvals to the historical earlier versions.
    `(event_id, route_key)` uniqueness, route eligibility, activation epochs,
    route-contract revisions, the Amendment-8 withdrawal/selection ordering,
    idempotency-key identity, which kinds may coalesce, the explicit, durable
    and audited coalescing requirement, `CURRENT_STATE` and `REVISION_BOUND`
    semantics, target-scoped serialization, applied-revision evidence, the
    attempt phases, claim-token fencing, WAITING and RECONCILIATION_REQUIRED,
    retry and reconciliation semantics, worker trust boundaries and the BE-04
    transition are unchanged. Candidate blob:
    `0fc33df9aca61a2c82af452426f5574bfd3493cf`.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: the material correction makes the ADR-0012 row's `APPROVED`
    assertion untrue for the new version.
  - behavioural effect: the ADR-0012 row records `PROPOSED` / `Pending`,
    binds the 2026-09-30, 2026-10-05, 2026-10-06 and 2026-10-07 approvals to
    the historical earlier exact versions and adds one summary of the
    coalescing-open/coalescing-sealed boundary and the attachment-versus-
    first-claim order; its partial-supersession, authority-condition, ADR-0013
    and implementation-authorization sentences are unchanged. The header
    `Last updated: 2026-10-07` already carries the date of this row change and
    is unchanged (section 5, decision 78). No other row changes. Candidate
    blob: `8b5ef6b5b6ca7f6d49ed5066aba5e49575de06be`.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: required changelog entry.
  - behavioural effect: exactly one new
    `THOTH-ASYNC-01-ADR-01-COALESCING-SEAL-CORRECTION` entry at the top of
    `[Unreleased] -> Changed`, describing the corrected version as `PROPOSED`
    and stating that the earlier
    `THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION`,
    `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION`,
    `THOTH-ASYNC-01-ADR-01-CORRECTION`, `THOTH-ASYNC-01-ADR-01-APPROVAL` and
    `THOTH-ASYNC-01-ADR-01-CLARIFICATION` entries and the approvals they
    record apply only to the historical earlier exact versions. No heading is
    added and every existing entry is byte-preserved. Candidate blob:
    `4d0e17ca0b11dea0ed4905ef9776693ee2c87bc4`.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: record the Amendment-9 candidate stage truthfully.
  - behavioural effect: documentation only. Made the candidate the
    then-current stage; recorded the exact-final-blob approval of the
    Amendment-8 approval-state blob, the final-head review of that head,
    `R-E258-01`, Amendment 9, its independent review and the candidate
    authorization as governing evidence; recorded the candidate's validation
    (section 9); and reframed statements that would otherwise describe the
    Amendment-8 approval-state commit as the commit containing the report,
    recording that commit's identities as historical facts. All still-true
    earlier evidence was preserved.
  - within authorized write budget: YES

Byte-identical to the approval-state head through the Amendment-9 candidate:
every path other than the four correction paths, including ADR-0008
`622060ad90eec41c792f110c373efffbb11a4b56`, ADR-0010
`aca2142a3387785e80db908b3a5c1b0afd82ab51` and ADR-0013
`d928f957bdf775d03e99fc73888ec8e2dec5f80b`.

Historical Amendment-8 approval-state changes in the approval-state commit
`e2588f7b7e63a5c71968d87a3409f0ff48635e6d`, relative to the content-approved
candidate
`42ae90a5932cf24ce90679196888cc3d83d88db8` (exactly the four authorized paths,
approval state only):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: record the CTO exact candidate-content approval of 2026-10-07.
  - behavioural effect: the header's `Status: PROPOSED` becomes
    `Status: APPROVED` and gains `Approved by: Javi, CTO` and
    `Approval date: 2026-10-07`; section 12's two leading candidate-state
    paragraphs ("Current decision state: **PROPOSED**." and the statement that
    the corrected content has not been approved) are replaced by
    `This ADR is **APPROVED**.`, the same approver and date, and a paragraph
    stating that the CTO approved this exact corrected content on 2026-10-07
    as the architecture content represented by candidate ADR-0012 blob
    `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`, that this version differs
    from that candidate only in its approval-state representation, that the
    approval does not by itself make the ADR repository-authoritative and
    that any ADR-0013 programme-local reliance on this exact version is a
    separate state not effective until its own conditions are satisfied.
    Every other line, including all architecture, the Amendment-8 ordering
    rule, invariant 41, the validation requirements, the historical-approval
    paragraph, the authority condition and the architecture-versus-
    implementation separation, is byte-identical to the candidate (section
    9). Approval-state blob: `85937a4e1ff9a27d8f5b36ca82651821711472b2`.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: record the approval state of the ADR-0012 row.
  - behavioural effect: the ADR-0012 row only - `PROPOSED` becomes `APPROVED`,
    `Pending` becomes `Satisfied`, and "its corrected content awaits CTO
    approval" becomes "the CTO approved its exact corrected content on
    2026-10-07". The `Last updated: 2026-10-07` header already carries the
    date of this change and is unchanged; every other row is unchanged.
    Approval-state blob: `246bd5e981d373447fccdd457c02645338bfee4b`.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: record the approval state of the withdrawal-order correction entry.
  - behavioural effect: the
    `THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION` entry only - "propose"
    becomes "record a CTO-approved", "`PROPOSED`: the corrected content is not
    approved" becomes "`APPROVED`: the CTO approved the exact corrected content
    on 2026-10-07", and "proposal only" becomes "decision only". Its
    statements that the earlier entries and the 2026-09-30, 2026-10-05 and
    2026-10-06 approvals apply only to historical exact versions are
    unchanged, and every other entry is byte-identical. Approval-state blob:
    `d1fbe1bdc7699bae18970cdcc9c61d9cbdcefa82`.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: record the Amendment-8 approval-state stage truthfully.
  - behavioural effect: documentation only. Made the approval-state commit
    the then-current stage; recorded the content-approved candidate, the CTO
    candidate-content approval and the approval-state authorization; recorded
    the approval-state validation and no-drift proof (section 9); and
    reframed statements that would otherwise describe the candidate as the
    commit containing the report, recording the candidate's identities as
    historical facts. All still-true earlier evidence was preserved.
  - within authorized write budget: YES

Byte-identical to the content-approved candidate through the approval-state
commit: every path other than the four authorized paths, including ADR-0008
`622060ad90eec41c792f110c373efffbb11a4b56`, ADR-0010
`aca2142a3387785e80db908b3a5c1b0afd82ab51` and ADR-0013
`d928f957bdf775d03e99fc73888ec8e2dec5f80b`.

Amendment-8 candidate changes in the historical content-approved candidate
`42ae90a5932cf24ce90679196888cc3d83d88db8`, relative to the clarification
head `eb3129f13a9c858b3c012ee182315801f0b8563a` (exactly the four authorized
paths):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: implement Specification Amendment 8 and record the corrected
    version as `PROPOSED`.
  - behavioural effect: architecture proposal only. The header carried
    `Status: PROPOSED` and no current approver or approval date. In section
    3.2, the per-`route_key` serialization paragraph states that revision
    withdrawal and every operation creating a new durable selection of a
    revision share one database-atomic order that composes with that
    serialization; the historical-enrollment validation requires the selected
    revision to be available, decided inside that order in the transaction
    that would create the disposition, and refuses a withdrawn revision before
    any disposition or job exists; the "Revision withdrawal and route
    retirement" subsection gains the Amendment-8 rule: the covered operations
    (first activation, reactivation and revision-transition epoch opening,
    first historical enrollment and the withdrawal itself as the minimum, with
    the general rule covering every other new durable selection or
    reference), the exclusion of non-durable preselection, both commit orders,
    the re-read-before-retry rule, the insufficiency of an application-level
    check-then-insert, the permitted mechanisms and composition requirements,
    and the exclusion of explicit replay/current-state work for an
    already-dispositioned event; and activation gate 4 is decided inside the
    same order. Invariant 41 is appended; section 8 item 1 and section 9
    rollout proof 3 name the ordering; section 11 adds the nine Amendment-8
    concurrency validation cases after the revision-withdrawal tests. Section
    12 records the `PROPOSED` state, adds the ordering rule to the description
    of this material correction and binds the 2026-09-30, 2026-10-05 and
    2026-10-06 approvals to the historical earlier versions. The existing
    withdrawal semantics (append-only, audited, future-selection only,
    non-destructive, strand prevention), route retirement and all unrelated
    architecture are unchanged. Candidate blob:
    `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: the material correction makes the ADR-0012 row's `APPROVED`
    assertion untrue for the new version.
  - behavioural effect: the ADR-0012 row records `PROPOSED` / `Pending`,
    binds the 2026-09-30, 2026-10-05 and 2026-10-06 approvals to the
    historical earlier exact versions and adds one summary of the
    withdrawal/selection ordering rule; its partial-supersession,
    authority-condition, ADR-0013 and implementation-authorization sentences
    are unchanged. The header `Last updated` line advances from `2026-10-06`
    to `2026-10-07`, the date of this row change (section 5, decision 57). No
    other row changes. Candidate blob: `45ce935149693d0e983fac1645f5363896e23a53`.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: required changelog entry.
  - behavioural effect: exactly one new
    `THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION` entry at the top of
    `[Unreleased] -> Changed`, describing the corrected version as `PROPOSED`
    and stating that the earlier
    `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION`,
    `THOTH-ASYNC-01-ADR-01-CORRECTION`, `THOTH-ASYNC-01-ADR-01-APPROVAL` and
    `THOTH-ASYNC-01-ADR-01-CLARIFICATION` entries and the approvals they
    record apply only to the historical earlier exact versions. No heading is
    added and every existing entry is byte-preserved. Candidate blob:
    `f3c2bc7976d5fb84e1a00cbfce05748197da900b`.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: record the Amendment-8 candidate stage truthfully.
  - behavioural effect: documentation only. Made the candidate the
    then-current stage; recorded the final-head review of the clarification
    head, `R-EB31-01`, Amendment 8, its independent review and the candidate
    authorization as governing evidence; recorded the candidate's validation
    (section 9); and reframed statements that would otherwise describe the
    clarification commit as the commit containing the report, recording that
    commit's identities as historical facts. All still-true earlier evidence
    was preserved.
  - within authorized write budget: YES

Byte-identical to the clarification head through the Amendment-8 candidate:
every path other than the four correction paths, including ADR-0008
`622060ad90eec41c792f110c373efffbb11a4b56`, ADR-0010
`aca2142a3387785e80db908b3a5c1b0afd82ab51` and ADR-0013
`d928f957bdf775d03e99fc73888ec8e2dec5f80b`.

Historical factual-clarification changes in the clarification commit
`eb3129f13a9c858b3c012ee182315801f0b8563a`, relative to the transient-state
correction head `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` (exactly the two
authorized paths):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: the configured final-head automated review of the transient-state
    correction head found that the unconditional sentence "exactly one durable
    disposition exists for each `(event_id, route_key)`" in section 3.2,
    section 3.5 and invariant 8 conflicted with the eligibility, backlog,
    in-epoch mismatch, historical-enrollment and reconciliation/divergence
    semantics already selected by the same ADR; the CTO classified the
    correction as a factual clarification of prior approved blob
    `a7d95ca918f418d596432071c8ba78a33c2630a9`.
  - behavioural effect: factual clarification only; no architecture decision
    changes. Three sentences change and nothing else: section 3.2 now states
    that at most one durable disposition may exist for a given
    `(event_id, route_key)` and that an eligible route obligation remains
    owed, visible in the route backlog, until it has received that one
    disposition; section 3.5 states the same at-most-one rule with eventual
    receipt of exactly one for an eligible route obligation; invariant 8
    states the same rule in its first sentence. The surrounding text of each
    sentence, the materialized/terminal non-materialized disposition model,
    the uniqueness boundary across epochs and revisions, the no-disposition
    cases (ineligible events, in-epoch contract mismatch, refused historical
    enrollment, approved reconciliation/divergence), the one allowed
    disposition from validated enrollment, duplicate prevention and job
    idempotency were byte-identical. The header kept `Status: APPROVED`,
    `Approved by: Javi, CTO` and `Approval date: 2026-10-06`; section 12 was
    unchanged. Clarified blob: `2db08058552ce901548ef3f71af4a294f5af7e61`.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: the same review found that the report still recorded which live
    PR #960 lifecycle gates were pending ("still requires", "does not
    include", a next-step list of Ready, review, thread-adjudication and
    merge-authorization gates); control accepted that as a blocking
    control-record defect.
  - behavioural effect: documentation only. Replaced every current-gate
    statement with the durable lifecycle model (the ADR-0012 authority
    condition; independent exact-head review, CTO merge authorization,
    guarded merge, programme-local reliance, migration, deployment,
    activation and observation as distinct controlled lifecycle categories;
    GitHub, through #958 and PR #960, as the authority for the current gate);
    recorded the final-head review of the transient-state correction head and
    the CTO classification substantively, without review identifiers;
    recorded the factual clarification and its validation (section 9); made
    that stage the then-current stage; and reframed statements that would
    otherwise describe the transient-state correction commit as the commit
    containing the report. All other historical evidence was preserved.
  - within authorized write budget: YES

Byte-identical to the transient-state correction head through the factual
clarification: every path other than ADR-0012 and this report, including the
decision register `b358acd5bbedf7482cfdc7b71866815c852773af` and
`CHANGELOG.md` `e2947275d19d3abb351a98854268d7db213370d7`.

Historical transient-state correction changes in the correction commit
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`, relative to the recovery head
`22bd94dc959f11fe49c8bef8cd2a836140aadff7` (exactly the one authorized path):

- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: the configured final-head automated review of the recovery head
    found that the recovery report treated the current number and
    resolved/unresolved state of PR #960 review threads as durable repository
    truth; control adjudication on #958 accepted that finding as a blocking
    control-record defect and required a report-only correction.
  - behavioural effect: documentation only. Rephrases every stage-time
    review-thread inspection as historical stage evidence; replaces the
    current-state assertion in section 14 with durable wording that points to
    PR #960 for live thread state; records the final-head review of the
    recovery head substantively, without its review identifiers, and the
    rejection of its topology premise (section 5, decisions 40-43); made that
    correction the then-current stage of this report; reframed statements
    that would otherwise describe the recovery commit as the commit containing
    the report; and recorded the correction's validation (section 9). All
    other historical evidence was preserved.
  - within authorized write budget: YES

Byte-identical to the recovery head through the transient-state correction:
every path other than this report, including ADR-0012
`a7d95ca918f418d596432071c8ba78a33c2630a9`, the decision register
`b358acd5bbedf7482cfdc7b71866815c852773af` and `CHANGELOG.md`
`e2947275d19d3abb351a98854268d7db213370d7`.

Historical control-record recovery changes in the recovery commit
`22bd94dc959f11fe49c8bef8cd2a836140aadff7`, relative to the approval-state
head `609a01427847233b7280557baa61825ac2079865` (exactly the one authorized
path):

- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: the approval-state report copied the GitHub identifier of the CTO
    content-approval record into repository source, contrary to the
    repository-authoritative Route-B rule in
    `docs/engineering/decisions/README.md`; control adjudication on #958
    required a report-only correction.
  - behavioural effect: documentation only. Removes every occurrence of that
    identifier; records the content approval through owning issue #958,
    approval date 2026-10-06, candidate head
    `b596536e7db0d6b8ace0a42fdd26bff42cabf104` and candidate ADR-0012 blob
    `cd897d576522a741c3dfa82f8de2f8d80d4288f6`; replaces the approval-state
    deviation framing with the reconciled control fact (section 5, decisions
    36-39); removes the review-focus item that asked a reviewer to accept the
    transcription; made the recovery the then-current stage of this report;
    and recorded its validation (section 9). All other historical evidence
    was preserved.
  - within authorized write budget: YES

Byte-identical to the approval-state head through the control-record recovery
and the transient-state correction:
every path other than this report, including ADR-0012
`a7d95ca918f418d596432071c8ba78a33c2630a9`, the decision register
`b358acd5bbedf7482cfdc7b71866815c852773af` and `CHANGELOG.md`
`e2947275d19d3abb351a98854268d7db213370d7`.

Amendment-6/7 approval-state changes in the historical approval-state commit
`609a01427847233b7280557baa61825ac2079865`, relative to the content-approved
candidate
`b596536e7db0d6b8ace0a42fdd26bff42cabf104` (exactly the four authorized paths,
approval state only):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: record the CTO exact candidate-content approval of 2026-10-06.
  - behavioural effect: the header's `Status: PROPOSED` becomes
    `Status: APPROVED` and gains `Approved by: Javi, CTO` and
    `Approval date: 2026-10-06`; section 12's two leading candidate-state
    paragraphs ("Current decision state: **PROPOSED**." and the statement that
    the corrected content has not been approved) are replaced by
    `This ADR is **APPROVED**.`, the same approver and date, and a paragraph
    stating that the CTO approved this exact corrected content on 2026-10-06,
    that this approval does not by itself make the ADR repository-authoritative
    and that any ADR-0013 programme-local reliance on this exact version is a
    separate state not effective until its own conditions are satisfied. Every
    other line, including all architecture, the historical-approval paragraph,
    the authority condition and the architecture-versus-implementation
    separation, is byte-identical to the candidate (section 9).
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: record the approval state of the ADR-0012 row.
  - behavioural effect: the ADR-0012 row only - `PROPOSED` becomes `APPROVED`,
    `Pending` becomes `Satisfied`, and "its corrected content awaits CTO
    approval" becomes "the CTO approved its exact corrected content on
    2026-10-06". The `Last updated: 2026-10-06` header already carries the
    date of this change and is unchanged; every other row is unchanged.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: record the approval state of the route-contract correction entry.
  - behavioural effect: the
    `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` entry only - "propose"
    becomes "record a CTO-approved", "`PROPOSED`: the corrected content is not
    approved" becomes "`APPROVED`: the CTO approved the exact corrected content
    on 2026-10-06", and "proposal only" becomes "decision only". Its statements
    that the earlier entries and the 2026-09-30 and 2026-10-05 approvals apply
    only to historical exact versions are unchanged, and every other entry is
    byte-identical.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: record the Amendment-6/7 approval-state stage truthfully.
  - behavioural effect: documentation only. Made the approval-state commit the
    current stage; recorded the content-approved candidate, the CTO
    candidate-content approval and the approval-state authorization; recorded
    the approval-state validation and no-drift proof (section 9); and reframed
    statements that would otherwise have described the candidate as the commit
    containing the report. It also copied the content-approval record's GitHub
    identifier into the report, the control-record defect corrected by the
    recovery above. All still-true earlier evidence was preserved.
  - within authorized write budget: YES

Amendment-6/7 candidate changes in the historical content-approved candidate
`b596536e7db0d6b8ace0a42fdd26bff42cabf104`, relative to the recovery head
`9db91af7bf3d9f969556c819187420fc31d25c10` (exactly the four authorized paths):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: implement the Amendment-6/7 immutable route-contract-revision
    architecture and record the corrected version as `PROPOSED`.
  - behavioural effect: architecture proposal only. The header carries
    `Status: PROPOSED` and no current approver or approval date. Section 3.2
    replaces the route-level event kind/version contract with append-only
    immutable route-contract revisions under the stable `route_key`; binds
    every epoch to exactly one revision; makes eligibility require both epoch
    membership and the epoch revision's kind/accepted-version predicate;
    defines the canonical semantic signature and database-atomic equivalent
    revision convergence; adds the `IN_EPOCH_CONTRACT_MISMATCH` fail-safe,
    field-by-field historical-enrollment validation, revision withdrawal and
    route-retirement rules, same-route versus new-route guidance and new-route
    duplicate-effect protection; and turns the event-emission floor into the
    event-emission floor and continuous producer-version coverage, with
    expand/contract transitions, cross-route fan-out and non-binary writers.
    Route backlog and materializer health become per revision and per
    emittable or outstanding accepted version (sections 3.2 and 3.15). Section
    3.3 binds route version acceptance to the revision's accepted-version set;
    section 3.9 adds explicit materializer capability declarations,
    deterministic compatibility and the materializer-retirement rule;
    invariants 3-6 and 8 are reconciled and invariants 35-40 added; sections
    8, 9, 10 and 11 are aligned, and section 11 adds the Amendment-6/7
    validation set. Section 12 records the `PROPOSED` state and binds the
    2026-09-30 and 2026-10-05 approvals to the historical earlier versions.
    All unrelated architecture is unchanged.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: the correction makes the ADR-0012 row's `APPROVED` assertion
    untrue for the new version.
  - behavioural effect: the ADR-0012 row records `PROPOSED` / `Pending`, binds
    the 2026-09-30 approval to the pre-correction versions and the 2026-10-05
    approval to the earlier activation-epoch corrected version, and adds one
    summary of the route-contract-revision correction; its partial
    supersession, authority-condition, ADR-0013 and
    implementation-authorization sentences are unchanged. The header
    `Last updated` line advances from `2026-10-05` to `2026-10-06`, the date of
    this row change (section 5, decision 25). No other row changes.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: required changelog entry.
  - behavioural effect: exactly one new
    `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` entry at the top of
    `[Unreleased] -> Changed`, describing the corrected version as `PROPOSED`
    and stating that the earlier `THOTH-ASYNC-01-ADR-01-CORRECTION`,
    `THOTH-ASYNC-01-ADR-01-APPROVAL` and
    `THOTH-ASYNC-01-ADR-01-CLARIFICATION` entries and the approvals they
    record apply only to the historical earlier exact versions. No heading is
    added and every existing entry is byte-preserved.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: record the Amendment-6/7 candidate stage truthfully.
  - behavioural effect: documentation only. Made the candidate the current
    stage; recorded the recovery, the final-head review of the recovery head,
    Amendments 6-7 and their review sequence as history; recorded the
    candidate's validation (section 9); and reframed statements that would
    otherwise have described the recovery commit as the commit containing the
    report. All still-true earlier evidence was preserved.
  - within authorized write budget: YES

Byte-identical to the recovery head through the Amendment-6/7 candidate, the
approval-state commit, the control-record recovery, the transient-state
correction, the factual clarification, the Amendment-8 candidate, the
Amendment-8 approval-state commit, the Amendment-9 candidate and the
Amendment-9 approval-state commit:

- ADR-0008 - blob `622060ad90eec41c792f110c373efffbb11a4b56`;
- ADR-0010 - blob `aca2142a3387785e80db908b3a5c1b0afd82ab51`;
- ADR-0013 - blob `d928f957bdf775d03e99fc73888ec8e2dec5f80b`;
- `docs/engineering/decisions/README.md` - blob
  `de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7`;
- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md`
  - blob `7bf7425f0527dbaa72e718df5264e399035a148b`;
- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md`
  - blob `d86cf215866750979eafea040426c51374d5155c`;
- ADR-0005 - blob `bdaa976e4893b1fc45f994236f9e56d433212d63`;
- `docs/engineering/AGENTS.md` - blob `e194b14e8c4fbb298db7ca6c38aebeb69547a29a`;
- `docs/engineering/ai-delivery/implementation-report-template.md` - blob
  `0ae39d3892bbca5b0ff90dc7ffa70038662351bc`.

Activation-epoch candidate-stage changes (content-approved candidate
`95769537cdc9871e42f9f3e5443604a96beb74da`) relative to the unmodified
merge-tree baseline `e15607878e66d2ff12b819dcea13293a46674f13`:

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: implement the CTO-selected append-only activation-epoch
    architecture and record the corrected version as `PROPOSED`.
  - behavioural effect: architecture proposal only. The header carries
    `Status: PROPOSED` and no current approver or approval date. Section 3.2
    replaces the single activation/deactivation pair per route with a stable
    logical `route_key` and an append-only history of activation epochs
    (half-open `[activation_generation, deactivation_generation)` intervals,
    at most one open epoch, total order by activation generation, write-once
    close facts, serialized open/close/retirement, terminal permanent
    retirement, historical eligibility through any open or closed epoch, no
    retroactive gap coverage, subordinate epoch identifiers) and states the
    disposition model consistently with section 3.5 and invariant 8. The
    event-emission floor, route health signals (section 3.15), invariants 3-6
    and 8, implementation decomposition, rollout, rollback and validation
    requirements are aligned with the epoch model. Section 12 records the
    `PROPOSED` state, binds the 2026-09-30 approval to the historical
    pre-correction versions only, keeps the repository-authority condition and
    the architecture-versus-implementation separation, and aligns the ADR's
    amendment clause with `docs/engineering/decisions/README.md`. All unrelated
    architecture is unchanged.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: the material correction invalidates the earlier exact-version
    approval represented by the ADR-0012 row.
  - behavioural effect: only the ADR-0012 row changes. It records `PROPOSED`,
    binds the 2026-09-30 approval to the historical pre-correction versions,
    summarizes the activation-epoch correction and preserves the partial
    supersession boundaries, the authority condition, the ADR-0013
    programme-local reliance distinction and the implementation-authorization
    distinction. No other row or header line changes.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: required changelog entry.
  - behavioural effect: exactly one new `THOTH-ASYNC-01-ADR-01-CORRECTION`
    entry at the top of `[Unreleased] -> Changed`, describing the corrected
    version as `PROPOSED` and stating that the earlier ADR-0012 approval and
    clarification entries apply only to the historical pre-correction exact
    versions. No new heading; existing ADR-0012 and ADR-0013 entries are
    byte-preserved.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: reconcile the report to the current template and the Route-B
    candidate state while preserving material history.
  - behavioural effect: documentation only. Sections 1-15 follow the template;
    the earlier record is preserved in Appendix A with headings demoted and
    only the tense or framing changes needed to keep it truthful as history.
  - within authorized write budget: YES

Approval-state changes in the historical approval-state commit
`c046e6bf6510f24ee1231f86293a3d499bceed7e` relative to the content-approved
candidate `95769537cdc9871e42f9f3e5443604a96beb74da` (the same four paths,
approval state only):

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: record the CTO content approval of 2026-10-05.
  - behavioural effect: the header's `Status: PROPOSED` becomes
    `Status: APPROVED` and gains `Approved by: Javi, CTO` and
    `Approval date: 2026-10-05`; section 12's candidate-state text ("Current
    decision state: **PROPOSED**." and the statement that the corrected content
    had not been approved) is replaced by `This ADR is **APPROVED**.`, the same
    approver and date, and a statement that the CTO approved this exact
    corrected content on 2026-10-05. No architecture text changes; the
    repository-authority condition, the ADR-0013 reliance distinction and the
    architecture-versus-implementation separation are unchanged.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: record the approval state of the ADR-0012 row.
  - behavioural effect: the ADR-0012 row only - `PROPOSED` becomes `APPROVED`,
    `Pending` becomes `Satisfied`, and "its corrected content awaits CTO
    approval" becomes "the CTO approved its exact corrected content on
    2026-10-05". All other row text, every other row and the header are
    unchanged.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: record the approval state of the new correction entry.
  - behavioural effect: the `THOTH-ASYNC-01-ADR-01-CORRECTION` entry only -
    "propose" becomes "record a CTO-approved", "`PROPOSED`: the corrected
    content is not approved" becomes "`APPROVED`: the CTO approved the exact
    corrected content on 2026-10-05", and "proposal only" becomes "decision
    only". Its statement that the earlier approval entries apply only to the
    historical pre-correction versions is unchanged, and no other entry
    changes.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: Amendment 3 C4/C5 approval-state and validation evidence.
  - behavioural effect: documentation only; records the content approval, the
    candidate as approval-state parent, the approval-state validation
    (section 9) and the stage-specific action matrix, and preserves the
    candidate-stage and historical evidence.
  - within authorized write budget: YES

Recovery changes in the historical recovery commit
`9db91af7bf3d9f969556c819187420fc31d25c10` relative to the historical
approval-state head `c046e6bf6510f24ee1231f86293a3d499bceed7e` (exactly the
two authorized recovery paths):

- `docs/engineering/decisions/decision-register.md`
  - reason: final-head automated review found the active register's own
    freshness metadata stale after the ADR-0012 row recorded an approval on
    2026-10-05 (Amendment 4 E1/E6).
  - behavioural effect: exactly one header line, `Last updated: 2026-09-30`
    becomes `Last updated: 2026-10-05`. The ADR-0012 row and every other row
    are byte-identical to the approval-state head; no other wording or metadata
    changes.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: Amendment 4 E7 / Amendment 5 F3 reconciliation so the durable
    record stays truthful once the report moves into the recovery commit.
  - behavioural effect: documentation only. Supersedes the earlier statement
    that leaving the register header unchanged was the correct final state;
    distinguishes the historical content-approved candidate, the historical
    approval-state head and the recovery commit; distinguishes the historical
    four-path budget from the two-path recovery budget; records the final-head
    review outcome substantively, without copying review identifiers; reframes
    statements that would otherwise describe the recovery commit as the
    approval-state commit; and records the recovery validation (section 9).
    All still-true candidate-stage, content-approval and approval-state
    evidence is preserved.
  - within authorized write budget: YES

Byte-identical to the approval-state head through the recovery:

- ADR-0012 - blob `4848470a252076268a9fac9da5c40f0b4ebff474` (architecture,
  approval date, approver and route/disposition semantics unchanged);
- `CHANGELOG.md` - blob `ff7089a4a93e6a8c2c27c75a0ebf56ce43405c27`;
- ADR-0005 - blob `bdaa976e4893b1fc45f994236f9e56d433212d63`;
- `docs/engineering/AGENTS.md` - blob `e194b14e8c4fbb298db7ca6c38aebeb69547a29a`;
- `docs/engineering/ai-delivery/implementation-report-template.md` - blob
  `0ae39d3892bbca5b0ff90dc7ffa70038662351bc`;
- the inherited and byte-preserved paths listed below.

Inherited unchanged from refreshed `develop` through parent 2 (the closed
inherited set between the task head and the merge-tree baseline):

- `CHANGELOG.md` - inherited merge contribution of two `[Unreleased] ->
  Changed` entries (`CTRL-ADR-PR-FIRST-01`, `CTRL-ADR-AMEND-01`), separate from
  the single new correction entry above;
- `docs/engineering/decisions/README.md` - blob
  `de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7`;
- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md`
  - blob `7bf7425f0527dbaa72e718df5264e399035a148b`;
- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md`
  - blob `d86cf215866750979eafea040426c51374d5155c`.

Byte-preserved task decision and control paths:

- ADR-0008 - blob `622060ad90eec41c792f110c373efffbb11a4b56`;
- ADR-0010 - blob `aca2142a3387785e80db908b3a5c1b0afd82ab51`;
- ADR-0013 - blob `d928f957bdf775d03e99fc73888ec8e2dec5f80b`.

Actual new files created: NONE

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS

The Amendment-9 approval-state commit differs from the content-approved
candidate `8699af5635b8f18691ceaeccd3b24707a71ef4f2` in exactly the four
authorized paths and from refreshed `feature/worker` in exactly the six
cumulative PR paths (section 9, "Amendment-9 approval-state validation"). The
Amendment-9 candidate differed from the Amendment-8 approval-state head
`e2588f7b7e63a5c71968d87a3409f0ff48635e6d` in exactly the four authorized
correction paths and from refreshed `feature/worker` in exactly the six
cumulative PR paths (section 9, "Amendment-9 candidate validation"). The
Amendment-8 approval-state commit differed from the content-approved
candidate `42ae90a5932cf24ce90679196888cc3d83d88db8` in exactly the four
authorized paths and from refreshed `feature/worker` in exactly the six
cumulative PR paths (section 9, "Amendment-8 approval-state validation"). The
Amendment-8 candidate differed from the clarification head
`eb3129f13a9c858b3c012ee182315801f0b8563a` in exactly the four authorized
correction paths and from refreshed `feature/worker` in exactly the six
cumulative PR paths (section 9, "Amendment-8 candidate validation"). The
factual clarification differed from the transient-state correction head
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` in exactly the two authorized
paths and from refreshed `feature/worker` in exactly the six cumulative PR
paths (section 9, "Factual-clarification validation"). The transient-state
correction differed from the recovery head
`22bd94dc959f11fe49c8bef8cd2a836140aadff7` in exactly the one authorized
report path and from refreshed `feature/worker` in exactly the six cumulative
PR paths (section 9, "Transient-state correction validation"). The
control-record recovery differed from the approval-state head in exactly the
one authorized report path and from refreshed `feature/worker` in exactly the
six cumulative PR paths (section 9, "Control-record recovery validation").
The Amendment-6/7 approval-state commit differed from the content-approved
candidate in exactly the four authorized paths and from refreshed
`feature/worker` in exactly the six cumulative PR paths (section 9,
"Amendment-6/7 approval-state validation"). The content-approved candidate
differed from the recovery head in exactly the four authorized correction paths
and from refreshed `feature/worker` in exactly the six cumulative PR paths
(section 9, "Amendment-6/7 candidate validation"). Historically, the
activation-epoch candidate differed from the unmodified
merge-tree baseline in exactly the four authorized correction paths and from
refreshed `feature/worker` in exactly the six cumulative PR paths; the
approval-state commit differed from the content-approved candidate in exactly
the same four paths and from refreshed `feature/worker` in exactly the six
cumulative PR paths; and the recovery commit differed from the approval-state
head in exactly the two authorized recovery paths and from refreshed
`feature/worker` in exactly the same six cumulative PR paths. No ADR-0008,
ADR-0010, ADR-0013, engineering-control doctrine, workflow, `AGENTS.md`,
runtime, migration or schema path is edited at any stage.

## 4.2 Authorized actions actually used

For the Route-B activation-epoch candidate stage (#958 comments `5993634460`
and `5998119577`), its approval-state stage (#958 comment `5999579357`), the
Amendment-4/5 recovery stage (#958 comment `6013267369`), the Amendment-6/7
candidate stage (#958 comment `6021574322`), the Amendment-6/7 approval-state
stage (#958 comment `6022486352`), the control-record recovery stage (#958
comment `6022850916`), the transient-state correction stage (#958 comment
`6036961945`), the factual-clarification stage (#958 comment `6041301223`),
the Amendment-8 candidate stage (#958 comment `6044220221`), the Amendment-8
approval-state stage (#958 comment `6045126538`), the Amendment-9 candidate
stage (#958 comment `6046895895`) and the Amendment-9 approval-state stage
(#958 comment `6065879000`). This matrix
states only what had been done when the commit that
contains this report was created; it is historical stage-time evidence and
does not describe the current PR #960 lifecycle gate.

Activation-epoch candidate stage - historical, completed before the
approval-state commit:

- repository inspection: USED (Git history, refs, doctrine and the #957/#958/PR
  #960 records, read-only)
- source edit: USED (the four authorized correction paths only)
- new file creation: NOT USED
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: NOT USED (detached local worktrees were used for
  construction; the existing task branch was the fast-forward target)
- commit: USED (the two-parent candidate merge commit
  `95769537cdc9871e42f9f3e5443604a96beb74da`; see section 5 for the earlier
  unpublished local candidate)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` to the candidate)
- PR creation/update: USED (PR #960 Ready -> Draft before any candidate was
  constructed, independently verified by control, and the PR description
  updated for the published candidate); PR #960 remained a draft

Approval-state stage - historical, completed before the recovery commit:

- repository inspection: USED (read-only re-verification of refs, PR #960
  state and the #958 approval records)
- source edit: USED (the same four paths, approval state only)
- commit: USED (the single-parent approval-state commit
  `c046e6bf6510f24ee1231f86293a3d499bceed7e`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `95769537cdc9871e42f9f3e5443604a96beb74da` to the approval-state commit;
  the remote head, tree, direct parent and ADR-0012 blob were verified)
- PR creation/update: USED (the PR #960 description was updated for the
  published approval-state head while the PR remained a draft)
- Draft -> Ready: NOT USED by the implementing agent; control later separately
  authorized and performed it, which triggered the configured final-head
  automated review of the approval-state head (section 2)

Amendment-4/5 recovery stage - historical, completed before the Amendment-6/7
candidate:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 state, commit topology and every protected blob named by the
  recovery authorization; all matched)
- PR creation/update: USED (PR #960 Ready -> Draft before any recovery source
  was staged, and one state-neutral PR #960 description update for the
  published recovery head; no title change)
- source edit: USED (the two authorized recovery paths only)
- new file creation, file deletion/move/rename, branch creation: NOT USED
- commit: USED (the single-parent recovery commit
  `9db91af7bf3d9f969556c819187420fc31d25c10`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `c046e6bf6510f24ee1231f86293a3d499bceed7e` to the recovery commit; the
  remote head, tree, parent and protected blobs were verified)
- Draft -> Ready: NOT USED by the implementing agent; control later marked
  PR #960 Ready, which triggered the configured final-head automated review of
  the recovery head (section 2)

Amendment-6/7 candidate stage - historical, completed before the
approval-state commit:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 state, mergeability, the six-path cumulative footprint, the absence
  of any source commit after the recovery head, the ADR-0012 blob at the
  recovery head and, as stage-time evidence, the PR #960 review-thread state
  in GitHub; all matched the authorization; current review-thread state
  remains GitHub-owned)
- PR creation/update: USED (PR #960 Ready -> Draft, the one authorized
  pre-publication PR-state mutation, performed after that verification and
  before any candidate source was staged; no PR title or description change)
- source edit: USED (the four authorized correction paths only)
- new file creation, file deletion/move/rename, branch creation: NOT USED
- commit: USED (the single-parent candidate commit
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `9db91af7bf3d9f969556c819187420fc31d25c10` to the candidate; the remote head,
  tree, parent and ADR-0012 blob were verified)

Amendment-6/7 approval-state stage - historical, completed before the
control-record recovery:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104` and target `feature/worker`, the
  candidate's tree, single parent and ADR-0012 blob, the six-path cumulative
  footprint and, as stage-time evidence, the PR #960 review-thread state in
  GitHub; all matched the authorization; current review-thread state remains
  GitHub-owned)
- PR creation/update: NOT USED (PR #960 was already a draft; no PR-state,
  title or description change)
- source edit: USED (the four authorized paths, approval state only)
- new file creation, file deletion/move/rename, branch creation: NOT USED
- commit: USED (the single-parent approval-state commit
  `609a01427847233b7280557baa61825ac2079865`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104` to the approval-state commit; the
  remote head, tree, parent and ADR-0012 blob and state were verified)

Control-record recovery stage - historical, completed before the
transient-state correction:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `609a01427847233b7280557baa61825ac2079865` and target `feature/worker`, the
  ADR-0012, report, decision-register and changelog blobs at that head, the
  six-path cumulative footprint and, as stage-time evidence, the PR #960
  review-thread state in GitHub; all matched the authorization; current
  review-thread state remains GitHub-owned)
- PR creation/update: NOT USED (PR #960 was already a draft; no PR-state,
  title or description change was authorized at that stage)
- source edit: USED (this report only)
- new file creation: NOT USED (not authorized)
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: NOT USED (not authorized; the same detached local worktree,
  at the approval-state head, was used for construction, and the existing task
  branch was the fast-forward target)
- commit: USED (the single-parent recovery commit
  `22bd94dc959f11fe49c8bef8cd2a836140aadff7`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `609a01427847233b7280557baa61825ac2079865` to the recovery commit; the
  remote head, tree, parent and protected blobs were verified)
- Draft -> Ready: NOT USED by the implementing agent; control later separately
  marked PR #960 Ready, which triggered the configured final-head automated
  review of the recovery head, and returned it to Draft before the
  transient-state correction (section 2)

Transient-state correction stage - historical, completed before the factual
clarification:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `22bd94dc959f11fe49c8bef8cd2a836140aadff7` and target `feature/worker`, the
  recovery head's tree and single parent, the ADR-0012, report,
  decision-register and changelog blobs at that head, the six-path cumulative
  footprint and, as stage-time evidence, the PR #960 review-thread state in
  GitHub; all matched the authorization; current review-thread state remains
  GitHub-owned)
- PR creation/update: NOT USED (control had already returned PR #960 to
  draft; no PR-state, title or description change was authorized at that
  stage)
- source edit: USED (this report only)
- new file creation: NOT USED (not authorized)
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: NOT USED (not authorized; a detached local worktree at the
  recovery head was used for construction, and the existing task branch was
  the fast-forward target)
- commit: USED (the single-parent correction commit
  `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `22bd94dc959f11fe49c8bef8cd2a836140aadff7` to the correction commit; the
  remote head, tree, parent and protected blobs were verified)
- Draft -> Ready: NOT USED by the implementing agent; control later separately
  marked PR #960 Ready, which triggered the configured final-head automated
  review of the correction head, and returned it to Draft before the factual
  clarification (section 2)

Factual-clarification stage - historical, completed before the Amendment-8
candidate:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` and target `feature/worker`, the
  correction head's tree and single parent, the ADR-0012, report,
  decision-register and changelog blobs at that head, the six-path cumulative
  footprint, the presence of the classification and authorization records on
  #958 with no later superseding control record, and, as stage-time
  evidence, the PR #960 review-thread state in GitHub; all matched the
  authorization; current review-thread state remains GitHub-owned)
- PR creation/update: NOT USED (control had already returned PR #960 to
  draft; no PR-state, title or description change was authorized at that
  stage)
- source edit: USED (ADR-0012 and this report only)
- new file creation: NOT USED (not authorized)
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: NOT USED (not authorized; a detached local worktree at the
  transient-state correction head was used for construction, and the existing
  task branch is the fast-forward target)
- commit: USED (the single-parent clarification commit
  `eb3129f13a9c858b3c012ee182315801f0b8563a`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` to the clarification commit; the
  remote head, tree, parent and ADR-0012 blob were verified)
- Draft -> Ready: NOT USED by the implementing agent; control later separately
  marked PR #960 Ready, which triggered the configured final-head automated
  review of the clarification head, and returned it to Draft before the
  Amendment-8 candidate (section 2)

Amendment-8 candidate stage - historical, completed before the approval-state
commit:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `eb3129f13a9c858b3c012ee182315801f0b8563a` and target `feature/worker`,
  the clarification head's tree and single parent, the ADR-0012, report,
  decision-register and changelog blobs at that head, the protected ADR-0008
  and ADR-0010 blobs, the six-path cumulative footprint, PR #960 as the only
  open pull request targeting `feature/worker`, the presence of the Amendment
  8, independent-review and authorization records on #958, and, as
  stage-time evidence, the PR #960 review-thread state in GitHub; all matched
  the authorization; current review-thread state remains GitHub-owned)
- PR creation/update: NOT USED (PR #960 was already a draft at preflight, so
  the one authorized Ready -> Draft mutation was not needed; no PR-state,
  title or description change was authorized at that stage)
- source edit: USED (the four authorized correction paths only)
- new file creation: NOT USED (not authorized)
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: NOT USED (not authorized; a detached local worktree at the
  clarification head was used for construction, and the existing task branch
  is the fast-forward target)
- commit: USED (the single-parent candidate commit
  `42ae90a5932cf24ce90679196888cc3d83d88db8`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `eb3129f13a9c858b3c012ee182315801f0b8563a` to the candidate; the remote
  head, tree, parent and ADR-0012, decision-register, changelog and report
  blobs were verified)

Amendment-8 approval-state stage - historical, completed before the
Amendment-9 candidate:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `42ae90a5932cf24ce90679196888cc3d83d88db8` and target `feature/worker`,
  the candidate's tree, single parent and ADR-0012, report, decision-register
  and changelog blobs, the protected ADR-0008 and ADR-0010 blobs, the six-path
  cumulative footprint, PR #960 as the only open pull request targeting
  `feature/worker`, the presence of the content-approval and approval-state
  authorization records on #958 with no later control record, and, as
  stage-time evidence, the PR #960 review-thread state in GitHub; all matched
  the authorization; current review-thread state remains GitHub-owned)
- PR creation/update: NOT USED (PR #960 was already a draft; no PR-state,
  title or description change was authorized at that stage)
- source edit: USED (the four authorized paths, approval state only)
- new file creation, file deletion/move/rename, branch creation: NOT USED
  (not authorized; the same detached local worktree, now at the
  content-approved candidate, was used for construction, and the existing
  task branch is the fast-forward target)
- commit: USED (the single-parent approval-state commit
  `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `42ae90a5932cf24ce90679196888cc3d83d88db8` to the approval-state commit; the
  remote head, tree, parent, ADR-0012, decision-register, changelog and report
  blobs and the ADR approval state were verified)
- Draft -> Ready: NOT USED by the implementing agent; control later separately
  marked PR #960 Ready, which triggered the configured final-head automated
  review of the approval-state head, and returned it to Draft before the
  Amendment-9 candidate (section 2)

Amendment-9 candidate stage - historical, completed before the Amendment-9
approval-state commit:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` and target `feature/worker`,
  the approval-state head's tree and single parent, the ADR-0012, report,
  decision-register and changelog blobs at that head, the protected ADR-0008
  and ADR-0010 blobs, the six-path cumulative footprint, PR #960 as the only
  open pull request targeting `feature/worker`, the presence of the Amendment
  9, independent-review and authorization records on #958 with no later
  control record, and, as stage-time evidence, the PR #960 review-thread
  state in GitHub; all matched the authorization; current review-thread state
  remains GitHub-owned)
- PR creation/update: NOT USED (PR #960 had been returned to draft by control
  before that stage, so no PR-state mutation was needed; no PR-state, title
  or description change was authorized at that stage)
- source edit: USED (the four authorized correction paths only)
- new file creation: NOT USED (not authorized)
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: NOT USED (not authorized; the same detached local
  worktree, now at the approval-state head, was used for construction, and
  the existing task branch is the fast-forward target)
- commit: USED (the single-parent candidate commit
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2`)
- push: USED (one non-force fast-forward of `feature/async/adr-0012` from
  `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` to the candidate; the remote head,
  tree, parent and ADR-0012, decision-register, changelog and report blobs
  were verified; no automated review ran on the draft candidate head)

Amendment-9 approval-state stage - done up to and including the creation of
the commit that contains this report:

- repository inspection: USED (read-only verification of the exact refs,
  PR #960 open, draft, unmerged and mergeable with head
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2` and target `feature/worker`,
  the candidate's tree, single parent and ADR-0012, report, decision-register
  and changelog blobs, the protected ADR-0008 and ADR-0010 blobs, the six-path
  cumulative footprint, PR #960 as the only open pull request targeting
  `feature/worker`, the presence of the content-approval and approval-state
  authorization records on #958 with no later control record, and, as
  stage-time evidence, the PR #960 review-thread state in GitHub; all matched
  the authorization; current review-thread state remains GitHub-owned)
- PR creation/update: NOT USED (PR #960 was already a draft; no PR-state,
  title or description change is authorized at this stage)
- source edit: USED (the four authorized paths, approval state only)
- new file creation, file deletion/move/rename, branch creation: NOT USED
  (not authorized; the same detached local worktree, now at the
  content-approved candidate, was used for construction, and the existing
  task branch is the fast-forward target)
- commit: USED (the single-parent approval-state commit that contains this
  report)

Authorized post-commit actions for the approval-state commit - GitHub-owned
lifecycle evidence under `ADR-0005`, not self-recorded here:

- the non-force push that fast-forwards `feature/async/adr-0012` from
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2` to the approval-state commit;
- post-push read-only verification of the remote head, tree, direct parent,
  ADR-0012, decision-register, changelog and report blobs, ADR approval
  state, changed path set, cumulative footprint and PR #960 state, and that
  no review thread was replied to, resolved or dismissed;
- the automatic PR CI that the push causes, and read-only inspection of any
  automated review that runs on the draft head.

These actions necessarily follow the creation of the commit that contains this
report. Their outcomes are recorded in the post-publication handoff and the
GitHub ledger rather than written into the source tree as completed.

Not authorized for the implementing agent at the Amendment-9 approval-state
stage, each a separate controlled lifecycle category where applicable
(historical stage-time evidence; GitHub, through #958 and PR #960, is
authoritative for the current state of each):

- any edit to a path other than the four authorized paths: NOT USED (not
  authorized)
- any ADR-0012 change outside the approval-state representation, including
  any architecture change: NOT USED (not authorized)
- independent inspection or exact-head review: NOT USED (never on the
  implementing agent's authority)
- exact-final-blob approval or confirmation: NOT USED (not authorized; never
  on the implementing agent's authority)
- PR #960 title or description update: NOT USED (not authorized at this stage)
- Draft -> Ready: NOT USED (not authorized)
- any second approval-state commit, amend, rebase, squash, force push or
  history rewrite: NOT USED (not authorized)
- issue/comment mutation: NOT USED (not authorized)
- review-thread resolution/dismissal/reply: NOT USED (not authorized)
- manual CI dispatch/rerun: NOT USED (not authorized)
- manual Codex/app review request: NOT USED (not authorized)
- provider/runtime read: NOT USED (not authorized)
- provider/runtime write: NOT USED (not authorized)
- migration execution: NOT USED (not authorized)
- release/tag/publication: NOT USED (not authorized)
- programme-pin reconciliation: NOT USED (not authorized)
- merge: NOT USED (not authorized; the implementing agent must not merge its
  own work)
- deployment: NOT USED (not authorized)
- production activation: NOT USED (not authorized)
- other: NONE

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed (for example a workflow triggered by
opening the PR, and whether it performed any external write such as a
container-registry push):

Pushing a new head to the PR triggers the repository's
`pull_request` workflows (`build-test-and-check`, `check-changelog`,
`publish-to-dockerhub`, `run-migrations`). Every path in the change set is
`CHANGELOG.md` or under `docs/`, which `.github/scripts/classify_ci_changes.py`
classifies as documentation-only, so the `classify` and `check-changelog` jobs
are expected to run and pass while the build, test, lint, format, migration and
Docker build/push jobs are expected to be skipped. No container-registry push
and no migration execution is expected. Under Amendment 3 C2/C11, Amendments
4-5, Amendments 6-7, Amendments 8-9, the transient-state correction
authorization, the factual-clarification authorization, the Amendment-8
candidate and approval-state authorizations and the Amendment-9 candidate and
approval-state authorizations, configured Codex/app review is not expected
from a push to a draft PR and is not relied upon at the Amendment-6/7
candidate, approval-state, control-record recovery, transient-state
correction, factual-clarification, Amendment-8 candidate, Amendment-8
approval-state, Amendment-9 candidate or Amendment-9 approval-state stage;
configured Codex/app review runs on separately authorized Ready transitions,
which are GitHub-owned lifecycle events, and any automated review that
nevertheless runs on a draft head is inspected read-only. The actual job
outcomes for the approval-state commit exist only after its publication; they
are GitHub-owned lifecycle evidence recorded in the pull
request, the post-publication handoff and the owning issue, not in this file.
If an automatic workflow unexpectedly intends to execute a migration or
publish an image rather than skip, that is a HOLD condition.

Historical automatic CI for earlier PR heads, as recorded by earlier stages of
this report or read back from GitHub at the transient-state correction,
factual-clarification, Amendment-8 candidate, Amendment-8 approval-state,
Amendment-9 candidate or Amendment-9 approval-state stage, followed the same
documentation-only classification, with no manual dispatch or rerun:

```text
7f7e62824eb40308688b46351ffea182e9e52942: 4 success, 6 skipped
227057cae8a4c10f7f7795e3afe68271e56681b9: 4 success, 6 skipped
b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20: 4 success (classify x3, check-changelog), 6 skipped
95769537cdc9871e42f9f3e5443604a96beb74da: 4 success (classify x3, check-changelog), 6 skipped (content-approved candidate)
c046e6bf6510f24ee1231f86293a3d499bceed7e: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (approval-state head)
9db91af7bf3d9f969556c819187420fc31d25c10: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (recovery head)
b596536e7db0d6b8ace0a42fdd26bff42cabf104: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (content-approved Amendment-6/7 candidate)
609a01427847233b7280557baa61825ac2079865: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (Amendment-6/7 approval-state head)
22bd94dc959f11fe49c8bef8cd2a836140aadff7: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (control-record recovery head)
9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (transient-state correction head)
eb3129f13a9c858b3c012ee182315801f0b8563a: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (factual-clarification head)
42ae90a5932cf24ce90679196888cc3d83d88db8: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (content-approved Amendment-8 candidate)
e2588f7b7e63a5c71968d87a3409f0ff48635e6d: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (Amendment-8 approval-state head)
8699af5635b8f18691ceaeccd3b24707a71ef4f2: 4 success (classify x3, check-changelog), 6 skipped (build, test, lint, format_check, run_migrations, build_and_push_staging_docker_image) (content-approved Amendment-9 candidate)
```

Manually initiated external actions (anything the implementing agent
triggered outside the normal push/PR flow, e.g. a manual workflow dispatch):
NONE

External writes/publication (releases, tags, packages, registries, third-party
services): NONE

## 5. Implementation decisions

List decisions made within the approved design.

Activation-epoch correction, approval-state and recovery stages (historical):

1. An activation epoch is expressed as the half-open routing-generation
   interval `[activation_generation, deactivation_generation)`; an open epoch
   is the absence of a deactivation boundary (Amendment 1 A2.2).
2. Closing an epoch is a write-once durable close/deactivation fact or an
   equivalently strong append-only close record; once closed, its boundary
   cannot change and the epoch cannot reopen, so the immutability rule never
   implies rewriting an already-written boundary (Amendment 3 C7).
3. To make "closing never rewrites historical eligibility" (A2.8) a property of
   the serialized mechanism rather than an aspiration, the ADR requires the
   mechanism to allocate each deactivation boundary so that closing an epoch
   never removes eligibility from an event that captured a routing generation
   inside it.
4. Retirement versus reactivation follows the durable serialization order
   exactly as A2.6 specifies: retirement after a reactivation closes the open
   epoch; reactivation after retirement is rejected.
5. Any epoch identifier is storage/audit metadata subordinate to `route_key`
   and is excluded from the `(event_id, route_key)` disposition key (A2.11).
6. Section 3.2 states the disposition model already ruled by the CTO and
   recorded in section 3.5 and invariant 8: exactly one disposition per
   `(event_id, route_key)`; a materialized disposition points to exactly one
   job; a terminal non-materialized disposition points to no job.
7. A retired route's successor, if separately authorized, is a different
   logical route with its own epochs and dispositions and receives earlier
   events only through explicitly authorized backfill/replay; the ADR defines
   no successor mechanism.
8. At the activation-epoch candidate stage the header dropped the earlier
   `Approved by` / `Approval date` fields, and the approval-state commit added
   fresh values (decision 12). Section 12 binds earlier approvals to the
   historical versions without embedding their identities; those identities
   live in this report and the owning issue.
9. Section 12's approval, authority and supersession text is written to remain
   true in both the `PROPOSED` and `APPROVED` states, so an approval-state
   delta is limited to the status line, the approver/date and the sentence
   that says the corrected content is not yet approved.
10. At the approval-state stage the decision-register header `Last updated`
    line was left at `2026-09-30`, because that authorization permitted only
    the ADR-0012 row to change. Final-head automated review of the
    approval-state head showed that this left the active register's own
    freshness metadata stale once the row recorded an approval on 2026-10-05;
    that finding was accepted as blocking. Leaving the header unchanged was
    therefore not the correct final state: the recovery commit advanced it to
    `Last updated: 2026-10-05` as its only register change (Amendment 4 E6).
11. The report is restructured to the template rather than appended to, so its
    durable current state is readable without implying that historical
    approvals cover a later corrected version.
12. The approval-state header used the same field order as the earlier
    approved ADR-0012 versions (`Status`, `Date`, `Approved by`,
    `Approval date`, `Decision owner`), and section 12 repeated the approver and
    date, so the approval metadata appeared in both places as before.
13. The approval date carried by the activation-epoch version was the date of
    the durable CTO content-approval record, 2026-10-05, as Route B requires;
    it was neither back-dated nor anticipated.
14. The recovery report did not self-pin the recovery commit's SHA, its tree or
    its own final blob (Amendment 5 F2); they were verified after push and
    recorded in the post-publication handoff. Section 1 records them here as
    historical facts because the recovery head is the Amendment-6/7 candidate's
    parent.
15. Final-head automated review outcomes are recorded here substantively
    (section 2, section 14); their review and comment identifiers remain
    GitHub-owned lifecycle evidence under `ADR-0005` and are not transcribed
    into source (Amendment 5 F3.5).

Amendment-6/7 candidate stage (historical content-approved candidate
`b596536e7db0d6b8ace0a42fdd26bff42cabf104`). Items 16, 19, 20, 22 and 25 were
interpretations of the approved specification recorded for explicit control
inspection; control inspected and accepted all five before the CTO
candidate-content approval:

16. The `IN_EPOCH_CONTRACT_MISMATCH` predicate is keyed to events of the epoch
    revision's consumed event kind whose payload/schema version is outside its
    accepted-version set. An event of a kind the revision does not consume is
    outside that route's contract and is neither an obligation nor an anomaly
    for it; otherwise every event of every unrelated kind would be a mismatch
    for every route. Whether producers still emit the consumed kind at all is
    governed by the event-emission floor, which already requires every serving
    and supported rollback producer to emit it.
17. The semantic signature records the absence of a compatibility policy as
    part of the signature, so a revision with a policy and one without remain
    distinct (Amendment 7 H6).
18. The database-level atomic property is stated as two acceptable forms: a
    database-enforced uniqueness constraint over the route's canonical
    semantic signature (or a deterministic canonical encoding of it), or
    revision creation serialized through the route's durable serialization
    mechanism with the equivalence check and insert in one transaction. An
    application-level read-then-insert is stated to be insufficient. Exact
    table, index and encoding names remain implementation-specification detail
    (Amendment 7 H6, clarification 4).
19. Historical-enrollment validation is enumerated field by field over the
    semantic signature (clarification 8). For the route-to-materializer/job
    contract version, the check is that deterministic compatibility identifies
    at least one compatible materializer for the event's kind and version, so
    enrollment never creates a disposition that no materializer can honour; a
    refusal leaves the `(event_id, route_key)` slot free for a later attempt.
20. Withdrawal from future selection is defined to prevent only new epochs from
    referencing the revision and its selection for new historical enrollment;
    it never removes the revision from materializing or reporting obligations
    already eligible through epochs that reference it. It is refused or held
    while it would strand undispositioned obligations or unresolved mismatch
    anomalies (Amendment 7 H8, clarification 5).
21. Materializer coverage is stated per revision and per accepted version that
    is currently emittable or has outstanding obligations; a newly emittable
    accepted version must be verified before producers may emit it
    (Amendment 7 H10, clarification 1 and its control clarification).
22. In-epoch mismatch resolution is append-only evidence linked to the anomaly;
    the anomaly and its detection evidence are never deleted (clarification 6).
    A mismatch that enrollment resolves receives the one allowed disposition
    under the selected compatible revision; one resolved by approved
    reconciliation/divergence receives none.
23. New invariants 35-40 are appended after invariant 34 rather than inserted
    after invariant 8, so the numbering of invariants 9-34, which other text
    and earlier review records cite, is unchanged. Invariants 3-6 and 8 are
    reconciled in place.
24. Section 3.2 gains five unnumbered subsections - route-contract revisions,
    in-epoch contract mismatch, historical enrollment/backfill/replay, revision
    withdrawal and route retirement, and the renamed "Event-emission floor and
    producer-version coverage" - following the existing unnumbered-subsection
    style of section 3.8. The "route introduced by a later release" paragraph
    moves into the historical-enrollment subsection.
25. The decision-register header `Last updated` line advances to
    `2026-10-06`, the date on which the candidate changed the ADR-0012 row.
    This keeps the active register's freshness metadata truthful, the property
    whose absence caused the accepted blocking final-head finding in decision
    10. It is a header line in the authorized register path, not another row.
26. The changelog gains one new entry instead of rewording the existing
    `THOTH-ASYNC-01-ADR-01-CORRECTION` entry, so every historical entry,
    including the record of the 2026-10-05 approval, stays byte-preserved; the
    new entry states which exact earlier versions those entries describe
    (Amendment 3 C8 pattern).
27. Section 12 binds both the 2026-09-30 approval and the 2026-10-05 approval
    to the historical earlier versions without embedding their identities,
    consistent with decision 8.
28. The candidate-stage report did not self-pin the candidate commit's SHA, its
    tree or its own final blob; it recorded only non-self-referential evidence
    and the ADR-0012, decision-register and changelog blobs, which did not
    depend on its bytes. Section 1 now records those candidate identities as
    historical facts because the candidate is the approval-state parent.

Amendment-6/7 approval-state stage (historical approval-state head
`609a01427847233b7280557baa61825ac2079865`):

29. The approval-state header uses the same field order as the earlier approved
    ADR-0012 versions (`Status`, `Date`, `Approved by`, `Approval date`,
    `Decision owner`), and section 12 repeats the approver and date.
30. The approval date carried by the ADR is the date of the CTO exact
    candidate-content approval record, 2026-10-06, as Route B requires; it is
    neither back-dated nor anticipated.
31. Section 12's approval-state delta is limited to its two leading
    candidate-state paragraphs, replaced by the approval statement, the
    approver/date and one paragraph stating that the approval does not by
    itself make the ADR repository-authoritative and that ADR-0013
    programme-local reliance on this exact version is separate and not
    effective until its own conditions are satisfied. The
    historical-approval, architecture-only, authority-condition,
    partial-supersession and amendment paragraphs were already true in the
    approved state (decision 9) and are unchanged.
32. The decision-register `Last updated: 2026-10-06` header already carries the
    date of this approval-state change, so it is unchanged; only the ADR-0012
    row changes.
33. Only the `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` changelog entry
    changes, following the earlier approval-state wording pattern; every other
    entry, including the historical approval entries, is byte-identical.
34. Architecture no-drift is proven by an exact normalized comparison rather
    than by assertion (section 9, "Amendment-6/7 approval-state validation").
35. The approval-state report did not self-pin the approval-state commit's
    SHA, its tree or its own final blob; it recorded only non-self-referential
    evidence and the ADR-0012, decision-register and changelog blobs, which did
    not depend on its bytes. Section 1 now records those approval-state
    identities as historical facts because that head is the recovery parent.
36. Approval-state implementation authorization #958 comment `6022486352`
    contained an erroneous instruction requiring the report to copy the GitHub
    identifier of the CTO content-approval record into repository source. The
    repository-authoritative Route-B rule in
    `docs/engineering/decisions/README.md` says the approval-state commit must
    not copy content-approval, exact-final-blob approval, review or merge
    identifiers into repository files; those remain GitHub-owned lifecycle
    evidence under `ADR-0005`, and repository records reference the owning
    issue and pull request. The implementing agent raised the conflict before
    committing and, on instruction, followed the task instruction, so the
    approval-state report copied the identifier. Repository doctrine takes
    precedence; control adjudication on #958 found that control-record defect
    nonconforming and required the report-only recovery below.

Report-only control-record recovery (historical recovery head
`22bd94dc959f11fe49c8bef8cd2a836140aadff7`):

37. The recovery removes every occurrence of the copied content-approval
    identifier and records the content approval only through durable facts:
    owning issue #958, approval date 2026-10-06, candidate head
    `b596536e7db0d6b8ace0a42fdd26bff42cabf104` and candidate ADR-0012 blob
    `cd897d576522a741c3dfa82f8de2f8d80d4288f6`. No other content-approval,
    exact-final-blob approval, review or merge identifier is substituted, and
    the adjudication record that required the recovery is identified only
    through owning issue #958.
38. The recovery changes only this report. ADR-0012
    (`a7d95ca918f418d596432071c8ba78a33c2630a9`), its architecture, approval
    state, approver and approval date, the decision register and the changelog
    are byte-identical to the approval-state head; the recovery restores
    conformity with Route-B doctrine and is not an ADR amendment, another
    candidate or another approval-state commit.
39. Because the report then lived in the recovery commit, statements that
    described the approval-state commit as the commit containing the report
    were reframed as history, and the approval-state commit's identities, the
    recovery parent, were recorded in section 1 as historical facts. The
    recovery report did not self-pin the recovery commit's SHA, its tree or
    its own final blob; section 1 records them here as historical facts
    because the recovery head is the transient-state correction's parent.

Report-only transient-state correction (historical correction head
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`):

40. The committed report records review-thread evidence only as durable
    substantive dispositions and as historical stage-time inspection evidence
    phrased as such. GitHub, through PR #960, owns the current number and
    resolved/unresolved state of review threads and their later adjudication
    (`docs/engineering/AGENTS.md`; `ADR-0005`). No sentence of this report
    depends for its truth on a thread remaining unresolved, and no later
    reply, resolution, dismissal or adjudication of a thread requires a
    further source edit to keep this report truthful.
41. The final-head automated review of the recovery head is recorded
    substantively (section 2, section 14) without its review, thread or
    comment identifiers, consistent with decision 15 (Amendment 5 F3.5). The
    control adjudication that classified its findings is identified only
    through owning issue #958.
42. The topology finding is recorded as rejected because its premise did not
    match the authoritative GitHub PR head/parent graph. The report's Route-B
    source lineage - the correction commit on
    `22bd94dc959f11fe49c8bef8cd2a836140aadff7`, on
    `609a01427847233b7280557baa61825ac2079865`, on
    `b596536e7db0d6b8ace0a42fdd26bff42cabf104` - is unchanged, and the
    non-head object cited by that finding is not adopted as source topology.
    The cumulative six-path PR footprint against refreshed `feature/worker`
    does not mean that any single commit changed six paths from its parent.
43. The correction changes only this report. ADR-0012
    (`a7d95ca918f418d596432071c8ba78a33c2630a9`), its architecture, approval
    state, approver and approval date, the decision register and the changelog
    were byte-identical to the recovery head; the correction was not an ADR
    amendment, another candidate or another approval-state commit. Because
    the report then lived in the correction commit, statements that described
    the recovery commit as the commit containing the report were reframed as
    history, and the recovery commit's identities, the correction parent,
    were recorded in section 1 as historical facts. The correction report did
    not self-pin the correction commit's SHA, its tree or its own final blob;
    section 1 records them here as historical facts because the correction
    head is the factual clarification's parent.

Factual clarification and durable-report correction (historical clarification
head `eb3129f13a9c858b3c012ee182315801f0b8563a`):

44. The ADR-0012 change was confined to the CTO-classified factual
    clarification of prior approved blob
    `a7d95ca918f418d596432071c8ba78a33c2630a9`: the unconditional sentence
    "exactly one durable disposition exists for each `(event_id,
    route_key)`" in section 3.2, section 3.5 and invariant 8 is replaced by
    "at most one durable disposition may exist for a given
    `(event_id, route_key)`", each time paired with the already-selected
    completion rule that an eligible route obligation remains owed until it
    has received exactly one disposition. The smallest wording that removes
    the contradiction was chosen; no other sentence of the ADR changes.
45. The clarification expresses semantics the ADR already selected rather
    than selecting new behaviour: the surrounding rule that the engine must
    eventually establish a durable disposition for every route for which a
    committed event is eligible is unchanged (section 3.2); in-epoch
    contract mismatch, refused historical enrollment and approved
    reconciliation/divergence already stated that no disposition is created
    and that the `(event_id, route_key)` slot stays unused; validated
    historical enrollment already created "the one allowed" disposition; and
    the uniqueness boundary across epochs and revisions, duplicate prevention
    and job idempotency were already stated as they remain. Had the
    contradiction required selecting new behaviour, the stage would have
    stopped as exceeding the factual-clarification boundary.
46. ADR-0012 kept `Status: APPROVED`, `Approved by: Javi, CTO` and
    `Approval date: 2026-10-06`, and section 12 was unchanged, because the
    Amendments rules of `docs/engineering/decisions/README.md` let a factual
    clarification update an approved ADR in place with the existing
    architecture approval remaining; no `PROPOSED` candidate, approval-state
    commit, changelog entry or register change belonged to that stage, and
    the clarified ADR blob is recorded in section 1 because it does not
    depend on the report's bytes.
47. The committed report records the ADR-0012 authority condition and the
    distinct controlled lifecycle categories (independent exact-head review,
    CTO merge authorization where required, guarded merge, programme-local
    reliance, migration, deployment, activation and observation) but not
    which PR #960 gate is currently pending, satisfied or blocked: GitHub,
    through #958 and PR #960, owns that state (`docs/engineering/AGENTS.md`;
    `ADR-0005`). Historical stages record what they did or did not perform
    as stage-time evidence only. No sentence of this report becomes false
    when PR metadata is reconciled, the PR changes Draft/Ready state, an
    automated or independent review runs, a thread is adjudicated, merge
    authorization is recorded or the PR merges.
48. The final-head automated review of the transient-state correction head
    and the CTO classification are recorded substantively (section 2,
    section 14) without review, thread or review-comment identifiers,
    consistent with decisions 15 and 41; the classification and authorization
    records are identified through owning issue #958 as every stage
    authorization is. Because the report then lived in the clarification
    commit, statements that described the transient-state correction commit
    as the commit containing the report were reframed as history, and that
    commit's identities, the clarification parent, were recorded in
    section 1 as historical facts. The clarification report did not self-pin
    the clarification commit's SHA, its tree or its own final blob; section 1
    records them here as historical facts because the clarification head is
    the Amendment-8 candidate's parent.

Amendment-8 Route-B candidate (historical content-approved candidate
`42ae90a5932cf24ce90679196888cc3d83d88db8`). Items 53 and 57 were
interpretations of the approved specification recorded for explicit control
inspection; control inspected the published candidate and the CTO approved its
exact content:

49. The Amendment-8 rule is stated once, in the "Revision withdrawal and route
    retirement" subsection of section 3.2, and the places that depend on it
    point to it: the per-`route_key` serialization paragraph states that
    withdrawal and new selection share one database-atomic order that
    composes with that serialization; activation gate 4 states that "not
    withdrawn" is decided inside that order; and historical-enrollment
    validation states that the selected revision's availability is decided
    inside that order in the transaction that would create the disposition.
    This places the correction where the independent review located the gap
    (the serialization rule covered epoch open, close and retirement but not
    withdrawal) without restating the rule in several places.
50. The general rule is carried, not only its enumerated floor: the subsection
    names first activation, reactivation and revision-transition epoch
    opening, first historical enrollment and the withdrawal itself as the
    minimum, and then states that every other operation that would create a
    new durable selection of or reference to the revision is covered
    (Amendment 8 I3/I4; independent-review observation 3). Non-durable
    preselection, preview or other choices are excluded exactly as Amendment
    8 I4 excludes them.
51. Both commit orders are written as Amendment 8 I5 defines them, including
    that a withdrawal-first epoch opening is refused before the epoch exists,
    that a withdrawal-first enrollment is refused before any disposition or
    job exists and leaves `(event_id, route_key)` unused, that a
    selection-first reference remains valid historical state, and that a
    losing transaction re-reads durable state before any retry and never
    commits from a stale pre-withdrawal observation.
52. The mechanism is left to the implementation specification. The ADR names
    the two forms Amendment 8 I6 permits (extending the existing
    per-`route_key` serialization boundary, or another database-enforced
    mechanism with equivalent atomic ordering), states that an
    application-level check-then-insert without database-enforced ordering is
    insufficient, and lists the composition requirements of I6 without
    selecting a table, lock or constraint.
53. Withdrawal semantics are otherwise unchanged (Amendment 8 I7). The
    candidate adds no withdrawal reversibility, no equivalent-signature
    re-enablement rule, no second lifecycle order and no new route
    abstraction (independent-review observations 2 and 4). For the reviewer's
    strand-check timing observation (observation 1), the candidate reflects
    only what Amendments 1-8 already determine: a selection that linearizes
    first makes the later withdrawal subject to every existing
    strand-prevention rule (I5), stated in the selection-first order and
    exercised by a validation case covering obligations that became eligible
    through an epoch whose opening linearized before the withdrawal. Whether
    the strand-prevention evaluation of a withdrawal may be stale relative to
    events or anomalies that commit concurrently with the withdrawal, which
    are not revision selections, is not determined by Amendments 1-8; it is
    left outside this correction and reported to control rather than decided
    here.
54. Explicit replay or current-state work for an already-dispositioned event is
    stated to be outside the first-selection rule and governed by section
    3.5, and not to become a new route disposition merely because a revision
    has been withdrawn (Amendment 8 I4). No statement is added about whether
    such work is refused.
55. Invariant 41 is appended after invariant 40 rather than inserted after
    invariant 39, so the numbering of invariants 1-40, which other text and
    earlier review records cite, is unchanged (decision 23 pattern).
56. The nine Amendment 8 I8 validation cases are added to section 11
    immediately after the existing revision-withdrawal tests, grouped so that
    each numbered case is identifiable: cases 1-2 (first activation, both
    orders), case 3 (reactivation and revision transition, both orders), case
    4 (first enrollment, both orders), case 5, case 6, case 7, case 8 and case
    9, the last naming equivalent-revision convergence where it shares the
    per-route mechanism (independent-review observation 5). All Amendment 1-7
    validation requirements are retained.
57. The decision-register header `Last updated` line advances to
    `2026-10-07`, the date on which the candidate changed the ADR-0012 row,
    for the reason recorded in decision 25: leaving it stale caused the
    accepted blocking final-head finding in decision 10. It is a header line
    in the authorized register path, not another row.
58. The header drops `Approved by` / `Approval date` and section 12 restores
    the two candidate-state paragraphs used by the earlier Route-B candidates
    (decisions 8 and 9), adds the ordering rule to the description of this
    material correction and binds the 2026-10-06 approval, like the
    2026-09-30 and 2026-10-05 approvals, to the historical earlier version
    without embedding its identities. The architecture-only,
    authority-condition, partial-supersession and amendment paragraphs are
    unchanged, so an approval-state delta would again be limited to the
    status line, the approver/date and the sentence that says the corrected
    content is not yet approved.
59. The changelog gains one new entry instead of rewording the existing
    `THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` entry, so every
    historical entry, including the record of the 2026-10-06 approval, stays
    byte-preserved; the new entry states which exact earlier versions those
    entries describe (Amendment 3 C8; decision 26).
60. The candidate-stage report did not self-pin the candidate commit's SHA,
    its tree or its own final blob; it recorded only non-self-referential
    evidence and the candidate ADR-0012, decision-register and changelog
    blobs, which did not depend on its bytes. Because the report then lived in
    the candidate commit, statements that described the clarification commit
    as the commit containing the report were reframed as history, and that
    commit's identities, the candidate parent, were recorded in section 1 as
    historical facts; section 1 now records the candidate's identities as
    historical facts because the candidate is the approval-state parent.
    `R-EB31-01` is recorded by its control-assigned name,
    and Amendment 8, its independent review receipt and the candidate
    authorization by their #958 comment identifiers, as the specification
    chain in section 2 already records every amendment, review receipt and
    authorization; no review, thread or review-comment identifier of the
    final-head automated review is copied.

Amendment-8 approval-state stage (historical approval-state head
`e2588f7b7e63a5c71968d87a3409f0ff48635e6d`):

61. The approval-state header uses the same field order as the earlier approved
    ADR-0012 versions (`Status`, `Date`, `Approved by`, `Approval date`,
    `Decision owner`), and section 12 repeats the approver and date.
62. The approval date carried by the ADR is the date of the CTO exact
    candidate-content approval record, 2026-10-07, as Route B requires; it is
    neither back-dated nor anticipated.
63. Section 12's approval-state delta is limited to its two leading
    candidate-state paragraphs, replaced by the approval statement, the
    approver/date and one paragraph stating that the CTO approved this exact
    corrected content on 2026-10-07 as the architecture content represented by
    candidate ADR-0012 blob `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`,
    that this version differs from that candidate only in its approval-state
    representation, that the approval does not by itself make the ADR
    repository-authoritative and that ADR-0013 programme-local reliance on
    this exact version is separate and not effective until its own conditions
    are satisfied. The candidate blob is named because the approval-state
    authorization requires the approved content to be identified by it; a Git
    blob identity is content identity, not a GitHub lifecycle identifier of
    the kind Route-B doctrine keeps out of repository files. The
    historical-approval, architecture-only, authority-condition,
    partial-supersession and amendment paragraphs were already true in the
    approved state (decision 58) and are unchanged.
64. The approval-state authorization asked the report to record the GitHub
    comment identifier of the CTO content-approval record. The
    repository-authoritative Route-B rule in
    `docs/engineering/decisions/README.md` says the approval-state commit
    must not copy content-approval, exact-final-blob approval, review or merge
    identifiers into repository files, and the control adjudication on #958
    of the earlier Amendment-6/7 approval-state report found exactly that
    transcription nonconforming and required a report-only recovery
    (decisions 36-37). Repository doctrine outranks the task instruction, so
    this report records the content approval only through owning issue #958,
    approval date 2026-10-07, candidate head
    `42ae90a5932cf24ce90679196888cc3d83d88db8` and candidate ADR-0012 blob
    `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`, without the comment
    identifier; the departure from the instruction is reported in the
    post-publication handoff.
65. The decision-register `Last updated: 2026-10-07` header already carries the
    date of this approval-state change, so it is unchanged; only the ADR-0012
    row changes.
66. Only the `THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION` changelog
    entry changes, following the earlier approval-state wording pattern; every
    other entry, including the historical approval entries, is byte-identical.
67. Architecture no-drift is proven by the exact normalized comparison in
    section 9 ("Amendment-8 approval-state validation"), not by assertion.
68. The approval-state report did not self-pin the approval-state commit's
    SHA, its tree or its own final blob; it recorded only non-self-referential
    evidence and the approval-state ADR-0012, decision-register and changelog
    blobs, which did not depend on its bytes. Because the report then lived in
    the approval-state commit, statements that described the candidate as the
    commit containing the report were reframed as history, and the
    candidate's identities, the approval-state parent, were recorded in
    section 1 as historical facts; section 1 now records the approval-state
    commit's identities as historical facts because that head is the
    Amendment-9 candidate's parent.

Amendment-9 Route-B candidate (historical content-approved candidate
`8699af5635b8f18691ceaeccd3b24707a71ef4f2`). Item 75 was an interpretation of the
approved specification recorded for explicit control inspection; control
inspected the published candidate, confirmed that it created no new coalescing
permission, and the CTO approved its exact content:

69. The Amendment-9 rule is stated once, in a new unnumbered subsection
    "Coalescing-open and coalescing-sealed jobs" at the end of section 3.5,
    following the unnumbered-subsection style of sections 3.2 and 3.8, and
    the places that depend on it point to it: the section-3.5 sentence that
    permits several materialized dispositions to point to one not-yet-started
    job now defines "not yet started" as coalescing-open until the first
    successful claim/start commits; the idempotency-conflict attachment is
    bound to the boundary; and section 3.15's coalescing paragraph and
    invariant 8 reference it. The heading outline gains exactly that one
    heading.
70. One semantic anchor is used: the subsection states that the
    "not-yet-started job" that coalescing may attach to and the "first
    successful claim/start" that seals it refer to the same durable
    transition, and that the seal is a property of the logical job, not of an
    attempt or a disposition (independent-review observations 1 and 4). The
    persisted encoding is left to the implementation specification, with the
    statement that a separate sealed flag is not required where the canonical
    job state, attempt or claim transition already enforces the property.
71. The database-atomic order is stated as Amendment 9 J5 states it: one
    serialization, lock or compare-and-set boundary or database-enforced
    equivalent; the forbidden observe-then-attach sequence; the attachment
    write itself proving the job is still open; the first claim/start
    participating so that no concurrent attachment commits after it; an
    application-level pre-check insufficient; no SQL, lock or constraint
    selected.
72. Both commit orders are written as J6 defines them, including that the
    claim/start-first refusal commits no disposition, leaves the
    `(event_id, route_key)` slot unused, leaves the obligation owed and
    visible and follows the existing idempotency-conflict error and
    attention/recovery path, and that the losing router re-reads durable
    state before any retry under existing semantics, with no new logical-job
    identity and no new replay or current-state rule.
73. The aborted-claim rule (J7) binds sealing to the durable committed
    transition and states that an attachment may still succeed after rollback
    when every ordinary condition holds.
74. Input/effect completeness (J8) is stated mechanism-neutrally: the kind's
    coalescing contract defines representation; `CURRENT_STATE` may use
    current-state/effect-fingerprint semantics; another kind may use a
    retained input set or equivalent; handlers are not required to enumerate
    dispositions; `CURRENT_STATE`/`REVISION_BOUND` semantics, effect
    fingerprints, target-scoped concurrency and applied-revision evidence are
    unchanged.
75. The attachment-scope sentence (authorization section 5; independent-review
    observation 5) is written conditionally: every attachment path "where this
    ADR already permits that attachment", including historical
    enrollment/backfill/replay and directly created logical jobs, uses the
    boundary, and the boundary "does not itself decide whether any path may
    coalesce". No path is newly permitted or newly forbidden to coalesce;
    whether a path may attach remains governed by the existing explicit
    durable coalescing rule and the idempotency-conflict rule. This
    interpretation is recorded for control inspection because the
    authorization requires HOLD if the sentence would have needed a new
    coalescing-permission decision; it did not.
76. The seal is stated to be earlier than and distinct from `EFFECT_STARTED`
    (J9), with the composition list of J9 and the statement that no second
    job identity, job lifecycle, route lifecycle, claim-lock mode or second
    order is introduced.
77. Invariant 42 is appended after invariant 41, and invariant 8 gains the
    coalescing-open qualifier and a reference to invariant 42; invariants 1-41
    are otherwise unrenumbered and unchanged (decision 23 pattern).
78. The decision-register header `Last updated: 2026-10-07` already carries
    the date on which the candidate changes the ADR-0012 row, so it is
    unchanged (decisions 25 and 57 pattern); only the ADR-0012 row changes.
79. The nine Amendment 9 J10 validation cases are added to section 11
    immediately after the existing coalescing tests as eight bullets: cases
    1-2 (both orders, with separate router and worker instances), case 3
    (with explicit `CURRENT_STATE` and retained-input evidence), case 4 (with
    the idempotency-conflict path), case 5, case 6 (several router instances),
    case 7, case 8 (with seal persistence across crash and restart) and case
    9; these carry the independent review's six evidence refinements
    (observations 1-6) without adding architecture. All Amendment 1-8
    validation requirements are retained.
80. The header drops `Approved by` / `Approval date` and section 12 restores
    the two candidate-state paragraphs used by the earlier Route-B candidates
    (decisions 8, 9 and 58), adds the coalescing boundary to the description
    of this material correction and binds the 2026-10-07 approval, like the
    2026-09-30, 2026-10-05 and 2026-10-06 approvals, to the historical earlier
    version without embedding its identities. The changelog gains one new
    entry and every historical entry, including the record of the 2026-10-07
    approval, stays byte-preserved (decisions 26 and 59). The candidate-stage
    report did not self-pin the candidate commit's SHA, its tree or its own
    final blob; because the report then lived in the candidate commit,
    statements that described the Amendment-8 approval-state commit as the
    commit containing the report were reframed as history, and that commit's
    identities, the candidate parent, were recorded in section 1 as
    historical facts; section 1 now records the candidate's identities as
    historical facts because the candidate is the approval-state parent. The
    exact-final-blob
    approval of the approval-state ADR blob is recorded through owning issue
    #958, its date and the blob and head it approved; its GitHub comment
    identifier is not copied, consistent with decisions 37 and 64, no review,
    thread or review-comment identifier of the final-head automated review is
    copied, and `R-E258-01` is recorded by its control-assigned name.

Amendment-9 approval-state stage (the commit that contains this report):

81. The approval-state header uses the same field order as the earlier
    approved ADR-0012 versions (`Status`, `Date`, `Approved by`,
    `Approval date`, `Decision owner`), and section 12 repeats the approver
    and date.
82. The approval date carried by the ADR is the date of the CTO exact
    candidate-content approval record, 2026-10-08, as Route B requires; it is
    neither back-dated nor anticipated.
83. Section 12's approval-state delta is limited to its two leading
    candidate-state paragraphs, replaced by "Current decision state:
    **APPROVED**.", the approver/date and one paragraph stating that the CTO
    approved this exact corrected content on 2026-10-08 as the architecture
    content represented by candidate ADR-0012 blob
    `0fc33df9aca61a2c82af452426f5574bfd3493cf`, that this version differs
    from that candidate only in its approval-state representation, that the
    approval does not by itself make the ADR repository-authoritative and
    that ADR-0013 programme-local reliance on this exact version is separate
    and not effective until its own conditions are satisfied (decision 63
    pattern; the authorization requires the approved content to be identified
    by the candidate blob). The historical-approval, architecture-only,
    authority-condition, partial-supersession and amendment paragraphs were
    already true in the approved state (decision 80) and are unchanged.
84. The content approval is recorded only through owning issue #958, approver
    Javi, CTO, approval date 2026-10-08, candidate head
    `8699af5635b8f18691ceaeccd3b24707a71ef4f2` and candidate ADR-0012 blob
    `0fc33df9aca61a2c82af452426f5574bfd3493cf`. Its GitHub comment identifier
    is not copied into repository source: the repository-authoritative Route-B
    rule in `docs/engineering/decisions/README.md` forbids it, the control
    adjudication on #958 of the earlier Amendment-6/7 approval-state report
    applied that rule to this task (decisions 36-37 and 64), and the
    approval-state authorization for this stage requires the same.
85. The decision-register header `Last updated` advances to `2026-10-08`,
    the date on which the approval-state commit changes the ADR-0012 row, as
    the approval-state authorization requires and for the reason recorded in
    decision 25; otherwise only the ADR-0012 row changes.
86. Only the `THOTH-ASYNC-01-ADR-01-COALESCING-SEAL-CORRECTION` changelog
    entry changes, following the earlier approval-state wording pattern and
    naming the approver as the authorization requires; every other entry,
    including the historical approval entries, is byte-identical.
87. Architecture no-drift is proven by the exact normalized comparison in
    section 9 ("Amendment-9 approval-state validation"), not by assertion.
88. This report does not self-pin the approval-state commit's SHA, its tree or
    its own final blob; it records only non-self-referential evidence and the
    approval-state ADR-0012, decision-register and changelog blobs, which do
    not depend on its bytes. Because the report now lives in the
    approval-state commit, statements that described the candidate as the
    commit containing this report are reframed as history, and the
    candidate's identities, now the approval-state parent, are recorded in
    section 1 as historical facts.

List any deviation from the specification requiring authorization:

- An earlier unpublished local activation-epoch candidate,
  `a0e0c2cd6809fb2b88d15235d07086f32e4a699c`, carried a version of this report
  that described the push, the task-branch fast-forward, the PR description
  update, the PR head and the PR diff as already complete before they had
  happened. Control rejected it before publication and authorized a
  replacement in #958 comment `5998119577`. The rejected candidate was not
  pushed and is not a parent of any published commit. The replacement
  `95769537cdc9871e42f9f3e5443604a96beb74da` was rebuilt from the same two
  authorized parents with the same ADR-0012, decision-register and changelog
  bytes; only the report differed, to correct that lifecycle wording.
- Activation-epoch approval-state stage: the approval-state authorization asked
  the report to record the identifier of the CTO content-approval comment. The
  repository-authoritative Route-B rule in
  `docs/engineering/decisions/README.md` says the approval-state commit must
  not copy content-approval, exact-final-blob approval, review or merge
  identifiers into repository files, and that repository records reference the
  owning issue and pull request instead. Control selected the
  README-conforming resolution before the approval-state commit: the report
  identifies the content approval by its owning issue (#958), its date and the
  exact candidate head and ADR blob it approved, without restating its comment
  identifier.
- Recovery stage: NONE. The recovery followed Amendments 4-5 and its separate
  implementation authorization exactly.
- Amendment-6/7 candidate stage: NONE. No fifth path, new file, deletion,
  second commit, amend, rebase, squash, force push or history rewrite was
  needed or performed. Decisions 16, 19, 20, 22 and 25 were recorded for
  control inspection as interpretations within the authorized scope, not as
  deviations, and control accepted them.
- Amendment-6/7 approval-state stage (historical, corrected): the
  approval-state report copied the CTO content-approval record's GitHub
  identifier into repository source because the approval-state authorization
  erroneously required it, contrary to the repository-authoritative Route-B
  rule (decision 36). Control adjudication on #958 found that control-record
  defect nonconforming, and the historical report-only recovery
  `22bd94dc959f11fe49c8bef8cd2a836140aadff7` corrected it; it is not a current
  source deviation. No fifth path, new file,
  deletion, architecture change, second commit, amend, rebase, squash, force
  push or history rewrite occurred at that stage.
- Control-record recovery stage: NONE. The recovery followed its separate
  authorization exactly; no second path, new file, deletion, ADR, register or
  changelog change, second commit, amend, rebase, squash, force push or history
  rewrite was needed or performed at that stage.
- Transient-state correction stage: NONE. The correction followed its
  separate authorization exactly; no second path, new file, deletion, ADR,
  register or changelog change, second commit, amend, rebase, squash, force
  push or history rewrite was needed or performed at that stage.
- Factual-clarification stage: NONE. The clarification followed its separate
  authorization exactly; no third path, new file, deletion, register or
  changelog change, ADR change outside the classified scope, second commit,
  amend, rebase, squash, force push or history rewrite was needed or performed
  before the clarification commit was created.
- Amendment-8 candidate stage: NONE. No fifth path, new file, deletion, ADR
  change outside Amendment 8 and the candidate state, second commit, amend,
  rebase, squash, force push or history rewrite was needed or performed
  before the candidate commit was created. Decisions 53 and 57 were recorded
  for control inspection as interpretations within the authorized scope, not
  as deviations, and control accepted them.
- Amendment-8 approval-state stage: the approval-state authorization asked the
  report to record the GitHub identifier of the CTO content-approval comment.
  The repository-authoritative Route-B rule in
  `docs/engineering/decisions/README.md` says the approval-state commit must
  not copy content-approval identifiers into repository files, and repository
  records reference the owning issue and pull request instead; the control
  adjudication on #958 of the earlier Amendment-6/7 approval-state report
  applied that rule to this task (decision 36). This report therefore
  identifies the content approval by its owning issue (#958), its date and
  the exact candidate head and ADR blob it approved, without restating its
  comment identifier (decision 64). No fifth path, new file, deletion,
  architecture change, second commit, amend, rebase, squash, force push or
  history rewrite was needed or performed before the approval-state commit was
  created.
- Amendment-9 candidate stage: NONE. No fifth path, new file, deletion, ADR
  change outside Amendment 9 and the candidate state, second commit, amend,
  rebase, squash, force push or history rewrite was needed or performed
  before the candidate commit was created. Decision 75 was recorded for
  control inspection as an interpretation within the authorized scope, not as
  a deviation, and control confirmed it.
- Amendment-9 approval-state stage: NONE. The approval-state transformation
  followed its separate authorization exactly; no fifth path, new file,
  deletion, architecture change, second commit, amend, rebase, squash, force
  push or history rewrite was needed or performed before the commit that
  contains this report was created.

## 6. Database and migration effects

Migration added: NO

No schema, data, index, constraint or `thoth-api/src/schema.rs` effect. The
activation-epoch and route-contract-revision model, including its required
database-level revision-equivalence property and its required database-atomic
ordering of revision withdrawal against new revision selection and of
coalescing attachment against a job's first claim/start, is architecture for a
future
separately authorized shared-engine implementation; it creates no migration
obligation by itself and names no table, column or index.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE
Backwards compatibility: unaffected; documentation/architecture only
Deprecations: NONE
Cross-repository dependencies: NONE for the corrections, clarification and
candidate recorded here. The verified BE-04
per-consumer impact record for the ADR's later legacy-contract retirement is
preserved unchanged in Appendix A; neither the activation-epoch correction,
the route-contract-revision correction, the withdrawal/selection ordering
correction nor the coalescing-seal ordering correction changes a released
contract or adds a consumer. The route, revision, anomaly and materializer-compatibility concepts
are internal to the future shared engine; any later external read or executor
contract derived from them needs its own specification, merged upstream
contract and downstream task.

## 8. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: NONE
Negative authorization tests: NOT APPLICABLE
Secret or personal-data handling: NONE
Security limitations: NONE

## 9. Tests and checks

Record exact commands and outcomes.

The subsections "Formatting" through "Content checks" record the
candidate-stage validation of the historical activation-epoch candidate
`95769537cdc9871e42f9f3e5443604a96beb74da`, preserved as candidate-stage
evidence. "Approval-state validation" records the validation of the historical
approval-state commit `c046e6bf6510f24ee1231f86293a3d499bceed7e`. "Recovery
validation" records the validation of the historical recovery commit
`9db91af7bf3d9f969556c819187420fc31d25c10`. "Amendment-6/7 candidate
validation" records the validation of the historical content-approved
candidate `b596536e7db0d6b8ace0a42fdd26bff42cabf104`. "Amendment-6/7
approval-state validation" records the validation of the historical
approval-state commit `609a01427847233b7280557baa61825ac2079865`.
"Control-record recovery validation" records the validation of the historical
recovery commit `22bd94dc959f11fe49c8bef8cd2a836140aadff7`. "Transient-state
correction validation" records the validation of the historical correction
commit `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`. "Factual-clarification
validation" records the validation of the historical clarification commit
`eb3129f13a9c858b3c012ee182315801f0b8563a`. "Amendment-8 candidate
validation" records the validation of the historical content-approved
candidate `42ae90a5932cf24ce90679196888cc3d83d88db8`. "Amendment-8
approval-state validation" records the validation of the historical
approval-state commit `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`. "Amendment-9
candidate validation" records the validation of the historical
content-approved candidate `8699af5635b8f18691ceaeccd3b24707a71ef4f2`. "Amendment-9
approval-state validation" at the end of this section records the validation
of the commit that contains this report.

Validation environment for the activation-epoch candidate: a full local Git
worktree of `thoth-pub/thoth`,
detached at `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`, with every object
fetched from `origin`. Immediately before construction, `git ls-remote origin`
returned `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe` and `feature/async/adr-0012` at
`b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`.

The candidate index was built by merging parent 2 into parent 1 without
committing, recording the unmodified merge tree, then overlaying only the four
correction paths. The checks below ran against that exact staged index before
the candidate commit was created from it. The created candidate commit's
parents and tree were then verified before push (results below). Commands that
compare with a fixed commit or tree use the full identities shown.

### Formatting

Command:

```text
git diff --check
git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
git diff --cached --check e15607878e66d2ff12b819dcea13293a46674f13
git diff --cached --check b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20
```

Result:

```text
each: exit 0; no output (no whitespace errors)
```

### Unit tests

Command:

```text
NOT APPLICABLE - documentation/architecture-only change; no Rust source changed
```

Result:

```text
NOT RUN
```

### Integration/database tests

Command:

```text
NOT APPLICABLE - no migration, schema or database code changed
```

Result:

```text
NOT RUN
```

### Lint/static analysis

Command:

```text
NOT APPLICABLE - no Rust source changed
```

Result:

```text
NOT RUN
```

### Other required checks

Merge geometry:

```text
git merge-tree --write-tree b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20 b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; e15607878e66d2ff12b819dcea13293a46674f13 (no conflicts)

git merge --no-ff --no-commit b44303c214498baf76c9d9a0a7cab374377f9cbe   (from detached b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20)
"Automatic merge went well; stopped before committing as requested"

git write-tree   (before any correction byte was staged)
e15607878e66d2ff12b819dcea13293a46674f13
```

Cumulative footprint against refreshed `feature/worker`:

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Correction delta against the unmodified merge-tree baseline:

```text
git diff --cached --name-only e15607878e66d2ff12b819dcea13293a46674f13
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Delta against the task head before the activation-epoch correction (four
correction paths plus the closed inherited set; `CHANGELOG.md` belongs to
both):

```text
git diff --cached --name-only b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/README.md
docs/engineering/decisions/decision-register.md

git diff --numstat b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20 e15607878e66d2ff12b819dcea13293a46674f13
2	0	CHANGELOG.md
362	0	docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
382	0	docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md
181	9	docs/engineering/decisions/README.md
```

Changelog and register composition:

```text
git diff --cached --numstat e15607878e66d2ff12b819dcea13293a46674f13 -- CHANGELOG.md docs/engineering/decisions/decision-register.md
1	0	CHANGELOG.md
1	1	docs/engineering/decisions/decision-register.md
```

The two inherited changelog lines are the `CTRL-ADR-PR-FIRST-01` and
`CTRL-ADR-AMEND-01` entries from refreshed `develop`; the single added line on
top of the merge-tree baseline is the new `THOTH-ASYNC-01-ADR-01-CORRECTION`
entry. The `[Unreleased]` section retains exactly one `### Added`, one
`### Changed` and one `### Fixed` heading, and every earlier ADR-0012 and
ADR-0013 entry is unchanged. The register's single changed line is the ADR-0012
row.

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated):

```text
git ls-files -s -- <path>
CHANGELOG.md                                          83882c489df11c0b7c8bf3ce76e7f25fe372cd18
ADR-0012                                              a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c
decision-register.md                                  ce8811566c27d5ff769c4fbb14b92ac120d71744
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c
```

Candidate ADR-0012 blob (`PROPOSED`): `a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c`. The
candidate-stage report blob was `594b2c914a123b4cb1b41260d66855e2e377a823`.

Candidate commit topology, verified after the candidate commit was created and
before it was pushed:

```text
git rev-list --parents -n 1 HEAD
95769537cdc9871e42f9f3e5443604a96beb74da b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20 b44303c214498baf76c9d9a0a7cab374377f9cbe

git rev-parse HEAD^{tree}
5084c6951a1b57b449f6045b139aacfbc488e534 (equal to the git write-tree output of the validated index)
```

After publication, the remote task-branch head, tree and ordered parents
matched these values.

Content checks:

- the ADR heading outline is unchanged; the header carries `Status: PROPOSED`
  and no current approver or approval date;
- ADR-0012's `Supersedes in part` header text is unchanged and still matches
  the partial-supersession text in ADR-0008 and ADR-0010;
- no ADR-0012 sentence outside section 12 asserts that the corrected content is
  approved;
- internal links in the changed files resolve to existing repository files.

### Approval-state validation

Environment: the same full local Git worktree, detached at the content-approved
candidate `95769537cdc9871e42f9f3e5443604a96beb74da`, after `git ls-remote
origin` and the PR #960 record showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `feature/async/adr-0012` at
`95769537cdc9871e42f9f3e5443604a96beb74da` and PR #960 open, draft and unmerged
with that head. The four paths were edited and staged, and the checks below ran
against that exact staged index before the approval-state commit was created
from it. The created commit was `c046e6bf6510f24ee1231f86293a3d499bceed7e`;
its single parent and tree, and the remote head after push, were verified as
recorded at the end of this subsection.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check 95769537cdc9871e42f9f3e5443604a96beb74da
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Approval-state delta against the content-approved candidate (exactly the four
authorized paths):

```text
git diff --cached --name-only 95769537cdc9871e42f9f3e5443604a96beb74da
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Size of the approval-state delta in the three decision files:

```text
git diff --cached --numstat 95769537cdc9871e42f9f3e5443604a96beb74da -- CHANGELOG.md docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md docs/engineering/decisions/decision-register.md
1	1	CHANGELOG.md
9	5	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
1	1	docs/engineering/decisions/decision-register.md
```

ADR-0012 candidate -> approval state (every changed line, diff header lines
omitted; no architecture text changes):

```text
git diff --cached -U0 95769537cdc9871e42f9f3e5443604a96beb74da -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
@@ -3 +3 @@
-Status: PROPOSED
+Status: APPROVED
@@ -4,0 +5,2 @@ Date: 2026-09-29
+Approved by: Javi, CTO
+Approval date: 2026-10-05
@@ -1970 +1972 @@ shared-engine tests.
-Current decision state: **PROPOSED**.
+This ADR is **APPROVED**.
@@ -1972,3 +1974,5 @@ Current decision state: **PROPOSED**.
-The corrected content of this version has not been approved. It may carry
-`APPROVED`, an approver and an approval date only after the CTO approves this
-exact corrected content under the repository decision process.
+Approved by: Javi, CTO
+Approval date: 2026-10-05
+
+The CTO approved this exact corrected content on 2026-10-05 under the
+repository decision process.
```

Decision-register candidate -> approval state (one line, the ADR-0012 row;
word-level changes, condensed to one pair per line):

```text
git diff --cached --word-diff=plain -U0 95769537cdc9871e42f9f3e5443604a96beb74da -- docs/engineering/decisions/decision-register.md
[-PROPOSED-] {+APPROVED+}
[-Pending-] {+Satisfied+}
{+the CTO approved+} {+exact+}
[-awaits CTO approval-] {+on 2026-10-05+}
```

Changelog candidate -> approval state (one line, the
`THOTH-ASYNC-01-ADR-01-CORRECTION` entry; word-level changes, condensed to one
pair per line):

```text
git diff --cached --word-diff=plain -U0 95769537cdc9871e42f9f3e5443604a96beb74da -- CHANGELOG.md
[-propose-] {+record+} {+CTO-approved+}
[-`PROPOSED`:-] {+`APPROVED`:+}
{+CTO approved the exact+}
[-is not approved.**-] {+on 2026-10-05.**+}
[-proposal-] {+decision+}
```

Approval-state blob identities (condensed `git ls-files -s` output and the
ADR-0012 `git hash-object --no-filters` result):

```text
ADR-0012 (APPROVED)                                   4848470a252076268a9fac9da5c40f0b4ebff474
decision-register.md                                  b19b1404dafa62a7aaebcd33708841cf44ce1831
CHANGELOG.md                                          ff7089a4a93e6a8c2c27c75a0ebf56ce43405c27
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
```

The approval-state report blob was `cd97eb441a38ff1885cf621e1af04519c470dffc`
and the approval-state tree `801e12ffc010470928da45919ede8d7239a491ce`. They
were not self-pinned by the approval-state report; they are recorded here as
historical facts because that commit is now the recovery commit's parent.

Topology of the approval-state commit, checked after it was created and before
it was pushed, and re-read from the repository at the recovery stage:

```text
git rev-list --parents -n 1 c046e6bf6510f24ee1231f86293a3d499bceed7e
c046e6bf6510f24ee1231f86293a3d499bceed7e 95769537cdc9871e42f9f3e5443604a96beb74da

git rev-parse c046e6bf6510f24ee1231f86293a3d499bceed7e^{tree}
801e12ffc010470928da45919ede8d7239a491ce (equal to the git write-tree output of the validated index)
```

After publication, the remote task-branch head, tree and parent matched these
values, and PR #960 carried that head when it was later marked Ready and
received the configured final-head automated review.

### Recovery validation

Environment: a full local Git worktree of `thoth-pub/thoth`, detached at the
historical approval-state head `c046e6bf6510f24ee1231f86293a3d499bceed7e`
(tree `801e12ffc010470928da45919ede8d7239a491ce`), with every object fetched
from `origin`. Immediately before construction, read-only GitHub inspection
showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `feature/async/adr-0012` at
`c046e6bf6510f24ee1231f86293a3d499bceed7e`, PR #960 open, ready, unmerged,
mergeable and clean with that head and target `feature/worker`, the
approval-state commit's single parent `95769537cdc9871e42f9f3e5443604a96beb74da`
and tree, and every protected blob named by the recovery authorization at its
required identity. PR #960 was then converted Ready -> Draft and the read-back
confirmed it open, draft and unmerged at the same head. The two recovery paths
were edited and staged, and every check below ran against that exact final
staged index before the recovery commit was created from it; the report's
numeric line counts below were filled in and the complete check set rerun with
identical results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check c046e6bf6510f24ee1231f86293a3d499bceed7e
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Recovery delta against the approval-state head (exactly the two authorized
paths):

```text
git diff --cached --name-only c046e6bf6510f24ee1231f86293a3d499bceed7e
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/decision-register.md

git diff --cached --name-status c046e6bf6510f24ee1231f86293a3d499bceed7e
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/decision-register.md

git diff --cached --numstat c046e6bf6510f24ee1231f86293a3d499bceed7e
439	106	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
1	1	docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Decision-register recovery delta (every changed line; the one authorized
header line and nothing else):

```text
git diff --cached -U0 c046e6bf6510f24ee1231f86293a3d499bceed7e -- docs/engineering/decisions/decision-register.md
@@ -5 +5 @@ Owner: CTO
-Last updated: 2026-09-30
+Last updated: 2026-10-05
```

ADR-0012 register row byte comparison against the approval-state head (the
row is line 20 of the register at both versions):

```text
diff <(git show c046e6bf6510f24ee1231f86293a3d499bceed7e:docs/engineering/decisions/decision-register.md | sed -n 20p) <(git show :docs/engineering/decisions/decision-register.md | sed -n 20p)
exit 0; no output (byte-identical)

git diff --cached --numstat c046e6bf6510f24ee1231f86293a3d499bceed7e -- docs/engineering/decisions/decision-register.md
1	1	docs/engineering/decisions/decision-register.md
```

Protected-blob checks on the staged index (condensed `git ls-files -s` output;
mode and stage omitted, paths abbreviated) and the ADR-0012
`git hash-object --no-filters` result:

```text
ADR-0012 (APPROVED, unchanged)                        4848470a252076268a9fac9da5c40f0b4ebff474
CHANGELOG.md                                          ff7089a4a93e6a8c2c27c75a0ebf56ce43405c27
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc
decision-register.md (recovered header)               247d077786a9176e628baa8de64c0c3a5e9b09d3

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
4848470a252076268a9fac9da5c40f0b4ebff474
```

The recovered decision-register blob is deterministic and not self-referential,
so it was recorded above. The recovery report's own final blob, the recovery
tree and the recovery commit SHA depended on that report's bytes and were not
self-pinned in it (Amendment 5 F2); they were verified against the remote head
after push and recorded in the post-publication handoff, and section 1 now
records them as historical facts.

Required topology of the recovery commit, checked after it was created and
before any push (results recorded in that post-publication handoff; the
published recovery head `9db91af7bf3d9f969556c819187420fc31d25c10` has the
single parent `c046e6bf6510f24ee1231f86293a3d499bceed7e` and tree
`fd9a300265c310388652ff94a4ef9f95224ce018`):

```text
git rev-list --parents -n 1 HEAD
required: <recovery commit> c046e6bf6510f24ee1231f86293a3d499bceed7e   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --name-only c046e6bf6510f24ee1231f86293a3d499bceed7e HEAD
required: exactly the two recovery paths

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Amendment-6/7 candidate validation

Environment: a full local Git worktree of `thoth-pub/thoth`, detached at the
historical recovery head `9db91af7bf3d9f969556c819187420fc31d25c10` (tree
`fd9a300265c310388652ff94a4ef9f95224ce018`), with every required object
present locally. Immediately before construction, read-only GitHub inspection
showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `feature/async/adr-0012` at
`9db91af7bf3d9f969556c819187420fc31d25c10` with single parent
`c046e6bf6510f24ee1231f86293a3d499bceed7e`, PR #960 open, ready, unmerged,
mergeable and clean with that head and target `feature/worker`, the cumulative
PR footprint exactly the six paths, ADR-0012 blob
`4848470a252076268a9fac9da5c40f0b4ebff474` at that head, and PR #960
review-thread state that matched the stage authorization (stage-time evidence;
current review-thread state remains GitHub-owned). PR #960 was then converted
Ready -> Draft and the read-back
confirmed it open, draft and unmerged at the same head. The four correction
paths were edited and staged, and every check below ran against that exact
final staged index before the candidate commit was created from it; the
report's numeric line counts below were filled in and the complete check set
rerun with identical results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check 9db91af7bf3d9f969556c819187420fc31d25c10
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Candidate delta against the recovery head (exactly the four authorized
paths):

```text
git diff --cached --name-only 9db91af7bf3d9f969556c819187420fc31d25c10
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md

git diff --cached --name-status 9db91af7bf3d9f969556c819187420fc31d25c10
M	CHANGELOG.md
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
M	docs/engineering/decisions/decision-register.md

git diff --cached --numstat 9db91af7bf3d9f969556c819187420fc31d25c10
1	0	CHANGELOG.md
781	259	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
704	179	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
2	2	docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Decision-register delta (only the `Last updated` header line and the ADR-0012
row; every other line byte-identical):

```text
git diff --cached -U0 9db91af7bf3d9f969556c819187420fc31d25c10 -- docs/engineering/decisions/decision-register.md | grep '^@@'
@@ -5 +5 @@ Owner: CTO
@@ -20 +20 @@ Last updated: 2026-10-05

git diff --cached -U0 9db91af7bf3d9f969556c819187420fc31d25c10 -- docs/engineering/decisions/decision-register.md | grep '^[-+]Last updated'
-Last updated: 2026-10-05
+Last updated: 2026-10-06

diff <(git show 9db91af7bf3d9f969556c819187420fc31d25c10:docs/engineering/decisions/decision-register.md | sed '5d;20d') <(git show :docs/engineering/decisions/decision-register.md | sed '5d;20d')
exit 0; no output

git show :docs/engineering/decisions/decision-register.md | sed -n 20p | grep -o '^| `ADR-0012` | [^|]* | [A-Z]* |'
| `ADR-0012` | [Shared asynchronous event and job execution architecture](ADR-0012-shared-asynchronous-event-and-job-execution.md) | PROPOSED |
```

Changelog delta (one added line, the new
`THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` entry at the top of
`[Unreleased] -> Changed`; every existing entry byte-preserved):

```text
git diff --cached -U0 9db91af7bf3d9f969556c819187420fc31d25c10 -- CHANGELOG.md | grep '^@@'
@@ -21,0 +22 @@ and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0

git show :CHANGELOG.md | sed -n '/^## \[Unreleased\]/,/^## \[\[1.8.0\]\]/p' | grep -c '^### '
3
```

ADR-0012 candidate state and outline:

```text
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n '^Status:\|^Approved by:\|^Approval date:\|^Current decision state:'
3:Status: PROPOSED
2492:Current decision state: **PROPOSED**.

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -c 'supported event kind/version contract'
0   (exit 1: no line matches)

diff <(git show 9db91af7bf3d9f969556c819187420fc31d25c10:docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^#') <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^#')
10c10,14
< ### Event-emission deployment floor
---
> ### Route-contract revisions
> ### In-epoch contract mismatch
> ### Historical enrollment, backfill and replay
> ### Revision withdrawal and route retirement
> ### Event-emission floor and producer-version coverage
exit 1 (the differences above are the complete outline change)
```

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (PROPOSED candidate)                         cd897d576522a741c3dfa82f8de2f8d80d4288f6
decision-register.md                                  18e3708d4247a9732778b8e71e5b52b95e9de776
CHANGELOG.md                                          db70c9827cde16887c11dbb4ef19258c36251d6f
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
cd897d576522a741c3dfa82f8de2f8d80d4288f6
```

The candidate ADR-0012, decision-register and changelog blobs were
deterministic and did not depend on the candidate report's bytes, so they were
recorded above. The candidate report's own final blob, the candidate tree and
the candidate commit SHA depended on that report's bytes and were not
self-pinned in it; they were verified against the remote head after push and
recorded in the post-publication handoff, and section 1 now records them as
historical facts.

Required topology of the candidate commit, checked after it was created and
before any push (results recorded in that post-publication handoff; the
published candidate `b596536e7db0d6b8ace0a42fdd26bff42cabf104` has the single
parent `9db91af7bf3d9f969556c819187420fc31d25c10` and tree
`d4d7dd359c3e20d37fcb5f664721b87cd2e480a8`):

```text
git rev-list --parents -n 1 HEAD
required: <candidate commit> 9db91af7bf3d9f969556c819187420fc31d25c10   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check 9db91af7bf3d9f969556c819187420fc31d25c10 HEAD
required: exit 0; no output

git diff --name-only 9db91af7bf3d9f969556c819187420fc31d25c10 HEAD
required: exactly the four candidate paths

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Amendment-6/7 approval-state validation

Environment: the same full local Git worktree, detached at the content-approved
candidate `b596536e7db0d6b8ace0a42fdd26bff42cabf104` (tree
`d4d7dd359c3e20d37fcb5f664721b87cd2e480a8`). Immediately before construction,
read-only GitHub inspection showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `feature/async/adr-0012` at
`b596536e7db0d6b8ace0a42fdd26bff42cabf104` with single parent
`9db91af7bf3d9f969556c819187420fc31d25c10`, ADR-0012 blob
`cd897d576522a741c3dfa82f8de2f8d80d4288f6` at that head, PR #960 open, draft,
unmerged and mergeable with that head and target `feature/worker`, the
cumulative PR footprint exactly the six paths, and PR #960 review-thread state
that matched the stage authorization (stage-time evidence; current
review-thread state remains GitHub-owned). The four paths were edited and
staged, and every check below ran against that exact final staged index before
the approval-state commit was created from it; the report's numeric line
counts below were filled in and the complete check set rerun with identical
results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check b596536e7db0d6b8ace0a42fdd26bff42cabf104
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Approval-state delta against the content-approved candidate (exactly the four
authorized paths):

```text
git diff --cached --name-only b596536e7db0d6b8ace0a42fdd26bff42cabf104
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md

git diff --cached --name-status b596536e7db0d6b8ace0a42fdd26bff42cabf104
M	CHANGELOG.md
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
M	docs/engineering/decisions/decision-register.md

git diff --cached --numstat b596536e7db0d6b8ace0a42fdd26bff42cabf104
1	1	CHANGELOG.md
623	158	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
12	5	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
1	1	docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

ADR-0012 candidate -> approval state (every changed line, diff header lines
omitted):

```text
git diff --cached -U0 b596536e7db0d6b8ace0a42fdd26bff42cabf104 -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
@@ -3 +3 @@
-Status: PROPOSED
+Status: APPROVED
@@ -4,0 +5,2 @@ Date: 2026-09-29
+Approved by: Javi, CTO
+Approval date: 2026-10-06
@@ -2492 +2494 @@ shared-engine tests.
-Current decision state: **PROPOSED**.
+This ADR is **APPROVED**.
@@ -2494,3 +2496,8 @@ Current decision state: **PROPOSED**.
-The corrected content of this version has not been approved. It may carry
-`APPROVED`, an approver and an approval date only after the CTO approves this
-exact corrected content under the repository decision process.
+Approved by: Javi, CTO
+Approval date: 2026-10-06
+
+The CTO approved this exact corrected content on 2026-10-06 under the
+repository decision process. That approval does not by itself make this ADR
+repository-authoritative: the authority condition below still applies. Any
+ADR-0013 programme-local reliance on this exact version is a separate state
+that is not effective until its own conditions are satisfied for this version.
```

Architecture no-drift proof. The normalization `N` removes only the authorized
approval-state representation: the header `Status`, `Approved by` and
`Approval date` fields before the first `---` rule, and the section-12
approval-state paragraphs between the `## 12. Approval and authority` heading
and the paragraph beginning "This version is a material architectural
correction". Everything else, including every other section-12 paragraph, is
kept:

```text
N='BEGIN{h=1} h&&/^---$/{h=0} h&&/^(Status|Approved by|Approval date): /{next} /^## 12\. Approval and authority$/{print;s=1;next} s&&/^This version is a material architectural correction/{s=0} s{next} {print}'

diff <(git cat-file -p cd897d576522a741c3dfa82f8de2f8d80d4288f6 | awk "$N") <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N")
exit 0; no output (NO DIFFERENCE)

git cat-file -p cd897d576522a741c3dfa82f8de2f8d80d4288f6 | awk "$N" | shasum -a 256
96e46cfe968b47cc56c05a6038a5732565c9e6cb17254971bd999cda7efddeae  -

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N" | shasum -a 256
96e46cfe968b47cc56c05a6038a5732565c9e6cb17254971bd999cda7efddeae  -
```

Lines the normalization removes from each version (trailing blanks stripped
for display):

```text
diff <(git cat-file -p cd897d576522a741c3dfa82f8de2f8d80d4288f6) <(git cat-file -p cd897d576522a741c3dfa82f8de2f8d80d4288f6 | awk "$N") | sed 's/[[:space:]]*$//'
3d2
< Status: PROPOSED
2491,2497d2489
<
< Current decision state: **PROPOSED**.
<
< The corrected content of this version has not been approved. It may carry
< `APPROVED`, an approver and an approval date only after the CTO approves this
< exact corrected content under the repository decision process.
<

diff <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md) <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N") | sed 's/[[:space:]]*$//'
3d2
< Status: APPROVED
5,6d3
< Approved by: Javi, CTO
< Approval date: 2026-10-06
2493,2504d2489
<
< This ADR is **APPROVED**.
<
< Approved by: Javi, CTO
< Approval date: 2026-10-06
<
< The CTO approved this exact corrected content on 2026-10-06 under the
< repository decision process. That approval does not by itself make this ADR
< repository-authoritative: the authority condition below still applies. Any
< ADR-0013 programme-local reliance on this exact version is a separate state
< that is not effective until its own conditions are satisfied for this version.
<
```

ADR-0012 approval state:

```text
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | sed -n 3,6p
Status: APPROVED
Date: 2026-09-29
Approved by: Javi, CTO
Approval date: 2026-10-06
```

Decision-register candidate -> approval state (one line, the ADR-0012 row;
every other line byte-identical; word-level changes, condensed to one pair per
line):

```text
git diff --cached -U0 b596536e7db0d6b8ace0a42fdd26bff42cabf104 -- docs/engineering/decisions/decision-register.md | grep '^@@'
@@ -20 +20 @@ Last updated: 2026-10-06

diff <(git show b596536e7db0d6b8ace0a42fdd26bff42cabf104:docs/engineering/decisions/decision-register.md | sed '20d') <(git show :docs/engineering/decisions/decision-register.md | sed '20d')
exit 0; no output

git diff --cached --word-diff=plain -U0 b596536e7db0d6b8ace0a42fdd26bff42cabf104 -- docs/engineering/decisions/decision-register.md
[-PROPOSED-] {+APPROVED+}
[-Pending-] {+Satisfied+}
{+the CTO approved+} {+exact+}
[-awaits CTO approval-] {+on 2026-10-06+}
```

Changelog candidate -> approval state (one line, the
`THOTH-ASYNC-01-ADR-01-ROUTE-CONTRACT-CORRECTION` entry; every other line
byte-identical; word-level changes, one pair per line):

```text
git diff --cached -U0 b596536e7db0d6b8ace0a42fdd26bff42cabf104 -- CHANGELOG.md | grep '^@@'
@@ -22 +22 @@ and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0

diff <(git show b596536e7db0d6b8ace0a42fdd26bff42cabf104:CHANGELOG.md | sed '22d') <(git show :CHANGELOG.md | sed '22d')
exit 0; no output

git diff --cached --word-diff=plain -U0 b596536e7db0d6b8ace0a42fdd26bff42cabf104 -- CHANGELOG.md
[-propose-] {+record+} {+CTO-approved+}
[-`PROPOSED`:-] {+`APPROVED`:+}
{+CTO approved the exact+}
[-is not approved.**-] {+on 2026-10-06.**+}
[-proposal-] {+decision+}
```

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (APPROVED)                                   a7d95ca918f418d596432071c8ba78a33c2630a9
decision-register.md                                  b358acd5bbedf7482cfdc7b71866815c852773af
CHANGELOG.md                                          e2947275d19d3abb351a98854268d7db213370d7
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
a7d95ca918f418d596432071c8ba78a33c2630a9
```

The approval-state ADR-0012, decision-register and changelog blobs were
deterministic and did not depend on the approval-state report's bytes, so they
were recorded above; the ADR-0012 blob
`a7d95ca918f418d596432071c8ba78a33c2630a9` was the exact final blob presented
for CTO exact-final-blob confirmation at that stage. The approval-state
report's own
final blob, the approval-state tree and the approval-state commit SHA depended
on that report's bytes and were not self-pinned in it; they were verified
against the remote head after push and recorded in the post-publication
handoff, and section 1 now records them as historical facts.

Required topology of the approval-state commit, checked after it was created
and before any push (results recorded in that post-publication handoff; the
published approval-state head `609a01427847233b7280557baa61825ac2079865` has
the single parent `b596536e7db0d6b8ace0a42fdd26bff42cabf104` and tree
`abc024bd16ebf8198475ef196c1f829228760abe`):

```text
git rev-list --parents -n 1 HEAD
required: <approval-state commit> b596536e7db0d6b8ace0a42fdd26bff42cabf104   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check b596536e7db0d6b8ace0a42fdd26bff42cabf104 HEAD
required: exit 0; no output

git diff --name-only b596536e7db0d6b8ace0a42fdd26bff42cabf104 HEAD
required: exactly the four authorized paths

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Control-record recovery validation

Environment: the same full local Git worktree, detached at the historical
approval-state head `609a01427847233b7280557baa61825ac2079865` (tree
`abc024bd16ebf8198475ef196c1f829228760abe`). Immediately before construction,
read-only GitHub inspection showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `feature/async/adr-0012` at
`609a01427847233b7280557baa61825ac2079865` with single parent
`b596536e7db0d6b8ace0a42fdd26bff42cabf104`, the ADR-0012, report,
decision-register and changelog blobs named by the recovery authorization at
that head, PR #960 open, draft, unmerged and mergeable with that head and
target `feature/worker`, the cumulative PR footprint exactly the six paths, and
PR #960 review-thread state that matched the stage authorization (stage-time
evidence; current review-thread state remains GitHub-owned). Only this report
was edited and staged, and every check below ran against that
exact final staged index before the recovery commit was created from it; the
report's numeric line counts below were filled in and the complete check set
rerun with identical results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check 609a01427847233b7280557baa61825ac2079865
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Recovery delta against the approval-state head (exactly the one authorized
path):

```text
git diff --cached --name-only 609a01427847233b7280557baa61825ac2079865
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md

git diff --cached --name-status 609a01427847233b7280557baa61825ac2079865
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md

git diff --cached --numstat 609a01427847233b7280557baa61825ac2079865
474	165	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Removed content-approval identifier (the command is shown with the identifier
replaced by a description, so that this report itself does not contain it):

```text
git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -F -c '<removed CTO content-approval comment identifier>'
0
```

Every other path byte-identical to the approval-state head:

```text
git diff --cached --name-only 609a01427847233b7280557baa61825ac2079865 -- . ':!docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md'
(no output)
```

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (APPROVED, unchanged)                        a7d95ca918f418d596432071c8ba78a33c2630a9
decision-register.md (unchanged)                      b358acd5bbedf7482cfdc7b71866815c852773af
CHANGELOG.md (unchanged)                              e2947275d19d3abb351a98854268d7db213370d7
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
a7d95ca918f418d596432071c8ba78a33c2630a9

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | sed -n 3,6p
Status: APPROVED
Date: 2026-09-29
Approved by: Javi, CTO
Approval date: 2026-10-06
```

The recovery report did not self-pin its own final blob, the recovery tree or
the recovery commit SHA; they were verified against the remote head after push
and recorded in the post-publication handoff, and section 1 now records them
as historical facts: commit `22bd94dc959f11fe49c8bef8cd2a836140aadff7`, tree
`7af3e0b8007eb6686e94043f57db63acc4edb4ab`, report blob
`74845b91dede2ba066bd0720598651e539196eb1`.

Required topology of the recovery commit, checked after it was created and
before push; the published recovery commit satisfied every requirement:

```text
git rev-list --parents -n 1 HEAD
required: <recovery commit> 609a01427847233b7280557baa61825ac2079865   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check 609a01427847233b7280557baa61825ac2079865 HEAD
required: exit 0; no output

git diff --name-only 609a01427847233b7280557baa61825ac2079865 HEAD
required: exactly this report

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Transient-state correction validation

Environment: a full local Git worktree of `thoth-pub/thoth`, detached at the
historical control-record recovery head
`22bd94dc959f11fe49c8bef8cd2a836140aadff7` (tree
`7af3e0b8007eb6686e94043f57db63acc4edb4ab`), with every required object
present locally. Immediately before construction, read-only GitHub inspection
showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `feature/async/adr-0012` at
`22bd94dc959f11fe49c8bef8cd2a836140aadff7` with single parent
`609a01427847233b7280557baa61825ac2079865`, the ADR-0012, report,
decision-register and changelog blobs named by the correction authorization at
that head, PR #960 open, draft, unmerged and mergeable with that head and
target `feature/worker`, the cumulative PR footprint exactly the six paths, and
PR #960 review-thread state that matched the stage authorization (stage-time
evidence; current review-thread state remains GitHub-owned). Only this report
was edited and staged, and every check below ran against that exact final
staged index before the correction commit was created from it; the report's
numeric line counts below were filled in and the complete check set rerun with
identical results.

Whitespace:

```text
git diff --cached --check
exit 0; no output

git diff --cached --check 22bd94dc959f11fe49c8bef8cd2a836140aadff7
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Correction delta against the recovery head (exactly the one authorized path):

```text
git diff --cached --name-only 22bd94dc959f11fe49c8bef8cd2a836140aadff7
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md

git diff --cached --name-status 22bd94dc959f11fe49c8bef8cd2a836140aadff7
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md

git diff --cached --numstat 22bd94dc959f11fe49c8bef8cd2a836140aadff7
555	166	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Transient review-thread state (the searched phrases are the exact-count and
current-state wordings named by the correction authorization; they are
described rather than restated so that this report does not contain them):

```text
git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -c -i -F -e '<exact unresolved-thread count phrase>' -e '<all-threads-remain-unresolved phrase>' -e '<threads-remain phrase>' -e '<unresolved-threads phrase>'
0

git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -n -i -E 'review.thread|unresolved'
every match read: each is historical stage-time evidence phrased as such,
durable substantive finding or disposition information, or the statement that
current review-thread state is GitHub-owned; none asserts a current thread
count or a current resolved/unresolved state
```

Final-head review identifiers (the review, thread and review-comment
identifiers of the final-head automated review of the recovery head, and the
identifier of the control adjudication that classified it, are described
rather than restated so that this report does not contain them):

```text
git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -c -F -e '<final-head review identifier>' -e '<first thread identifier>' -e '<second thread identifier>' -e '<first review-comment identifier>' -e '<second review-comment identifier>' -e '<control adjudication comment identifier>'
0
```

Every other path byte-identical to the recovery head:

```text
git diff --cached --name-only 22bd94dc959f11fe49c8bef8cd2a836140aadff7 -- . ':!docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md'
(no output)
```

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (APPROVED, unchanged)                        a7d95ca918f418d596432071c8ba78a33c2630a9
decision-register.md (unchanged)                      b358acd5bbedf7482cfdc7b71866815c852773af
CHANGELOG.md (unchanged)                              e2947275d19d3abb351a98854268d7db213370d7
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
a7d95ca918f418d596432071c8ba78a33c2630a9

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | sed -n 3,6p
Status: APPROVED
Date: 2026-09-29
Approved by: Javi, CTO
Approval date: 2026-10-06
```

The correction report did not self-pin its own final blob, the correction
tree or the correction commit SHA; they were verified against the remote head
after push and recorded in the post-publication handoff, and section 1 now
records them as historical facts: commit
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`, tree
`862a8e32207f1e0f61cab532b08b65034979f5ac`, report blob
`81db6d33bc0d103e149dd99e1f35150f35c079e3`.

Required topology of the correction commit, checked after it was created and
before push; the published correction commit satisfied every requirement:

```text
git rev-list --parents -n 1 HEAD
required: <correction commit> 22bd94dc959f11fe49c8bef8cd2a836140aadff7   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check 22bd94dc959f11fe49c8bef8cd2a836140aadff7 HEAD
required: exit 0; no output

git diff --name-only 22bd94dc959f11fe49c8bef8cd2a836140aadff7 HEAD
required: exactly this report

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Factual-clarification validation

Environment: a full local Git worktree of `thoth-pub/thoth`, detached at the
historical transient-state correction head
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` (tree
`862a8e32207f1e0f61cab532b08b65034979f5ac`), with every required object
present locally. Immediately before construction, read-only GitHub inspection
showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `feature/async/adr-0012` at
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` with single parent
`22bd94dc959f11fe49c8bef8cd2a836140aadff7`, the ADR-0012, report,
decision-register and changelog blobs named by the clarification
authorization at that head, PR #960 open, draft, unmerged and mergeable with
that head and target `feature/worker`, the cumulative PR footprint exactly the
six paths, the classification and authorization records present on #958 with
no later control record, and PR #960 review-thread state that matched the
stage authorization (stage-time evidence; current review-thread state remains
GitHub-owned). ADR-0012 was edited first and its deterministic blob recorded;
then only this report was edited; both were staged, and every check below ran
against that exact final staged index before the clarification commit was
created from it; the report's numeric line counts below were filled in and
the complete check set rerun with identical results.

Whitespace:

```text
git diff --cached --check
exit 0; no output

git diff --cached --check 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Clarification delta against the transient-state correction head (exactly the
two authorized paths):

```text
git diff --cached --name-only 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md

git diff --cached --name-status 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md

git diff --cached --numstat 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a
738	196	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
17	12	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Exact ADR-0012 delta from the prior approved blob
`a7d95ca918f418d596432071c8ba78a33c2630a9` to the clarified blob
`2db08058552ce901548ef3f71af4a294f5af7e61` (three hunks; nothing else changes):

```text
git diff --cached -U0 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
@@ -359,6 +359,8 @@ in-epoch contract-mismatch anomaly for every route for which it is mismatched.
-Exactly one durable disposition exists for each `(event_id, route_key)`. The
-normal disposition materializes a job: a materialized disposition points to
-exactly one job and remains unique on `(event_id, route_key)` regardless of
-whether that job is pending, running, waiting or terminal. The only alternative
-is the explicit, authorized and audited terminal non-materialized disposition
-described below, which points to no job (section 3.5 and invariant 8).
+At most one durable disposition may exist for a given `(event_id, route_key)`,
+and an eligible route obligation remains owed, visible in the route backlog,
+until it has received that one disposition. The normal disposition
+materializes a job: a materialized disposition points to exactly one job and
+remains unique on `(event_id, route_key)` regardless of whether that job is
+pending, running, waiting or terminal. The only alternative is the explicit,
+authorized and audited terminal non-materialized disposition described below,
+which points to no job (section 3.5 and invariant 8).
@@ -983,2 +985,3 @@ distinction between retry, replay and current-state redistribution.
-Event routing identity and job idempotency are related but distinct. Exactly one
-durable disposition exists for each `(event_id, route_key)`. A successfully
+Event routing identity and job idempotency are related but distinct. At most
+one durable disposition may exist for a given `(event_id, route_key)`, and an
+eligible route obligation eventually receives exactly one. A successfully
@@ -1827,4 +1830,6 @@ execution until a separate approved architecture decision says otherwise.
-8. Exactly one durable disposition exists for each `(event_id, route_key)`.
-   A successfully materialized disposition maps to exactly one job; many
-   materialized dispositions may map to one not-yet-started job only under an
-   explicit durable coalescing rule. An explicit authorized/audited terminal
+8. At most one durable disposition may exist for a given
+   `(event_id, route_key)`; an eligible route obligation remains owed until it
+   has received exactly one. A successfully materialized disposition maps to
+   exactly one job; many materialized dispositions may map to one
+   not-yet-started job only under an explicit durable coalescing rule. An
+   explicit authorized/audited terminal

git diff --cached 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -c -E '^[-+](Status:|Approved by:|Approval date:|## 12)'
0

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | sed -n 3,6p
Status: APPROVED
Date: 2026-09-29
Approved by: Javi, CTO
Approval date: 2026-10-06
```

ADR semantic validation of the clarified blob (line numbers refer to that
blob):

```text
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n -i -E 'exactly one durable|exactly one disposition|each \(event_id|every \(event_id'
(no output)

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n -i 'at most one'
lines 274, 322 (epochs, pre-existing), 359 (section 3.2), 1830 (invariant 8);
section 3.5 carries the same rule across lines 985-987
```

- cardinality: at most one durable disposition per `(event_id, route_key)`
  (lines 359, 985-986, 1830-1831), with the uniqueness boundary spanning every
  epoch and revision and excluding their identifiers (lines 367-373,
  1838-1841);
- completion: the engine must eventually establish a durable route disposition
  for every route for which a committed event is eligible (line 356), and an
  eligible route obligation remains owed until it has received exactly one
  (lines 360-361, 987, 1831-1832);
- case 1, ineligible event: no obligation and no anomaly (lines 334-341), so
  no slot is consumed;
- case 2, eligible backlog: an eligible obligation may temporarily have no
  disposition while owed and visible (line 360; undispositioned eligible-event
  backlog, line 552);
- case 3, in-epoch contract mismatch: no disposition or job, slot not consumed
  (lines 533-534; invariant 35, lines 1911-1912);
- case 4, refused historical enrollment: no disposition or job, slot unused
  (lines 589-591; invariant 6, lines 1826-1827);
- case 5, validated enrollment: creates the one allowed disposition (lines
  540-541; validation requirement, lines 2365-2366);
- case 6, approved reconciliation/divergence: records why no route disposition
  is created (lines 543-545; lines 2328-2330);
- case 7, already-dispositioned event: no second disposition through
  enrollment, reactivation, revision transition or replay (lines 595-600,
  1838-1841), and materialized and terminal non-materialized dispositions
  cannot coexist (lines 992-994, 1837-1838).

None of the seven cases is ambiguous or contradicted by the clarified text,
and the clarification selects no behaviour that the prior approved blob did
not already select.

Report lifecycle validation (the current-gate formulations named by the
clarification authorization are described rather than restated, so that this
record does not reintroduce them):

```text
git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -n -i -E '<still-requires phrase>|<requires-in-order phrase>|<current-gate phrase>|<later-gate phrase>|<not-yet phrase>|<awaits phrase>|<pending phrase>'
every match read: each is inside this validation record's own description,
historical stage-time evidence phrased as such, a quotation of a historical
register or ADR state, or a durable rule; none states which PR #960
review, Ready, thread-adjudication or merge-authorization gate is currently
pending or completed
```

Final-head review identifiers (the review, thread and review-comment
identifiers of the final-head automated review of the transient-state
correction head are described rather than restated so that this report does
not contain them):

```text
git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -c -F -e '<final-head review identifier>' -e '<first thread identifier>' -e '<second thread identifier>' -e '<first review-comment identifier>' -e '<second review-comment identifier>'
0
```

Every other path byte-identical to the transient-state correction head:

```text
git diff --cached --name-only 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a -- . ':!docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md' ':!docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md'
(no output)
```

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (APPROVED, clarified)                        2db08058552ce901548ef3f71af4a294f5af7e61
decision-register.md (unchanged)                      b358acd5bbedf7482cfdc7b71866815c852773af
CHANGELOG.md (unchanged)                              e2947275d19d3abb351a98854268d7db213370d7
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
2db08058552ce901548ef3f71af4a294f5af7e61
```

The clarification report did not self-pin its own final blob, the
clarification tree or the clarification commit SHA; they were verified against
the remote head after push and recorded in the post-publication handoff, and
section 1 now records them as historical facts: commit
`eb3129f13a9c858b3c012ee182315801f0b8563a`, tree
`5244d6ac76578a6f78cde79671762da5b09f2fc1`, report blob
`c47b874f1a48449e6c8f60a97f00ed1506d55fb9`.

Required topology of the clarification commit, checked after it was created
and before push; the published clarification commit satisfied every
requirement:

```text
git rev-list --parents -n 1 HEAD
required: <clarification commit> 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a HEAD
required: exit 0; no output

git diff --name-only 9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a HEAD
required: exactly ADR-0012 and this report

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Amendment-8 candidate validation

Environment: a full local Git worktree of `thoth-pub/thoth`, detached at the
historical factual-clarification head
`eb3129f13a9c858b3c012ee182315801f0b8563a` (tree
`5244d6ac76578a6f78cde79671762da5b09f2fc1`), with every required object
present locally. Immediately before construction, read-only GitHub inspection
showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `master` at
`4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff`, `feature/async/adr-0012` at
`eb3129f13a9c858b3c012ee182315801f0b8563a` with single parent
`9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` and tree
`5244d6ac76578a6f78cde79671762da5b09f2fc1`, the ADR-0012, report,
decision-register, changelog, ADR-0008 and ADR-0010 blobs named by the
candidate authorization at that head, PR #960 open, draft, unmerged and
mergeable with that head and target `feature/worker` and the only open pull
request targeting `feature/worker`, the cumulative PR footprint exactly the six
paths, the Amendment 8, independent-review and authorization records present on
#958 with no later control record, and PR #960 review-thread state that
matched the stage authorization (stage-time evidence; current review-thread
state remains GitHub-owned). ADR-0012, the decision register and the changelog
were edited first and their deterministic blobs recorded; then only this report
was edited; all four were staged, and every check below ran against that exact
final staged index before the candidate commit was created from it; the
report's numeric line counts below were filled in and the complete check set
rerun with identical results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check eb3129f13a9c858b3c012ee182315801f0b8563a
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Candidate delta against the clarification head (exactly the four authorized
paths):

```text
git diff --cached --name-only eb3129f13a9c858b3c012ee182315801f0b8563a
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md

git diff --cached --name-status eb3129f13a9c858b3c012ee182315801f0b8563a
M	CHANGELOG.md
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
M	docs/engineering/decisions/decision-register.md

git diff --cached --numstat eb3129f13a9c858b3c012ee182315801f0b8563a
1	0	CHANGELOG.md
839	191	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
142	26	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
2	2	docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Decision-register delta (only the `Last updated` header line and the ADR-0012
row; every other line byte-identical):

```text
git diff --cached -U0 eb3129f13a9c858b3c012ee182315801f0b8563a -- docs/engineering/decisions/decision-register.md | grep '^@@'
@@ -5 +5 @@ Owner: CTO
@@ -20 +20 @@ Last updated: 2026-10-06

git diff --cached -U0 eb3129f13a9c858b3c012ee182315801f0b8563a -- docs/engineering/decisions/decision-register.md | grep '^[-+]Last updated'
-Last updated: 2026-10-06
+Last updated: 2026-10-07

diff <(git show eb3129f13a9c858b3c012ee182315801f0b8563a:docs/engineering/decisions/decision-register.md | sed '5d;20d') <(git show :docs/engineering/decisions/decision-register.md | sed '5d;20d')
exit 0; no output

git show :docs/engineering/decisions/decision-register.md | sed -n 20p | grep -o '^| `ADR-0012` | [^|]* | [A-Z]* |'
| `ADR-0012` | [Shared asynchronous event and job execution architecture](ADR-0012-shared-asynchronous-event-and-job-execution.md) | PROPOSED |
```

Changelog delta (one added line, the new
`THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION` entry at the top of
`[Unreleased] -> Changed`; every existing entry byte-preserved):

```text
git diff --cached -U0 eb3129f13a9c858b3c012ee182315801f0b8563a -- CHANGELOG.md | grep '^@@'
@@ -21,0 +22 @@ and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0

git show :CHANGELOG.md | sed -n '/^## \[Unreleased\]/,/^## \[\[1.8.0\]\]/p' | grep -c '^### '
3
```

ADR-0012 candidate state, outline and hunks:

```text
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n '^Status:\|^Approved by:\|^Approval date:\|^Current decision state:'
3:Status: PROPOSED
2613:Current decision state: **PROPOSED**.

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n 'APPROVED'
every match is in section 12 (lines 2616-2655): the sentence that the version
may carry APPROVED only after CTO approval, the historical-approval paragraph
and the authority condition; no sentence asserts that this version is approved

diff <(git show eb3129f13a9c858b3c012ee182315801f0b8563a:docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^#') <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^#')
exit 0; no output (heading outline unchanged)

git diff --cached -U0 eb3129f13a9c858b3c012ee182315801f0b8563a -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^@@'
@@ -3 +3 @@
@@ -5,2 +4,0 @@ Date: 2026-09-29
@@ -295,0 +294,8 @@ or record boundaries out of order.
@@ -590,5 +596,12 @@ its semantic signature:
@@ -633,0 +647,54 @@ disposition plan closes or transfers that work safely.
@@ -656 +723,3 @@ deployment evidence for that epoch's exact revision proves all of:
@@ -1937,0 +2007,16 @@ execution until a separate approved architecture decision says otherwise.
@@ -1949 +2034,2 @@ Approval of this ADR should lead to separate bounded tasks, at minimum:
@@ -2160 +2246,2 @@ Rollout should prove in order:
@@ -2346,0 +2434,27 @@ Before shared implementation can be approved, evidence must include:
@@ -2499,4 +2613 @@ shared-engine tests.
@@ -2504,5 +2615,3 @@ Approval date: 2026-10-06
@@ -2519 +2628,4 @@ atomic revision equivalence, deterministic materializer compatibility,
@@ -2524,5 +2636,9 @@ activation-epoch model without immutable route-contract revisions, carried

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n '^41\. '
2007:41. For one route, revision withdrawal and every operation that creates a new

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -c 'pre-withdrawal observation'
3   (the withdrawal subsection, invariant 41 and the section-11 case)
```

The fourteen hunks are: the header status and approval fields (two hunks); the
section 3.2 serialization paragraph; the historical-enrollment availability
check and refusal; the withdrawal/selection ordering paragraphs of the
"Revision withdrawal and route retirement" subsection; activation gate 4;
invariant 41; section 8 item 1; section 9 rollout proof 3; the nine section-11
validation cases; and the four section-12 hunks that record the candidate
state, extend the description of this material correction and bind the
2026-10-06 approval to the historical earlier version. No other line of the
ADR changes; in particular the withdrawal paragraphs the candidate did not
touch (append-only/audited, future-selection only, non-destructive, strand
prevention and route retirement) are byte-identical to the clarification head.

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (PROPOSED candidate)                         cbb31a53f6f4ed6b96faafb79b273b55664f3ae0
decision-register.md                                  45ce935149693d0e983fac1645f5363896e23a53
CHANGELOG.md                                          f3c2bc7976d5fb84e1a00cbfce05748197da900b
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
cbb31a53f6f4ed6b96faafb79b273b55664f3ae0
```

The candidate ADR-0012, decision-register and changelog blobs were
deterministic and did not depend on the candidate report's bytes, so they were
recorded above. The candidate report's own final blob, the candidate tree and
the candidate commit SHA depended on that report's bytes and were not
self-pinned in it; they were verified against the remote head after push and
recorded in the post-publication handoff, and section 1 now records them as
historical facts: commit `42ae90a5932cf24ce90679196888cc3d83d88db8`, tree
`45d8a0bdb05ca95b61c01cdc83dcecf3a3f60e98`, report blob
`f85da63b89654d4774462de674eb02b4a1a0c38d`.

Required topology of the candidate commit, checked after it was created and
before push; the published candidate satisfied every requirement:

```text
git rev-list --parents -n 1 HEAD
required: <candidate commit> eb3129f13a9c858b3c012ee182315801f0b8563a   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check eb3129f13a9c858b3c012ee182315801f0b8563a HEAD
required: exit 0; no output

git diff --name-only eb3129f13a9c858b3c012ee182315801f0b8563a HEAD
required: exactly the four candidate paths

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Amendment-8 approval-state validation

Environment: the same full local Git worktree, detached at the content-approved
candidate `42ae90a5932cf24ce90679196888cc3d83d88db8` (tree
`45d8a0bdb05ca95b61c01cdc83dcecf3a3f60e98`). Immediately before construction,
read-only GitHub inspection showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `master` at
`4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff`, `feature/async/adr-0012` at
`42ae90a5932cf24ce90679196888cc3d83d88db8` with single parent
`eb3129f13a9c858b3c012ee182315801f0b8563a`, ADR-0012 blob
`cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`, report blob
`f85da63b89654d4774462de674eb02b4a1a0c38d`, decision-register blob
`45ce935149693d0e983fac1645f5363896e23a53` and changelog blob
`f3c2bc7976d5fb84e1a00cbfce05748197da900b` at that head, the protected
ADR-0008 and ADR-0010 blobs, PR #960 open, draft, unmerged and mergeable with
that head and target `feature/worker` and the only open pull request targeting
`feature/worker`, the cumulative PR footprint exactly the six paths, the
content-approval and approval-state authorization records present on #958 with
no later control record, and PR #960 review-thread state that matched the
stage authorization (stage-time evidence; current review-thread state remains
GitHub-owned). ADR-0012, the decision register and the changelog were edited
first and their deterministic blobs recorded; then only this report was
edited; all four were staged, and every check below ran against that exact
final staged index before the approval-state commit was created from it; the
report's numeric line counts below were filled in and the complete check set
rerun with identical results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check 42ae90a5932cf24ce90679196888cc3d83d88db8
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Approval-state delta against the content-approved candidate (exactly the four
authorized paths):

```text
git diff --cached --name-only 42ae90a5932cf24ce90679196888cc3d83d88db8
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md

git diff --cached --name-status 42ae90a5932cf24ce90679196888cc3d83d88db8
M	CHANGELOG.md
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
M	docs/engineering/decisions/decision-register.md

git diff --cached --numstat 42ae90a5932cf24ce90679196888cc3d83d88db8
1	1	CHANGELOG.md
739	171	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
15	5	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
1	1	docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

ADR-0012 candidate -> approval state (every changed line, diff header lines
omitted):

```text
git diff --cached -U0 42ae90a5932cf24ce90679196888cc3d83d88db8 -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
@@ -3 +3 @@
-Status: PROPOSED
+Status: APPROVED
@@ -4,0 +5,2 @@ Date: 2026-09-29
+Approved by: Javi, CTO
+Approval date: 2026-10-07
@@ -2613 +2615 @@ shared-engine tests.
-Current decision state: **PROPOSED**.
+This ADR is **APPROVED**.
@@ -2615,3 +2617,11 @@ Current decision state: **PROPOSED**.
-The corrected content of this version has not been approved. It may carry
-`APPROVED`, an approver and an approval date only after the CTO approves this
-exact corrected content under the repository decision process.
+Approved by: Javi, CTO
+Approval date: 2026-10-07
+
+The CTO approved this exact corrected content on 2026-10-07 under the
+repository decision process, as the architecture content represented by the
+Route-B candidate ADR-0012 blob `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`;
+this version differs from that candidate only in this approval-state
+representation. That approval does not by itself make this ADR
+repository-authoritative: the authority condition below still applies. Any
+ADR-0013 programme-local reliance on this exact version is a separate state
+that is not effective until its own conditions are satisfied for this version.
```

Architecture no-drift proof. The normalization `N` removes only the authorized
approval-state representation: the header `Status`, `Approved by` and
`Approval date` fields before the first `---` rule, and the section-12
approval-state paragraphs between the `## 12. Approval and authority` heading
and the paragraph beginning "This version is a material architectural
correction". Everything else, including every other section-12 paragraph, is
kept:

```text
N='BEGIN{h=1} h&&/^---$/{h=0} h&&/^(Status|Approved by|Approval date): /{next} /^## 12\. Approval and authority$/{print;s=1;next} s&&/^This version is a material architectural correction/{s=0} s{next} {print}'

diff <(git cat-file -p cbb31a53f6f4ed6b96faafb79b273b55664f3ae0 | awk "$N") <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N")
exit 0; no output (NO DIFFERENCE)

git cat-file -p cbb31a53f6f4ed6b96faafb79b273b55664f3ae0 | awk "$N" | shasum -a 256
1ecb63976863e91900cd35fcea5179ef46e4778932ece74c0edef4b2aa18d9c8  -

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N" | shasum -a 256
1ecb63976863e91900cd35fcea5179ef46e4778932ece74c0edef4b2aa18d9c8  -
```

Lines the normalization removes from each version (trailing blanks stripped
for display):

```text
diff <(git cat-file -p cbb31a53f6f4ed6b96faafb79b273b55664f3ae0) <(git cat-file -p cbb31a53f6f4ed6b96faafb79b273b55664f3ae0 | awk "$N") | sed 's/[[:space:]]*$//'
3d2
< Status: PROPOSED
2612,2618d2610
<
< Current decision state: **PROPOSED**.
<
< The corrected content of this version has not been approved. It may carry
< `APPROVED`, an approver and an approval date only after the CTO approves this
< exact corrected content under the repository decision process.
<

diff <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md) <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N") | sed 's/[[:space:]]*$//'
3d2
< Status: APPROVED
5,6d3
< Approved by: Javi, CTO
< Approval date: 2026-10-07
2614,2628d2610
<
< This ADR is **APPROVED**.
<
< Approved by: Javi, CTO
< Approval date: 2026-10-07
<
< The CTO approved this exact corrected content on 2026-10-07 under the
< repository decision process, as the architecture content represented by the
< Route-B candidate ADR-0012 blob `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`;
< this version differs from that candidate only in this approval-state
< representation. That approval does not by itself make this ADR
< repository-authoritative: the authority condition below still applies. Any
< ADR-0013 programme-local reliance on this exact version is a separate state
< that is not effective until its own conditions are satisfied for this version.
<
```

ADR-0012 approval state:

```text
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | sed -n 3,7p
Status: APPROVED
Date: 2026-09-29
Approved by: Javi, CTO
Approval date: 2026-10-07
Decision owner: CTO
```

Decision-register candidate -> approval state (one line, the ADR-0012 row;
every other line byte-identical; word-level changes, condensed to one pair per
line):

```text
git diff --cached -U0 42ae90a5932cf24ce90679196888cc3d83d88db8 -- docs/engineering/decisions/decision-register.md | grep '^@@'
@@ -20 +20 @@ Last updated: 2026-10-07

diff <(git show 42ae90a5932cf24ce90679196888cc3d83d88db8:docs/engineering/decisions/decision-register.md | sed '20d') <(git show :docs/engineering/decisions/decision-register.md | sed '20d')
exit 0; no output

git diff --cached --word-diff=plain -U0 42ae90a5932cf24ce90679196888cc3d83d88db8 -- docs/engineering/decisions/decision-register.md
[-PROPOSED-] {+APPROVED+}
[-Pending-] {+Satisfied+}
{+the CTO approved+} {+exact+}
[-awaits CTO approval-] {+on 2026-10-07+}
```

Changelog candidate -> approval state (one line, the
`THOTH-ASYNC-01-ADR-01-WITHDRAWAL-ORDER-CORRECTION` entry; every other line
byte-identical; word-level changes, one pair per line):

```text
git diff --cached -U0 42ae90a5932cf24ce90679196888cc3d83d88db8 -- CHANGELOG.md | grep '^@@'
@@ -22 +22 @@ and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0

diff <(git show 42ae90a5932cf24ce90679196888cc3d83d88db8:CHANGELOG.md | sed '22d') <(git show :CHANGELOG.md | sed '22d')
exit 0; no output

git diff --cached --word-diff=plain -U0 42ae90a5932cf24ce90679196888cc3d83d88db8 -- CHANGELOG.md
[-propose-] {+record+} {+CTO-approved+}
[-`PROPOSED`:-] {+`APPROVED`:+}
{+CTO approved the exact+}
[-is not approved.**-] {+on 2026-10-07.**+}
[-proposal-] {+decision+}
```

Content-approval identifier (the command is shown with the identifier replaced
by a description, so that this report itself does not contain it):

```text
git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -F -c '<CTO content-approval comment identifier>'
0
```

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (APPROVED)                                   85937a4e1ff9a27d8f5b36ca82651821711472b2
decision-register.md                                  246bd5e981d373447fccdd457c02645338bfee4b
CHANGELOG.md                                          d1fbe1bdc7699bae18970cdcc9c61d9cbdcefa82
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
85937a4e1ff9a27d8f5b36ca82651821711472b2
```

The approval-state ADR-0012, decision-register and changelog blobs were
deterministic and did not depend on the approval-state report's bytes, so they
were recorded above; the ADR-0012 blob `85937a4e1ff9a27d8f5b36ca82651821711472b2`
was the exact final blob presented for CTO exact-final-blob approval at that
stage and received it on 2026-10-07. The approval-state report's own final
blob, the approval-state tree and the approval-state commit SHA depended on
that report's bytes and were not self-pinned in it; they were verified against
the remote head after push and recorded in the post-publication handoff, and
section 1 now records them as historical facts: commit
`e2588f7b7e63a5c71968d87a3409f0ff48635e6d`, tree
`bec3637f333a12ed1895b2aae6f1078510067f5e`, report blob
`307a46ed153cd1287b24d921e7286c62328e8cc1`.

Required topology of the approval-state commit, checked after it was created
and before push; the published approval-state commit satisfied every
requirement:

```text
git rev-list --parents -n 1 HEAD
required: <approval-state commit> 42ae90a5932cf24ce90679196888cc3d83d88db8   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check 42ae90a5932cf24ce90679196888cc3d83d88db8 HEAD
required: exit 0; no output

git diff --name-only 42ae90a5932cf24ce90679196888cc3d83d88db8 HEAD
required: exactly the four authorized paths

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Amendment-9 candidate validation

Environment: the same full local Git worktree, detached at the historical
Amendment-8 approval-state head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` (tree
`bec3637f333a12ed1895b2aae6f1078510067f5e`). Immediately before construction,
read-only GitHub inspection showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `master` at
`4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff`, `feature/async/adr-0012` at
`e2588f7b7e63a5c71968d87a3409f0ff48635e6d` with single parent
`42ae90a5932cf24ce90679196888cc3d83d88db8`, ADR-0012 blob
`85937a4e1ff9a27d8f5b36ca82651821711472b2`, report blob
`307a46ed153cd1287b24d921e7286c62328e8cc1`, decision-register blob
`246bd5e981d373447fccdd457c02645338bfee4b` and changelog blob
`d1fbe1bdc7699bae18970cdcc9c61d9cbdcefa82` at that head, the protected
ADR-0008 and ADR-0010 blobs, PR #960 open, draft, unmerged and mergeable with
that head and target `feature/worker` and the only open pull request targeting
`feature/worker`, the cumulative PR footprint exactly the six paths, the
Amendment 9, independent-review and authorization records present on #958 with
no later control record, and PR #960 review-thread state that matched the
stage authorization (stage-time evidence; current review-thread state remains
GitHub-owned). ADR-0012, the decision register and the changelog were edited
first and their deterministic blobs recorded; then only this report was
edited; all four were staged, and every check below ran against that exact
final staged index before the candidate commit was created from it; the
report's numeric line counts below were filled in and the complete check set
rerun with identical results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check e2588f7b7e63a5c71968d87a3409f0ff48635e6d
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Candidate delta against the approval-state head (exactly the four authorized
paths):

```text
git diff --cached --name-only e2588f7b7e63a5c71968d87a3409f0ff48635e6d
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md

git diff --cached --name-status e2588f7b7e63a5c71968d87a3409f0ff48635e6d
M	CHANGELOG.md
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
M	docs/engineering/decisions/decision-register.md

git diff --cached --numstat e2588f7b7e63a5c71968d87a3409f0ff48635e6d
1	0	CHANGELOG.md
869	191	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
183	28	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
1	1	docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Decision-register delta (only the ADR-0012 row; every other line, including
the `Last updated: 2026-10-07` header, byte-identical):

```text
git diff --cached -U0 e2588f7b7e63a5c71968d87a3409f0ff48635e6d -- docs/engineering/decisions/decision-register.md | grep '^@@'
@@ -20 +20 @@ Last updated: 2026-10-07

diff <(git show e2588f7b7e63a5c71968d87a3409f0ff48635e6d:docs/engineering/decisions/decision-register.md | sed '20d') <(git show :docs/engineering/decisions/decision-register.md | sed '20d')
exit 0; no output

git show :docs/engineering/decisions/decision-register.md | sed -n 20p | grep -o '^| `ADR-0012` | [^|]* | [A-Z]* |'
| `ADR-0012` | [Shared asynchronous event and job execution architecture](ADR-0012-shared-asynchronous-event-and-job-execution.md) | PROPOSED |
```

Changelog delta (one added line, the new
`THOTH-ASYNC-01-ADR-01-COALESCING-SEAL-CORRECTION` entry at the top of
`[Unreleased] -> Changed`; every existing entry byte-preserved):

```text
git diff --cached -U0 e2588f7b7e63a5c71968d87a3409f0ff48635e6d -- CHANGELOG.md | grep '^@@'
@@ -21,0 +22 @@ and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0

diff <(git show e2588f7b7e63a5c71968d87a3409f0ff48635e6d:CHANGELOG.md) <(git show :CHANGELOG.md | sed '22d')
exit 0; no output

git show :CHANGELOG.md | sed -n '/^## \[Unreleased\]/,/^## \[\[1.8.0\]\]/p' | grep -c '^### '
3
```

ADR-0012 candidate state, outline and hunks:

```text
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n '^Status:\|^Approved by:\|^Approval date:\|^Current decision state:'
3:Status: PROPOSED
2771:Current decision state: **PROPOSED**.

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n 'APPROVED'
every match is in section 12 (lines 2774-2820): the sentence that the version
may carry APPROVED only after CTO approval, the historical-approval paragraph
and the authority condition; no sentence asserts that this version is approved

diff <(git show e2588f7b7e63a5c71968d87a3409f0ff48635e6d:docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^#') <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^#')
17a18
> ### Coalescing-open and coalescing-sealed jobs
exit 1 (the one added heading is the complete outline change)

git diff --cached -U0 e2588f7b7e63a5c71968d87a3409f0ff48635e6d -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep '^@@'
@@ -3 +3 @@
@@ -5,2 +4,0 @@ Date: 2026-09-29
@@ -1061 +1059,3 @@ materialized route dispositions may point to one **not-yet-started** job only
@@ -1073 +1073,2 @@ An idempotency conflict never silently means "route satisfied". It either:
@@ -1096,0 +1098,97 @@ kind must not consume all worker capacity indefinitely.
@@ -1577 +1675,4 @@ domain rule and must retain enough correlation to explain which events or
@@ -1905,2 +2006,2 @@ execution until a separate approved architecture decision says otherwise.
@@ -2024,0 +2126,17 @@ execution until a separate approved architecture decision says otherwise.
@@ -2040 +2158,3 @@ Approval of this ADR should lead to separate bounded tasks, at minimum:
@@ -2251 +2371,2 @@ Rollout should prove in order:
@@ -2519,0 +2641,35 @@ Before shared implementation can be approved, evidence must include:
@@ -2615,4 +2771 @@ shared-engine tests.
@@ -2620,8 +2773,3 @@ Approval date: 2026-10-07
@@ -2641 +2789,5 @@ database-atomic linearized order, so that a stale availability observation can
@@ -2650,5 +2802,8 @@ carried `Status: APPROVED` with CTO approval dated 2026-10-06, retained through

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -n '^### Coalescing-open\|^42\. '
1098:### Coalescing-open and coalescing-sealed jobs
2126:42. For every logical job of a kind that permits many-route-to-one-job

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -c 'coalescing-open'
8
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -c 'coalescing-sealed'
5
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -c 'compare-and-set'
3
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | grep -c 'pre-claim observation'
2
```

The fifteen hunks are: the header status and approval fields (two hunks); the
section-3.5 not-yet-started sentence; the idempotency-conflict attachment
item; the new "Coalescing-open and coalescing-sealed jobs" subsection; the
section-3.15 coalescing paragraph; invariant 8; invariant 42; section 8 item
1; section 9 rollout proof 5; the nine section-11 validation cases; and the
four section-12 hunks that record the candidate state, extend the description
of this material correction and bind the 2026-10-07 approval to the
historical earlier version. No other line of the ADR changes; in particular
the Amendment-8 withdrawal/selection text, the claim, lease and attempt-phase
text of sections 3.4 and 3.6, the idempotency-key, target-identity and
target-concurrency text of sections 3.3 and 3.5 and the disposition-uniqueness
text of section 3.2 are byte-identical to the approval-state head.

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (PROPOSED candidate)                         0fc33df9aca61a2c82af452426f5574bfd3493cf
decision-register.md                                  8b5ef6b5b6ca7f6d49ed5066aba5e49575de06be
CHANGELOG.md                                          4d0e17ca0b11dea0ed4905ef9776693ee2c87bc4
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
0fc33df9aca61a2c82af452426f5574bfd3493cf
```

The candidate ADR-0012, decision-register and changelog blobs were
deterministic and did not depend on the candidate report's bytes, so they were
recorded above. The candidate report's own final blob, the candidate tree and
the candidate commit SHA depended on that report's bytes and were not
self-pinned in it; they were verified against the remote head after push and
recorded in the post-publication handoff, and section 1 now records them as
historical facts: commit `8699af5635b8f18691ceaeccd3b24707a71ef4f2`, tree
`ebff7219611e7855c62856019fc051fcf41cb2b5`, report blob
`ffbb44f9f250d325da6846796dfae8051a3a839e`.

Required topology of the candidate commit, checked after it was created and
before push; the published candidate satisfied every requirement:

```text
git rev-list --parents -n 1 HEAD
required: <candidate commit> e2588f7b7e63a5c71968d87a3409f0ff48635e6d   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check e2588f7b7e63a5c71968d87a3409f0ff48635e6d HEAD
required: exit 0; no output

git diff --name-only e2588f7b7e63a5c71968d87a3409f0ff48635e6d HEAD
required: exactly the four candidate paths

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

### Amendment-9 approval-state validation

Environment: the same full local Git worktree, detached at the content-approved
candidate `8699af5635b8f18691ceaeccd3b24707a71ef4f2` (tree
`ebff7219611e7855c62856019fc051fcf41cb2b5`). Immediately before construction,
read-only GitHub inspection showed `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe`, `master` at
`4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff`, `feature/async/adr-0012` at
`8699af5635b8f18691ceaeccd3b24707a71ef4f2` with single parent
`e2588f7b7e63a5c71968d87a3409f0ff48635e6d`, ADR-0012 blob
`0fc33df9aca61a2c82af452426f5574bfd3493cf`, report blob
`ffbb44f9f250d325da6846796dfae8051a3a839e`, decision-register blob
`8b5ef6b5b6ca7f6d49ed5066aba5e49575de06be` and changelog blob
`4d0e17ca0b11dea0ed4905ef9776693ee2c87bc4` at that head, the protected
ADR-0008 and ADR-0010 blobs, PR #960 open, draft, unmerged and mergeable with
that head and target `feature/worker` and the only open pull request targeting
`feature/worker`, the cumulative PR footprint exactly the six paths, the
content-approval and approval-state authorization records present on #958 with
no later control record, and PR #960 review-thread state that matched the
stage authorization (stage-time evidence; current review-thread state remains
GitHub-owned). ADR-0012, the decision register and the changelog were edited
first and their deterministic blobs recorded; then only this report was
edited; all four were staged, and every check below ran against that exact
final staged index before the approval-state commit was created from it; the
report's numeric line counts below were filled in and the complete check set
rerun with identical results.

Whitespace:

```text
git diff --check
exit 0; no output

git diff --cached --check 8699af5635b8f18691ceaeccd3b24707a71ef4f2
exit 0; no output

git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; no output
```

Approval-state delta against the content-approved candidate (exactly the four
authorized paths):

```text
git diff --cached --name-only 8699af5635b8f18691ceaeccd3b24707a71ef4f2
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md

git diff --cached --name-status 8699af5635b8f18691ceaeccd3b24707a71ef4f2
M	CHANGELOG.md
M	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
M	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
M	docs/engineering/decisions/decision-register.md

git diff --cached --numstat 8699af5635b8f18691ceaeccd3b24707a71ef4f2
1	1	CHANGELOG.md
762	191	docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
15	5	docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
2	2	docs/engineering/decisions/decision-register.md
```

Cumulative footprint against refreshed `feature/worker` (exactly the six PR
paths):

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

ADR-0012 candidate -> approval state (every changed line, diff header lines
omitted):

```text
git diff --cached -U0 8699af5635b8f18691ceaeccd3b24707a71ef4f2 -- docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
@@ -3 +3 @@
-Status: PROPOSED
+Status: APPROVED
@@ -4,0 +5,2 @@ Date: 2026-09-29
+Approved by: Javi, CTO
+Approval date: 2026-10-08
@@ -2771 +2773 @@ shared-engine tests.
-Current decision state: **PROPOSED**.
+Current decision state: **APPROVED**.
@@ -2773,3 +2775,11 @@ Current decision state: **PROPOSED**.
-The corrected content of this version has not been approved. It may carry
-`APPROVED`, an approver and an approval date only after the CTO approves this
-exact corrected content under the repository decision process.
+Approved by: Javi, CTO
+Approval date: 2026-10-08
+
+The CTO approved this exact corrected content on 2026-10-08 under the
+repository decision process, as the architecture content represented by the
+Route-B candidate ADR-0012 blob `0fc33df9aca61a2c82af452426f5574bfd3493cf`;
+this version differs from that candidate only in this approval-state
+representation. That approval does not by itself make this ADR
+repository-authoritative: the authority condition below still applies. Any
+ADR-0013 programme-local reliance on this exact version is a separate state
+that is not effective until its own conditions are satisfied for this version.
```

Architecture no-drift proof. The normalization `N` removes only the authorized
approval-state representation: the header `Status`, `Approved by` and
`Approval date` fields before the first `---` rule, and the section-12
approval-state paragraphs between the `## 12. Approval and authority` heading
and the paragraph beginning "This version is a material architectural
correction". Everything else, including every other section-12 paragraph, is
kept:

```text
N='BEGIN{h=1} h&&/^---$/{h=0} h&&/^(Status|Approved by|Approval date): /{next} /^## 12\. Approval and authority$/{print;s=1;next} s&&/^This version is a material architectural correction/{s=0} s{next} {print}'

diff <(git cat-file -p 0fc33df9aca61a2c82af452426f5574bfd3493cf | awk "$N") <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N")
exit 0; no output (NO DIFFERENCE)

git cat-file -p 0fc33df9aca61a2c82af452426f5574bfd3493cf | awk "$N" | shasum -a 256
33bb6972129c88c537608b740d8f284e8c4a5c86927bbb3e693a1f2c86036a02  -

git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N" | shasum -a 256
33bb6972129c88c537608b740d8f284e8c4a5c86927bbb3e693a1f2c86036a02  -
```

Lines the normalization removes from each version (trailing blanks stripped
for display):

```text
diff <(git cat-file -p 0fc33df9aca61a2c82af452426f5574bfd3493cf) <(git cat-file -p 0fc33df9aca61a2c82af452426f5574bfd3493cf | awk "$N") | sed 's/[[:space:]]*$//'
3d2
< Status: PROPOSED
2770,2776d2768
<
< Current decision state: **PROPOSED**.
<
< The corrected content of this version has not been approved. It may carry
< `APPROVED`, an approver and an approval date only after the CTO approves this
< exact corrected content under the repository decision process.
<

diff <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md) <(git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | awk "$N") | sed 's/[[:space:]]*$//'
3d2
< Status: APPROVED
5,6d3
< Approved by: Javi, CTO
< Approval date: 2026-10-08
2772,2786d2768
<
< Current decision state: **APPROVED**.
<
< Approved by: Javi, CTO
< Approval date: 2026-10-08
<
< The CTO approved this exact corrected content on 2026-10-08 under the
< repository decision process, as the architecture content represented by the
< Route-B candidate ADR-0012 blob `0fc33df9aca61a2c82af452426f5574bfd3493cf`;
< this version differs from that candidate only in this approval-state
< representation. That approval does not by itself make this ADR
< repository-authoritative: the authority condition below still applies. Any
< ADR-0013 programme-local reliance on this exact version is a separate state
< that is not effective until its own conditions are satisfied for this version.
<
```

ADR-0012 approval state:

```text
git show :docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md | sed -n 3,7p
Status: APPROVED
Date: 2026-09-29
Approved by: Javi, CTO
Approval date: 2026-10-08
Decision owner: CTO
```

Decision-register candidate -> approval state (the `Last updated` header line
and the ADR-0012 row; every other line byte-identical; row changes as
word-level pairs):

```text
git diff --cached -U0 8699af5635b8f18691ceaeccd3b24707a71ef4f2 -- docs/engineering/decisions/decision-register.md | grep '^@@'
@@ -5 +5 @@ Owner: CTO
@@ -20 +20 @@ Last updated: 2026-10-07

git diff --cached -U0 8699af5635b8f18691ceaeccd3b24707a71ef4f2 -- docs/engineering/decisions/decision-register.md | grep '^[-+]Last updated'
-Last updated: 2026-10-07
+Last updated: 2026-10-08

diff <(git show 8699af5635b8f18691ceaeccd3b24707a71ef4f2:docs/engineering/decisions/decision-register.md | sed '5d;20d') <(git show :docs/engineering/decisions/decision-register.md | sed '5d;20d')
exit 0; no output

git diff --cached --word-diff=plain -U0 8699af5635b8f18691ceaeccd3b24707a71ef4f2 -- docs/engineering/decisions/decision-register.md
[-2026-10-07-] {+2026-10-08+}
[-PROPOSED-] {+APPROVED+}
[-Pending-] {+Satisfied+}
{+Javi, CTO approved+} {+exact+}
[-awaits CTO approval-] {+on 2026-10-08+}
```

Changelog candidate -> approval state (one line, the
`THOTH-ASYNC-01-ADR-01-COALESCING-SEAL-CORRECTION` entry; every other line
byte-identical; word-level changes, one pair per line):

```text
git diff --cached -U0 8699af5635b8f18691ceaeccd3b24707a71ef4f2 -- CHANGELOG.md | grep '^@@'
@@ -22 +22 @@ and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0

diff <(git show 8699af5635b8f18691ceaeccd3b24707a71ef4f2:CHANGELOG.md | sed '22d') <(git show :CHANGELOG.md | sed '22d')
exit 0; no output

git diff --cached --word-diff=plain -U0 8699af5635b8f18691ceaeccd3b24707a71ef4f2 -- CHANGELOG.md
[-propose-] {+record+} {+CTO-approved+}
[-`PROPOSED`:-] {+`APPROVED`: Javi, CTO approved+} {+exact+}
[-is not approved.**-] {+on 2026-10-08.**+}
[-proposal-] {+decision+}
```

Content-approval identifier (the command is shown with the identifier replaced
by a description, so that this report itself does not contain it):

```text
git show :docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md | grep -F -c '<CTO content-approval comment identifier>'
0
```

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated) and the ADR-0012 `git hash-object --no-filters`
result:

```text
ADR-0012 (APPROVED)                                   cbe8322d95fe8d5e9911d421a5f10f1ae4afd55c
decision-register.md                                  c618d9d2eec622660ee7c1868111f747efa27629
CHANGELOG.md                                          2731521f363189063b9ab09068df7852889121a2
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c
ADR-0005                                              bdaa976e4893b1fc45f994236f9e56d433212d63
docs/engineering/AGENTS.md                            e194b14e8c4fbb298db7ca6c38aebeb69547a29a
implementation-report-template.md                     0ae39d3892bbca5b0ff90dc7ffa70038662351bc

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
cbe8322d95fe8d5e9911d421a5f10f1ae4afd55c
```

The approval-state ADR-0012, decision-register and changelog blobs are
deterministic and do not depend on this report's bytes, so they are recorded
above; the ADR-0012 blob `cbe8322d95fe8d5e9911d421a5f10f1ae4afd55c` is the exact
final blob presented for CTO exact-final-blob approval at this stage. This
report's own final blob, the approval-state tree and the approval-state commit
SHA depend on this report's bytes and are not self-pinned; they are verified
against the remote head after push and recorded in the post-publication
handoff.

Required topology of the approval-state commit, checked after it is created
and before any push, with results recorded in the post-publication handoff:

```text
git rev-list --parents -n 1 HEAD
required: <approval-state commit> 8699af5635b8f18691ceaeccd3b24707a71ef4f2   (exactly one parent)

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index

git diff --check 8699af5635b8f18691ceaeccd3b24707a71ef4f2 HEAD
required: exit 0; no output

git diff --name-only 8699af5635b8f18691ceaeccd3b24707a71ef4f2 HEAD
required: exactly the four authorized paths

git diff --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe HEAD
required: exactly the six cumulative PR paths
```

## 10. Manual verification

Environment: the local worktrees described in section 9.
Steps: compare the corrected ADR against the prior blob
`dec6665353804e42f22474954bf878308092b717` for every Amendment 1 A2 invariant,
Amendment 3 C7 and base-specification section 3; confirm that no unrelated
architecture changed; confirm the register, changelog and report state
`PROPOSED` and bind the 2026-09-30 approval to the historical versions only.
Observed result: all twelve A2 invariants are expressed in section 3.2 and
invariant 3 and exercised by section 11 validation requirements; C7's
write-once close wording is used; text unrelated to route activation, the
disposition model and approval state is byte-identical to the prior blob.
Approval-state stage: compared the staged ADR-0012 with candidate blob
`a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c`, and the staged register and
changelog with the candidate blobs; the observed changes are exactly the
approval-state lines listed in section 4 and in "Approval-state validation".
Recovery stage: compared the staged register with the approval-state blob
`b19b1404dafa62a7aaebcd33708841cf44ce1831`; the only changed line is the
`Last updated` header line and the ADR-0012 row is byte-identical; confirmed
that ADR-0012, the changelog and every protected decision and control file
keep their approval-state blob identities.
Amendment-6/7 candidate stage: compared the staged ADR-0012 with the recovery
head's ADR blob `4848470a252076268a9fac9da5c40f0b4ebff474` against every
requirement of Amendment 6 G3-G9, Amendment 7 H3-H11 and the eight
candidate-inspection clarifications. Observed result: each is expressed in
section 3.2 and its subsections, sections 3.3, 3.9 and 3.15, invariants 3-6,
8 and 35-40, and sections 8-12, and each required validation case appears in
section 11. Every changed ADR hunk concerns route contracts, eligibility,
mismatch, enrollment, withdrawal, emission/producer coverage, materializer
compatibility, route health, rollout/rollback/validation of those concerns or
the approval state; the ADR-0008/ADR-0010 partial-supersession header text and
all other architecture are byte-identical to the recovery head. The phrase
"supported event kind/version contract" and every other route-level
kind/version contract statement are gone; the only remaining "kind/version"
phrases describe the revision predicate, recorded anomaly facts or job
kind/version retirement. The candidate's register, changelog and report
described the corrected version as `PROPOSED` and bound both earlier approvals
to the earlier exact versions; its ADR-0012 carried no current approver or
approval date.
Amendment-6/7 approval-state stage: compared the staged ADR-0012 with the
content-approved candidate blob `cd897d576522a741c3dfa82f8de2f8d80d4288f6` by
the exact normalized comparison in section 9 (no difference), and read the
raw delta line by line; compared the staged register and changelog with the
candidate blobs (only the ADR-0012 row and the route-contract correction entry
change); confirmed the protected decision and control blobs are unchanged; and
read the report for statements that would become false once it moved into the
approval-state commit and reframed them.
Control-record recovery stage: confirmed that only this report is staged and
that ADR-0012, the decision register, the changelog and every protected file
keep their approval-state blobs; confirmed by a fixed-string search that the
removed content-approval identifier no longer occurs in this report; and read
the report for statements that would become false once it moves into the
recovery commit, or that treated the identifier transcription as an accepted
deviation or a review item, and reframed or removed them.
Transient-state correction stage: confirmed that only this report is staged
and that ADR-0012, the decision register, the changelog and every protected
file keep their recovery-head blobs; searched the whole report for every
exact-count and current-state review-thread wording named by the correction
authorization and for equivalents, and read every remaining mention of review
threads to confirm it is historical stage-time evidence phrased as such,
durable substantive finding or disposition information, or the statement that
current thread state is GitHub-owned; confirmed by fixed-string search that no
review, thread or comment identifier of the final-head review of the recovery
head, and no identifier of the control adjudication that classified it, occurs
in this report; confirmed that the report's source lineage is unchanged; and
read the report for statements that would become false once it moves into the
correction commit and reframed them.
Factual-clarification stage: compared the staged ADR-0012 with the prior
approved blob `a7d95ca918f418d596432071c8ba78a33c2630a9` hunk by hunk (three
hunks, each one cardinality sentence and its rewrapped continuation);
confirmed for each of the seven disposition cases listed in section 9 that
the clarified text states it without ambiguity and that the surrounding
mismatch, enrollment, divergence, uniqueness-boundary and idempotency text is
byte-identical; confirmed that header lines 3-6 and section 12 are unchanged;
searched the report for every current-gate formulation named by the
clarification authorization and reframed each as the durable lifecycle model
or as explicitly historical stage evidence; confirmed by fixed-string search
that no review, thread or review-comment identifier of the final-head review
of the transient-state correction head occurs in this report; and read the
report for statements that would become false once it moves into the
clarification commit and reframed them.
Amendment-8 candidate stage: compared the staged ADR-0012 with the
clarification head's ADR blob `2db08058552ce901548ef3f71af4a294f5af7e61` hunk
by hunk against every requirement of Amendment 8 I3-I8 and the five
candidate-inspection observations; confirmed each is expressed in section 3.2
(the serialization paragraph, the historical-enrollment availability check,
the "Revision withdrawal and route retirement" subsection and activation gate
4), invariant 41, sections 8, 9 and 11, and the candidate state in the header
and section 12; confirmed that every changed hunk concerns withdrawal/selection
ordering, its validation or the candidate state, that the withdrawal and
retirement paragraphs the candidate did not change are byte-identical, and
that the ADR-0008/ADR-0010 partial-supersession header text and all other
architecture are byte-identical to the clarification head; compared the staged
register and changelog with the clarification-head blobs (only the header line
and the ADR-0012 row, and only one added entry); confirmed the protected
decision and control blobs are unchanged; and read the report for statements
that would become false once it moves into the candidate commit and reframed
them.
Amendment-8 approval-state stage: compared the staged ADR-0012 with the
content-approved candidate blob `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0` by
the exact normalized comparison in section 9 (no difference), and read the raw
delta line by line; compared the staged register and changelog with the
candidate blobs (only the ADR-0012 row and the withdrawal-order correction
entry change); confirmed the protected decision and control blobs are
unchanged; confirmed by fixed-string search that the content-approval comment
identifier does not occur in this report; and read the report for statements
that would become false once it moves into the approval-state commit and
reframed them.
Amendment-9 candidate stage: compared the staged ADR-0012 with the
approval-state head's ADR blob `85937a4e1ff9a27d8f5b36ca82651821711472b2` hunk
by hunk against every requirement of Amendment 9 J4-J11 and the six
candidate-inspection observations; confirmed each is expressed in section 3.5
(the not-yet-started sentence, the idempotency-conflict item and the new
subsection), section 3.15, invariants 8 and 42, sections 8, 9 and 11, and the
candidate state in the header and section 12; confirmed that every changed
hunk concerns coalescing attachment versus first claim/start, its validation
or the candidate state, and that the ADR-0008/ADR-0010 partial-supersession
header text, the Amendment-8 ordering text, the claim, attempt-phase,
idempotency, target-concurrency and disposition-uniqueness text and all other
architecture are byte-identical to the approval-state head; compared the
staged register and changelog with the approval-state-head blobs (only the
ADR-0012 row, and only one added entry); confirmed the protected decision and
control blobs are unchanged; and read the report for statements that would
become false once it moves into the candidate commit and reframed them.
Amendment-9 approval-state stage: compared the staged ADR-0012 with the
content-approved candidate blob `0fc33df9aca61a2c82af452426f5574bfd3493cf` by
the exact normalized comparison in section 9 (no difference), and read the raw
delta line by line; compared the staged register and changelog with the
candidate blobs (only the `Last updated` header and the ADR-0012 row, and only
the coalescing-seal correction entry, change); confirmed the protected
decision and control blobs are unchanged; confirmed by fixed-string search
that the content-approval comment identifier does not occur in this report;
and read the report for statements that would become false once it moves into
the approval-state commit and reframed them.
Evidence link/screenshot/log reference: the local validation in section 9.
Pull-request diffs of earlier heads, including the published activation-epoch
candidate `95769537cdc9871e42f9f3e5443604a96beb74da`, its approval-state head
`c046e6bf6510f24ee1231f86293a3d499bceed7e`, the recovery head
`9db91af7bf3d9f969556c819187420fc31d25c10`, the content-approved
Amendment-6/7 candidate `b596536e7db0d6b8ace0a42fdd26bff42cabf104`, the
Amendment-6/7 approval-state head `609a01427847233b7280557baa61825ac2079865`
the control-record recovery head
`22bd94dc959f11fe49c8bef8cd2a836140aadff7`, the transient-state correction
head `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a`, the factual-clarification
head `eb3129f13a9c858b3c012ee182315801f0b8563a`, the content-approved
Amendment-8 candidate `42ae90a5932cf24ce90679196888cc3d83d88db8`, the
Amendment-8 approval-state head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` and
the content-approved Amendment-9 candidate
`8699af5635b8f18691ceaeccd3b24707a71ef4f2`, are historical evidence; the live
PR #960 diff of the approval-state commit exists only after its publication
and is GitHub-owned lifecycle evidence.

## 11. CI

CI status: GitHub-owned lifecycle evidence for each PR head; expected
documentation-only classification (`classify` and `check-changelog` pass;
build, test, lint, format, migration and Docker jobs skipped).
Checks: as listed in section 4.3.
Failures or warnings: NONE EXPECTED. Under Amendment 3 C10, the Amendment-4/5
recovery authorization, the Amendment-6/7 candidate and approval-state
authorizations, the control-record recovery authorization, the
transient-state correction authorization, the factual-clarification
authorization, the Amendment-8 candidate authorization, the Amendment-8
approval-state authorization, the Amendment-9 candidate authorization and the
Amendment-9 approval-state authorization, an automatic CI failure, or any
migration execution or Docker publication, is a HOLD condition.

## 12. Rollout and rollback

Initial state after merge: after a guarded merge into `feature/worker`, the
approved ADR-0012 version would be reachable from the programme integration
branch only. It is not repository-authoritative until its authority condition,
including reachability from `develop`, is met, and ADR-0013 programme-local
reliance additionally requires the programme-pin conditions on #957.
Independent exact-head review of whichever head is presented for merge, CTO
merge authorization where the task's risk requires it, the guarded merge,
programme-pin/reliance reconciliation and any later migration, deployment,
activation and observation are distinct controlled lifecycle categories under
the repository release gates and the task specification; GitHub, through #958
and PR #960, is authoritative for which of them is pending, satisfied or
blocked for any head, and this report does not record that state.
Activation required: NONE. Merging the eventual approved version would make no
runtime, schema, provider or deployment change.
Feature flag/configuration: NONE
Migration sequence: NONE
Rollback/disable procedure: before merge, stop at whichever lifecycle gate
GitHub records as pending; no history rewrite, force push or amend is
permitted. The Amendment-6/7 authorizations permitted exactly one candidate
commit and exactly one approval-state commit, the control-record recovery and
transient-state correction authorizations each permitted exactly one
report-only commit, the factual-clarification authorization permitted exactly
one two-path clarification commit, the Amendment-8 candidate authorization
permitted exactly one four-path candidate commit, the Amendment-8
approval-state authorization permitted exactly one direct-child approval-state
commit, the Amendment-9 candidate authorization permitted exactly one
four-path candidate commit, and the Amendment-9 approval-state authorization
permits exactly one direct-child approval-state commit; any further source
correction exhausts those authorizations and requires a fresh CTO
authorization (Amendment 3 C10; Amendments 6-9;
`docs/engineering/decisions/README.md`).
Monitoring required: NONE

## 13. Known limitations and deferred work

- Historical stage-time evidence: at the Amendment-9 approval-state stage the
  implementing agent performed no independent inspection or review,
  exact-final-blob approval or confirmation, PR #960 title/description
  reconciliation, Draft/Ready change, automated-review request, thread
  adjudication, exact-head review or merge (section 4.2).
  Those are separate controlled lifecycle categories; GitHub, through #958 and
  PR #960, is authoritative for their current state, including whether the
  PR title and description match the current head.
- Programme-local reliance on ADR-0012 under ADR-0013 is exact-version-bound
  and separately controlled. Any #957 pin bound to an earlier ADR-0012 blob or
  PR #960 head is stale for the corrected version until superseded using the
  exact eventually approved ADR-0012 blob and an independently reviewed
  PR #960 head;
  dependent `feature/worker` slices must be rechecked and recorded as affected
  or `NONE` with the inspected evidence (Amendment 1 A9, Amendment 2 B9,
  Amendment 3 C9) before that supersession; ADR-0013 conditions 7 and 8 follow
  the actual merge and reachability evidence. #957 is authoritative for the
  current pin state.
- The repository-wide Route-B recovery ambiguity raised on PR #967 (finding
  `4168223564`) is not resolved here. For #958, Amendment 3 C10 fails closed:
  any correction needing another candidate commit, another approval-state
  commit, amend, rebase, force push or history rewrite requires a fresh CTO
  specification amendment/authorization. The final-head register-freshness
  finding was handled that way (Amendments 4-5 and one recovery commit), and
  so was the final-head route-contract finding (Amendments 6-7, one candidate
  commit and one approval-state commit). The approval-state control-record
  defect was likewise corrected only under a separate authorization for one
  report-only recovery commit, the transient review-thread-state defect found
  by the final-head review of that recovery head only under a separate
  authorization for one report-only correction commit, and the overbroad
  disposition-cardinality sentence and remaining live-gate wording found by
  the final-head review of that correction head only under the CTO
  factual-clarification classification and a separate authorization for one
  two-path clarification commit, and the withdrawal/selection ordering defect
  found by the final-head review of that clarification head only under
  Specification Amendment 8, its independent review and a separate
  authorization for one four-path candidate commit, its approval state only
  under a separate authorization for one direct-child approval-state commit,
  and the coalescing-attachment/first-claim ordering defect found by the
  final-head review of that approval-state head only under Specification
  Amendment 9, its independent review and a separate authorization for one
  four-path candidate commit, and its approval state only under a separate
  authorization for this one direct-child approval-state commit.
- The strand-check timing question left outside this correction (section 5,
  decision 53): Amendments 1-8 do not determine whether a withdrawal's
  strand-prevention evaluation may be stale relative to events or anomalies
  that commit concurrently with the withdrawal and are not revision
  selections. The candidate neither decides it nor implies a mechanism for
  it; it belongs to a later specification or to the implementation
  specification.
- Exact table, column, index and encoding names for route-contract revisions,
  anomalies and capability declarations are deliberately left to the future
  shared-engine implementation specification.
- The projection-evidence limitation of the historical heads
  `227057cae8a4c10f7f7795e3afe68271e56681b9` and
  `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` (Appendix A) remains historical;
  section 9 is exact-worktree evidence for the activation-epoch candidate, its
  approval-state commit, the recovery commit, the Amendment-6/7 candidate, its
  approval-state commit, the control-record recovery, the transient-state
  correction, the factual clarification, the Amendment-8 candidate and
  approval-state commit, the Amendment-9 candidate and this Amendment-9
  approval-state commit.

## 14. Unresolved issues

- The PR #960 review threads `4146838604` and `4148985175` concern
  implementation-report validation evidence. Their Route-B disposition under
  Amendment 3 C5 was the approval-state report update, preserved and extended
  by the later stages: section 9 records the literal
  validation commands and results of every source stage, the candidate ADR
  blobs, the six-path and four-path footprints and CI applicability, without
  self-referential containing-commit metadata.
- Thread `4146838614` concerns the disposition-to-job mapping; the prior blob
  corrected section 3.5 and invariant 8, and section 3.2 states the same
  model.
- Thread `4148985154` concerns BE-04 consumer impact; the verified
  per-consumer record is in Appendix A.
- Thread `4156929919` concerns the task-branch namespace; the base
  specification retains the approved transitional exception for
  `feature/async/adr-0012` (section 1).
- Thread `4157508169` concerns repeated route activation; it is the
  `R-b467-01` defect that the activation-epoch correction addresses.
- The configured final-head automated review of the approval-state head opened
  two further threads, identified here substantively only. One asserted that
  the approval-state commit was not the direct child of the content-approved
  candidate; its premise did not match the authoritative PR #960 commit graph
  (the approval-state head's single parent is the candidate) and it required
  no source correction. The other identified the stale `Last updated` register
  header; it was accepted as blocking and was addressed by the recovery commit.
- The configured final-head automated review of the recovery head opened one
  further thread, identified here substantively only: the route-level event
  kind/version contract could reinterpret historical epochs (`R-9DB-01`). It
  was accepted as valid and blocking and closed at specification level by
  Amendments 6-7; the content-approved candidate implements that correction
  in source and the approval-state commit
  `609a01427847233b7280557baa61825ac2079865` records its approval.
- The configured final-head automated review of the control-record recovery
  head `22bd94dc959f11fe49c8bef8cd2a836140aadff7` opened two further threads,
  identified here substantively only. One asserted that the recovery head's
  lineage matched a cited object whose sole parent was the refreshed
  `feature/worker` base and which changed all six PR paths; that object is not
  the PR #960 head, its premise did not match the authoritative GitHub PR
  head/parent graph (the recovery head's single parent is the approval-state
  head `609a01427847233b7280557baa61825ac2079865` and its delta from that
  parent is this report alone), and control rejected it as an invalid premise
  and non-blocking; no source or lineage correction was made. The other
  identified that the recovery report treated the current number and
  resolved/unresolved state of PR #960 review threads as durable repository
  truth; control accepted it as a valid blocking control-record defect, and
  the transient-state correction commit
  `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` addressed only that defect.
- The configured final-head automated review of the transient-state
  correction head `9b4bb1b02204aa2267a4c0b75016fa96c5e22f4a` opened two
  further threads, identified here substantively only; control accepted both
  as valid and blocking. One found that ADR-0012's unconditional "exactly one
  durable disposition exists for each `(event_id, route_key)`" wording in
  section 3.2, section 3.5 and invariant 8 was broader than the eligibility,
  backlog, mismatch, enrollment and divergence semantics the same ADR selects;
  the CTO classified its correction as a factual clarification of prior
  approved blob `a7d95ca918f418d596432071c8ba78a33c2630a9`, and the
  historical clarification commit `eb3129f13a9c858b3c012ee182315801f0b8563a`
  applied it (section 4; section 5, decisions 44-46). The other found that the
  report still recorded which live PR #960 lifecycle gates were pending; the
  same commit replaced that wording with the durable lifecycle model (section
  5, decision 47).
- The configured final-head automated review of the factual-clarification
  head `eb3129f13a9c858b3c012ee182315801f0b8563a` opened one further thread,
  identified here substantively and by its control-assigned finding name
  `R-EB31-01`: revision withdrawal could race future revision selection,
  because the ADR did not require withdrawal to share a database-atomic order
  with the operations that newly select a revision for an epoch or a first
  historical enrollment, so a stale availability observation could commit a
  new reference after the withdrawal. Control accepted it as valid, HIGH and
  blocking and classified the correction as a material architectural
  correction; the CTO approved Specification Amendment 8; a fresh independent
  CRITICAL specification review of the base specification and Amendments 1-8
  returned `APPROVED`, confirming the finding, the classification and the
  validation matrix and recording five non-blocking candidate-inspection
  observations; the content-approved Route-B candidate
  `42ae90a5932cf24ce90679196888cc3d83d88db8` implements the correction in
  source (section 4; section 5, decisions 49-60); control's independent
  inspection of that candidate found no blocking exact-candidate finding; the
  CTO approved its exact content on 2026-10-07; the historical approval-state
  commit `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` recorded that approval; and
  the CTO recorded exact-final-blob approval of the resulting ADR-0012 blob
  `85937a4e1ff9a27d8f5b36ca82651821711472b2`. `R-EB31-01` is closed at
  specification level by Amendment 8 and, per the CTO's Amendment-9 record,
  closed in the source of that approval-state head; its thread adjudication
  belongs to the review controls that GitHub records.
- The configured final-head automated review of the Amendment-8
  approval-state head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d` opened one further
  thread, identified here substantively and by its control-assigned finding
  name `R-E258-01`: a router could observe a coalescible job as not yet
  started, a worker could claim/start it, and the router could then attach a
  new route disposition to the started job from its stale observation, so
  that the event was recorded as represented by a job whose inputs or effect
  may already have been fixed without it; the ADR required database-enforced
  claims but not that attachment and first claim/start share one
  database-atomic order. Control accepted it as valid, HIGH and blocking and
  classified the correction as a material architectural correction; the CTO
  approved Specification Amendment 9; a fresh independent CRITICAL
  specification review of the base specification and Amendments 1-9 returned
  `APPROVED`, confirming the finding, the classification, the seal boundary
  and the validation matrix and recording six non-blocking
  candidate-inspection observations; the content-approved Route-B candidate
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2` implements the correction in
  source (section 4; section 5, decisions 69-80); control's independent
  inspection of that candidate found no blocking exact-candidate finding;
  Javi, CTO approved its exact content on 2026-10-08; and the approval-state
  commit that contains this report records that approval. `R-E258-01` is
  closed at specification level by Amendment 9; its source closure is
  adjudicated by the independent exact-head review controls that precede any
  merge, which GitHub records.

PR #960 holds the historical and current review threads covering the findings
summarized above. GitHub is authoritative for their current number,
resolved/unresolved state and adjudication; this report records only their
durable substantive dispositions and source evidence. Adjudication of review
threads belongs to the independent exact-head review controls that precede any
merge, not to this report, which neither resolves nor dismisses any thread.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

The implementing agent does not approve, mark merge-ready or merge its own
work. The implementing agent of this Amendment-9 approval-state commit also
implemented the Amendment-9 candidate, the Amendment-8 approval-state commit
and candidate, the factual clarification, the transient-state correction, the
control-record recovery, the Amendment-6/7 approval-state commit and candidate
and earlier Route-B source stages of this task and prepared the earlier
uncommitted
activation-epoch candidate referred to in Amendment 1 A10, so it is not
eligible to act as an independent inspector or reviewer of this task
(Amendment 5 F8; Amendment 6 G12; Amendment 7 H13).

Suggested review focus:

- that the approval-state delta from the content-approved Amendment-9
  candidate `8699af5635b8f18691ceaeccd3b24707a71ef4f2` is limited to the ADR
  header approval fields, the two leading section-12 paragraphs, the ADR-0012
  register row and `Last updated` header and the coalescing-seal correction
  changelog entry, that the normalized comparison in section 9 shows no
  architecture drift from candidate blob
  `0fc33df9aca61a2c82af452426f5574bfd3493cf`, and that the approval-state
  ADR-0012 blob is `cbe8322d95fe8d5e9911d421a5f10f1ae4afd55c`;
- that the content approval is recorded only through owning issue #958,
  approver Javi, CTO, approval date 2026-10-08, candidate head
  `8699af5635b8f18691ceaeccd3b24707a71ef4f2` and candidate ADR-0012 blob
  `0fc33df9aca61a2c82af452426f5574bfd3493cf`, that the approval date matches
  the CTO candidate-content approval record, and that no content-approval,
  exact-final-blob approval, review or merge identifier is copied into this
  report (section 5, decision 84);
- that the candidate changed exactly the four authorized paths from the
  Amendment-8 approval-state head `e2588f7b7e63a5c71968d87a3409f0ff48635e6d`,
  that the candidate ADR-0012 blob is `0fc33df9aca61a2c82af452426f5574bfd3493cf`,
  and that ADR-0008, ADR-0010 and ADR-0013 keep their protected blobs;
- that every Amendment 9 J4-J11 requirement is expressed in ADR-0012: the
  coalescing-open/coalescing-sealed boundary anchored to the first successful
  claim/start as the same transition as "not yet started", the permanence of
  the seal through retry, lease reclaim, WAITING/resume, reconciliation,
  cancellation, recovery and later claims, the database-atomic
  attachment-versus-first-claim order with the attachment write proving the
  job open and pre-checks insufficient, both commit orders, the aborted-claim
  rule, mechanism-neutral input/effect completeness, the same boundary for
  every already-permitted attachment path, the seal's distinction from
  `EFFECT_STARTED`, the composition list, invariant 42 and the nine section-11
  cases with their evidence refinements, and that the six independent-review
  observations are reflected as recorded in section 5, decisions 70, 75 and
  79;
- that the candidate changes no coalescing permission, idempotency-key
  identity, `CURRENT_STATE`/`REVISION_BOUND`, target-concurrency,
  applied-revision, attempt-phase, claim-token, WAITING/RECONCILIATION_REQUIRED
  or Amendment-8 ordering semantics, and that decision 75 decides no new
  coalescing path;
- that every ADR-0012 hunk outside those concerns is limited to the approval
  state (header and section 12) and that all other architecture is
  byte-identical to the Amendment-8 approval-state head and to the
  content-approved Amendment-9 candidate;
- that the approval-state delta from the content-approved Amendment-8
  candidate `42ae90a5932cf24ce90679196888cc3d83d88db8` is limited to the ADR
  header approval fields, the two leading section-12 paragraphs, the ADR-0012
  register row and the withdrawal-order correction changelog entry, that the
  normalized comparison in section 9 shows no architecture drift from
  candidate blob `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`, and that the
  approval-state ADR-0012 blob is `85937a4e1ff9a27d8f5b36ca82651821711472b2`;
- that the content approval is recorded only through owning issue #958, the
  approval date 2026-10-07, candidate head
  `42ae90a5932cf24ce90679196888cc3d83d88db8` and candidate ADR-0012 blob
  `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`, that the approval date matches
  the CTO candidate-content approval record, and that no content-approval,
  exact-final-blob approval, review or merge identifier is copied into this
  report (section 5, decision 64);
- that the candidate changed exactly the four authorized paths from the
  clarification head `eb3129f13a9c858b3c012ee182315801f0b8563a`, that the
  candidate ADR-0012 blob is `cbb31a53f6f4ed6b96faafb79b273b55664f3ae0`, and that
  ADR-0008, ADR-0010 and ADR-0013 keep their protected blobs;
- that every Amendment 8 I3-I8 requirement is expressed in ADR-0012: the
  general "every new durable selection/reference" rule and its enumerated
  minimum, the exclusion of non-durable preselection, both commit orders, the
  re-read-before-retry rule, the insufficiency of application-level
  check-then-insert, the permitted mechanisms and composition requirements,
  the exclusion of explicit replay/current-state work, invariant 41 and the
  nine section-11 concurrency cases, and that the five independent-review
  observations are reflected as recorded in section 5, decisions 50, 53 and
  56;
- that the candidate changes no withdrawal semantics other than the ordering
  rule, invents no withdrawal reversibility, equivalent-signature
  re-enablement, second lifecycle order or route abstraction, and leaves the
  strand-check timing question outside the correction as decision 53 records;
- that every ADR-0012 hunk outside those concerns is limited to the approval
  state (header and section 12) and that all other architecture is
  byte-identical to the clarification head and to the content-approved
  candidate;
- that the approval-state delta from the content-approved candidate
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104` is limited to the ADR header
  approval fields, the two leading section-12 paragraphs, the ADR-0012
  register row and the route-contract correction changelog entry, and that the
  normalized comparison in section 9 shows no architecture drift;
- that the content approval is recorded only through owning issue #958, the
  approval date 2026-10-06, candidate head
  `b596536e7db0d6b8ace0a42fdd26bff42cabf104` and candidate ADR-0012 blob
  `cd897d576522a741c3dfa82f8de2f8d80d4288f6`, that the approval date matches
  the CTO candidate-content approval record, and that the approval-state
  ADR-0012 blob `a7d95ca918f418d596432071c8ba78a33c2630a9` is the exact prior
  approved blob that the factual clarification identifies;
- that the factual clarification changes ADR-0012 only in the three
  cardinality sentences recorded in section 4 and section 9
  ("Factual-clarification validation"), that the clarified blob is
  `2db08058552ce901548ef3f71af4a294f5af7e61`, that it continues to require
  an eventual single disposition for every eligible route obligation, to
  prevent a second disposition for any `(event_id, route_key)` and to state
  every no-disposition case unchanged, that the header approval fields
  and section 12 are unchanged, and that no sentence of this report states
  which PR #960 lifecycle gate is currently pending (section 5, decisions
  44-48);
- that the transient-state correction changed only this report, that
  ADR-0012, the decision register and the changelog keep their recovery-head
  blobs, that no sentence presents the current count or resolved/unresolved
  state of PR #960 review threads as durable repository truth, that the
  report's source lineage is unchanged, and that no review, thread or comment
  identifier of the final-head review of the recovery head is copied into the
  report (section 5, decisions 40-43; section 9, "Transient-state correction
  validation");
- that the control-record recovery changed only this report, that ADR-0012,
  the decision register and the changelog keep their approval-state blobs, and
  that no GitHub content-approval, exact-final-blob approval, review or merge
  identifier is copied into the report (section 5, decisions 36-39; section 9,
  "Control-record recovery validation");
- that every Amendment 6 G3-G9 and Amendment 7 H3-H11 requirement, and each of
  the eight candidate-inspection clarifications, is expressed in ADR-0012
  without retaining contradictory route-level-contract wording;
- the interpretations recorded in section 5 as items 16, 19, 20, 22 and 25:
  the scope of the in-epoch mismatch predicate, the materializer check in
  enrollment validation, the meaning of withdrawal from future selection,
  anomaly-resolution recording and the register `Last updated` header;
- that the database-level revision-equivalence property (section 3.2) rules
  out an application-level check-then-insert race;
- that the producer-version coverage text covers rollback targets, cross-route
  fan-out, newly emittable versions and non-binary writers;
- that the new invariants 35-40 and the reconciled invariants 3-6 and 8 agree
  with section 3.2;
- that the header and section 12 carry `APPROVED` with Javi, CTO and
  2026-10-08, identify the approved content by candidate ADR-0012 blob
  `0fc33df9aca61a2c82af452426f5574bfd3493cf`, bind the 2026-09-30, 2026-10-05,
  2026-10-06 and 2026-10-07 approvals to the earlier exact versions and keep
  the repository authority condition and the ADR-0013 reliance distinction;
- that the register row, the coalescing-seal correction changelog entry and
  this report describe the corrected version as `APPROVED` on 2026-10-08,
  that the withdrawal-order correction entry and every other historical
  changelog entry are byte-preserved, and that the register header
  `Last updated` is `2026-10-08`;
- that ADR-0008, ADR-0010, ADR-0013, the decision README and the other
  protected control files are byte-identical to the approval-state heads, the
  clarification head, the recovery head and the content-approved candidates.

## Appendix A - Historical authoring and correction record

This appendix preserves the earlier task record. Headings are demoted and only
the tense or framing needed to keep it truthful as history is changed.
Statements such as "the ADR now requires" describe ADR-0012 as of the round in
which they were written; later rounds and the Route-B corrections supersede them
where they differ. In particular, the round-4 route model of one activation
boundary and one optional deactivation boundary per route is superseded by the
append-only activation-epoch model, and the route-level event kind/version
contract and "emits K" emission floor described in earlier rounds are
superseded by immutable route-contract revisions and producer-version coverage
in the current section 3.2. The 2026-09-30, 2026-10-05, 2026-10-06 and
2026-10-07 approvals and every review recorded here are bound to the exact
historical versions and heads they name.

### Initial candidate and review round 1

Initial candidate:

```text
a69921682cd02e060110304f8f9868d0c39da65e
```

The independent CRITICAL review returned `CHANGES REQUIRED`; durable receipt:
#958 comment `5896883454`.

F-01 through F-09 were accepted. Round-2 candidate:

```text
9d329e063b2731b691c62a412555b470f2a9fe39
```

The round-1 corrections established precise ADR-0008/ADR-0010 boundaries,
crash-safe route identity, effect-phase checkpoints, bounded scheduling,
released-contract handling, worker task-role-only AWS authority, operability and
documentation hygiene.

### Independent review round 2

The independent CRITICAL review of
`9d329e063b2731b691c62a412555b470f2a9fe39` again returned
`CHANGES REQUIRED`; durable receipt: #958 comment `5909933000`.

Seven blockers were accepted:

- R2-01 - incomplete WAITING/resume/attempt/concurrency/reconciliation lifecycle;
- R2-02 - resource-ambiguous idempotency keys and coalescing contradiction;
- R2-03 - event-routing completeness vulnerable to commit-order and mixed-version
  worker assumptions;
- R2-04 - incomplete BE-04 consumer/API coverage, including `thoth-app`;
- R2-05 - unsafe same-release BE-04 table DROP under the actual rolling
  migrate-on-start deployment model;
- R2-06 - untrusted publisher-file parsing sharing high Hosting authority;
- R2-07 - unbounded immutable payload snapshots/personal-data duplication.

### Round-3 corrections

The final tightening also explicitly qualifies ADR-0010 section 4.4's statement
that ADR-0008 remains fully binding: `ServiceOperation` remains non-queue audit
infrastructure, while ADR-0012 supplies the later explicit shared-framework
exception ADR-0008 anticipated.

#### Job lifecycle

The ADR now requires:

- WAITING to atomically persist checkpoint state and release claim/lease;
- WAITING/poll attempts not to consume the pre-write retry budget, instead being
  bounded by the waiting deadline/poll policy;
- resume from WAITING as a new claim/attempt;
- separate bounded pre-write retry budget and post-write waiting/deadline;
- post-write deadline exhaustion to become `RECONCILIATION_REQUIRED`, not
  `FAILED`;
- WAITING and RECONCILIATION_REQUIRED to retain their concurrency key by
  default;
- effectful work to start only when its remaining lease/runtime can cover the
  declared effect window;
- reconciliation itself to be a claimed, claim-token-fenced, kind-authorized
  attempt;
- durable `EFFECT_CONFIRMED` evidence to permit non-duplicating recovery where
  a kind defines it.

#### Effect-scoped idempotency and coalescing

Idempotency now identifies a specific intended effect, not merely a resource.

Keys are derived from the event/route, source revision/fingerprint, schedule
slot, command/request id or explicit coalescing window as appropriate.

Each `(event_id, route_key)` has its own durable route record. Several route
records may target one not-yet-started job only under an explicit coalescing
rule. Idempotency conflicts never silently mark a route complete.

#### Routing completeness

Route registrations and activation boundaries are durable PostgreSQL state.
Activation uses a durable routing generation/epoch or equivalently strong
serialized database mechanism captured transactionally by events/routes, so
out-of-order commits cannot create an ambiguous boundary.

Routing completeness is determined per eligible route, not from worker-local
handler knowledge or an unproved sequence high-water mark. Mixed worker
versions cannot silently complete an unknown active route, and backfill uses the
same route materialization records.

#### Payload minimization

Event/job payloads default to canonical IDs, source revision/fingerprint and
minimal command/routing parameters.

Credentials are prohibited. Personal data or immutable domain snapshots require
kind-specific justification and retention/erasure handling. Current-state work
reads canonical state at execution time.

#### BE-04 expand-migrate-contract transition

The previous state-translating compatibility-facade design has been removed.

The architecture now requires:

1. **Expand** - Release N adds the generic schema/API while retaining legacy
   BE-04 schema/API and keeping legacy automatic job creation inactive.
2. **Migrate consumers** - independently migrate both known released consumers:
   `thoth-dissemination` and `thoth-app`.
3. **Retire legacy API** - later remove legacy lifecycle/read/filter/toggle
   GraphQL contract after all consumers are verified migrated.
4. **Contract storage** - only in a still-later migration, after old code is no
   longer a supported rollback target; the migration itself asserts all three
   legacy tables contain zero rows and aborts otherwise.

Known `thoth-app` dependencies are explicitly recorded:
`latestBackCatalogueJob`, `jobStatuses`,
`withoutBackCatalogueJob` and generated `DistributionJobStatus` contract.

Historical `20260814_v1.7.0` remains immutable.
`distribution_platform` remains.

#### Deployment and rollback

The ADR now incorporates the authoritative current deployment property that the
GraphQL image defaults to `thoth init`, which runs migrations before serving,
while production uses multiple rolling ECS tasks.

Therefore destructive contraction cannot occur in the same release that moves
the API. The previous binary/schema remains rollback-compatible throughout
expand/migrate/API-retirement phases; after generic data exists, runtime rollback
must preserve that data and use forward recovery rather than deleting/reverting
it.

#### Trust-separated worker pools

The default trusted worker still uses one routine task role containing the union
of approved routine trusted capabilities.

Handlers that parse publisher-controlled/untrusted content are a distinct
`UNTRUSTED_CONTENT` risk class and run in a separate task/service/process
boundary with minimal authority and no Hosting DNS/ACM/CloudFront
tenant-management permissions unless explicit CTO risk acceptance records the
blast radius. That pool also does not inherit the primary application database
credential: it must use the kind-scoped executor API or a separately reviewed
minimal database principal.

This is a real trust-boundary exception, not one IAM role per ordinary handler.

#### Validation additions

The ADR now requires tests for:

- mixed worker versions;
- out-of-order commits;
- route completeness/backfill;
- effect-scoped idempotency and coalescing;
- WAITING claim release/resume;
- pre-write versus post-write exhaustion;
- concurrency-key retention;
- claimed reconciliation races;
- payload minimization;
- untrusted-worker IAM isolation;
- both downstream consumer migrations;
- rolling deploy/rollback safety;
- contraction abort on any legacy row.

### Independent review round 3

The independent CRITICAL review of
`2d5eb1ae1e25bc222c4ca94a4f6dc5e81ecceba8` returned
`CHANGES REQUIRED`; durable receipt: #958 comment `5911717880`.

Four blockers were accepted:

- R3-01 HIGH - route activation had no durable deactivation/retirement boundary
  and was not gated on every serving/rollback binary emitting the consumed event
  kind;
- R3-02 MEDIUM - Publisher Services still accessed `distribution_job*` outside
  the named GraphQL surfaces, making later DROP unsafe during rolling deploy and
  rollback;
- R3-03 MEDIUM - untrusted-content isolation still allowed served-object writes
  or trusted-handler steering through untrusted destination data;
- R3-04 MEDIUM - job kinds did not declare current-state versus revision-bound
  semantics, permitting an older retried revision to overwrite newer applied
  state.

### Round-4 corrections

#### Route activation, retirement and emission floor

Routes now have durable activation and optional deactivation boundaries under the
same routing-generation mechanism.

Activation/deactivation is separately authorized and audited rather than a side
effect of deploying handler code. Events eligible before deactivation remain
owed after route retirement; route registrations are never deleted to erase
outstanding obligations. An eligible route that cannot be materialized requires
an explicit authorized/audited terminal route disposition.

A route for event kind/version K may activate only after every serving binary
and supported rollback binary capable of the relevant business mutation emits K
atomically. Rollback below that emission floor requires prior deactivation or an
explicitly authorized current-state reconciliation/backfill covering the exact
gap.

#### Revision semantics

Every job kind now declares either:

```text
CURRENT_STATE
REVISION_BOUND
```

CURRENT_STATE jobs read current canonical state when they execute and may safely
coalesce/no-op under their declared rule.

REVISION_BOUND jobs bind to a retained revision/fingerprint and must not apply an
older revision after a newer revision of the same effect target has already been
applied unless the operation is an explicitly authorized historical
replay/rollback.

#### Full BE-04 runtime retirement before contraction

The Phase-C retirement release now covers every runtime access to
`distribution_job*`, not only named legacy GraphQL fields.

The ADR explicitly includes the current
`replacePublisherServiceConfiguration` runtime dependencies:

- automatic back-catalogue creation;
- the fail-closed AUTOMATIC_PUSH activation guard;
- assignment-disable legacy-job cancellation.

Those paths must be redirected to the generic lifecycle or remain conservatively
fail-closed before the legacy tables can be dropped.

A repository-wide source/runtime audit must prove the deployed and supported
rollback Phase-C binaries have zero runtime legacy-table access.

Phase D then performs only the storage DROP plus removal of remaining inert
schema/model declarations. Its migration takes locks preventing a concurrent
legacy write, performs the final zero-row assertion and DROP in the same
fail-closed migration transaction, and retains `distribution_platform`.

#### Untrusted-content quarantine and trusted promotion

UNTRUSTED_CONTENT remains part of the same canonical async engine but executes
in a separate minimal-authority pool.

It may read authorized input and write only quarantine/staging output. It cannot
write served publisher/Hosting locations or choose final domain/bucket/key/
Hosting targets.

Untrusted completion results are explicitly untrusted data. A trusted promotion
job validates the artifact and resolves all authority-bearing destination values
from canonical Thoth state before publishing.

The untrusted worker may report only its declared completion/evidence facts and
cannot create arbitrary downstream work.

If direct database access is chosen instead of the protected kind-scoped
executor API, ordinary table grants are insufficient: the separately reviewed
database boundary must enforce row/kind scope through RLS, security-definer
functions/procedures or an equivalent mechanism.

#### Additional safety clarifications

Round 4 also records that:

- `EFFECT_CONFIRMED` is anti-replay execution evidence and is not by itself
  ADR-0010 observed publication/acceptance/current-state truth;
- attempt diagnostics/provider evidence are covered by data
  minimization/retention/erasure rules;
- jobs blocked behind WAITING/RECONCILIATION_REQUIRED concurrency keys must be
  operationally visible;
- high-volume job families require retention/coalescing/admission/backpressure
  policy before activation;
- graceful-shutdown rules apply to every worker pool.

### Independent review round 4

The independent CRITICAL review of
`ced4cb19224d7acc1824863a85d83b2387617449` returned `CHANGES REQUIRED`.

Three MEDIUM blockers were accepted:

- R4-01 - durable route obligations lacked route-level backlog/materializer
  visibility and could be terminally dispositioned without explicit divergence
  attention;
- R4-02 - UNTRUSTED_CONTENT IAM/database separation did not prevent cross-tenant
  persistence or network access to shared private services;
- R4-03 - stale-write protection and concurrency were still kind-scoped rather
  than shared by every job kind mutating the same external effect target.

The accepted non-blocking clarifications also covered the BE-04 migration-backfill
runtime path, queue-specific error-contract retirement, canonical writes outside
normal event-emitting binaries, safe job-kind retirement, explicit upstream
Publisher Services generic creation ownership and contraction-revert semantics.

### Round-5 corrections

#### Route obligations and materializer health

The ADR now requires per-route count/oldest-age signals for eligible events with
no disposition, explicit visibility when no compatible materializer is deployed,
and route activation only after both the event-emission floor and a compatible
materializer exist. A terminal route disposition without covering
reconciliation/backfill is an accepted divergence requiring durable attention;
bulk actions still create one audited disposition per event/route.

#### Canonical external effect targets

Effectful kinds now resolve domain-owned canonical external effect-target
identities independent of job kind. Every writer of one target shares the same
target-scoped concurrency namespace and durable applied-revision/fingerprint
evidence. Batch jobs decompose by target by default; retained multi-target jobs
must acquire complete target serialization deterministically and maintain
per-target evidence.

#### Stronger untrusted-content isolation

UNTRUSTED_CONTENT execution now requires cross-publisher process/state isolation,
job-scoped staging authority, independently verified trusted promotion and a
deny-by-default network boundary. The pool cannot inherit private-subnet access to
Redis, EFS, unrelated RDS/database endpoints or other internal services merely
because IAM/database credentials are restricted.

#### Retirement and rollback clarifications

The ADR now names the current migration-backfill runtime path and released
`DISTRIBUTION_JOB_CREATION_DISABLED` compatibility surface in Phase C, requires
non-emitting migration/repair writes to emit transactionally or receive exact-scope
reconciliation/backfill, prevents job-kind retirement from orphaning non-terminal
work, assigns the Publisher Services generic creation/cancellation/activation
contract upstream to `thoth`, and makes Phase-D contraction a forward-repair
boundary rather than an automatic restoration of BE-04.

#### Validation additions

Round 5 adds explicit validation for route backlog/materializer health, accepted
divergence, cross-kind target serialization/evidence, provider-without-revision
ordering, cross-publisher untrusted isolation, per-job staging, private-network
denial, trusted staging provenance, job-kind retirement, non-binary domain writes,
migration-backfill/error-contract retirement and contraction-revert behaviour.

### Independent review round 5

The independent CRITICAL review of
`d25bb1ecd1665cc4e635b16dbd5fd09582a5889b` returned `CHANGES REQUIRED`.
R4-01 through R4-03 were verified closed with no regression identified in the
earlier F/R2/R3 findings.

One MEDIUM blocker was accepted:

- R5-01 - trusted promotion validated a staged object but did not bind
  publication to the exact immutable bytes/version validated or require the
  untrusted writer's authority to end first, leaving a validate-then-swap TOCTOU
  path.

The missing round-4/round-5 lifecycle chain was reconciled in GitHub under #958
comment `5912886635`; the programme gate was reconciled under #957 comment
`5912887392`.

### Round-6 corrections

#### Trusted promotion integrity

Untrusted staging-write authority is now current-claim-scoped and bounded in
lifetime. Trusted validation cannot begin until write authority over the selected
artifact has ended (or an unrevocable capability has expired / an immutable
provider-version boundary is established). The trusted side seals and resolves
the artifact identity, independently validates it, and may publish only the exact
immutable bytes/version that passed validation. A mutable key cannot bridge the
validation/publication boundary; any replacement requires fresh validation.

#### Effect-target fencing and identity

Target evidence is now current-claim-fenced, updated only while holding the
target serialization boundary and committed atomically with the durable local
effect/job outcome. It is monotonic except under authorized historical replay.
CURRENT_STATE fingerprints cover every canonical input determining the effect.
Exactly one domain owns a shared effect-target identity; identity changes have
explicit old/new-target disposition rules; multi-target locking is all-or-nothing
rather than waiting while holding a subset.

#### Untrusted execution and egress

The isolation rule is job-to-job, including jobs for the same publisher. Staging
capabilities are claim-bound and excluded from durable payload/attempt records.
The deny-by-default network boundary explicitly covers outbound traffic, limits
object storage to approved Thoth input/staging locations and requires explicit
allowlisting of other dependencies.

#### Operator and validation tightening

Staff-created terminal route dispositions use ADR-0010's protected audited command
seam. Validation now covers complete CURRENT_STATE fingerprints, target-evidence
claim fencing and monotonicity, target-identity transitions, all-or-nothing
multi-target locking, same-publisher job isolation, staging-capability expiry,
egress restrictions and staged-object replacement after validation.

### Migration/data effect

No migration was created, modified or executed.

Production's v1.7.0 legacy schema fact remains an architecture input. The ADR no
longer proposes immediate replacement/drop: generic schema expansion precedes
consumer migration/API retirement, and legacy storage contraction is a later
separately authorized migration.

### Authorization/security effect

No authorization implementation changed.

No IAM role/policy, credential, provider configuration or environment variable
was created, rotated or changed.

ADR-0008 domain-specific application authorization remains binding. Worker-pool
IAM authority and ZITADEL/application authorization remain separate concepts.

### Approval-state and review history (pre-correction versions)

The architecture content at
`631e28d1f05495f24ce88369d4557f1033f970b3` received CTO exact-content
approval before the durable approval-state reconciliation.

That approval-state reconciliation made the following durable control changes
without altering the selected architecture:

- ADR-0012 recorded `Status: APPROVED`, `Approved by: Javi, CTO`,
  `Approval date: 2026-09-30` and the approval/authority rule in section 12;
- ADR-0008's header recorded only the explicit partial-supersession boundary
  selected by ADR-0012, leaving its unaffected machine-role and least-privilege
  controls binding;
- ADR-0010's header recorded only the explicit partial-supersession boundary
  selected by ADR-0012, leaving its unaffected Staff Operations,
  `ServiceOperation`, desired/execution/observed-state, attention,
  reconciliation and staff-command controls binding;
- the engineering decision register recorded ADR-0012 as `APPROVED` with its
  repository-authority condition and preserved the distinction between
  architecture approval and implementation authorization;
- the Unreleased changelog gained the durable ADR-0012 approval entry while
  preserving the earlier proposal entry.

The five-path approval-state source authorization existed before that mutation.
Its omitted GitHub authorization receipt was reconciled as historical provenance
in #958 comment `5914033493`; the reconciliation explicitly does not portray
the later comment as retroactive authorization.

The approval-state candidate reached
`c43aa750ea4d971fe4f803cf761868876f0783bd` and received
`CHANGES REQUIRED` for AS-01. The direct-child AS-01 correction
`bd5223deee44b465dd860ff96b75d81ad41cbf2d` restored the repository ADR
amendment rule. Fresh independent CRITICAL review then returned `APPROVED`
for that exact head in #958 comment `5914564111`.

The later programme-integration compatibility reconciliation produced
`7f7e62824eb40308688b46351ffea182e9e52942` against
`feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462`. Fresh independent
CRITICAL review of that exact integration candidate returned `CHANGES REQUIRED`
in #958 comment `5919509096`. The accepted blockers were missing exact
validation evidence, transient/incomplete durable report history and the
contradiction between section 3.2's terminal route dispositions and the
section 3.5/invariant-8 every-route-record-maps-to-a-job wording. The latter is
a factual clarification of the already-selected section 3.2 semantics, not a
new architecture decision.

### Verified BE-04 consumer impact

The independent consumer-impact check for the retiring BE-04 identifiers is
durably recorded as follows:

- `thoth-pub/thoth-app`: downstream migration required;
- `thoth-pub/thoth-dissemination`: downstream migration required;
- `thoth-pub/thoth-pyramid`: unaffected by the retiring BE-04 identifiers;
- standalone `thoth-pub/thoth-client`: unaffected;
- internal Rust `thoth-client` / `thoth-export-server`: unaffected;
- `thoth-pub/metrics-dashboard`: unaffected;
- `thoth-pub/metrics-widget`: unaffected;
- `thoth-pub/baboon`: unaffected;
- `thoth-pub/thoth-sphinx`: not a verified active consumer;
- `thoth-pub/infrastructure`: runtime/IAM substrate, not a GraphQL consumer.

This record is compatibility evidence for the BE-04 retirement plan. It does
not authorize either downstream migration or removal of the legacy contract.

### Programme-integration compatibility reconciliation

After ADR-0013 became repository-authoritative and THOTH-ASYNC-01 explicitly
opted into programme-integration reliance, the programme created
`feature/worker` from current `develop` and retargeted PR #960 to that
programme integration branch.

The compatibility reconciliation is bound to:

```text
reviewed ADR-0012 source before reconciliation:
bd5223deee44b465dd860ff96b75d81ad41cbf2d

programme integration base:
feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462

historical common base:
923545d5c9028bc04c40e38efeb7de674efed3fd
```

The histories had diverged. The only content changed on both sides since the
historical common base was `CHANGELOG.md` and
`docs/engineering/decisions/decision-register.md`.

The compatibility reconciliation preserved the then-reviewed ADR-0012,
ADR-0008 and ADR-0010 decision blobs byte-for-byte, while carrying the
repository-authoritative ADR-0013, branching workflow and operating-model
doctrine from `feature/worker` unchanged. The subsequent factual ADR-0012
clarification changed the exact ADR-0012 version but did not alter ADR-0013 or
those shared control documents.

ADR-0012's normal repository-authority condition remains reachability from
`develop`. ADR-0013's narrower programme-local reliance mechanism applies only
to an exact approved and independently reviewed ADR version that is merged into
the designated `feature/worker` line and durably pinned by the programme. A
change to ADR-0012 makes any earlier exact-version programme pin stale until the
changed version has received fresh independent CRITICAL review, explicit CTO
approval and separate programme-pin reconciliation. This report does not assert
that programme-local reliance is currently effective.

### Historical external/runtime effects

```text
provider reads: 0
provider writes: 0
migration execution: 0
CI dispatch/rerun: 0
deployment: 0
release/publication: 0
production activation: 0
```

### Historical validation evidence and provenance

#### Correction at `227057cae8a4c10f7f7795e3afe68271e56681b9`

Before the route-disposition clarification commit, the proposed source bytes
were frozen, the three modified-file blob identities were computed, and the
created GitHub blobs matched those expected identities exactly. The cumulative
six-path footprint, the three-path correction boundary, the unchanged control
blobs and full-file whitespace/composition checks were also verified.

The literal commands recorded by the report version at that head:

```text
git diff --check HEAD
git diff --name-only HEAD
```

were run in a controlled six-path **projection** workspace. That workspace did
not contain all six exact GitHub file bytes. Those command results are therefore
historical projection evidence only and are not represented here as exact-byte
Git validation of that candidate. This provenance defect was recorded
durably in #958 comment `5934574460` and the corresponding programme HOLD in
#957 comment `5934575236`.

After push, GitHub verification of exact head
`227057cae8a4c10f7f7795e3afe68271e56681b9` established:

```text
tree:
1645d0ebdca5df231d3fa72ede229cccab76771e

parent:
7f7e62824eb40308688b46351ffea182e9e52942

correction paths:
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md

cumulative PR paths against feature/worker:
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Exact final blobs at that head were:

```text
CHANGELOG.md:
66cfbb5b56dfe7a9382614e274a872c68df26dcb

implementation report:
104df8c02d8aa8077e720a63bb3c16415afe54c3

ADR-0008:
622060ad90eec41c792f110c373efffbb11a4b56

ADR-0010:
aca2142a3387785e80db908b3a5c1b0afd82ab51

ADR-0012:
dec6665353804e42f22474954bf878308092b717

decision register:
fc64dddbf6beddd602d31d694d9dae7c40c417c2

ADR-0013:
d928f957bdf775d03e99fc73888ec8e2dec5f80b

branching-and-release-workflow:
6edce35dd29307b0554bcd122ed5518038f57df5

operating-model:
fc8b9c90a7d0ae98a44645d2e316a0573c13ce2a
```

Automatic PR CI for that head completed with four successful jobs and six
classification-skipped jobs, with no failure or in-progress job and no manual
dispatch/rerun.

#### Report-only provenance correction at `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`

The correction authorized by #958 comment `5934661557` changed only the
implementation report from parent
`227057cae8a4c10f7f7795e3afe68271e56681b9`.

Before commit, the complete candidate bytes for all six cumulative PR paths were
read from the exact GitHub parent, with only the report replaced by its
proposed corrected bytes. Full-file byte-level whitespace checks found no
trailing whitespace on any candidate file.

The exact cumulative-base blobs are:

```text
CHANGELOG.md:
cf662733b61354f08f7d6b26ead3b8079f10d8ba

implementation report:
ABSENT

ADR-0008:
19273fe60520b8a907c8b5a28e76d4fe741bb832

ADR-0010:
0e3b047435dbdcd2fdd15cbfc9ac4c1fc2e601ce

ADR-0012:
ABSENT

decision register:
ed03bcc6766f05fb7f026468689727b3c42f4c94
```

The five unchanged candidate paths retained their exact parent blob identities:

```text
CHANGELOG.md:
66cfbb5b56dfe7a9382614e274a872c68df26dcb

ADR-0008:
622060ad90eec41c792f110c373efffbb11a4b56

ADR-0010:
aca2142a3387785e80db908b3a5c1b0afd82ab51

ADR-0012:
dec6665353804e42f22474954bf878308092b717

decision register:
fc64dddbf6beddd602d31d694d9dae7c40c417c2
```

The proposed report bytes were Git-blob hashed before commit; the exact proposed
blob is recorded in the owning #958 completion handoff rather than self-pinned
inside the bytes whose hash it would change.

GitHub's exact base-to-parent comparison contains exactly the six cumulative PR
paths listed above. Because that correction changed only the report, the
cumulative path set remained those same six paths and its direct correction
delta was exactly:

```text
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
```

The execution shell available for that control step could not resolve GitHub
and could not materialize an exact repository worktree. Consequently that report
version did **not** claim that literal local Git commands were executed against a full
exact-byte worktree.

The required literal commands were nevertheless re-run without restrictive
pathspecs in a six-path path/projection workspace after the exact GitHub-byte
checks:

```text
git diff --check HEAD

exit: 0
stdout: ""
stderr: ""

git diff --name-only HEAD

exit: 0
stdout:
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
stderr: ""
```

Those literal command results are projection evidence only. Exact-byte evidence
was instead the complete GitHub file reads, Git blob identities, full-file
whitespace checks and exact GitHub compare path sets above. This limitation
remains part of the historical record; it is not converted into stronger
evidence by assertion.

Composition and doctrine checks at that head:

- changelog had one `[Unreleased]` structure and retained ADR-0013, ADR-0012
  proposal, approval and factual-clarification entries;
- decision register was byte-identical to its parent and recorded ADR-0012 as
  `APPROVED` while retaining ADR-0013's programme-integration rule;
- section 3.2's terminal-disposition semantics remained unchanged and section 3.5
  plus invariant 8 used the same materialized-versus-terminal disposition model;
- ADR-0013, branching workflow and operating-model doctrine remained
  byte-preserved;
- repository names used canonical spellings, including `thoth-sphinx`.

Test applicability:

```text
runtime/unit tests: NOT APPLICABLE
database/migration tests: NOT APPLICABLE
provider/runtime tests: NOT APPLICABLE
manual CI dispatch/rerun: NO
final-head automatic CI: GitHub-owned post-push evidence
```

### Historical deviation history

During initial issue creation, the GitHub connector omitted the issue number in
its normalized create response, briefly producing a `#undefined` parent
reference in #958. The returned URLs established #957/#958 and the issue was
corrected before branch/source mutation.

The source write budget remained compliant. The validation-evidence provenance
deviation at head `227057cae8a4c10f7f7795e3afe68271e56681b9` is recorded in
#958 comment `5934574460`; the report-only correction at
`b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` removed the unsupported
exact-worktree claim rather than retroactively curing it.

## Appendix B - Durable control conditions

These conditions remain true regardless of the current live PR/review/merge
stage:

1. ADR-0012 is an architecture decision only. Approval or merge does not
   authorize runtime implementation, schema/data migration, provider/IAM/runtime
   mutation, external writes, deployment, release or production activation.
2. Any exact ADR-0012 version used for merge-readiness or programme-local
   reliance requires the review/approval evidence required by repository
   doctrine for that exact version; a later source change invalidates
   exact-head approval for the earlier version.
3. ADR-0012 becomes repository-authoritative only under its section-12 authority
   condition, including reachability from `develop`.
4. ADR-0013 programme-local reliance, when used, is exact-version-bound: the
   relied-upon ADR version must be approved, independently reviewed, merged into
   the designated `feature/worker` line and durably pinned. Any later ADR-0012
   change makes the earlier programme pin stale until separately reconciled.
5. Every async implementation slice remains separately specified, authorized,
   independently reviewed and integrated. Downstream repositories may not guess
   an unmerged upstream contract and must use an explicitly authorized pinned
   version/preview where applicable.
6. Merge authorization, merge, migration execution, provider/IAM/runtime
   mutation, deployment, release, production activation and observation remain
   separate lifecycle gates.
7. Historical migration `20260814_v1.7.0` remains immutable; BE-04 retirement
   continues to require the approved expand -> consumer migration -> runtime/API
   retirement -> rollback-floor closure -> later storage-contraction sequence.
8. The terminal route-disposition clarification does not permit silent work
   loss: terminal non-materialization remains explicit, authorized, audited and
   subject to the section 3.2 reconciliation/backfill and attention semantics.
9. The activation-epoch model does not permit silent work loss either: events
   eligible through any epoch remain owed after deactivation and retirement,
   and events from an inactive inter-epoch gap are processed only through
   explicitly authorized backfill/replay under the same `(event_id, route_key)`
   deduplication boundary.
10. A materially corrected ADR-0012 version may carry `APPROVED` only after CTO
    approval of that exact corrected content; earlier approvals and reviews,
    including the 2026-09-30, 2026-10-05, 2026-10-06 and 2026-10-07
    approvals, remain bound to the earlier exact versions and heads they name.
11. The route-contract-revision model does not permit silent reinterpretation
    or loss either: every epoch is bound to one immutable revision, an event
    outside its epoch revision's accepted versions becomes a visible anomaly
    rather than disappearing, historical enrollment validates its selected
    revision before consuming the `(event_id, route_key)` slot, and revision
    withdrawal or route retirement never strands obligations or anomalies.
12. The withdrawal/selection ordering rule does not permit a stale
    availability observation to create a new epoch reference or first
    enrollment for a withdrawn revision: withdrawal and new selection have
    one durable linearized order, a selection that linearized first remains
    valid historical state, and a withdrawal that linearized first refuses
    later selection before any epoch, disposition or job exists.
13. The coalescing-seal rule does not permit a route disposition to be
    attached from a stale observation to a job that has already started: a
    logical job of a coalescing kind is coalescing-open only until its first
    successful claim/start commits, attachment and that transition have one
    durable linearized order, an attachment that linearized first is genuinely
    represented by the started job, and a claim/start that linearized first
    refuses the later attachment with no disposition committed, the
    `(event_id, route_key)` slot unused and the obligation still owed.

This report authorizes none of those later actions.
