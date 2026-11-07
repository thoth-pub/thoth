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

    def test_fail_edit_existing_migration(self):
        head = self.pr({"thoth-api/migrations/20260101_v1.0.0/up.sql": "edited\n"})
        self.target({"src/b.txt": "b2\n"})
        code, out, _ = self.land(head)
        self.assertEqual(code, 20, out)
        self.assertIn("HOLD", out)

    def test_non_dated_migration_entry_ignored(self):
        self.repo.branch("setup", self.base)
        self.base = self.repo.commit({"thoth-api/migrations/README.md": "r\n"}, "readme")
        self.repo.branch("pr", self.base)
        head = self.repo.commit({"thoth-api/migrations/20260301_v1.0.0/up.sql": "y\n"}, "pr")
        self.repo.branch("target", self.base)
        self.repo.commit({"thoth-api/migrations/README.md": "r2\n"}, "t")
        code, out, _ = self.land(head)
        self.assertEqual(code, 0, out)

    def test_no_changelog_union_makes_changelog_conflict_fail(self):
        head = self.pr({"CHANGELOG.md": changelog("pr entry")})
        self.target({"CHANGELOG.md": changelog("target entry")})
        code, out, _ = self.land(head, None, "--no-changelog-union")
        self.assertEqual(code, 20, out)

    def test_runs_from_subdirectory_with_diff_relative(self):
        self.repo.git("config", "diff.relative", "true")
        sub = os.path.join(self.tmp, "thoth-api")
        # S1: PR migration date equal to an existing one, run below the root.
        head = self.pr({"thoth-api/migrations/20260101_v1.1.0/up.sql": "y\n"})
        self.target({"src/b.txt": "b2\n"})
        proc = subprocess.run(
            [sys.executable, LAND, "--repo-dir", sub, "--reviewed", head, "--head", head,
             "--target", "target"], stdout=subprocess.PIPE, text=True)
        self.assertEqual(proc.returncode, 10, proc.stdout)
        # S2: target gains a migration, run below the root with diff.relative.
        head = self.pr({"src/a.txt": "a2\n"})
        self.target({"thoth-api/migrations/20260115_v1.0.0/up.sql": "z\n"})
        proc = subprocess.run(
            [sys.executable, LAND, "--reviewed", head, "--head", head, "--target", "target"],
            cwd=sub, stdout=subprocess.PIPE, text=True)
        self.assertEqual(proc.returncode, 10, proc.stdout)

    def _refresh(self, reviewed, resolve=None):
        self.repo.branch("refreshed", reviewed)
        subprocess.run(["git", "-C", self.tmp, "merge", "--no-ff", "--no-commit", "-q", "target"],
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if resolve:
            self.repo.write(resolve)
        self.repo.git("add", "-A")
        self.repo.git("commit", "-q", "-m", "refresh")
        return self.repo.sha("HEAD")

    def _advance_target(self, files):
        self.repo.git("checkout", "-q", "target")
        return self.repo.commit(files, "target2")

    def test_chain_one_step_accepted(self):
        reviewed = self.pr({"src/a.txt": "a2\n"})
        self.target({"src/b.txt": "b2\n"})
        new = self._refresh(reviewed)
        code, out, _ = self.land(new, None, "--authorized", reviewed)
        self.assertEqual(code, 0, out)
        self.assertIn("pure-merge refresh chain", out)
        self.assertIn(reviewed[:12], out)

    def test_chain_two_steps_accepted(self):
        reviewed = self.pr({"src/a.txt": "a2\n"})
        self.target({"src/b.txt": "b2\n"})
        n1 = self._refresh(reviewed)
        self._advance_target({"src/c.txt": "c\n"})
        n2 = self._refresh(n1)
        code, out, _ = self.land(n2, None, "--authorized", reviewed)
        self.assertEqual(code, 0, out)
        self.assertIn(n1[:12], out)

    def test_chain_two_steps_one_impure_rejected(self):
        reviewed = self.pr({"src/a.txt": "a2\n"})
        self.target({"src/b.txt": "b2\n"})
        n1 = self._refresh(reviewed, {"src/a.txt": "sneaky\n"})
        self._advance_target({"src/c.txt": "c\n"})
        n2 = self._refresh(n1)
        code, out, _ = self.land(n2, None, "--authorized", reviewed)
        self.assertEqual(code, 20, out)

    def test_chain_bound_allows_twenty_refreshes_not_twenty_one(self):
        # parallel-landing.md section 5 step 6: at most 20 refresh steps.
        reviewed = self.pr({"src/a.txt": "a2\n"})
        heads = []
        cur = reviewed
        for i in range(21):
            step = self.target if i == 0 else self._advance_target
            step({"src/c%02d.txt" % i: "c\n"})
            cur = self._refresh(cur)
            heads.append(cur)
        code, out, _ = self.land(heads[19], None, "--authorized", reviewed)
        self.assertEqual(code, 0, out)
        code, out, _ = self.land(heads[20], None, "--authorized", reviewed)
        self.assertEqual(code, 20, out)
        self.assertIn("within 20 steps", out)

    def test_hand_edited_refresh_rejected(self):
        reviewed = self.pr({"src/a.txt": "a2\n"})
        self.target({"src/b.txt": "b2\n"})
        new = self._refresh(reviewed, {"src/a.txt": "sneaky\n"})
        code, out, _ = self.land(new, None, "--authorized", reviewed)
        self.assertEqual(code, 20, out)

    def test_t1_unauthorized_intermediate_head_rejected(self):
        a = self.pr({"src/a.txt": "a2\n"})
        b = self.repo.commit({"src/a.txt": "unauthorized\n"}, "hand edit on A")
        self.target({"src/b.txt": "b2\n"})
        c = self._refresh(b)
        code, out, _ = self.land(c, None, "--authorized", a)
        self.assertEqual(code, 20, out)
        self.assertIn("not a two-parent refresh merge", out)

    def test_t2_second_parent_not_on_target_rejected(self):
        a = self.pr({"src/a.txt": "a2\n"})
        self.repo.branch("side", self.base)
        self.repo.commit({"src/s.txt": "new code\n"}, "side")
        self.target({"src/b.txt": "b2\n"})
        self.repo.branch("refreshed", a)
        subprocess.run(["git", "-C", self.tmp, "merge", "--no-ff", "-q", "-m", "m", "side"],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        c = self.repo.sha("HEAD")
        code, out, _ = self.land(c, None, "--authorized", a)
        self.assertEqual(code, 20, out)
        self.assertIn("not on the target", out)


class RefreshCheckTests(Base):
    def setUp(self):
        super().setUp()
        self.reviewed = self.pr({"src/a.txt": "a2\n", "CHANGELOG.md": changelog("pr")})
        self.tip = self.target({"src/b.txt": "b2\n"})

    def refresh(self, new, reviewed=None, tip=None):
        return run_script(
            REFRESH, self.repo, "--reviewed", reviewed or self.reviewed,
            "--target-tip", tip or self.tip, "--new-head", new, "--target", "target",
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
        self.assertIn("not the git merge-file --union", out)

    def test_not_pure_changelog_foreign_line(self):
        reviewed = self.pr({"CHANGELOG.md": changelog("pr entry")})
        tip = self.target({"CHANGELOG.md": changelog("target entry")})
        union = changelog("pr entry", "target entry", "sneaky entry")
        new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": union})
        code, out, _ = self.refresh(new, reviewed, tip)
        self.assertEqual(code, 20, out)
        self.assertIn("not the git merge-file --union", out)

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

    def test_not_pure_tip_not_on_target(self):
        self.repo.branch("side", self.base)
        side = self.repo.commit({"src/s.txt": "s\n"}, "side")
        self.repo.branch("refreshed", self.reviewed)
        subprocess.run(["git", "-C", self.tmp, "merge", "--no-ff", "-q", "-m", "m", side],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        new = self.repo.sha("HEAD")
        code, out, _ = self.refresh(new, self.reviewed, side)
        self.assertEqual(code, 20, out)
        self.assertIn("not on the target", out)

    def test_not_pure_single_parent(self):
        self.repo.branch("refreshed", self.reviewed)
        single = self.repo.commit({"src/b.txt": "b2\n"}, "squashed")
        code, out, _ = self.refresh(single)
        self.assertEqual(code, 20, out)
        self.assertIn("1 parents: FAIL", out)

    def test_not_pure_changelog_edit_on_clean_merge(self):
        # C1/C5: CHANGELOG.md merged cleanly, so any difference is a hand edit.
        base_cl = "# Changelog\n## [Unreleased]\n### Added\n  - old entry\n## [1.0.0]\n### Added\n  - released\n"
        self.repo.branch("setup", self.base)
        self.base = self.repo.commit({"CHANGELOG.md": base_cl}, "cl")
        reviewed = self.pr({"CHANGELOG.md": base_cl.replace("  - old entry\n", "  - pr entry\n  - old entry\n")})
        tip = self.target({"src/b.txt": "b3\n"})
        moved = base_cl.replace("  - released\n", "  - released\n  - pr entry\n")
        new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": moved})
        code, out, _ = self.refresh(new, reviewed, tip)
        self.assertEqual(code, 20, out)
        self.assertIn("CHANGELOG.md differs from the clean merge", out)

    def test_not_pure_changelog_permuted_or_duplicated(self):
        # C2/C3: right lines, wrong order or duplicated headings.
        reviewed = self.pr({"CHANGELOG.md": changelog("pr entry")})
        tip = self.target({"CHANGELOG.md": changelog("target entry")})
        for bad in (
            "  - old entry\n### Added\n## [Unreleased]\n# Changelog\n  - pr entry\n  - target entry\n",
            changelog("pr entry", "target entry") + "### Added\n  - pr entry\n",
        ):
            new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": bad})
            code, out, _ = self.refresh(new, reviewed, tip)
            self.assertEqual(code, 20, out)

    def test_not_pure_when_changelog_union_disabled(self):
        reviewed = self.pr({"CHANGELOG.md": changelog("pr entry")})
        tip = self.target({"CHANGELOG.md": changelog("target entry")})
        new = self.merge(reviewed, tip, resolve={"CHANGELOG.md": changelog("pr entry", "target entry")})
        code, out, _ = run_script(
            REFRESH, self.repo, "--reviewed", reviewed, "--target-tip", tip,
            "--new-head", new, "--target", "target", "--no-changelog-union")
        self.assertEqual(code, 20, out)

    def test_not_pure_hand_edit_from_subdirectory(self):
        self.repo.git("config", "diff.relative", "true")
        subprocess.run(
            ["git", "-C", self.repo.path, "merge", "--no-ff", "--no-commit", "-q", self.tip],
            check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.repo.write({"src/a.txt": "sneaky\n"})
        self.repo.git("add", "-A")
        self.repo.git("commit", "-q", "-m", "x")
        edited = self.repo.sha("HEAD")
        proc = subprocess.run(
            [sys.executable, REFRESH, "--repo-dir", os.path.join(self.tmp, "thoth-api"),
             "--reviewed", self.reviewed, "--target-tip", self.tip, "--new-head", edited,
             "--target", "target"],
            stdout=subprocess.PIPE, text=True)
        self.assertEqual(proc.returncode, 20, proc.stdout)


if __name__ == "__main__":
    unittest.main()
