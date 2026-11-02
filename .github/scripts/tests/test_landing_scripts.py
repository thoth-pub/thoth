"""Tests for land_check.py and refresh_check.py using throwaway git repos."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
LAND = os.path.join(SCRIPTS, "land_check.py")
REFRESH = os.path.join(SCRIPTS, "refresh_check.py")

CHANGELOG_BASE = "# Changelog\n## [Unreleased]\n### Added\n  - old entry\n"


def changelog(*top):
    return "# Changelog\n## [Unreleased]\n### Added\n" + "".join(
        "  - %s\n" % t for t in top
    ) + "  - old entry\n"


class Repo:
    def __init__(self, path):
        self.path = path
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "Test User")
        self.git("config", "user.email", "test@example.com")
        self.git("config", "commit.gpgsign", "false")

    def git(self, *args, check=True):
        proc = subprocess.run(
            ["git", "-C", self.path, *args],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        if check and proc.returncode != 0:
            raise AssertionError("git %s: %s" % (args, proc.stderr))
        return proc.stdout.strip()

    def write(self, files):
        for name, content in files.items():
            full = os.path.join(self.path, name)
            os.makedirs(os.path.dirname(full), exist_ok=True)
            with open(full, "w") as handle:
                handle.write(content)

    def commit(self, files, msg="c"):
        self.write(files)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", msg)
        return self.git("rev-parse", "HEAD")

    def branch(self, name, start):
        self.git("checkout", "-q", "-B", name, start)

    def sha(self, rev):
        return self.git("rev-parse", rev)


def run_script(script, repo, *args):
    proc = subprocess.run(
        [sys.executable, script, "--repo-dir", repo.path, *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return proc.returncode, proc.stdout, proc.stderr


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.repo = Repo(self.tmp)
        self.base = self.repo.commit(
            {
                "src/a.txt": "a\n",
                "src/b.txt": "b\n",
                "CHANGELOG.md": CHANGELOG_BASE,
                "Cargo.lock": "lock1\n",
                "thoth-api/migrations/20260101_v1.0.0/up.sql": "x\n",
            },
            "base",
        )

    def pr(self, files):
        self.repo.branch("pr", self.base)
        return self.repo.commit(files, "pr")

    def target(self, files):
        self.repo.branch("target", self.base)
        return self.repo.commit(files, "target")

    def land(self, head, reviewed=None, *extra):
        return run_script(
            LAND, self.repo, "--reviewed", reviewed or head, "--head", head,
            "--target", "target", *extra,
        )


class LandCheckTests(Base):
    def test_pass(self):
        head = self.pr({"src/a.txt": "a2\n", "CHANGELOG.md": changelog("pr")})
        self.target({"src/b.txt": "b2\n"})
        code, out, _ = self.land(head)
        self.assertEqual(code, 0, out)
        self.assertTrue(out.strip().endswith("PASS"))
        self.assertIn("verify on GitHub; not checked by this script", out)

    def test_pass_with_clean_authorized_and_migration_after(self):
        head = self.pr({"thoth-api/migrations/20260201_v1.0.0/up.sql": "y\n"})
        self.target({"src/b.txt": "b2\n"})
        code, out, _ = self.land(head, None, "--authorized", head)
        self.assertEqual(code, 0, out)

    def test_refresh_overlap_clean_merge(self):
        self.repo.branch("setup", self.base)
        base2 = self.repo.commit({"src/c.txt": "1\n2\n3\n4\n5\n6\n7\n8\n9\n"}, "c")
        self.base = base2
        self.repo.branch("pr", base2)
        head = self.repo.commit({"src/c.txt": "ONE\n2\n3\n4\n5\n6\n7\n8\n9\n"}, "pr")
        self.repo.branch("target", base2)
        self.repo.commit({"src/c.txt": "1\n2\n3\n4\n5\n6\n7\n8\nNINE\n"}, "t")
        code, out, _ = self.land(head)
        self.assertEqual(code, 10, out)
        self.assertIn("3 no file overlap: REFRESH", out)
        self.assertIn("2 conflict-free merge: ok", out)

    def test_refresh_target_added_migration(self):
        head = self.pr({"src/a.txt": "a2\n"})
        self.target({"thoth-api/migrations/20260115_v1.0.0/up.sql": "z\n"})
        code, out, _ = self.land(head)
        self.assertEqual(code, 10, out)
        self.assertIn("4 target migrations and lockfiles: REFRESH", out)

    def test_refresh_target_lockfile_change(self):
        head = self.pr({"src/a.txt": "a2\n"})
        self.target({"Cargo.lock": "lock2\n"})
        code, out, _ = self.land(head)
        self.assertEqual(code, 10, out)
        self.assertIn("4 target migrations and lockfiles: REFRESH", out)

    def test_refresh_target_nested_manifest_change(self):
        head = self.pr({"src/a.txt": "a2\n"})
        self.target({"crate/sub/Cargo.toml": "[package]\n"})
        code, out, _ = self.land(head)
        self.assertEqual(code, 10, out)

    def test_refresh_pr_migration_date_not_last(self):
        # The base already has 20260101, so an equal date is not after it.
        head = self.pr({"thoth-api/migrations/20260101_v1.1.0/up.sql": "y\n"})
        self.target({"src/b.txt": "b2\n"})
        code, out, _ = self.land(head)
        self.assertEqual(code, 10, out)
        self.assertIn("5 migration order: REFRESH", out)
        self.assertIn("full delta review", out)

    def test_refresh_changelog_only_conflict(self):
        head = self.pr({"CHANGELOG.md": changelog("pr entry")})
        self.target({"CHANGELOG.md": changelog("target entry")})
        code, out, _ = self.land(head)
        self.assertEqual(code, 10, out)
        self.assertIn("only CHANGELOG.md conflicts", out)

    def test_fail_head_mismatch(self):
        head = self.pr({"src/a.txt": "a2\n"})
        other = self.repo.commit({"src/a.txt": "a3\n"}, "pr2")
        self.repo.branch("target", self.base)
        self.repo.commit({"src/b.txt": "b2\n"}, "t")
        code, out, _ = self.land(other, head)
        self.assertEqual(code, 20, out)
        self.assertIn("1 head binding: FAIL", out)

    def test_fail_authorized_mismatch(self):
        head = self.pr({"src/a.txt": "a2\n"})
        other = self.repo.commit({"src/a.txt": "a3\n"}, "pr2")
        self.target({"src/b.txt": "b2\n"})
        code, out, _ = self.land(head, None, "--authorized", other)
        self.assertEqual(code, 20, out)
        self.assertIn("authorized", out)

    def test_fail_real_conflict(self):
        head = self.pr({"src/a.txt": "pr side\n"})
        self.target({"src/a.txt": "target side\n"})
        code, out, _ = self.land(head)
        self.assertEqual(code, 20, out)
        self.assertIn("src/a.txt", out)

    def test_fail_conflict_plus_changelog(self):
        head = self.pr({"src/a.txt": "pr side\n", "CHANGELOG.md": changelog("p")})
        self.target({"src/a.txt": "target side\n", "CHANGELOG.md": changelog("t")})
        code, out, _ = self.land(head)
        self.assertEqual(code, 20, out)


class RefreshCheckTests(Base):
    def setUp(self):
        super().setUp()
        self.reviewed = self.pr({"src/a.txt": "a2\n", "CHANGELOG.md": changelog("pr")})
        self.tip = self.target({"src/b.txt": "b2\n"})

    def refresh(self, new, reviewed=None, tip=None):
        return run_script(
            REFRESH, self.repo, "--reviewed", reviewed or self.reviewed,
            "--target-tip", tip or self.tip, "--new-head", new,
        )

    def merge(self, branch_from, other, message_files=None, resolve=None):
        self.repo.branch("refreshed", branch_from)
        proc = subprocess.run(
            ["git", "-C", self.repo.path, "merge", "--no-ff", "-q", "-m", "refresh", other],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        if resolve is not None:
            self.repo.write(resolve)
            self.repo.git("add", "-A")
            self.repo.git("commit", "-q", "-m", "refresh")
        else:
            self.assertEqual(proc.returncode, 0, proc.stderr)
        return self.repo.sha("HEAD")

    def test_pure(self):
        new = self.merge(self.reviewed, self.tip)
        code, out, _ = self.refresh(new)
        self.assertEqual(code, 0, out)
        self.assertTrue(out.strip().endswith("PURE"))

    def test_pure_changelog_union(self):
        reviewed = self.pr({"CHANGELOG.md": changelog("pr entry")})
        tip = self.target({"CHANGELOG.md": changelog("target entry")})
        union = changelog("pr entry", "target entry")
        new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": union})
        code, out, _ = self.refresh(new, reviewed, tip)
        self.assertEqual(code, 0, out)
        self.assertIn("PURE", out)
        self.assertIn("union", out)

    def test_not_pure_changelog_marker_left(self):
        reviewed = self.pr({"CHANGELOG.md": changelog("pr entry")})
        tip = self.target({"CHANGELOG.md": changelog("target entry")})
        bad = "<<<<<<< HEAD\n" + changelog("pr entry", "target entry")
        new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": bad})
        code, out, _ = self.refresh(new, reviewed, tip)
        self.assertEqual(code, 20, out)
        self.assertIn("conflict markers", out)

    def test_not_pure_changelog_lost_line(self):
        reviewed = self.pr({"CHANGELOG.md": changelog("pr entry")})
        tip = self.target({"CHANGELOG.md": changelog("target entry")})
        new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": changelog("pr entry")})
        code, out, _ = self.refresh(new, reviewed, tip)
        self.assertEqual(code, 20, out)
        self.assertIn("lost", out)

    def test_not_pure_changelog_foreign_line(self):
        reviewed = self.pr({"CHANGELOG.md": changelog("pr entry")})
        tip = self.target({"CHANGELOG.md": changelog("target entry")})
        union = changelog("pr entry", "target entry", "sneaky entry")
        new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": union})
        code, out, _ = self.refresh(new, reviewed, tip)
        self.assertEqual(code, 20, out)
        self.assertIn("no input version", out)

    def test_not_pure_hand_edit(self):
        self.repo.branch("refreshed2", self.reviewed)
        subprocess.run(
            ["git", "-C", self.repo.path, "merge", "--no-ff", "--no-commit", "-q", self.tip],
            check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        self.repo.write({"src/a.txt": "sneaky\n"})
        self.repo.git("add", "-A")
        self.repo.git("commit", "-q", "-m", "refresh with edit")
        edited = self.repo.sha("HEAD")
        code, out, _ = self.refresh(edited)
        self.assertEqual(code, 20, out)
        self.assertIn("src/a.txt", out)

    def test_not_pure_hand_resolved_conflict(self):
        reviewed = self.pr({"src/a.txt": "pr side\n"})
        tip = self.target({"src/a.txt": "target side\n"})
        new = self.merge(reviewed, tip, resolve={"src/a.txt": "resolved\n"})
        code, out, _ = self.refresh(new, reviewed, tip)
        self.assertEqual(code, 20, out)

    def test_not_pure_wrong_parent_order(self):
        self.repo.branch("refreshed", self.tip)
        subprocess.run(
            ["git", "-C", self.repo.path, "merge", "--no-ff", "-q", "-m", "swap", self.reviewed],
            check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        swapped = self.repo.sha("HEAD")
        code, out, _ = self.refresh(swapped)
        self.assertEqual(code, 20, out)
        self.assertIn("parents", out)

    def test_not_pure_single_parent(self):
        self.repo.branch("refreshed", self.reviewed)
        single = self.repo.commit({"src/b.txt": "b2\n"}, "squashed")
        code, out, _ = self.refresh(single)
        self.assertEqual(code, 20, out)
        self.assertIn("1 parents: FAIL", out)


if __name__ == "__main__":
    unittest.main()
