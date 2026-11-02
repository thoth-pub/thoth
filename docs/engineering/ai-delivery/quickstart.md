# Delivery Quickstart

Status: Proposed until merged and approved
Owner: CTO

Read this first. It is a summary, not a replacement. When it and a full document differ, the full document wins.

Full documents: `operating-model.md`, `release-gates.md`, `parallel-landing.md`, `model-selection.md`, `risk-classification.md`, `branching-and-release-workflow.md`.

## 1. Start of a thread

Read, in this order: the state block in the stage issue body, the last three comments, the PR head and CI. Do not resume from chat memory (`operating-model.md` section 3.2).

## 2. State block

Edit it in place at the top of the stage issue body (`operating-model.md` section 9.1).

```text
Stage:     <TASK-ID> · Risk <LEVEL> · Programme: <programme>
Gate:      <current gate>
Spec:      <issue body or committed spec>
Base:      <repo> <branch> @ <sha>  ->  <target>
PR:        <number or none> · branch <name>
Surface:   <declared write surface> · Migration date: <date or none>
Grant:     <bundle and risk>
Review:    <state> · Codex: <state>
Next:      <next action> · Owner: <who>
```

## 3. Gate comments

Five kinds. Each starts `<KIND> <TASK-ID> @ <sha>: <ruling>` and stays under 15 lines.

- `GRANT`: the bundle, the action list, the reserved migration date.
- `REVIEW`: `APPROVED`, `CHANGES REQUIRED` or `BLOCKED`; model and effort; findings.
- `MERGE AUTH`: CTO authorization for the exact head.
- `LANDED`: merge commit, post-merge CI, model and effort.
- `HOLD`: why work stopped and what releases it.

## 4. Authorization bundles

| Risk | One grant covers | Merge |
| --- | --- | --- |
| LOW, MEDIUM | Implementation through guarded merge | Once review is `APPROVED` at the head, CI is green and Codex (where installed) has no P0 or P1 |
| HIGH | Implementation through `APPROVED` review | SHA-bound CTO `MERGE AUTH`; may be batched per landing window |
| CRITICAL | As HIGH, with two independent reviews, one cross-family | SHA-bound CTO `MERGE AUTH`; never batched with another CRITICAL |

Never bundled, each needs its own explicit grant: release or tag; deployment; migration runs outside CI; production activation; provider reads or writes; secrets; direct pushes to `develop`, `dev`, `main` or `master`; force-push; a new integration branch; a change to an approved ADR; growth of scope or write surface.

If the spec gates its own merge on the CTO, that gate applies at any risk.

## 5. Landing

`parallel-landing.md` has the rules. In short:

1. Run `python3 .github/scripts/land_check.py --reviewed <sha> --head <sha> --target <ref> [--authorized <sha>]`. It uses git only.
2. `PASS` (exit 0): merge the reviewed head with the PR merge API and the expected `sha`. Then post `LANDED`.
3. `REFRESH` (exit 10): merge the target tip into the PR branch as Javi, with no other change. Run `refresh_check.py --reviewed <old> --target-tip <tip> --new-head <new> --target <ref>`. On `PURE`, a light re-check posts `REVIEW <TASK-ID> @ <new sha>: APPROVED (refresh of <old sha>)`. Re-run `land_check.py --reviewed <new> --head <new> --authorized <MERGE AUTH sha> --target <ref>` (HIGH and CRITICAL). The script proves the whole refresh chain back to the authorized SHA and prints it for the `LANDED` evidence.
4. `FAIL` (exit 20): stop. The head moved or the merge conflicts. Post `HOLD`.
5. Check 6, CI green at the head and the target tip, is yours to verify on GitHub.
6. In `thoth-pub/thoth`, a conflict only in `CHANGELOG.md` is resolved by keeping both entries.
7. One lander per repository lands serially. Rerun the land-check if the target tip moved after it ran.
8. Outside thoth, pass `--no-changelog-union` unless the CTO extended D5.
9. Migrations land one at a time, in date order. The date is reserved on the Delivery board at grant time.

Do a full re-review of the refresh delta when a conflict was resolved by hand, a migration date changed, the light re-check flags an interaction, or the stage is CRITICAL.

A red post-merge CI run pauses that repository's landings and goes to the CTO.

## 6. Models

Names evolve. Record model and effort in every `REVIEW` and `LANDED` comment (`model-selection.md` section 6).

| Task | Model, effort |
| --- | --- |
| Cross-programme architecture, ADRs, CRITICAL design, large audits | Fable 5.1 (named only) |
| Specs and stage decomposition | Opus 5.5, high |
| Stage orchestrator; LOW and MEDIUM implementation | Sonnet 5.5, medium |
| HIGH implementation | Opus 5.5, high |
| CRITICAL implementation | Opus 5.5, max |
| LOW and MEDIUM review | Fresh Sonnet 5.5, high, plus Codex |
| HIGH review | Fresh Opus 5.5, high, plus Codex |
| CRITICAL review | Fresh Opus 5.5 and fresh Fable 5.1, plus Codex |
| Land-check, refresh re-check, landing, CI watching, state block and board edits, handoffs | Haiku 5.5, low |

## 7. Token rules

- Read this page, not all the full documents. Open a full document only for the section you need.
- Give a reviewer the spec, the diff and the CI evidence. Do not paste the implementer's narrative.
- Run scripts for mechanical checks. Do not spend a model on what a script proves.
- Keep gate comments short. Link to evidence; do not copy it.
- Edit the state block. Do not append status comments.
- Do not re-read unchanged files or re-run unchanged checks.
- Use the smallest model in the table that fits the task.

## 8. Standing rules

- Review binds to the exact head. A changed head needs a new review, except a proved pure-merge refresh.
- An implementer never approves or merges its own work.
- Merge is not deployment.
- Commits are authored and committed as Javier Arias. No AI trailers.
- Review rulings are COMMENT reviews (`operating-model.md` section 9.1).
- Stop and post `HOLD` if a step needs an action the grant does not name.
