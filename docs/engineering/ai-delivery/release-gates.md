# Merge and Release Gates

PR approval, merge readiness, programme-integration readiness and production readiness are distinct states.

Branching and releases must follow `branching-and-release-workflow.md`.

## 1. Merge-ready gate

A task is merge ready only when:

- the task specification is approved;
- the diff matches the approved scope;
- all acceptance criteria have evidence;
- no unresolved P0 or P1 review findings remain;
- required unit, integration, database and authorization tests pass;
- CI is green;
- migrations are verified as required;
- generated schemas and clients are current;
- compatibility is assessed;
- documentation and programme status are updated;
- rollout and rollback are documented;
- no unrelated changes are present.

High-risk tasks require explicit CTO merge approval. So does any task whose
specification or governing control explicitly gates its merge, at any risk level.

For a slice targeting `feature/<programme>`, this gate approves only the slice merge into the programme integration branch. It does not approve the final programme merge into `develop` or production activation.

### 1.1 Guarded merge

Independent review binds to an exact PR head, and CTO merge authorization, where
required, binds to that same head.

Perform the merge with an expected-head guard, so that the merge fails rather
than proceeding if the reviewed head has moved. For example:

```bash
gh pr merge <number> --merge --match-head-commit <reviewed-sha>
```

The implementing agent must not merge its own PR.

### 1.2 Terminal merge evidence

Under `ADR-0005`, the successful GitHub merge event and the resulting merge
commit are terminal evidence that the task merged.

Do not open a further PR solely to record that a PR merged, its merge SHA or
timestamp, a review or authorization identifier, or a transition to "merged" or
"complete". Approval-state-only commits are prohibited when their sole purpose
is copying existing GitHub review or approval metadata into repository files.

Optional terminal evidence may be posted as a comment on the merged PR:

```text
TERMINAL MERGE EVIDENCE

Reviewed head:            <sha>
Independent review:       <id / APPROVED>
CTO merge authorization:  <id>
Merge commit:             <sha>
Merged at:                <timestamp>
Post-merge verification:  <results>
Runtime activation:       NOT AUTHORIZED / NOT APPLICABLE / separately authorized
```

That comment does not modify the repository and requires no further review,
merge authorization or PR.

A material post-merge correction still requires its own bounded task and PR.


## 2. Programme integration gate

Before opening or approving `feature/<programme> -> develop`:

- every included slice PR is approved and merged;
- all slice branches are deleted;
- the programme integration branch is updated with the current `develop`;
- integrated CI is green;
- migrations apply in the final order;
- cross-slice and cross-repository contracts are verified;
- programme-wide acceptance criteria have evidence;
- temporary bypasses, mocks and feature-only schema inconsistencies are removed;
- rollout and rollback cover the integrated feature;
- an independent reviewer assesses the complete integration diff against `develop`.

The final programme PR decision is `APPROVED`, `CHANGES REQUIRED` or `BLOCKED`.

## 3. Safe post-merge state

Prefer merged changes to be:

- additive;
- backwards compatible;
- disabled by default;
- unused by existing production paths;
- safe before downstream repositories deploy.

If merge itself changes production behaviour, the task must also satisfy the production-ready gate before merge.

## 4. Staging/preview gate

Before production activation:

- deploy the exact candidate commit;
- run required migrations in a production-like environment;
- verify authorization with allowed and denied actors;
- exercise failure and retry paths;
- verify monitoring and logs;
- verify rollback or disable procedures;
- capture evidence in the release issue or PR.

## 5. Production-ready gate

Production activation requires:

- staging/preview acceptance;
- approved migration plan;
- feature flag or controlled configuration where practical;
- named pilot or bounded activation scope;
- monitoring and alert thresholds;
- runbook;
- rollback/kill switch;
- explicit activation owner;
- observation period;
- CTO approval for high or critical risk.

## 6. Comparison-mode gate

When replacing an existing source of truth or operational configuration:

- run old and new paths in comparison/shadow mode;
- define expected equivalence and permitted divergence;
- produce a difference report;
- resolve unexplained differences;
- fail closed on new-source errors;
- preserve the old path through the observation window.

Do not cut over solely because the new path executes successfully.

## 7. Migration gate

For backfills or canonical-data migration:

- dry run first;
- record expected row/object counts;
- validate a representative sample;
- verify rerun behaviour;
- verify no unintended external side effects;
- snapshot or back up irreducible state;
- define abort thresholds;
- define rollback or forward-repair procedure;
- require explicit execution approval.

## 8. Observation gate

During observation:

- monitor correctness, errors, latency and backlog;
- compare expected and observed counts;
- keep rollback available;
- record incidents and divergence;
- do not remove compatibility paths.

The observation period ends only with an explicit sign-off.

## 9. Closure gate

A task closes when:

- observation is complete;
- acceptance remains valid in production;
- cleanup tasks are created or completed;
- temporary flags and compatibility paths have owners and deadlines;
- programme status is current;
- operational ownership is documented.

Closure is evidenced by the GitHub merge record and, where applicable, the
recorded observation sign-off. It does not require a repository commit that
declares the task closed.


## 10. Release branch gate

The production release path is `develop -> master`.

Before merging or releasing `develop` into `master`:

- identify all included task and programme PRs;
- confirm `develop` is green at the selected commit;
- review all migrations and required deployment ordering;
- prepare release notes;
- confirm feature flags and post-deploy activation;
- confirm rollback and restore procedures;
- obtain required CTO approval;
- record the resulting release/tag and production observation owner.
