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

Decision recorded by this report:
PR #914 persisted the approved R52B bytes exactly; its control deviations are
recorded in sections 5 and 12; no BE-06 runtime implementation occurred.

Authority condition:
This record is repository-authoritative when this exact content is reachable
from `feature/publisher-services-v1-10`.

Live review, authorization and merge evidence for the recovery pull request:
GitHub pull-request history, per `ADR-0005`.

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
Final programme PR required: NO
```

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
Pull request:                the draft pull request opened from this task branch
Head commit:                 authoritative in GitHub pull-request history
Implementing model:          Claude Opus 5 (claude-opus-5), high reasoning
```

The recovery handoff named Claude Sonnet 5 as the implementing model. The session
ran as Claude Opus 5. Recovery specification `5648330082` does not name a model.

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
