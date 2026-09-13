# BE-06-R52B-PERSIST-01 Implementation Report

## 0. Nature of this report

This is the implementation report for `BE-06-R52B-PERSIST-01`, the
documentation/control task that persisted the independently approved BE-06 R52B
specification through [PR #914](https://github.com/thoth-pub/thoth/pull/914).

It is added **retrospectively** by the recovery task
`BE-06-R52B-PERSIST-REPORT-01`, under the CTO-authorized recovery specification
[#848 comment 5648330082](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5648330082).
Root `AGENTS.md` section 14 requires an implementation report before review. No
report existed while PR #914 was reviewed or when it merged. That omission was
found by a post-ready automated review and reconciled after merge (section 12).
This file does not retroactively make a report present during PR #914 review or
merge, and nothing in it should be read that way.

The report records the original task from durable GitHub and Git evidence. Where
a fact comes only from the implementing agent session and has no durable GitHub
record, it is labelled **session record (not durable)**.

Sections 1 to 16 report the original persistence task `BE-06-R52B-PERSIST-01`
(PR #914). Section 17 is the implementation report for the recovery task
`BE-06-R52B-PERSIST-REPORT-01` itself (PR #915). Section 18 is the
implementation report for the durability correction
`BE-06-R52B-PERSIST-REPORT-DURABILITY-01`.

Decision recorded by this report:
PR #914 persisted the approved R52B bytes exactly; its control deviations are
recorded in sections 5 and 12; no BE-06 runtime implementation occurred.

Authority condition:
This record is repository-authoritative when this exact content is reachable
from the repository's authoritative integration branch (`develop`).
`feature/publisher-services-v1-10` is the Publisher Services programme
integration branch and the target of PR #914 and PR #915. Reaching it does not
make this record repository-authoritative.

Live review, authorization and merge evidence:
GitHub pull-request history.

## 1. Repository state

### 1.1 Original persistence task (`BE-06-R52B-PERSIST-01`)

```text
Programme:                   Publisher Services and Distribution Configuration
Owning GitHub issue:         #848
Parent programme issue:      #765
Repository:                  thoth-pub/thoth
Task ID:                     BE-06-R52B-PERSIST-01
Workflow:                    PROGRAMME_INTEGRATION
Underlying BE-06 risk:       CRITICAL
Nature:                      documentation/control only
Base branch / PR target:     feature/publisher-services-v1-10
Programme integration branch: feature/publisher-services-v1-10
Authorized base commit:      395cc16ac770bc8bbf8a708662a1b31d85b15398
Actual base commit:          395cc16ac770bc8bbf8a708662a1b31d85b15398
Task branch:                 feature/publisher-services-v1-10--be-06-r52b-spec-record
First pushed head:           f7a1cc670f3ebd4f26f367eb0cc2e1275240586e
Final reviewed head:         6cdb4ee43050b4d33a80907581a899db18645d37
Pull request:                #914
Merge commit:                8a711db7f61d0af9658261fc338badbee8431610
Merge parents:               1. 395cc16ac770bc8bbf8a708662a1b31d85b15398
                             2. 6cdb4ee43050b4d33a80907581a899db18645d37
Merge tree:                  1d8f47366f24ed3805f476cdbb857baedf474509
Merged at / by:              2026-09-12T19:17:42Z / ja573
Expected branch deletion after merge: YES
Final programme PR required: YES
```

The first version of this report recorded `Final programme PR required: NO` for
PR #914. That was incorrect: PR #914 merged into the programme integration branch,
which reaches `develop` only through the separately gated final programme
integration. The review correction in section 17 changes the field to `YES`.

Implementing models (session record, not durable):

- first persistence commit `f7a1cc67`: Claude Sonnet 5, high reasoning;
- attribution/changelog correction producing `6cdb4ee4`: Claude Opus 5, high
  reasoning.

The persistence task ran in the same agent session that had earlier performed the
independent R52B specification review recorded in
[comment 5646962729](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5646962729).
That session did not author R52B. The persistence work itself was independently
reviewed separately ([comment 5648111825](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5648111825)).

### 1.2 Recovery task (`BE-06-R52B-PERSIST-REPORT-01`)

```text
Task ID:                     BE-06-R52B-PERSIST-REPORT-01
Risk:                        MEDIUM documentation/control
Workflow:                    PROGRAMME_INTEGRATION
Authorization/specification: #848 comment 5648330082
Base branch / PR target:     feature/publisher-services-v1-10
Authorized base commit:      8a711db7f61d0af9658261fc338badbee8431610
Actual base commit:          8a711db7f61d0af9658261fc338badbee8431610
Task branch:                 feature/publisher-services-v1-10--be-06-r52b-persist-report
Pull request:                #915 (opened as draft)
Head commit:                 authoritative in GitHub pull-request history
Implementing model:          Claude Opus 5 (claude-opus-5), high reasoning
```

The recovery handoff named Claude Sonnet 5 as the implementing model. The session
ran as Claude Opus 5. Recovery specification `5648330082` does not name a model.

The complete implementation report for this recovery task is section 17.

## 2. Scope confirmation

Approved specification:
`docs/publisher-services/specifications/BE-06-R52B.md`, approved at the
specification gate by
[#848 comment 5646962729](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5646962729):

```text
SHA-256: 584683ca02611afb064335db83d1fb22685fc3dae87f4eeeee2e1490b320f3ce
bytes:   699166
lines:   5107
```

Persistence objective:
Move the exact approved bytes out of disposable agent scratch state into the
repository, byte-identical, without altering the specification. Comment
`5646962729` permitted "a separately bounded documentation-only persistence
branch/PR" for that purpose. It did not authorize BE-06 implementation.

Specification bytes unchanged: YES. The known LOW §8.7/T324 wording that calls the
new BE-06 permit guard the "released permit guard" was deliberately left
uncorrected, because any byte change would invalidate the exact-SHA approval.
Comment `5646962729` records that the phrase is to be read as "the new BE-06
permit guard".

This recovery task does not modify the specification.

Out-of-scope repository changes made by PR #914: NONE. The final diff contained
only the two paths in section 4. The control deviations are in section 5.

## 3. Commits

Original persistence branch:

- `f7a1cc670f3ebd4f26f367eb0cc2e1275240586e` -
  `docs: persist approved BE-06 R52B specification`, first pushed head, with
  trailer `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Replaced by
  a force-with-lease push (GitHub `head_ref_force_pushed` event,
  2026-09-12T19:13:19Z).
- `6cdb4ee43050b4d33a80907581a899db18645d37` -
  `docs: persist approved BE-06 R52B specification`, the amended replacement, with
  no trailer or body. This is the reviewed and merged head.

Both commits have exactly one parent,
`395cc16ac770bc8bbf8a708662a1b31d85b15398`. The branch was exactly one commit
ahead of that base at the reviewed head.

## 4. Files changed

Authorized new-file path in the original persistence handoff (session record, not
durable):

- `docs/publisher-services/specifications/BE-06-R52B.md`

Authorized existing-file modification, as finally retained (section 5.2):

- `CHANGELOG.md`, one persistence entry under `## [Unreleased]` / `### Added`

Actual files changed between `395cc16a` and reviewed head `6cdb4ee4`:

- `docs/publisher-services/specifications/BE-06-R52B.md`
  - reason: durable persistence of the approved specification;
  - behavioural effect: none (documentation);
  - within authorized new-file list: YES;
  - `git diff --numstat`: `5107 0`.
- `CHANGELOG.md`
  - reason: root `AGENTS.md` section 13 requires every PR to add an entry;
  - behavioural effect: none;
  - within authorized write budget: YES as finally retained, with the provenance
    gap recorded in section 5.2;
  - `git diff --numstat`: `1 0`. The entry references issue #848, PR #914 and
    approval comment `5646962729`.

Between `f7a1cc67` and `6cdb4ee4` only `CHANGELOG.md` changed (`1 1`). That was
the insertion of the PR #914 reference.

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS on the final reviewed diff. Changelog authorization provenance is recorded as
a deviation in section 5.2.

### 4.2 Actions used by the original persistence task

- repository inspection: used;
- source edit: `CHANGELOG.md` only;
- new file creation: `docs/publisher-services/specifications/BE-06-R52B.md`;
- file deletion/move/rename: not used;
- branch creation: `feature/publisher-services-v1-10--be-06-r52b-spec-record`
  from `395cc16a`;
- commit: used; amended once under CTO correction authorization;
- push: used; one normal push, then one force-with-lease push leased to
  `f7a1cc67`;
- PR creation/update: PR #914 created as draft; the implementing agent did not
  edit its body after creation, so the body does not describe the section 5
  deviations;
- issue/comment mutation: not used by the implementing agent;
- manual CI dispatch/rerun: not used;
- provider/runtime read or write: not used;
- migration execution: not used;
- release/tag/publication: not used;
- merge, deployment, production activation: not used by the implementing agent.
  The merge in section 11 was separately authorized and performed by `ja573`.

Local workspace handling (session record, not durable): the implementing session
first created a separate local worktree for the new branch. The agent harness
refused writes outside the session's own worktree, so that newly created local
worktree was removed and the branch was checked out in the session's own worktree.
No repository file other than the two above was written.

Unauthorized actions performed: see section 5. The attribution trailer was a
bounded-handoff violation and was corrected before review.

### 4.3 Automatic and manual external effects

Automatic effects: PR-triggered workflows at both heads (section 10). At both
heads `build_and_push_staging_docker_image` was **skipped**, so no staging Docker
image was published by PR #914.

Manually initiated external actions by the implementing agent: NONE.

External writes/publication: NONE.

## 5. Deviations and provenance

### 5.1 Unauthorized AI attribution

The first pushed commit `f7a1cc67` included:

```text
Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
```

The persistence handoff prohibited AI attribution, so the trailer violated the
bounded handoff. It was not authorized when made. The implementing session's
report justified it as a non-waivable platform policy. That justification was
wrong, and the later correction session withdrew it.

The CTO then explicitly authorized a correction: amend the single commit to remove
the trailer and force-with-lease push the existing branch. The exact authorization
text reached the correction session in its handoff (session record, not durable).
Recovery specification `5648330082` durably records "the initial unauthorized
AI-attribution trailer and its CTO-authorized correction".

The amended commit `6cdb4ee4` contains no `Co-Authored-By` trailer, no body and no
generated-by text. Review comment `5648111825` records that the reviewed head
"contains no AI-attribution trailer". This correction does not make the original
trailer authorized.

### 5.2 Changelog authorization provenance gap

The original persistence handoff authorized exactly one new file and no
`CHANGELOG.md` change. It instructed the agent to stop if repository policy
required a changelog entry. Root `AGENTS.md` section 13 does require one for every
PR.

Session record (not durable): the implementing agent stopped at preflight and
reported the conflict. It then received an in-session instruction authorizing the
changelog write-budget expansion, and proceeded. Its completion report described
the change as explicitly reauthorized. That authorization was not independently
recorded in the durable control ledger.

The CTO subsequently and explicitly authorized:

- retaining the existing changelog persistence entry;
- adding the PR #914 reference to it;
- acknowledging that the earlier authorization was not durably evidenced;
- not treating the later authorization as retroactive proof that the earlier
  authorization existed.

Recovery specification `5648330082` durably records "the earlier
changelog-authorization provenance gap and the CTO authorization that
retained/corrected it". The retained entry does not prove that the earlier
authorization was durable.

### 5.3 Missing implementation report

PR #914 was reviewed and merged without the implementation report required by
root `AGENTS.md` section 14. See section 12. This file is the additive remedy.

## 6. Database and migration effects

Migration added: NO.

## 7. API and compatibility effects

```text
Runtime effect:                      NONE
Database/schema effect:              NONE
Migration effect:                    NONE
GraphQL/API effect:                  NONE
Authorization implementation effect: NONE
Provider effect:                     NONE
Crossref external-write effect:      NONE
Workflow definition effect:          NONE
Deployment effect:                   NONE
Production activation effect:        NONE
Cross-repository contract effect:    NONE
Metrics effect:                      NONE
```

Generated schema/client updates: NONE. Deprecations: NONE.

## 8. Authorization and security

Authorization paths changed: NONE. Roles/scopes involved: NONE.
Secret or personal-data handling: NONE.

Governance effect: persisting R52B, approving its persistence and merging PR #914
did not authorize BE-06 implementation, the BE-06 implementation branch, source or
migration work, migration execution, release, deployment or activation.

## 9. Tests and checks

These commands were re-run by the recovery task on 2026-09-12 against the durable
Git objects. Outputs are exact.

### R52B committed-blob identity at reviewed head

The persistence and correction sessions ran the equivalent `git show HEAD:...`
check before their respective pushes.

```bash
git show 6cdb4ee43050b4d33a80907581a899db18645d37:docs/publisher-services/specifications/BE-06-R52B.md | sha256sum
git show 6cdb4ee43050b4d33a80907581a899db18645d37:docs/publisher-services/specifications/BE-06-R52B.md | wc -c
git show 6cdb4ee43050b4d33a80907581a899db18645d37:docs/publisher-services/specifications/BE-06-R52B.md | wc -l
```

```text
584683ca02611afb064335db83d1fb22685fc3dae87f4eeeee2e1490b320f3ce  -
699166
5107
```

### R52B blob unchanged across correction and merge

```bash
git rev-parse \
  f7a1cc670f3ebd4f26f367eb0cc2e1275240586e:docs/publisher-services/specifications/BE-06-R52B.md \
  6cdb4ee43050b4d33a80907581a899db18645d37:docs/publisher-services/specifications/BE-06-R52B.md \
  8a711db7f61d0af9658261fc338badbee8431610:docs/publisher-services/specifications/BE-06-R52B.md
```

```text
cabeddb84edb8fbfb5a3cdf3c1dc3e19c7799cfa
cabeddb84edb8fbfb5a3cdf3c1dc3e19c7799cfa
cabeddb84edb8fbfb5a3cdf3c1dc3e19c7799cfa
```

### Commit topology

```bash
git rev-list --parents -n 1 f7a1cc670f3ebd4f26f367eb0cc2e1275240586e
git rev-list --parents -n 1 6cdb4ee43050b4d33a80907581a899db18645d37
git rev-list --parents -n 1 8a711db7f61d0af9658261fc338badbee8431610
git rev-list --count 395cc16ac770bc8bbf8a708662a1b31d85b15398..6cdb4ee43050b4d33a80907581a899db18645d37
```

```text
f7a1cc670f3ebd4f26f367eb0cc2e1275240586e 395cc16ac770bc8bbf8a708662a1b31d85b15398
6cdb4ee43050b4d33a80907581a899db18645d37 395cc16ac770bc8bbf8a708662a1b31d85b15398
8a711db7f61d0af9658261fc338badbee8431610 395cc16ac770bc8bbf8a708662a1b31d85b15398 6cdb4ee43050b4d33a80907581a899db18645d37
1
```

Tree of `6cdb4ee4` and of merge `8a711db7`:
`1d8f47366f24ed3805f476cdbb857baedf474509`. Tree of `f7a1cc67`:
`4c8927479965eda970a5ff4d9b21f329e4a4dc37`.

### Diff scope

```bash
git diff --name-status 395cc16ac770bc8bbf8a708662a1b31d85b15398 6cdb4ee43050b4d33a80907581a899db18645d37
git diff --name-status f7a1cc670f3ebd4f26f367eb0cc2e1275240586e 6cdb4ee43050b4d33a80907581a899db18645d37
```

```text
M	CHANGELOG.md
A	docs/publisher-services/specifications/BE-06-R52B.md

M	CHANGELOG.md
```

### Attribution trailers

```bash
git log -1 --format='%(trailers:key=Co-Authored-By)' f7a1cc670f3ebd4f26f367eb0cc2e1275240586e
git log -1 --format='%(trailers:key=Co-Authored-By)' 6cdb4ee43050b4d33a80907581a899db18645d37
```

```text
Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>

(no output)
```

### Formatting, unit, integration and lint checks

Not applicable. No Rust, SQL or generated file changed, and the repository
classified the change as documentation-only (section 10).

## 10. Manual verification and CI

Manual verification: the byte-identity checks above. Before the first push the
persistence session also compared the destination with the discovered scratch
source (`cmp`: identical) and hashed the working tree and committed blob. Before
the force-with-lease push the correction session hashed the committed blob and
confirmed the R52B blob id was unchanged (session record, not durable; the
re-derived Git evidence above is durable).

Workflow runs, all with event `pull_request` and none manually dispatched, rerun or
cancelled:

First pushed head `f7a1cc670f3ebd4f26f367eb0cc2e1275240586e`:

```text
34704994741 check-changelog:      success  (check-changelog: success)
34704994760 build-test-and-check: success  (classify: success; build, test, lint, format_check: skipped)
34704994753 run-migrations:       success  (classify: success; run_migrations: skipped)
34704994763 publish-to-dockerhub: success  (classify: success; build_and_push_staging_docker_image: skipped)
```

Reviewed head `6cdb4ee43050b4d33a80907581a899db18645d37`:

```text
34713526690 check-changelog:      success  (check-changelog: success)
34713526694 build-test-and-check: success  (classify: success; build, test, lint, format_check: skipped)
34713526638 run-migrations:       success  (classify: success; run_migrations: skipped)
34713526682 publish-to-dockerhub: success  (classify: success; build_and_push_staging_docker_image: skipped)
```

CI status: PASSING at workflow level. `check-changelog` and the classifiers
succeeded. The runtime jobs `build`, `test`, `lint`, `format_check` and
`run_migrations`, and the staging image job, were **skipped** under
documentation-only classification. They did not pass, and no staging Docker image
was published.

## 11. Independent review and merge

- Specification-level approval of the R52B bytes:
  [comment 5646962729](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5646962729).
- Exact-head persistence review, `APPROVED` for head `6cdb4ee4`:
  [comment 5648111825](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5648111825)
  (2026-09-12T19:17:25Z).
- The same comment records separate CTO authorization to mark PR #914 ready and
  merge that exact head, only if `feature/publisher-services-v1-10` was still
  `395cc16ac770bc8bbf8a708662a1b31d85b15398` immediately before merge.
- PR #914 was marked ready at 2026-09-12T19:17:33Z and merged at
  2026-09-12T19:17:42Z by `ja573` into
  `8a711db7f61d0af9658261fc338badbee8431610`, whose first parent is that
  authorized target SHA.
- Merge reconciliation:
  [comment 5648166766](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5648166766).

Neither the persistence approval nor the merge authorized BE-06 implementation.

## 12. Post-ready control finding

Marking PR #914 ready triggered the repository's automated Codex review
(`chatgpt-codex-connector[bot]`, review trigger "Draft marked ready"). It reviewed
`6cdb4ee4` and submitted review
[`5187716235`](https://github.com/thoth-pub/thoth/pull/914#pullrequestreview-5187716235)
at 2026-09-12T19:21:32Z, a P1 finding "Add the mandatory implementation report".

The review was submitted after the merge at 2026-09-12T19:17:42Z. The control
plane independently verified the finding against root `AGENTS.md` section 14.
Comment `5648166766` records the resulting post-merge hold on BE-06 implementation
authorization. The recovery is additive: this report, through
`BE-06-R52B-PERSIST-REPORT-01`, rather than rewriting or reverting PR #914.

## 13. Rollout and rollback

Initial state after merge: R52B present on `feature/publisher-services-v1-10` as
documentation only.
Activation required: NONE. Feature flag/configuration: NONE.
Migration sequence: NONE. Monitoring required: NONE.

Rollout: repository documentation/control only. No service rollout, migration or
provider action.

Rollback: if this report is materially wrong, correct or revert it through another
bounded, reviewed documentation PR. No production or runtime rollback applies.
Reverting the R52B persistence itself would need its own authorization, because
that file carries an exact-SHA approval.

## 14. Known limitations and deferred work

- This report is retrospective. It did not exist during PR #914 review or merge,
  and adding it does not change that.
- The deviations in section 5 remain historical facts. Later authorization
  corrected their repository effect but does not make the original acts
  authorized.
- Some facts in sections 1, 4 and 5 rest on implementing-session records rather
  than durable GitHub records, and are labelled as such.
- The recovery pull request for this report requires fresh independent exact-head
  review. Its merge requires separate CTO authorization. Its final head, review and
  merge lifecycle are authoritative in GitHub.
- BE-06 source and migration implementation remain unauthorized until this
  recovery has been independently reviewed, merged and reconciled, and then until
  separately authorized.
- Root `AGENTS.md` section 5 expects a slice branch to be deleted after merge.
  `feature/publisher-services-v1-10--be-06-r52b-spec-record` still existed on the
  remote when this report was prepared (2026-09-12). Deleting it is not
  authorized by either task.

## 15. Unresolved issues

- NONE within this report's scope. The post-merge control finding of section 12 is
  closed only by independent review and reconciliation of the recovery pull
  request, not by this file.

## 16. Agent self-assessment

The implementing agent does not approve this report.

Suggested review focus:

- whether sections 5.1 and 5.2 record the deviations without softening or
  backdating them;
- whether each session-record label is appropriate, or whether a durable record
  exists that should be cited instead;
- CI wording: skipped runtime jobs are not described as passing;
- that no statement here becomes false when the recovery pull request is reviewed
  or merged.

## 17. Recovery task implementation report (`BE-06-R52B-PERSIST-REPORT-01`)

This section is the implementation report for the recovery task represented by
[PR #915](https://github.com/thoth-pub/thoth/pull/915). Sections 1 to 16 report
the original persistence task (PR #914).

### 17.1 Repository state

```text
Task ID:                      BE-06-R52B-PERSIST-REPORT-01
Owning issue:                 #848
Parent programme issue:       #765
Repository:                   thoth-pub/thoth
Workflow:                     PROGRAMME_INTEGRATION
Risk:                         MEDIUM documentation/control
Authorization/specification:  #848 comment 5648330082
Base branch:                  feature/publisher-services-v1-10
Authorized base commit:       8a711db7f61d0af9658261fc338badbee8431610
Actual base commit:           8a711db7f61d0af9658261fc338badbee8431610
PR target:                    feature/publisher-services-v1-10
Programme integration branch: feature/publisher-services-v1-10
Task branch:                  feature/publisher-services-v1-10--be-06-r52b-persist-report
Pull request:                 #915
Implementing model:           Claude Opus 5 (claude-opus-5)
Reasoning level:              High
Expected branch deletion after merge: YES
Final programme PR required:  YES
```

The recovery handoff and the review-correction handoff both preferred Claude
Sonnet 5. Both the implementation and the review correction ran as Claude Opus 5
with high reasoning.

Point-in-time implementation commits, in order:

- `9e23214f0a21978f3b2a805cbcd37a835c5d03cf` -
  `docs: add BE-06 R52B persistence report` (parent `8a711db7`);
- `04ae644776fcff914407c73c96145576a98c635b` -
  `docs: reference PR 915 in BE-06 R52B persistence report` (parent `9e23214f`).

`04ae6447` was the head independently reviewed in
[#848 comment 5648534677](https://github.com/thoth-pub/thoth/issues/848#issuecomment-5648534677)
and by the automated review in section 17.2. It is not the final head. The review
correction adds a further commit on the same branch. Per `ADR-0005`, the
correction commit, the final exact head, and the live exact-head review, CI,
authorization and merge state are authoritative in PR #915's GitHub history and
are not embedded here.

### 17.2 Scope confirmation and objective

Approved specification for this task: #848 comment `5648330082`.

Implemented objective:

- add the implementation report required by root `AGENTS.md` section 14 that
  PR #914 lacked, retrospectively (sections 0 to 16);
- record PR #914's history and control deviations faithfully, without softening
  or backdating them;
- add the required `CHANGELOG.md` entry for this task;
- make no change to R52B, source, runtime, schema, migrations or workflows.

Review correction. PR #915 was marked ready at head `04ae6447`, which triggered
the configured Codex review. Review `5187949827` (submitted
2026-09-12T20:39:45Z) raised two P1 findings. The control plane independently
verified both against repository doctrine and accepted them:

1. **Recovery-task coverage** (thread `3997494293`): the first version reported
   PR #914 in full but did not itself give a complete implementation report of
   PR #915's own diff, checks, CI and effects. Corrected by adding this
   section 17.
2. **Authority condition** (thread `3997494296`): the first version declared the
   record repository-authoritative when reachable from
   `feature/publisher-services-v1-10`, which is the programme integration branch.
   Corrected in section 0: the condition is now reachability from `develop`, the
   repository's authoritative integration branch (`ADR-0005`, `ADR-0009`).

Both defects were introduced by the first PR #915 implementation and were not
known to it. They were found by the post-ready review. The CTO authorized a
bounded correction: convert PR #915 to draft, modify only this report, commit and
push normally.

One further factual error in the same file, closely related to finding 2, was
corrected in the same edit and is disclosed here. Section 1.1 recorded
`Final programme PR required: NO` for PR #914. It now reads `YES`, because PR #914
targeted the programme integration branch, which reaches `develop` only through
the final programme integration. Section 1.1 notes the change. No other PR #914
fact was altered.

Out-of-scope changes: NONE beyond that disclosed field correction, which lies
within the single authorized correction path.

### 17.3 Files changed

Authorized recovery write paths (`5648330082`):

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
```

Actual PR #915 changed paths before the review correction (base `8a711db7` to
`04ae6447`):

```text
M	CHANGELOG.md
A	docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
```

- `CHANGELOG.md`: one added line, the `BE-06-R52B-PERSIST-REPORT-01` entry
  under `## [Unreleased]` / `### Added`, referencing issue #848, PR #915 and
  comment `5648330082`. The existing PR #914 entry is byte-identical to the base.
  Behavioural effect: none. Within budget: YES.
- `docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md`:
  new report. Behavioural effect: none. Within budget: YES.

Review-correction write path (CTO correction authorization):

```text
docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
```

`CHANGELOG.md` is not modified by the review correction.

```text
Files deleted/moved/renamed: NONE
Unauthorized paths changed:  NONE
R52B edited:                 NO
```

Write-budget compliance: PASS.

### 17.4 Authorized actions actually used

Recovery implementation (`5648330082`):

- repository and GitHub read inspection: used;
- branch creation from `8a711db7`: used;
- new report file creation: used;
- `CHANGELOG.md` edit, one entry: used;
- commit: used (`9e23214f`);
- push: used (normal push);
- draft PR creation: used (PR #915);
- second commit and normal push after the PR number existed: used (`04ae6447`);
- issue/comment mutation by the implementing agent: not used;
- manual CI dispatch/rerun/cancel: not used;
- migration execution: not used;
- provider/runtime read or write: not used;
- release/tag/publication: not used;
- merge: not used;
- deployment: not used;
- production activation: not used;
- branch deletion: not used.

The recovery session was already an isolated harness worktree. The task branch was
created there, and no additional worktree was created.

Review correction (CTO correction authorization):

- convert PR #915 to draft: authorized and used;
- edit of this report file: used;
- correction commit and normal push: used;
- `CHANGELOG.md` edit: not used;
- amend, rebase or force-push: not used;
- Codex review-thread reply or resolution: not used;
- PR title, body or base change: not used;
- ready transition after correction: not used;
- issue/comment mutation, merge, branch deletion, manual CI: not used.

Unauthorized actions performed: NONE.

### 17.5 Tests and checks

Documentation/control change. No Rust, SQL or generated file changed, so
formatting, unit, integration and lint suites are not applicable.

Recovery implementation checks, re-derived from the durable commits. Outputs are
exact; parenthetical notes inside output blocks are annotations, not command
output.

```bash
git diff --check 8a711db7f61d0af9658261fc338badbee8431610 04ae644776fcff914407c73c96145576a98c635b
```

```text
exit 0
no output
```

```bash
git diff --name-status 8a711db7f61d0af9658261fc338badbee8431610 04ae644776fcff914407c73c96145576a98c635b
git diff --numstat 8a711db7f61d0af9658261fc338badbee8431610 04ae644776fcff914407c73c96145576a98c635b
git diff --diff-filter=DR --name-status 8a711db7f61d0af9658261fc338badbee8431610 04ae644776fcff914407c73c96145576a98c635b
```

```text
M	CHANGELOG.md
A	docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md

1	0	CHANGELOG.md
518	0	docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md

(no output: nothing deleted or renamed)
```

```bash
git show 04ae644776fcff914407c73c96145576a98c635b:CHANGELOG.md |
  awk '/^## \[Unreleased\]/{f=1;next} f&&/^## /{exit} f&&/^### /{print}' | sort | uniq -c
```

```text
   1 ### Added
   1 ### Fixed
```

No duplicate Unreleased heading.

```bash
git rev-parse \
  8a711db7f61d0af9658261fc338badbee8431610:docs/publisher-services/specifications/BE-06-R52B.md \
  04ae644776fcff914407c73c96145576a98c635b:docs/publisher-services/specifications/BE-06-R52B.md
git show 04ae644776fcff914407c73c96145576a98c635b:docs/publisher-services/specifications/BE-06-R52B.md | sha256sum
git show 04ae644776fcff914407c73c96145576a98c635b:docs/publisher-services/specifications/BE-06-R52B.md | wc -c
git show 04ae644776fcff914407c73c96145576a98c635b:docs/publisher-services/specifications/BE-06-R52B.md | wc -l
```

```text
cabeddb84edb8fbfb5a3cdf3c1dc3e19c7799cfa
cabeddb84edb8fbfb5a3cdf3c1dc3e19c7799cfa
584683ca02611afb064335db83d1fb22685fc3dae87f4eeeee2e1490b320f3ce  -
699166
5107
```

Also performed by the recovery implementation (session record, not durable):

- every 40-character SHA in the new content resolved to the expected Git object,
  and every comment, review and run identifier matched GitHub;
- a transient-status scan of the newly introduced report and changelog entry found
  no match. All matches in `CHANGELOG.md` were pre-existing historical entries;
- neither commit message carries a trailer or AI attribution.

Review-correction checks, run against the corrected working tree before commit:

```bash
git diff --check 8a711db7f61d0af9658261fc338badbee8431610
git diff --name-status 8a711db7f61d0af9658261fc338badbee8431610
git diff --name-status 04ae644776fcff914407c73c96145576a98c635b
git diff --quiet 04ae644776fcff914407c73c96145576a98c635b -- CHANGELOG.md docs/publisher-services/specifications/BE-06-R52B.md
grep -nE 'A[W]AITING|P[E]NDING MERGE|MERGE NOT Y[E]T' docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
```

```text
exit 0, no output
M	CHANGELOG.md
A	docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
M	docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
exit 0: CHANGELOG.md and R52B unchanged relative to 04ae6447
no match
```

The same scope and R52B identity checks against the committed correction head are
exact-head evidence. They are recorded with the implementation handoff and PR
#915's GitHub history rather than embedded in the commit they describe.

### 17.6 CI

Natural `pull_request` workflow runs at `04ae644776fcff914407c73c96145576a98c635b`.
None was manually dispatched, rerun, cancelled or approved.

```text
34717169819 check-changelog:      success  (check-changelog: success)
34717169774 build-test-and-check: success  (classify: success; build, test, lint, format_check: skipped)
34717169795 run-migrations:       success  (classify: success; run_migrations: skipped)
34717169809 publish-to-dockerhub: success  (classify: success; build_and_push_staging_docker_image: skipped)
```

The intermediate head `9e23214f` produced the same classification
(runs 34717147839, 34717147849, 34717147835, 34717147816).

`check-changelog` and the classifiers succeeded. `build`, `test`, `lint`,
`format_check`, `run_migrations` and `build_and_push_staging_docker_image` were
**skipped** under documentation-only classification. They did not pass. No
migration ran and no staging image was published.

CI triggered by the review-correction push is exact-head evidence owned by GitHub.
It is independently checked before any review, ready transition or merge, per
`ADR-0005`, and is not embedded here.

### 17.7 Effects

```text
Runtime effect:                      NONE
Database/schema effect:              NONE
Migration effect:                    NONE
GraphQL/API effect:                  NONE
Authorization implementation effect: NONE
Provider effect:                     NONE
Crossref external-write effect:      NONE
Workflow definition effect:          NONE
Deployment effect:                   NONE
Production activation effect:        NONE
Cross-repository contract effect:    NONE
Metrics effect:                      NONE
```

Automatic PR-triggered CI is the only operational side effect. External writes or
publication: NONE.

### 17.8 Rollout and rollback

Rollout:

- documentation/control merge only, into the Publisher Services programme
  integration branch `feature/publisher-services-v1-10`;
- repository-authoritative status only when this exact content reaches `develop`
  through the programme's final integration;
- no service rollout, migration, provider action or production activation.

Rollback: if this report is materially incorrect, revert or correct it through
another authorized, reviewed documentation change. No runtime rollback applies.

### 17.9 Known limitations and deferred work

- The report is retrospective and did not exist during PR #914 review or merge.
- The first PR #915 head (`04ae6447`) did not completely report the recovery task
  itself, and it named the programme integration branch as the
  repository-authority condition. The post-ready Codex review found both defects,
  and they are corrected additively here without rewriting history.
- The final correction head's review, CI, authorization and merge lifecycle is
  owned by GitHub.
- PR #915 / `BE-06-R52B-PERSIST-REPORT-01` did not authorize BE-06
  implementation. Current and future BE-06 implementation authorization is owned
  by the GitHub task ledger.
- Branch deletion for the merged PR #914 persistence slice
  (`feature/publisher-services-v1-10--be-06-r52b-spec-record`) was not performed
  by `BE-06-R52B-PERSIST-REPORT-01`. Later branch state is owned by GitHub.
- Branch deletion for PR #915
  (`feature/publisher-services-v1-10--be-06-r52b-persist-report`) was not
  performed by `BE-06-R52B-PERSIST-REPORT-01`. Later branch state is owned by
  GitHub.

### 17.10 Agent self-assessment

The implementing agent does not approve this recovery task or its correction.

Suggested review focus:

- that section 17 is a complete section 14 report for PR #915 without embedding
  its own final head;
- that section 0's authority condition names `develop`;
- the disclosed `Final programme PR required` correction in section 1.1;
- that no PR #914 historical fact or deviation was softened.

## 18. Durability correction implementation report (`BE-06-R52B-PERSIST-REPORT-DURABILITY-01`)

This section is the implementation report for the durability correction task. It
follows `docs/engineering/ai-delivery/implementation-report-template.md` within
this existing file, because no new file is authorized. Sections 0 to 17 are
preserved apart from the changes listed in section 18.2.

### 18.1 Repository state

```text
Task ID:                      BE-06-R52B-PERSIST-REPORT-DURABILITY-01
Owning issue:                 #848
Parent programme issue:       #765
Repository:                   thoth-pub/thoth
Workflow:                     PROGRAMME_INTEGRATION
Risk:                         MEDIUM documentation/control
Authorization/specification:  #848 comment 5649130376
Base branch:                  feature/publisher-services-v1-10
Authorized base commit:       73a3709d0211063b154b3d42da8d51d5f4db7628
Actual base commit:           73a3709d0211063b154b3d42da8d51d5f4db7628
PR target:                    feature/publisher-services-v1-10
Programme integration branch: feature/publisher-services-v1-10
Task branch:                  feature/publisher-services-v1-10--be-06-r52b-report-durability
Pull request:                 the draft pull request opened from this task branch
Implementing model:           Claude Opus 5 (claude-opus-5)
Reasoning level:              High
Expected branch deletion after merge: YES
Final programme PR required:  YES
```

Specification `5649130376` and its handoff named Claude Sonnet 5 as the
implementation model. The session ran as Claude Opus 5 with high reasoning, and
this record states the model actually used.

The authorized base `73a3709d` is the merge of PR #915 (parents `8a711db7` and
`c441ce7b`). Per `ADR-0005`, this task's commits, final exact head, independent
review, CI, authorization and merge lifecycle are authoritative in the task pull
request's GitHub history and are not embedded here.

### 18.2 Scope confirmation and objective

Approved specification: #848 comment `5649130376`.

Motivation: PR #915's final head `c441ce7b` received Codex P2 review thread
`3997644819` (review `5188133116`, 2026-09-12T21:23:19Z). The thread identified
section 17.9's sentence "BE-06 implementation remains unauthorized" as live
authorization state that a later authorization would falsify, contrary to
`docs/engineering/AGENTS.md` section 1.1. PR #915 merged at
2026-09-12T21:25:13Z, which also made section 17.9's branch-cleanup wording about
PR #915 stale.

Implemented objective:

- correct P2 `3997644819` in section 17.9: the statement now records that PR #915 /
  `BE-06-R52B-PERSIST-REPORT-01` did not authorize BE-06 implementation, and that
  current and future authorization is owned by the GitHub task ledger;
- convert section 17.9's branch-cleanup statements into durable historical
  statements: branch deletion for the PR #914 and PR #915 task branches was not
  performed by `BE-06-R52B-PERSIST-REPORT-01`, and later branch state is owned by
  GitHub;
- add a one-sentence pointer in section 0 identifying this section 18;
- make the existing PR #915 changelog entry's closing wording durable ("this
  recovery did not authorize BE-06 implementation"), changing nothing else in that
  entry;
- add one `### Fixed` changelog entry for this task;
- add this section 18.

Not changed: the retrospective history, the missing-report history, the
unauthorized AI-attribution history, the changelog-authorization provenance gap,
the P1 recovery-report and authority-condition corrections, the
`Final programme PR required` correction, and all model and session disclosures.
No R52B, runtime, schema, migration, API, authorization, workflow, provider or
production change.

Durability review of retained wording. Occurrences outside section 17.9 that
mention authorization or branch lifecycle were reviewed against
`docs/engineering/AGENTS.md` section 1.1 and left unchanged:

- section 5.1, "It was not authorized when made": historical fact;
- section 14, the recovery pull request "requires fresh independent exact-head
  review" and its merge "requires separate CTO authorization": requirements that
  applied to that pull request;
- section 14, BE-06 implementation remains unauthorized "until this recovery has
  been independently reviewed, merged and reconciled, and then until separately
  authorized": a conditional gate statement that stays true after a later
  authorization;
- section 14, the PR #914 branch "still existed on the remote when this report
  was prepared (2026-09-12)": a dated observation.

These sit in the historical PR #914 report sections that specification
`5649130376` requires to be preserved.

Out-of-scope changes: NONE.

### 18.3 Files changed

Authorized existing-file paths (`5649130376`):

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
```

Authorized new files: NONE.

Actual files changed:

- `docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md`
  - reason: P2 `3997644819` durability correction and this task's report;
  - behavioural effect: none;
  - within authorized write budget: YES.
- `CHANGELOG.md`
  - reason: durable closing wording in the PR #915 entry, and this task's
    `### Fixed` entry required by root `AGENTS.md` section 13;
  - behavioural effect: none;
  - within authorized write budget: YES.

```text
New files:                  NONE
Deleted files:              NONE
Moved files:                NONE
Renamed files:              NONE
Unauthorized paths changed: NONE
R52B edited:                NO
```

Write-budget compliance: PASS.

### 18.4 Authorized actions actually used

- repository and GitHub read inspection: used;
- task branch creation from `73a3709d`: used. The session was already an
  isolated harness worktree, so no additional worktree was created;
- edits to the two authorized files: used;
- local validation: used;
- commit: used (normal, no amend);
- push: used (normal, no force);
- draft PR creation: used;
- bounded follow-up commit and normal push solely to add the resulting PR number:
  used.

Not used:

- new file creation, deletion, move or rename;
- issue or comment mutation;
- PR ready transition;
- review-thread reply or resolution, including thread `3997644819`;
- manual CI dispatch, rerun, cancel or approval;
- provider or runtime read or write;
- migration execution;
- release, tag or publication;
- merge;
- deployment;
- production activation;
- branch deletion.

Unauthorized actions performed: NONE.

### 18.5 Tests and checks

Documentation/control change only. No Rust, SQL or generated file changed, so
formatting, unit, integration and lint suites are not applicable.

Checks run against the corrected working tree before commit:

```bash
git diff --check 73a3709d0211063b154b3d42da8d51d5f4db7628
git diff --name-status 73a3709d0211063b154b3d42da8d51d5f4db7628
git diff --diff-filter=ADR --name-status 73a3709d0211063b154b3d42da8d51d5f4db7628
git diff --quiet 73a3709d0211063b154b3d42da8d51d5f4db7628 -- docs/publisher-services/specifications/BE-06-R52B.md
grep -c '^## \[Unreleased\]' CHANGELOG.md
awk '/^## \[Unreleased\]/{f=1;next} f&&/^## /{exit} f&&/^### /{print}' CHANGELOG.md | sort | uniq -c
grep -c 'BE-06-R52B-PERSIST-REPORT-DURABILITY-01' CHANGELOG.md
```

```text
exit 0, no output
M	CHANGELOG.md
M	docs/engineering/ai-delivery/implementation-reports/BE-06-R52B-PERSIST-01-implementation-report.md
no output (nothing added, deleted or renamed)
exit 0 (R52B unchanged)
1
   1 ### Added
   1 ### Fixed
1
```

Transient-wording review of the changed material, against the concepts listed in
the handoff ("remains unauthorized", "is not authorized", "has not been
performed", "eventually merges", "awaiting", "pending merge", "merge not yet"):
no such current-state prose remains in section 17.9, in the PR #915 changelog
entry's closing wording, or in the new changelog entry. The new changelog entry
quotes the former wording only as a labelled description of what was replaced.
Retained occurrences elsewhere are classified in section 18.2.

R52B identity at the authorized base:

```bash
git rev-parse 73a3709d0211063b154b3d42da8d51d5f4db7628:docs/publisher-services/specifications/BE-06-R52B.md
git show 73a3709d0211063b154b3d42da8d51d5f4db7628:docs/publisher-services/specifications/BE-06-R52B.md | sha256sum
git show 73a3709d0211063b154b3d42da8d51d5f4db7628:docs/publisher-services/specifications/BE-06-R52B.md | wc -c
git show 73a3709d0211063b154b3d42da8d51d5f4db7628:docs/publisher-services/specifications/BE-06-R52B.md | wc -l
```

```text
cabeddb84edb8fbfb5a3cdf3c1dc3e19c7799cfa
584683ca02611afb064335db83d1fb22685fc3dae87f4eeeee2e1490b320f3ce  -
699166
5107
```

The same scope, whitespace and R52B checks against the committed head are
exact-head evidence. They are recorded with the implementation handoff and the
task pull request's GitHub history, not embedded in the commit they describe.

### 18.6 CI

Expected natural `pull_request` classification for this documentation-only diff:
`check-changelog` and the classifiers run, while `build`, `test`, `lint`,
`format_check`, `run_migrations` and `build_and_push_staging_docker_image` are
skipped. Actual run IDs and job conclusions for this task's heads are exact-head
evidence owned by GitHub and are checked independently before any review, ready
transition or merge, per `ADR-0005`. No workflow is manually dispatched, rerun,
cancelled or approved by this task.

### 18.7 Effects

```text
Runtime effect:                      NONE
Database/schema effect:              NONE
Migration effect:                    NONE
GraphQL/API effect:                  NONE
Authorization implementation effect: NONE
Provider effect:                     NONE
Crossref external-write effect:      NONE
Workflow definition effect:          NONE
Deployment effect:                   NONE
Production activation effect:        NONE
Cross-repository contract effect:    NONE
Metrics effect:                      NONE
```

Automatic PR-triggered CI is the only expected operational side effect.

### 18.8 Rollout and rollback

Rollout:

- documentation/control only, into the Publisher Services programme integration
  branch `feature/publisher-services-v1-10`;
- repository-authoritative status only when this exact content later reaches
  `develop` through the separately gated final programme integration;
- no service rollout, migration, provider action or runtime activation.

Rollback: if this correction is materially wrong, correct or revert it through
another separately authorized, reviewed documentation change. No runtime or data
rollback applies.

### 18.9 Known limitations and deferred work

- This task corrects documentation durability only.
- This task did not authorize BE-06 implementation.
- This task did not change historical branch state and deleted no branch.
- Current and future branch, authorization, review and merge state is owned by
  GitHub.
- The final exact-head review and CI for this task are GitHub-owned evidence.

### 18.10 Agent self-assessment

The implementing agent does not approve this task.

Suggested review focus:

- that section 17.9 no longer asserts live authorization or branch state, and that
  no historical deviation was softened;
- the classification of retained wording in section 18.2;
- that the PR #915 changelog entry changed only in its closing authorization
  wording;
- that section 18 does not embed its own final head or CI run identifiers.
