# Independent Review Brief - CTRL-FOUNDATION-01

Review status: READY FOR FRESH INDEPENDENT REVIEW
Task: `CTRL-FOUNDATION-01`
Repository: `thoth-pub/thoth`
Pull request: [#764](https://github.com/thoth-pub/thoth/pull/764)
Base: `develop` at `652a499dfdfbaa7594537e0865c41ec617f52dc2`
Risk: LOW, documentation-only
Production effect: None

## Reviewer independence

The reviewer must not be the ChatGPT conversation that authored these documents. Use a separate Codex or Claude instance at high reasoning.

## Required inputs

Review the versions at the exact PR head:

1. [`../tasks/CTRL-FOUNDATION-01.md`](../tasks/CTRL-FOUNDATION-01.md)
2. [`../implementation-reports/CTRL-FOUNDATION-01-implementation-report.md`](../implementation-reports/CTRL-FOUNDATION-01-implementation-report.md)
3. [Private Publisher Services design](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit), Drive revision `3`
4. [Private Thoth Metrics design](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit), Drive revision `6`
5. [`../../design-references.md`](../../design-references.md)
6. repository/environment map;
7. ADR-0001 and ADR-0002;
8. issues #765 and #766;
9. complete PR diff and current-head CI.

The reviewer must have explicit Google Drive access to both private designs.
The repository intentionally does not contain their contents.

## Review objectives

Verify:

- approved task specification completeness;
- implementation-report evidence;
- fidelity to both private approved designs at the recorded Drive revisions;
- authority, task, review and release controls;
- proposed-decision status;
- Publisher Services direct task-branch workflow;
- Metrics repository-local integration workflow;
- live repository/branch accuracy;
- canonical naming and the recorded private-source spelling discrepancy;
- agent safety and no production effects;
- complete trackers/issues and explicit blockers;
- valid control identifiers and links.

## Required commands

```bash
base=652a499dfdfbaa7594537e0865c41ec617f52dc2

git diff --check "$base"...HEAD

git diff --name-only "$base"...HEAD

git diff --stat "$base"...HEAD

grep -RniE \
  '(^|[^[:alnum:]_-])CTRL-(01|02)([^[:digit:]]|$)' \
  docs \
  || true

grep -RniE \
  'CG-(05|06|09)' \
  docs/engineering/repository-map/environments.md \
  docs/engineering/repository-map/repositories \
  || true

grep -Rni 'Publisher Services integration bran[c]h' docs || true

grep -Rni 'NOT YET CREATE[D]' docs || true

grep -Rni 'sph[y]nx' AGENTS.md .github thoth-* docs || true
```

Confirm:

- only `CHANGELOG.md`, AGENTS instruction files and `docs/**` changed;
- no workflow behaviour, Rust, SQL, migration, package or deployment files changed;
- all current-head required CI is green;
- both private design file IDs, revision IDs and modification times match `design-references.md`;
- master issues exist and link to the exact reviewed commit;
- ADRs remain proposed;
- acceptance checkboxes match evidence;
- no unreviewed head movement occurs after evidence capture.

## Red-team questions

1. Could any document be read as permission to merge, deploy or access secrets?
2. Is the task specification actually approved and complete?
3. Does the implementation report contain exact evidence rather than narrative claims?
4. Does Publisher Services incorrectly use a programme integration branch?
5. Does Metrics correctly use repository-local integration branches only after readiness?
6. Are missing or normalized branches accurately represented?
7. Are package business decisions silently settled?
8. Is OPERAS inbound completeness overstated?
9. Is the Publisher Services inventory presented as approved?
10. Are package and platform assignments independent?
11. Are distribution and metrics platforms separate?
12. Are missing CI, deployment and schema-generation controls explicit?
13. Are the private design revision records exact, and does the reviewer have authorized access?
14. Are all links valid at the exact review head and after merge?

## Decision format

Return exactly one:

```text
APPROVED
CHANGES REQUIRED
BLOCKED
```

For every finding provide:

```text
Severity:
File and line:
Requirement:
Evidence:
Required change:
```

Approval requires no unresolved P0/P1 findings, complete specification/report evidence, passing required commands, green final CI and no unreviewed head movement. The reviewer must not merge the PR.
