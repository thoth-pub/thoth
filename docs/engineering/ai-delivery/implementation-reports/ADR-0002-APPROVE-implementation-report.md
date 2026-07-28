# ADR-0002-APPROVE Implementation Report

## 1. Repository and delivery record

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Original task: `ADR-0002-APPROVE`
Approved specification:
`docs/engineering/ai-delivery/tasks/ADR-0002-APPROVE.md`
Original base branch: `develop`
Original base commit: `f2e09bd9b138e8ba2ca47a791533f4aae4ffab28`
Original PR target: `develop`
Original task branch: `feature/engineering/adr-0002-approve`
Original PR: [#769](https://github.com/thoth-pub/thoth/pull/769)
Original reviewed head: `78307578f680050581f1a4e16a9668d9dfcc037a`
Merge commit: `e124221f8444bd738228f1b609c536639be8789e`
Exact merge timestamp: `2026-07-28T09:24:58Z`
Implementing agent/model: Claude Code / `claude-opus-4-8`
Independent reviewer/model: ChatGPT / GPT-5.6 Thinking, operating in a fresh
non-implementing review context
Risk: MEDIUM
Implementation reasoning: High
Independent review reasoning: High

The independent-review identity is taken from the original connected ChatGPT
review record that issued the exact-head decision and from the concrete model
identity recorded by the approved evidence-restoration amendment. It is not
inferred from the GitHub account that posted the durable review comment.

The original PR recorded the CTO approval of ADR-0002 as written. The ADR
architecture was not amended; only approval metadata and dependent control
records changed.

## 2. Scope confirmation and CTO approval

Implemented objective: record the CTO's approval of `ADR-0002 - Distribution and
Metrics Platform Domain Boundaries` exactly as written and reconcile the
engineering, Publisher Services and Metrics control records with that decision.

Exact CTO approval:

> I approve ADR-0002 - Distribution and Metrics Platform Domain Boundaries as
> written.
> Do not amend the architectural decision.

Approved by: Javi, CTO
Approval date: 2026-07-27

Scope result:

- ADR-0002 was approved as written.
- No other ADR was approved.
- No implementation task or work package was approved or made `READY`.
- All 14 original changes were within the approved specification allowlist.
- Issues #765 and #766 were not written.
- Deviations from the approved specification: none.

## 3. Original and corrective commit evidence

Original implementation commits:

- `ddf635fd4c0ed2a0e322324b0d6dce17f1257300` -
  `docs: approve ADR-0002 recording task`; approved task specification committed
  first.
- `7a82680ccafe18c99b18d6be991e1b42a5fbca28` -
  `docs: record ADR-0002 approval`; approval metadata and control-record
  reconciliation.
- `f4ef99c97c21f86d0dccd29081a450e3bdf4ce54` -
  `docs: record ADR-0002 approval evidence`; initial implementation report.

Original evidence-correction commit:

- `78307578f680050581f1a4e16a9668d9dfcc037a` -
  `docs: correct ADR-0002 approval evidence`; pre-review evidence correction
  limited to this report.

Merge record:

- `e124221f8444bd738228f1b609c536639be8789e` - merge commit for PR #769, with
  original base `f2e09bd9b138e8ba2ca47a791533f4aae4ffab28` and reviewed head
  `78307578f680050581f1a4e16a9668d9dfcc037a` as parents.

PR #770 post-merge corrective work:

- `e4392d5b1eae661fa5ab7d11a8670bf748d462f2` - approve the corrective task.
- `cc381a13d41ec97a3a902d61315878563e99cd03` - correct the ADR-0002 approval
  evidence and embed complete issue bodies.
- `3c48daf183a1065d1aae8af258df5fd4aaf9bf24` - reconcile agent rollout status.
- `dd292df8276c0b8d024dd772a96672925b9b8268` - record corrective evidence.
- `8b1647e9f77ec478c0209a82086f638d20ffe16a` - record the superseded
  no-changelog amendment.
- `8158603b30e87074326b5729bcf661678a4dccd5` - record changelog-check amendment
  and evidence.
- `ca8e90645957c50c25fecd8b220772837bb522d3` - address independent review.
- `cb82ce799ca3afecf8f243faa7ebb9d10c5d049b` - address post-ready review.
- `docs: restore PR 769 implementation evidence` - current bounded
  evidence-restoration correction; its exact SHA is recorded in the post-push
  handoff because a commit cannot contain its own SHA.

## 4. Full original 14-file evidence

The original PR #769 cumulative diff changed exactly these 14 allowlisted files.

- `docs/engineering/ai-delivery/tasks/ADR-0002-APPROVE.md`
  - Reason: record the approved bounded task before implementation.
  - Control effect: establishes objective, scope, invariants, tests, rollout and
    review gates.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: none; specification alone made no task ready.
- `docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md`
  - Reason: record `APPROVED`, CTO approver, approval date and approval note.
  - Control effect: makes the existing cross-programme architectural decision
    authoritative without changing Sections 1-9.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: removes only the ADR-0002 dependency.
- `docs/engineering/decisions/decision-register.md`
  - Reason: preserve the approval blocker as a satisfied audit-trail entry and
    reference PR #769.
  - Control effect: ADR-0002 becomes approved while ADR-0001 and all
    implementation rules remain unchanged.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: no task became ready.
- `docs/engineering/README.md`
  - Reason: reconcile the active engineering overview with ADR-0002 approval and
    the completed issue #765 foundation synchronization.
  - Control effect: records the approved decision without claiming programme
    implementation readiness.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: remaining programme gates stay active.
- `docs/engineering/repository-map/control-gaps.md`
  - Reason: narrow CG-06 from both shared ADRs to the still-proposed ADR-0001.
  - Control effect: preserves the gap and audit trail rather than erasing it.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: no task became ready.
- `docs/publisher-services/README.md`
  - Reason: remove ADR-0002 from active blockers and record it as approved.
  - Control effect: retains `BLOCKED FOR IMPLEMENTATION`, ADR-0001, ADR-01,
    final-inventory and branch-readiness gates.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: no Publisher Services task became ready.
- `docs/publisher-services/decisions.md`
  - Reason: move ADR-0002 to approved shared architecture while preserving its
    exact separate-domain meaning.
  - Control effect: retains ADR-0001 and Publisher Services ADR-01 as unresolved.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: none.
- `docs/publisher-services/task-status.md`
  - Reason: remove only satisfied P0-01/ADR-0002 dependencies and reconcile ADR-01
    and BE-02 blockers.
  - Control effect: ADR-01 remains `BLOCKED` by its bounded specification and
    final inventory; BE-02 remains `BLOCKED` by ADR-01.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: no task became `READY`.
- `docs/publisher-services/rollout-plan.md`
  - Reason: record P0-01 closure, issue #765 synchronization and ADR-0002 approval
    as achieved Stage 0 evidence.
  - Control effect: retains ADR-0001, ADR-01/inventory, branch-readiness and task
    specification/reviewer gates.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: no later stage was activated.
- `docs/metrics/README.md`
  - Reason: record ADR-0002 approval and remove only that dependency.
  - Control effect: retains `BLOCKED FOR IMPLEMENTATION`, `MET-CTRL-01`,
    ADR-0001 and all other Metrics blockers.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: none.
- `docs/metrics/decisions.md`
  - Reason: record approved separate `MetricPlatform` and
    `DistributionPlatform` domains with no initial mapping.
  - Control effect: preserves ADR-0001 and all Metrics-specific unresolved
    decisions.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: none.
- `docs/metrics/task-status.md`
  - Reason: set the ADR-0002 control row to `APPROVED` and narrow WP1's shared
    ADR blocker to ADR-0001.
  - Control effect: retains `MET-CTRL-01 CHANGES REQUIRED` and every blocked work
    package.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: no work package became `READY`.
- `CHANGELOG.md`
  - Reason: add one bounded Unreleased / Changed entry for PR #769.
  - Control effect: satisfies repository delivery traceability.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: none.
- `docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md`
  - Reason: record implementation, verification, issue-baseline, CI, review,
    rollout and rollback evidence.
  - Control effect: provides the durable evidence record for PR #769.
  - Runtime/authorization/migration effect: none.
  - Readiness effect: evidence did not itself make any task ready.

## 5. Implementation decisions and ADR integrity

1. ADR-0002 changes were limited to approval metadata: `PROPOSED` to `APPROVED`,
   CTO approver, approval date and the approval note.
2. The decision-register approval blocker was retained as a satisfied statement
   so the audit trail remained visible.
3. CG-06 was narrowed to ADR-0001 rather than deleted.
4. Historical evidence that accurately recorded ADR-0002 as proposed at an
   earlier time was left historically accurate.
5. Issues #765 and #766 were not written; their guarded proposed bodies remain
   recorded evidence only.
6. No shared abstraction, automatic conversion, name-based conversion or
   cross-domain mapping was introduced.

The exact ADR diff is:

```diff
@@ -1,6 +1,6 @@
 # ADR-0002 - Distribution and Metrics Platform Domain Boundaries

-Status: PROPOSED
+Status: APPROVED
 Date: 2026-07-24
 Decision owner: CTO
 Programmes affected: Publisher Services, Thoth Metrics
@@ -293,6 +293,7 @@ Rollback:
 ## 10. Approval

 Approval required from: CTO
-Approved by:
-Approval date:
-Notes:
+Approved by: Javi, CTO
+Approval date: 2026-07-27
+Notes: Approved as written. DistributionPlatform and MetricPlatform remain
+separate domain types, with no initial cross-domain mapping.
```

Sections 1-9 remained byte-for-byte unchanged.

## 6. Migration, operational, API and compatibility effects

```text
Database migration: none
Data migration: none
Schema change: none
Generated-code change: none
Deployment effect: none
Production activation: none
Operational action: documentation merge only
```

GraphQL/API changes: none.
Generated schema/client changes: none.
Backwards compatibility: unaffected.
Deprecations: none.
Cross-repository runtime dependency changes: none.

## 7. Authorization and security assessment

```text
Authorization paths changed: none
Roles/scopes involved: none
Negative authorization tests: not applicable
Secret handling: none
Personal-data handling: none
Security limitations: none; no runtime surface changed
```

## 8. Exact local commands and truthful results

### Original changed-file boundary

Command:

```bash
git diff --name-only \
  f2e09bd9b138e8ba2ca47a791533f4aae4ffab28...78307578f680050581f1a4e16a9668d9dfcc037a
```

Result: exit `0`, exactly the 14 paths listed in Section 4.

### Historical whitespace check

Command:

```bash
git diff --check \
  f2e09bd9b138e8ba2ca47a791533f4aae4ffab28...78307578f680050581f1a4e16a9668d9dfcc037a
```

Actual result: exit `2`.

```text
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md:327: trailing whitespace.
+
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md:335: trailing whitespace.
+
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md:385: trailing whitespace.
+
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md:388: trailing whitespace.
+
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md:390: trailing whitespace.
+
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md:421: trailing whitespace.
+
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md:427: trailing whitespace.
+
```

This supersedes the inaccurate historical clean-check claim. The original green
CI did not prove the report was whitespace-clean or complete.

### Current correction whitespace check

Command:

```bash
git diff --check \
  e124221f8444bd738228f1b609c536639be8789e...HEAD
```

Result before this evidence-restoration edit at head
`cb82ce799ca3afecf8f243faa7ebb9d10c5d049b`: exit `0`, no output. The same
command is rerun against the new committed head before push and handoff.

### State assertions at the original reviewed head

Commands:

```bash
git show 78307578f680050581f1a4e16a9668d9dfcc037a:docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md |
  rg -n '^Status: APPROVED$'
git show 78307578f680050581f1a4e16a9668d9dfcc037a:docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md |
  rg -n '^Status: PROPOSED$'
git show 78307578f680050581f1a4e16a9668d9dfcc037a:docs/publisher-services/platform-inventory.md |
  rg -n 'FINAL ENUM NOT APPROVED'
git show 78307578f680050581f1a4e16a9668d9dfcc037a:docs/metrics/task-status.md |
  rg -n 'MET-CTRL-01.*CHANGES REQUIRED'
git show 78307578f680050581f1a4e16a9668d9dfcc037a:docs/publisher-services/task-status.md |
  rg -n 'ADR-01.*BLOCKED'
git grep -n -E '\| (READY|ACTIVE) \|' \
  78307578f680050581f1a4e16a9668d9dfcc037a -- \
  docs/publisher-services/task-status.md docs/metrics/task-status.md
```

Exact outputs:

```text
3:Status: APPROVED
3:Status: PROPOSED
3:Status: VERIFIED BASELINE; FINAL ENUM NOT APPROVED
17:| MET-CTRL-01 Programme controls | `thoth` | LOW | CHANGES REQUIRED | PR #764 merged into `develop` as `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` | Shared foundation closed (P0-01 closeout PR #767 independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`); MET-CTRL-01's own `CHANGES REQUIRED` remediation outstanding | [#766](https://github.com/thoth-pub/thoth/issues/766) |
22:| ADR-01 Platform inventory/final architecture | `thoth` | MEDIUM | BLOCKED | `develop` / `develop` | missing approved bounded ADR-01 specification; final distribution-platform inventory decision | #765 | TBD | NOT STARTED |
```

The final `READY|ACTIVE` search exited `1` with no output, proving no Publisher
Services task or Metrics work package was `READY` or `ACTIVE`.

### Relative-link validation

Command:

```bash
node - <<'NODE'
const {execFileSync} = require('child_process');
const path = require('path').posix;
const base = 'f2e09bd9b138e8ba2ca47a791533f4aae4ffab28';
const head = '78307578f680050581f1a4e16a9668d9dfcc037a';
const files = execFileSync(
  'git',
  ['diff', '--name-only', `${base}...${head}`, '--', '*.md'],
  {encoding: 'utf8'}
).trim().split('\n').filter(Boolean);
const failures = [];
let checked = 0;
for (const file of files) {
  const text = execFileSync('git', ['show', `${head}:${file}`], {encoding: 'utf8'});
  for (const match of text.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)) {
    let target = match[1].trim();
    if (target.startsWith('<') && target.endsWith('>')) {
      target = target.slice(1, -1);
    } else {
      target = target.split(/\s+["']/)[0];
    }
    if (/^(?:[a-z][a-z0-9+.-]*:|#)/i.test(target)) continue;
    target = target.split('#')[0].split('?')[0];
    if (!target) continue;
    try { target = decodeURI(target); } catch {}
    const resolved = path.normalize(path.join(path.dirname(file), target));
    checked++;
    try {
      execFileSync('git', ['cat-file', '-e', `${head}:${resolved}`], {stdio: 'ignore'});
    } catch {
      failures.push(`${file} -> ${target} (${resolved})`);
    }
  }
}
if (failures.length) {
  console.log(failures.join('\n'));
  console.log(`RELATIVE_LINKS_FAILED=${failures.length}`);
  process.exit(1);
}
console.log(`RELATIVE_LINKS_OK files=${files.length} links=${checked}`);
NODE
```

Result:

```text
RELATIVE_LINKS_OK files=14 links=24
```

## 9. Exact-head CI

At reviewed head `78307578f680050581f1a4e16a9668d9dfcc037a`:

```text
30342715466 - build-test-and-check
  build: success
  test: success
  lint: success
  format_check: success

30342715472 - run-migrations
  run_migrations: success

30342715538 - check-changelog
  check-changelog: success

30342715481 - publish-to-dockerhub
  build_and_push_staging_docker_image: success
```

All seven jobs were green at the exact reviewed head. That CI established the
workflow results only; it did not establish that the report was complete or that
the historical diff was whitespace-clean.

## 10. Manual verification

Environment: local `thoth-pub/thoth` worktree with the original base and reviewed
head available as immutable commits.

Files inspected: all 14 original changed files, ADR-0001, ADR-0002, the Publisher
Services platform inventory, and active Publisher Services and Metrics task
trackers.

Steps:

1. Diffed ADR-0002 from the exact original base to reviewed head.
2. Verified the ADR diff was limited to approval metadata.
3. Ran the exact state assertions in Section 8.
4. Inspected the complete 14-file boundary and per-file effects.
5. Validated every relative Markdown link at the original reviewed head.
6. Re-ran the historical whitespace check and recorded all seven failures.

Observed result: ADR-0002 was approved as written; ADR-0001 remained proposed;
the final platform enum remained unapproved; `MET-CTRL-01` remained changes
required; ADR-01 and every implementation task/work package remained blocked.

Evidence location: immutable Git commits
`f2e09bd9b138e8ba2ca47a791533f4aae4ffab28`,
`78307578f680050581f1a4e16a9668d9dfcc037a` and merge
`e124221f8444bd738228f1b609c536639be8789e`, plus PR #769 CI and review records.

## 11. Independent review and post-merge findings

ChatGPT / GPT-5.6 Thinking, operating in a fresh non-implementing review
context, returned `APPROVED` at the reviewed head with no P0, P1 or P2 finding
before merge.

After PR #769 was marked ready, an automated Codex review completed after the
merge and raised three actionable P1 findings:

1. the issue proposals were abbreviated rather than embedded completely;
2. the active agent rollout plan still described issue #765 synchronization as
   outstanding;
3. the original report contained trailing whitespace while claiming its
   historical `git diff --check` was clean.

PR #770 corrects these evidence and control defects without changing the
architectural approval.

## 12. Rollout and rollback

Original ADR-0002 rollout:

- Merge records ADR-0002 as approved.
- Publisher Services and Metrics remain blocked by their remaining gates.
- Activation required: none.
- Feature flag or configuration change: none.
- Migration sequence: none.
- Runtime monitoring: none, because no runtime surface changed.

Original rollback:

- Revert PR #769 using the normal repository revert process.
- Any issue rollback requires a fresh complete-body and `updatedAt` re-fetch,
  exact comparison, a minimal reviewed reversal and explicit CTO authorization.
- Never restore an old complete issue snapshot blindly.

PR #770 corrective rollout:

- Documentation reconciliation only.
- No issue synchronization.
- No deployment, release or production activation.
- Roll back by reverting PR #770.

## 13. Known limitations and deferred work

- PR #769 merged before its automated post-ready findings were addressed.
- PR #770 corrects evidence and active-control defects only.
- Issues #765 and #766 remain open and unchanged.
- Issue synchronization remains separately gated and unauthorized.
- ADR-0001 remains `PROPOSED`.
- Publisher Services ADR-01 remains unapproved.
- The final platform enum remains unapproved.
- `MET-CTRL-01` remains `CHANGES REQUIRED`.
- Repository branch-readiness decisions remain outstanding.
- All Publisher Services implementation tasks and Metrics work packages remain
  blocked.
- No implementation task becomes `READY`.
- Final PR #770 merge and post-merge PR #769 thread evidence cannot exist until
  those operations occur.

## 14. Live issue baselines

This task and PR #770 did not write either issue.

```text
Issue #765
state: OPEN
baseline updatedAt: 2026-07-27T15:50:33Z
complete-body sha256: 96c31089a3046eadf51a0fc39b12d0275ce26f4d752c64282f5dcb933f78ca15
proposed-body sha256: da12243b2a1898fd3fd574aada1dede3296ff13f38943e4fbb78a3dcb5ae1a35

Issue #766
state: OPEN
baseline updatedAt: 2026-07-24T17:17:11Z
complete-body sha256: 6b1bb092f3f0b436c01faaabbf4fb5df331268f4d687463b3c715fb4ea9d6dbc
proposed-body sha256: f4e8aa7e855b2b3c44b4cf38c60475861079698cc7f5cd95a6ac319b892cb772
```

## 15. Exact proposed body for issue #765

The complete proposed replacement body is:

```markdown
## Objective

Implement the approved Publisher Services and Distribution Configuration design across Thoth, thoth-app, thoth-dissemination and cc-license with additive schema, explicit authorization, audited migration, comparison-mode cutover, bounded pilots, monitoring and rollback.

## Immutable authority at foundation review head

- [Private approved design](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit) - Drive revision `3`
- [Private design reference metadata](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/design-references.md#publisher-services-and-distribution-configuration)
- [Programme README](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/README.md)
- [Task tracker](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/task-status.md)
- [Platform inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/platform-inventory.md)
- [Acceptance matrix](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/acceptance-matrix.md)
- [Rollout plan](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/rollout-plan.md)
- [Foundation specification](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md)
- [Foundation implementation report](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md)
- [Foundation PR #764](https://github.com/thoth-pub/thoth/pull/764)
- [P0-01 closeout PR #767](https://github.com/thoth-pub/thoth/pull/767)
- [P0-01 finalization PR #768](https://github.com/thoth-pub/thoth/pull/768)

The Publisher Services design requires one fresh task branch and one PR per task. There is no long-lived `feature/publisher-services` integration branch.

## Synchronization guard

Before applying this replacement: re-fetch the complete live issue body; re-fetch its current `updatedAt`; compare both exactly against the reviewed baseline `updatedAt: 2026-07-27T15:50:33Z` and body. If either the live body or `updatedAt` differs, do not write. Regenerate the minimal diff from the new live body, obtain fresh independent review, and obtain separate explicit CTO authorization before writing. Any later rollback must likewise re-fetch and compare the live body and `updatedAt`, preserve unrelated edits, and apply only a reviewed minimal reversal under explicit CTO authorization; it must never restore an old complete snapshot blindly.

## Current gate

- [x] P0-01 independently approved, repository-finalized and merged
- [ ] ADR-0001 approved
- [x] ADR-0002 approved
- [ ] ADR-01 platform inventory approved
- [ ] repository branch-readiness decisions recorded

No production implementation begins before the applicable gate passes.

## Tasks

### Foundation

- [x] P0-01 - Project control documents and tracker - CLOSED
- [ ] ADR-01 - Platform inventory and final architecture
- [ ] LIC-01 - Expand cc-license
- [ ] LIC-02 - Enforce supported licences in Thoth

### Backend

- [ ] BE-01 - Publisher package model
- [ ] BE-02 - Distribution platform model
- [ ] BE-03 - Protected service configuration
- [ ] BE-04 - Durable distribution jobs

### Migration and interfaces

- [ ] MIG-01 - Audit and production backfill
- [ ] APP-01 - Publisher service configuration UI
- [ ] APP-02 - Staff subscription report
- [ ] APP-03 - API-backed licence options

### Cutover and downstream services

- [ ] DIS-01 - API publisher discovery and comparison mode
- [ ] DIS-02 - Back-catalogue job worker
- [ ] EXP-01 - OCLC KBART feed index
- [ ] OAI-01 - Package and licence gating

### Stabilization

- [ ] OPS-01 - Monitoring, runbooks and cleanup
- [ ] E2E-01 - Full workflow verification

P0-01 closure records completion of the engineering-control foundation only. It does not approve an ADR, approve the final inventory, satisfy branch readiness, or make another task ready.

Do not close a task at PR creation or CI success. Close only after independent approval, merge, required rollout/observation and repository tracker update.
```

## 16. Exact proposed body for issue #766

The complete proposed replacement body is:

```markdown
## Objective

Make Thoth the canonical datastore and API for usage and sales metrics, with restartable Sphinx collection, publisher imports, coverage-aware rollups, protected dashboard/widget queries, OPERAS synchronization and reconciled historical migration.

## Immutable authority at foundation review head

- [Private approved design](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit) - Drive revision `6`
- [Private design reference metadata](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/design-references.md#thoth-metrics)
- [Programme README](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/README.md)
- [Task tracker](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/task-status.md)
- [Source inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/source-inventory.md)
- [Contract register](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/contract-register.md)
- [Migration inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/migration-inventory.md)
- [Acceptance matrix](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/acceptance-matrix.md)
- [Rollout plan](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/metrics/rollout-plan.md)
- [Foundation specification](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md)
- [Foundation implementation report](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md)
- [Foundation PR #764](https://github.com/thoth-pub/thoth/pull/764)

The Metrics design uses repository-local `feature/metrics` integration branches only after each repository's branch-readiness gate.

## Synchronization guard

Before applying this replacement: re-fetch the complete live issue body; re-fetch its current `updatedAt`; compare both exactly against the reviewed baseline `updatedAt: 2026-07-24T17:17:11Z` and body. If either the live body or `updatedAt` differs, do not write. Regenerate the minimal diff from the new live body, obtain fresh independent review, and obtain separate explicit CTO authorization before writing. Any later rollback must likewise re-fetch and compare the live body and `updatedAt`, preserve unrelated edits, and apply only a reviewed minimal reversal under explicit CTO authorization; it must never restore an old complete snapshot blindly.

## Current gate

- [ ] MET-CTRL-01 independently approved and merged
- [ ] ADR-0001 approved
- [x] ADR-0002 approved
- [ ] BR-SPHINX-01 complete
- [ ] SPHINX-BOOT-01 complete
- [ ] THOTH-DB-CTRL-01 complete
- [ ] client repository branch-readiness decisions recorded
- [ ] service-role codes approved before WP5

## Work packages

- [ ] WP1 - Metrics domain and database foundation
- [ ] WP2 - Canonical ingestion service
- [ ] WP3 - Import upload and thoth-app experience
- [ ] WP4 - Rollups and dashboard GraphQL
- [ ] WP5 - Service authentication and entitlements
- [ ] WP6 - Sphinx core
- [ ] WP7 - CloudFront driver
- [ ] WP8 - Additional platform drivers and COUNTER
- [ ] WP9 - OPERAS adapter and reconciliation
- [ ] WP10 - Dashboard and widget clients
- [ ] WP11 - Deployment, monitoring and migration
- [ ] MET-E2E-01 - Integrated acceptance and cutover

Do not close at code completion or CI success. Close only after independent approval, merge, required rollout, reconciliation and tracker update.




```

## 17. Deferred-write controls

The embedded bodies are evidence only. They do not authorize a write. Each issue
write still requires, in order:

1. immediate re-fetch of the complete live body and `updatedAt`;
2. exact comparison with the reviewed baseline;
3. stop on any mismatch;
4. regeneration and fresh independent review of any changed proposal;
5. separate explicit CTO authorization;
6. a minimal write that keeps the issue open.

Rollback must likewise re-fetch and compare, preserve unrelated edits, and apply
only a reviewed minimal reversal. A blind full-body restoration is prohibited.

## 18. Residual blockers

```text
ADR-0001: PROPOSED
Publisher Services ADR-01: unapproved
Platform inventory: VERIFIED BASELINE; FINAL ENUM NOT APPROVED
MET-CTRL-01: CHANGES REQUIRED
All Publisher Services implementation tasks: BLOCKED
All Metrics work packages: BLOCKED
Branch-readiness decisions: outstanding
```

ADR-0002 approval removes exactly one dependency. No implementation task becomes
`READY`.
