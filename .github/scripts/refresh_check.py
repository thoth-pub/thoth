#!/usr/bin/env python3
"""Prove that a refresh commit is a pure merge (CTRL-DELIVERY-03).

Usage:
    refresh_check.py --reviewed <sha> --target-tip <sha> --new-head <sha>
                     --target <ref> [--repo-dir <path>]

Local git only (no network, no GitHub API). Prints one line per check, then:

    PURE      exit 0   the new head is the reviewed head merged with the target
                       tip and nothing else.
    NOT PURE  exit 20  at least one check failed; reasons are printed.

Options:
    --changelog-union / --no-changelog-union   (default: on, as in thoth)

Checks:
  0. The target tip is an ancestor of (or equal to) --target. The script does
     not trust --target-tip on its own.
  1. The new head has exactly two parents: the reviewed head first, the target
     tip second.
  2. The new head's tree equals `git merge-tree --write-tree` of those parents.
     The only allowed difference is CHANGELOG.md, and only under decision D5
     (thoth): when CHANGELOG.md conflicted in the merge, the new file must be
     byte-identical to `git merge-file -p --union` of the reviewed, base and
     target versions (either side order) and have no conflict markers. When
     CHANGELOG.md merged cleanly, no difference is allowed. With
     --no-changelog-union a CHANGELOG.md conflict is NOT PURE.

The conflict-marker check also rejects a Markdown setext underline (a line of
exactly `=======`) in CHANGELOG.md. That fails closed.

Exit 2 means the script could not run (bad ref, git older than 2.38).
The script writes tree objects to the object store; it changes no ref, index
or working tree.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from typing import List, Optional, Sequence, Tuple

CHANGELOG = "CHANGELOG.md"
MIN_GIT = (2, 38)
MARKER = re.compile(r"^(<{7}|>{7})( |$)|^={7}$")


class ToolError(Exception):
    """The script cannot run (not a verdict)."""


def git(repo: str, *args: str, ok: Sequence[int] = (0,)) -> Tuple[int, str]:
    proc = subprocess.run(
        ["git", "-C", repo, *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="surrogateescape",
    )
    if proc.returncode not in ok:
        raise ToolError(
            "git %s failed (%d): %s" % (" ".join(args), proc.returncode, proc.stderr.strip())
        )
    return proc.returncode, proc.stdout


def require_git_version() -> None:
    try:
        out = subprocess.run(
            ["git", "--version"], stdout=subprocess.PIPE, text=True, check=True
        ).stdout
    except (OSError, subprocess.CalledProcessError) as exc:
        raise ToolError("git is not available: %s" % exc)
    match = re.search(r"(\d+)\.(\d+)", out)
    if not match or (int(match.group(1)), int(match.group(2))) < MIN_GIT:
        raise ToolError(
            "git >= %d.%d is required for merge-tree --write-tree (found: %s)"
            % (MIN_GIT[0], MIN_GIT[1], out.strip())
        )


def resolve(repo: str, rev: str) -> str:
    _, out = git(repo, "rev-parse", "--verify", "--quiet", rev + "^{commit}")
    return out.strip()


def toplevel(repo: str) -> str:
    _, out = git(repo, "rev-parse", "--show-toplevel")
    return out.strip()


def git_bytes(repo: str, *args: str) -> Tuple[int, bytes]:
    proc = subprocess.run(["git", "-C", repo, *args], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return proc.returncode, proc.stdout


def blob(repo: str, rev: str, path: str) -> bytes:
    code, out = git_bytes(repo, "show", "%s:%s" % (rev, path))
    return out if code == 0 else b""


def union_candidates(repo: str, base: str, reviewed: str, tip: str) -> List[bytes]:
    """Return git merge-file --union results for both side orders."""
    import tempfile

    results: List[bytes] = []
    with tempfile.TemporaryDirectory() as tmp:
        paths = {}
        for name, rev in (("base", base), ("reviewed", reviewed), ("tip", tip)):
            paths[name] = os.path.join(tmp, name)
            with open(paths[name], "wb") as handle:
                handle.write(blob(repo, rev, CHANGELOG))
        for ours, theirs in (("reviewed", "tip"), ("tip", "reviewed")):
            proc = subprocess.run(
                ["git", "merge-file", "-p", "--union", paths[ours], paths["base"], paths[theirs]],
                cwd=tmp, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            )
            if proc.returncode >= 0 and proc.returncode < 128:
                results.append(proc.stdout)
    return results


def on_target(repo: str, tip: str, target: str) -> bool:
    """True when tip is an ancestor of (or equal to) target."""
    code, _ = git(repo, "merge-base", "--is-ancestor", tip, target, ok=(0, 1))
    return code == 0


def check_pure(
    repo: str, reviewed: str, tip: str, new: str, changelog_union: bool = True,
    target: Optional[str] = None,
) -> Tuple[bool, List[str], List[str]]:
    """Return (pure, per-check lines, reasons). Arguments are resolved SHAs."""
    lines: List[str] = []
    reasons: List[str] = []

    if target is not None:
        if on_target(repo, tip, target):
            lines.append("0 target tip is on the target: ok")
        else:
            reasons.append("target tip %s is not on the target" % tip[:12])
            lines.append("0 target tip is on the target: FAIL")

    _, out = git(repo, "rev-list", "--parents", "-n", "1", new)
    parents = out.split()[1:]
    if parents == [reviewed, tip]:
        lines.append("1 parents: ok (reviewed head, then target tip)")
    else:
        reason = "parents are [%s], expected [%s, %s]" % (
            ", ".join(p[:12] for p in parents), reviewed[:12], tip[:12])
        reasons.append(reason)
        lines.append("1 parents: FAIL (%s)" % reason)

    code, out = git(
        repo, "merge-tree", "--write-tree", "--name-only", "-z", reviewed, tip, ok=(0, 1)
    )
    parts = out.split("\0")
    expected_tree = parts[0].strip()
    conflicts: List[str] = []
    if code == 1:
        for part in parts[1:]:
            if part == "":
                break
            conflicts.append(part)
        if not conflicts:
            reasons.append("merge-tree reported a conflict but named no path")
    _, out = git(repo, "diff-tree", "-r", "--no-renames", "--name-only", "-z",
                 expected_tree, new + "^{tree}")
    differing = [p for p in out.split("\0") if p]

    problems: List[str] = []
    allowed_conflicts = [CHANGELOG] if changelog_union else []
    other_conflicts = [c for c in conflicts if c not in allowed_conflicts]
    if other_conflicts:
        problems.append("conflict that needs hand resolution: " + ", ".join(other_conflicts))
    other_diff = [p for p in differing if p != CHANGELOG]
    if other_diff:
        problems.append("tree differs from merge result at: " + ", ".join(other_diff))
    note = ""
    if not problems:
        if CHANGELOG in conflicts:
            _, mb = git(repo, "merge-base", reviewed, tip)
            new_blob = blob(repo, new, CHANGELOG)
            if any(MARKER.match(l) for l in new_blob.decode("utf-8", "replace").split("\n")):
                problems.append("CHANGELOG.md has conflict markers")
            elif new_blob not in union_candidates(repo, mb.strip(), reviewed, tip):
                problems.append("CHANGELOG.md is not the git merge-file --union of both sides")
            else:
                note = " (CHANGELOG.md is the union of both sides)"
        elif CHANGELOG in differing:
            problems.append("CHANGELOG.md differs from the clean merge result")
    if problems:
        reasons.extend(problems)
        lines.append("2 tree equals merge result: FAIL")
    else:
        lines.append("2 tree equals merge result: ok" + note)
    return (not reasons), lines, reasons


def run(args: argparse.Namespace) -> int:
    require_git_version()
    repo = toplevel(args.repo_dir)
    reviewed = resolve(repo, args.reviewed)
    tip = resolve(repo, args.target_tip)
    new = resolve(repo, args.new_head)
    target = resolve(repo, args.target)
    pure, lines, reasons = check_pure(repo, reviewed, tip, new, args.changelog_union, target)
    for line in lines:
        print(line)
    if pure:
        print("PURE")
        return 0
    print("NOT PURE")
    for reason in reasons:
        print("  - " + reason)
    return 20


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--reviewed", required=True, help="reviewed head SHA")
    parser.add_argument("--target-tip", required=True, help="target tip SHA")
    parser.add_argument("--new-head", required=True, help="refreshed PR head SHA")
    parser.add_argument("--target", required=True, help="target ref the tip must be on")
    parser.add_argument("--repo-dir", default=".")
    parser.add_argument("--changelog-union", action=argparse.BooleanOptionalAction, default=True,
                        help="apply decision D5 (thoth only); default on")
    args = parser.parse_args(argv)
    try:
        return run(args)
    except ToolError as exc:
        print("ERROR: %s" % exc, file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
