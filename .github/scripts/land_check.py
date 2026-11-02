#!/usr/bin/env python3
"""Land-check for a reviewed pull request head (CTRL-DELIVERY-03).

Usage:
    land_check.py --reviewed <sha> --head <sha> --target <ref>
                  [--authorized <sha>] [--migrations-dir <path>]
                  [--lockfiles <glob,...>] [--repo-dir <path>]
                  [--changelog-union | --no-changelog-union]

Runs checks 1 to 5 of docs/engineering/ai-delivery/parallel-landing.md with
local git only (no network, no GitHub API). Prints one line per check, then a
verdict:

    PASS     exit 0   all checks hold; guarded merge of the reviewed head.
    REFRESH  exit 10  check 3, 4 or 5 failed, or only CHANGELOG.md conflicts.
    FAIL     exit 20  check 1 failed, or a conflict outside CHANGELOG.md.

--authorized is always the MERGE AUTH SHA. When head differs from it, the script
walks first parents from head back to it (at most 20 steps). Each step must be
a two-parent merge whose second parent is on --target and which refresh_check
rates PURE. The chain is printed in check 1 for the LANDED evidence.
--no-changelog-union turns off decision D5 (default on, as in thoth):
any CHANGELOG.md conflict is then FAIL.

Exit 2 means the script could not run (bad ref, git older than 2.38). That is
not a verdict. Check 6 (CI) is not checked here; verify it on GitHub.

The script writes tree objects to the repository object store through
`git merge-tree --write-tree`. It changes no ref, index or working tree.
"""

from __future__ import annotations

import argparse
import fnmatch
import os
import re
import subprocess
import sys
from typing import List, Optional, Sequence, Tuple

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import refresh_check  # noqa: E402

CHANGELOG = "CHANGELOG.md"
DEFAULT_MIGRATIONS_DIR = "thoth-api/migrations"
DEFAULT_LOCKFILES = (
    "Cargo.lock,**/Cargo.toml,package-lock.json,**/package.json,"
    "requirements*.txt,pyproject.toml,poetry.lock,uv.lock"
)
MIN_GIT = (2, 38)


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


def toplevel(repo: str) -> str:
    _, out = git(repo, "rev-parse", "--show-toplevel")
    return out.strip()


def resolve(repo: str, rev: str) -> str:
    _, out = git(repo, "rev-parse", "--verify", "--quiet", rev + "^{commit}")
    return out.strip()


def changed_paths(repo: str, base: str, tip: str) -> List[str]:
    _, out = git(repo, "diff", "--no-relative", "--name-only", "--no-renames", "-z", base, tip)
    return [p for p in out.split("\0") if p]


def merge_conflicts(repo: str, head: str, target: str) -> Tuple[bool, List[str]]:
    """Return (clean, conflicted paths) for a trial merge of head and target."""
    code, out = git(
        repo, "merge-tree", "--write-tree", "--name-only", "-z", head, target, ok=(0, 1)
    )
    if code == 0:
        return True, []
    parts = out.split("\0")
    files: List[str] = []
    for part in parts[1:]:
        if part == "":
            break
        files.append(part)
    return False, files


def glob_match(path: str, pattern: str) -> bool:
    if fnmatch.fnmatchcase(path, pattern):
        return True
    if pattern.startswith("**/") and fnmatch.fnmatchcase(path, pattern[3:]):
        return True
    if "/" not in pattern and fnmatch.fnmatchcase(path.rsplit("/", 1)[-1], pattern):
        return True
    return False


DATED = re.compile(r"^\d{8}_")


def migration_names(repo: str, rev: str, mig_dir: str) -> List[str]:
    code, out = git(
        repo, "ls-tree", "--full-tree", "--name-only", "-z", rev,
        mig_dir.rstrip("/") + "/", ok=(0, 128),
    )
    if code != 0:
        return []
    names = [p.rsplit("/", 1)[-1] for p in out.split("\0") if p]
    return [n for n in names if DATED.match(n)]


def migration_date(name: str) -> str:
    return name.split("_", 1)[0]


def under(path: str, directory: str) -> bool:
    return path.startswith(directory.rstrip("/") + "/")


def dated_migration_of(path: str, mig_dir: str) -> Optional[str]:
    prefix = mig_dir.rstrip("/") + "/"
    if not path.startswith(prefix):
        return None
    name = path[len(prefix):].split("/", 1)[0]
    return name if DATED.match(name) else None


def pr_migration_names(paths: Sequence[str], mig_dir: str) -> List[str]:
    return sorted({n for n in (dated_migration_of(p, mig_dir) for p in paths) if n})


MAX_CHAIN = 20


