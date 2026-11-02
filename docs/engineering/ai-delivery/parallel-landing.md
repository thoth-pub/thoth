# Parallel Landing

Status: Proposed until merged and approved
Owner: CTO

## 1. Purpose

This document is the landing doctrine for the AI-led delivery controls. It lets independent changes land in parallel with few approval round trips. It does not weaken exact-head review, CTO merge authorization for HIGH and CRITICAL work, or the deny-by-default action model in `operating-model.md` section 2.4.

Terms: the reviewed head is the PR head SHA that an independent review approved. The target is the branch the PR merges into.

## 2. Lanes

- Repositories land independently.
- Within a repository, stages with disjoint declared write surfaces run and land in parallel (`operating-model.md` section 7).
- `thoth-pub/thoth` has one migration lane. A migration date (`thoth-api/migrations/YYYYMMDD_vX.Y.Z`) is reserved on the Delivery board at grant time. Migration PRs land one at a time, in date order.
- A new integration branch needs CTO approval. It is never created to avoid a conflict.

## 3. Land-check

The land-check is a script, not a model run: `.github/scripts/land_check.py`. It uses local git only. It passes only when all six checks hold.

1. The PR head equals the reviewed SHA. For HIGH and CRITICAL work it also equals the merge-authorized SHA.
2. The merge into the target is conflict-free.
3. The PR's changed files and the files changed on the target since the merge base do not overlap. `CHANGELOG.md` is ignored when git merges it cleanly.
4. The target gained no migration and no dependency lockfile or manifest change since the merge base. Any change under the migrations directory to a dated entry counts, even an edit to an existing one. This is deliberately conservative: it costs at most one refresh. Lockfile globs without a slash (for example `Cargo.lock`) match at any depth, because nested lockfiles are real.
5. Every migration in the PR still sorts after every migration on the target. An equal date fails. Only entries named `YYYYMMDD_*` count as migrations. A PR that modifies or deletes an existing migration is `FAIL`: post `HOLD`, because no refresh can clear it.
6. CI is green at the reviewed head and at the target tip. The lander checks this on GitHub. The script prints "verify on GitHub; not checked by this script".

Verdicts and exit codes:

| Verdict | Exit | Meaning |
| --- | --- | --- |
| `PASS` | 0 | Land the reviewed head. |
| `REFRESH` | 10 | Check 3, 4 or 5 failed, or the only conflict is in `CHANGELOG.md`. |
| `FAIL` | 20 | Check 1 failed, a conflict outside `CHANGELOG.md` (or any conflict whose path cannot be read), or a PR edit to an existing migration. |

Decision D5 (a `CHANGELOG.md`-only conflict is a refresh, not a failure) applies to `thoth-pub/thoth`. The scripts default to it (`--changelog-union`). Other repositories pass `--no-changelog-union` unless the CTO extends D5 to them; then any `CHANGELOG.md` conflict is `FAIL`.

Both scripts resolve the repository root with `git rev-parse --show-toplevel`, so they run from any directory of a checkout.

Exit 2 means the script could not run (for example git older than 2.38). It is not a verdict. A re-date under check 5 is a source change, so it needs a full delta review (section 6).

The run itself needs no reasoning. It is a Haiku-class task.

## 4. Landing on PASS

Land the reviewed head with GitHub's PR merge API, passing the reviewed head as the expected `sha` (`release-gates.md` section 1.1). The head is unchanged, so the independent review and any CTO merge authorization still bind to it.

Merge commits show Javi as author and GitHub as committer. This is accepted under the commit-authorship rule. Nothing is pushed directly to `develop`, `dev`, `main` or `master`.

Merge is not deployment. After landing, post a `LANDED` comment (`operating-model.md` section 9.1). For HIGH and CRITICAL work it records the authorized head, the `MERGE AUTH` reference, every refresh head and the `PURE` result of each. Any optional terminal-evidence comment records the same.

A red post-merge CI run on the target pauses landings in that repository and goes to the CTO. A revert is its own grant.

## 5. Refresh

When check 3, 4 or 5 fails, or the only conflict is in `CHANGELOG.md`, refresh the PR:

1. Merge the target tip into the PR branch. The commit is authored and committed as Javi. It makes no other change. The first parent is the reviewed head and the second parent is the target tip.
2. In `thoth-pub/thoth` a conflict only in `CHANGELOG.md` is resolved by keeping both sides' entries (decision D5).
3. Run `.github/scripts/refresh_check.py --reviewed <old> --target-tip <tip> --new-head <new> --target <ref>`. `PURE` means:
   - the target tip is an ancestor of (or equal to) the target ref;
   - the parents are the reviewed head and the target tip, in that order;
   - the new head's tree equals `git merge-tree --write-tree` of those parents;
   - if `CHANGELOG.md` merged cleanly, no difference at all is allowed;
   - if `CHANGELOG.md` conflicted (decision D5, thoth only), the new file must be byte-identical to `git merge-file -p --union` of the reviewed, base and target versions, in either side order, and must have no conflict markers.
4. On `PURE`, a Haiku light re-check of the overlapping files, including rename destinations, posts `REVIEW <TASK-ID> @ <new sha>: APPROVED (refresh of <old sha>)`. This counts as the fresh exact-head review for the new head (decision D2).
5. A CTO merge authorization for the reviewed head carries over to a pure-merge refresh (decision D2). It carries over to nothing else.
6. Rerun the land-check on the new head: `land_check.py --reviewed <new> --head <new> --authorized <MERGE AUTH sha> --target <ref>`. `--authorized` is always the SHA the CTO authorized (HIGH and CRITICAL). When the head differs from it, the script walks first parents from the head back to the authorized SHA, at most 20 steps. Each step must be a two-parent merge whose second parent is on the target and which `refresh_check` rates `PURE`. If the walk does not reach the authorized SHA exactly, check 1 fails. On success check 1 prints the chain (authorized SHA, then each refresh head with its target parent). Paste it into the `LANDED` evidence.
7. Then land.

On `NOT PURE`, treat the new head as a changed head. Review it fully.

## 6. Full re-review of the refresh delta

Use the stage's review model (`model-selection.md`) on the refresh delta, instead of the light re-check, when any of these holds:

- a conflict was resolved by hand (a `CHANGELOG.md` union under D5 is not a conflict resolved by hand);
- a migration date must change;
- the light re-check flags an interaction between the two sides;
- the stage is CRITICAL.

The delta is what the refresh changed relative to the reviewed head. Any head that `refresh_check.py` rates `NOT PURE` is reviewed in full.

## 7. Lander order

1. Migration lane, by migration date, then by age.
2. Other stages, by age.
3. After each landing, rerun the land-check on every remaining PR for that repository.

One lander per repository lands serially. The `sha` guard protects only the PR head. If the target tip moved after the land-check ran, rerun the land-check before merging.

## 8. Limits

- The land-check proves that the merge is mechanical. It does not prove that two changes work together. The light re-check and CI cover that.
- The scripts are git-only. They use no network and no GitHub API.
- Release, tag, deployment, migration runs outside CI and production activation stay separate grants (`operating-model.md` section 9.2).
