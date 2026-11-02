#!/usr/bin/env python3
"""Prove that a refresh commit is a pure merge (CTRL-DELIVERY-03).

Usage:
    refresh_check.py --reviewed <sha> --target-tip <sha> --new-head <sha>
                     [--repo-dir <path>]

Local git only (no network, no GitHub API). Prints one line per check, then:

    PURE      exit 0   the new head is the reviewed head merged with the target
                       tip and nothing else.
    NOT PURE  exit 20  at least one check failed; reasons are printed.

Checks:
  1. The new head has exactly two parents: the reviewed head first, the target
     tip second.
  2. The new head's tree equals `git merge-tree --write-tree` of those parents.
     The only allowed difference is CHANGELOG.md (decision D5, union
     resolution). It is accepted only when the new file has no conflict
     markers, holds every line added on each side since the merge base, and
     holds no line that is absent from the base, reviewed and target versions.
     Also, a base line that both sides kept must still be there.

Exit 2 means the script could not run (bad ref, git older than 2.38).
The script writes tree objects to the object store; it changes no ref, index
or working tree.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from typing import List, Optional, Sequence, Set, Tuple

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


def file_lines(repo: str, rev: str, path: str) -> List[str]:
    code, out = git(repo, "show", "%s:%s" % (rev, path), ok=(0, 128))
    return out.split("\n") if code == 0 else []


def nonblank(lines: Sequence[str]) -> Set[str]:
    return {line for line in lines if line.strip()}


def changelog_union_problems(
    base: Sequence[str], reviewed: Sequence[str], target: Sequence[str], new: Sequence[str]
) -> List[str]:
    problems: List[str] = []
    markers = [line for line in new if MARKER.match(line)]
    if markers:
        problems.append("CHANGELOG.md has conflict markers")
    b, r, t, n = nonblank(base), nonblank(reviewed), nonblank(target), nonblank(new)
    missing_r = (r - b) - n
    missing_t = (t - b) - n
    if missing_r:
        problems.append("CHANGELOG.md lost %d line(s) added by the reviewed head" % len(missing_r))
    if missing_t:
        problems.append("CHANGELOG.md lost %d line(s) added by the target" % len(missing_t))
    foreign = n - (b | r | t)
    if foreign:
        problems.append("CHANGELOG.md has %d line(s) found in no input version" % len(foreign))
    dropped = (b & r & t) - n
    if dropped:
        problems.append("CHANGELOG.md dropped %d line(s) both sides kept" % len(dropped))
    return problems


def run(args: argparse.Namespace) -> int:
    require_git_version()
    repo = args.repo_dir
    reviewed = resolve(repo, args.reviewed)
    tip = resolve(repo, args.target_tip)
    new = resolve(repo, args.new_head)
    reasons: List[str] = []

    # Check 1: parents.
    _, out = git(repo, "rev-list", "--parents", "-n", "1", new)
    parents = out.split()[1:]
    if parents == [reviewed, tip]:
        print("1 parents: ok (reviewed head, then target tip)")
    else:
        reason = "parents are [%s], expected [%s, %s]" % (
            ", ".join(p[:12] for p in parents),
            reviewed[:12],
            tip[:12],
        )
        reasons.append(reason)
        print("1 parents: FAIL (%s)" % reason)

    # Check 2: tree equals the merge-tree result.
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
    _, out = git(repo, "diff-tree", "-r", "--no-renames", "--name-only", "-z", expected_tree, new + "^{tree}")
    differing = [p for p in out.split("\0") if p]
    other_conflicts = [c for c in conflicts if c != CHANGELOG]
    other_diff = [p for p in differing if p != CHANGELOG]

    tree_ok = True
    if other_conflicts:
        tree_ok = False
        reasons.append("conflict outside CHANGELOG.md: " + ", ".join(other_conflicts))
    if other_diff:
        tree_ok = False
        reasons.append("tree differs from merge result at: " + ", ".join(other_diff))
    if tree_ok and (CHANGELOG in differing or CHANGELOG in conflicts):
        _, mb = git(repo, "merge-base", reviewed, tip)
        base = mb.strip()
        problems = changelog_union_problems(
            file_lines(repo, base, CHANGELOG),
            file_lines(repo, reviewed, CHANGELOG),
            file_lines(repo, tip, CHANGELOG),
            file_lines(repo, new, CHANGELOG),
        )
        if problems:
            tree_ok = False
            reasons.extend(problems)
        else:
            print("2 tree equals merge result: ok (CHANGELOG.md differs only by union of both sides)")
    elif tree_ok:
        print("2 tree equals merge result: ok")
    if not tree_ok:
        print("2 tree equals merge result: FAIL")

    if reasons:
        print("NOT PURE")
        for reason in reasons:
            print("  - " + reason)
        return 20
    print("PURE")
    return 0


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--reviewed", required=True, help="reviewed head SHA")
    parser.add_argument("--target-tip", required=True, help="target tip SHA")
    parser.add_argument("--new-head", required=True, help="refreshed PR head SHA")
    parser.add_argument("--repo-dir", default=".")
    args = parser.parse_args(argv)
    try:
        return run(args)
    except ToolError as exc:
        print("ERROR: %s" % exc, file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