def refresh_chain(
    repo: str, head: str, authorized: str, target: str, changelog_union: bool
) -> Tuple[bool, str]:
    """Walk first parents from head to authorized; every step must be PURE."""
    cur = head
    chain = [authorized[:12]]
    steps: List[str] = []
    for _ in range(MAX_CHAIN):
        if cur == authorized:
            for item in reversed(steps):
                chain.append(item)
            return True, " -> ".join(chain)
        _, out = git(repo, "rev-list", "--parents", "-n", "1", cur)
        parents = out.split()[1:]
        if len(parents) != 2:
            return False, "%s is not a two-parent refresh merge" % cur[:12]
        p1, p2 = parents
        pure, _, reasons = refresh_check.check_pure(
            repo, p1, p2, cur, changelog_union, target
        )
        if not pure:
            return False, "%s is not a PURE refresh of %s (%s)" % (
                cur[:12], p1[:12], "; ".join(reasons))
        steps.append("%s (target parent %s)" % (cur[:12], p2[:12]))
        cur = p1
    return False, "chain from head does not reach authorized within %d steps" % MAX_CHAIN


def run(args: argparse.Namespace) -> int:
    require_git_version()
    repo = toplevel(args.repo_dir)
    reviewed = resolve(repo, args.reviewed)
    head = resolve(repo, args.head)
    target = resolve(repo, args.target)
    authorized = resolve(repo, args.authorized) if args.authorized else None
    _, mb = git(repo, "merge-base", head, target)
    base = mb.strip()
    lockfiles = [g.strip() for g in args.lockfiles.split(",") if g.strip()]

    fail = False
    refresh = False

    # Check 1: head equals the reviewed SHA (and the authorized SHA, if given).
    problems = []
    note = ""
    if head != reviewed:
        problems.append("head %s != reviewed %s" % (head[:12], reviewed[:12]))
    if authorized is not None and head != authorized:
        ok, detail = refresh_chain(repo, head, authorized, target, args.changelog_union)
        if ok:
            note = " (pure-merge refresh chain: %s)" % detail
        else:
            problems.append("head %s != authorized %s and %s" % (head[:12], authorized[:12], detail))
    if problems:
        fail = True
    print("1 head binding: %s" % ("FAIL (%s)" % "; ".join(problems) if problems else "ok" + note))

    # Check 2: conflict-free merge.
    clean, conflicts = merge_conflicts(repo, head, target)
    allowed = [CHANGELOG] if args.changelog_union else []
    other = [c for c in conflicts if c not in allowed]
    if clean:
        print("2 conflict-free merge: ok")
    elif not conflicts:
        fail = True
        print("2 conflict-free merge: FAIL (conflict reported but no path named)")
    elif not other:
        refresh = True
        print("2 conflict-free merge: REFRESH (only %s conflicts; union resolution)" % CHANGELOG)
    else:
        fail = True
        print("2 conflict-free merge: FAIL (conflicts: %s)" % ", ".join(other))

    pr_files = changed_paths(repo, base, head)
    target_files = changed_paths(repo, base, target)

    # Check 3: no overlap of changed files (CHANGELOG.md ignored).
    overlap = sorted((set(pr_files) & set(target_files)) - {CHANGELOG})
    if overlap:
        refresh = True
        print("3 no file overlap: REFRESH (%s)" % ", ".join(overlap))
    else:
        print("3 no file overlap: ok")

    # Check 4: target gained no migration and no lockfile/manifest change.
    gained = sorted(
        p
        for p in target_files
        if dated_migration_of(p, args.migrations_dir) or any(glob_match(p, g) for g in lockfiles)
    )
    if gained:
        refresh = True
        print("4 target migrations and lockfiles: REFRESH (%s)" % ", ".join(gained))
    else:
        print("4 target migrations and lockfiles: ok")

    # Check 5: PR migrations sort after every target migration.
    mine = pr_migration_names(pr_files, args.migrations_dir)
    existing = set(migration_names(repo, base, args.migrations_dir))
    edited = [n for n in mine if n in existing]
    theirs = migration_names(repo, target, args.migrations_dir)
    if edited:
        fail = True
        print(
            "5 migration order: FAIL (PR modifies existing migration %s; HOLD, "
            "a refresh cannot clear this)" % ", ".join(edited)
        )
    elif not mine:
        print("5 migration order: ok (PR adds no migration)")
    else:
        newest: Optional[str] = max((migration_date(n) for n in theirs), default=None)
        bad = [n for n in mine if newest is not None and migration_date(n) <= newest]
        if bad:
            refresh = True
            print(
                "5 migration order: REFRESH (%s not after target newest %s; "
                "a re-date is a source change needing a full delta review)"
                % (", ".join(bad), newest)
            )
        else:
            print("5 migration order: ok")

    print("6 CI green at reviewed head and target tip: verify on GitHub; not checked by this script")

    if fail:
        print("FAIL")
        return 20
    if refresh:
        print("REFRESH")
        return 10
    print("PASS")
    return 0


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--reviewed", required=True, help="reviewed head SHA")
    parser.add_argument("--head", required=True, help="current PR head")
    parser.add_argument("--target", required=True, help="target ref or SHA")
    parser.add_argument("--authorized", help="merge-authorized SHA (HIGH/CRITICAL)")
    parser.add_argument("--migrations-dir", default=DEFAULT_MIGRATIONS_DIR)
    parser.add_argument("--lockfiles", default=DEFAULT_LOCKFILES)
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
