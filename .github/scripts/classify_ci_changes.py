#!/usr/bin/env python3
"""Classify a complete Git change set for GitHub Actions CI gating."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path, PurePosixPath
from typing import Mapping, Sequence

SHA_PATTERN = re.compile(r"^[0-9a-fA-F]{40}$")
ALL_ZERO_SHA = "0" * 40

BUILD_CONTROL_PATHS = {
    ".github/scripts/classify_ci_changes.py",
    ".github/workflows/build_test_and_check.yml",
    ".github/workflows/build_test_and_check_no_action.yml",
}
MIGRATION_CONTROL_PATHS = {
    ".github/scripts/classify_ci_changes.py",
    ".github/workflows/run_migrations.yml",
    ".github/workflows/run_migrations_no_action.yml",
    # THOTH-DB-CTRL-01 Diesel schema control surfaces. A change to any of these
    # must run the migration-control verification and must never be treated as
    # documentation-only.
    "AGENTS.md",
    "thoth-api/AGENTS.md",
    "diesel.toml",
    "Makefile",
    ".github/scripts/diesel_schema.py",
    ".github/scripts/test_diesel_schema.py",
    "thoth-api/diesel-schema-control.toml",
    "thoth-api/src/schema.rs",
}


class ClassificationError(RuntimeError):
    """Raised when a change set cannot be classified reliably."""


@dataclass(frozen=True)
class Classification:
    docs_only: bool
    run_build: bool
    run_migrations: bool
    run_docker: bool

    @classmethod
    def heavy(cls) -> "Classification":
        return cls(
            docs_only=False,
            run_build=True,
            run_migrations=True,
            run_docker=True,
        )

    def as_outputs(self) -> dict[str, str]:
        return {
            key: str(value).lower()
            for key, value in asdict(self).items()
        }


def normalize_path(raw_path: str) -> str:
    """Validate and normalize a repository-relative Git path."""
    if not raw_path or "\x00" in raw_path or "\\" in raw_path:
        raise ClassificationError(f"invalid changed path: {raw_path!r}")

    path = PurePosixPath(raw_path)
    if path.is_absolute() or ".." in path.parts or "." in path.parts:
        raise ClassificationError(f"unsafe changed path: {raw_path!r}")

    normalized = path.as_posix()
    if normalized in {"", "."}:
        raise ClassificationError(f"invalid changed path: {raw_path!r}")
    return normalized


def is_documentation_path(path: str) -> bool:
    return path == "CHANGELOG.md" or path.startswith("docs/")


def is_build_path(path: str) -> bool:
    return (
        path in BUILD_CONTROL_PATHS
        or path == "Cargo.lock"
        or path.endswith("Cargo.toml")
        or path == "diesel.toml"
        or path.endswith((".rs", ".js", ".json", ".html"))
    )


def is_migration_path(path: str) -> bool:
    return (
        path in MIGRATION_CONTROL_PATHS
        or path.startswith("src/bin/")
        or path.endswith(("up.sql", "down.sql", "db.rs"))
    )


def classify_paths(raw_paths: Sequence[str]) -> Classification:
    """Classify a complete, non-empty changed-file set."""
    if not raw_paths:
        raise ClassificationError("changed-file set is empty")

    paths = tuple(normalize_path(path) for path in raw_paths)
    docs_only = all(is_documentation_path(path) for path in paths)
    if docs_only:
        return Classification(
            docs_only=True,
            run_build=False,
            run_migrations=False,
            run_docker=False,
        )

    return Classification(
        docs_only=False,
        run_build=any(is_build_path(path) for path in paths),
        run_migrations=any(is_migration_path(path) for path in paths),
        run_docker=True,
    )


def validate_sha(value: object, label: str) -> str:
    if not isinstance(value, str) or not SHA_PATTERN.fullmatch(value):
        raise ClassificationError(f"{label} is not a full Git SHA")
    return value.lower()


def changed_paths(
    base_sha: str,
    head_sha: str,
    cwd: Path | None = None,
    *,
    merge_base: bool = False,
) -> list[str]:
    """Return all paths changed between two complete Git trees."""
    base = validate_sha(base_sha, "base SHA")
    head = validate_sha(head_sha, "head SHA")
    if base == ALL_ZERO_SHA or head == ALL_ZERO_SHA:
        raise ClassificationError("an all-zero Git SHA cannot define a change range")

    comparison = f"{base}...{head}" if merge_base else f"{base}..{head}"
    try:
        result = subprocess.run(
            [
                "git",
                "diff",
                "--name-only",
                "--no-renames",
                "-z",
                comparison,
                "--",
            ],
            cwd=cwd,
            check=True,
            capture_output=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        diagnostic = getattr(error, "stderr", b"")
        detail = diagnostic.decode("utf-8", "replace").strip()
        raise ClassificationError(
            f"unable to calculate complete change range: {detail or error}"
        ) from error

    paths = [
        entry.decode("utf-8", "surrogateescape")
        for entry in result.stdout.split(b"\x00")
        if entry
    ]
    if not paths:
        raise ClassificationError("Git change range produced an empty file set")
    return paths


def load_event(path: str) -> Mapping[str, object]:
    if not path:
        raise ClassificationError("GITHUB_EVENT_PATH is not set")
    try:
        with Path(path).open(encoding="utf-8") as event_file:
            event = json.load(event_file)
    except (OSError, json.JSONDecodeError) as error:
        raise ClassificationError(f"unable to read GitHub event: {error}") from error
    if not isinstance(event, dict):
        raise ClassificationError("GitHub event payload is not an object")
    return event


def classify_event(
    event_name: str,
    event: Mapping[str, object],
    github_sha: str | None,
    cwd: Path | None = None,
) -> Classification:
    """Classify a supported GitHub Actions event."""
    if event_name == "workflow_dispatch":
        return Classification.heavy()

    if event_name == "pull_request":
        pull_request = event.get("pull_request")
        if not isinstance(pull_request, dict):
            raise ClassificationError("pull_request payload is missing")
        base = pull_request.get("base")
        head = pull_request.get("head")
        if not isinstance(base, dict) or not isinstance(head, dict):
            raise ClassificationError("pull_request base or head is missing")
        paths = changed_paths(
            validate_sha(base.get("sha"), "pull-request base SHA"),
            validate_sha(head.get("sha"), "pull-request head SHA"),
            cwd=cwd,
            merge_base=True,
        )
        return classify_paths(paths)

    if event_name == "push":
        before = validate_sha(event.get("before"), "push before SHA")
        head_value = github_sha or event.get("after")
        head = validate_sha(head_value, "push head SHA")
        paths = changed_paths(before, head, cwd=cwd)
        return classify_paths(paths)

    raise ClassificationError(f"unsupported GitHub event: {event_name or '<empty>'}")


def emit_outputs(
    classification: Classification,
    output_path: str | None,
) -> None:
    outputs = classification.as_outputs()
    if output_path:
        with Path(output_path).open("a", encoding="utf-8") as output_file:
            for key, value in outputs.items():
                output_file.write(f"{key}={value}\n")
    print(json.dumps(outputs, sort_keys=True))


def git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def run_self_tests() -> None:
    cases = [
        (
            "documentation_only",
            ["docs/engineering/example.md", "docs/publisher-services/README.md"],
            Classification(True, False, False, False),
        ),
        (
            "changelog_only",
            ["CHANGELOG.md"],
            Classification(True, False, False, False),
        ),
        (
            "mixed_docs_and_rust",
            ["docs/example.md", "thoth-api/src/lib.rs"],
            Classification(False, True, False, True),
        ),
        (
            "migration_only",
            ["thoth-api/migrations/example/up.sql"],
            Classification(False, False, True, True),
        ),
        (
            "dockerfile",
            ["Dockerfile"],
            Classification(False, False, False, True),
        ),
        (
            "workflow_change",
            [".github/workflows/build_test_and_check.yml"],
            Classification(False, True, False, True),
        ),
        (
            "classifier_change",
            [".github/scripts/classify_ci_changes.py"],
            Classification(False, True, True, True),
        ),
        (
            "deleted_build_no_action_workflow",
            [".github/workflows/build_test_and_check_no_action.yml"],
            Classification(False, True, False, True),
        ),
        (
            "deleted_migration_no_action_workflow",
            [".github/workflows/run_migrations_no_action.yml"],
            Classification(False, False, True, True),
        ),
        (
            "root_readme",
            ["README.md"],
            Classification(False, False, False, True),
        ),
        # THOTH-DB-CTRL-01 migration-control surfaces must run migration
        # verification and must never be documentation-only.
        (
            "diesel_synchronizer",
            [".github/scripts/diesel_schema.py"],
            Classification(False, False, True, True),
        ),
        (
            "diesel_synchronizer_tests",
            [".github/scripts/test_diesel_schema.py"],
            Classification(False, False, True, True),
        ),
        (
            "diesel_convention_file",
            ["thoth-api/diesel-schema-control.toml"],
            Classification(False, False, True, True),
        ),
        (
            "diesel_config",
            ["diesel.toml"],
            Classification(False, True, True, True),
        ),
        (
            "makefile_control",
            ["Makefile"],
            Classification(False, False, True, True),
        ),
        (
            "root_agents",
            ["AGENTS.md"],
            Classification(False, False, True, True),
        ),
        (
            "api_agents",
            ["thoth-api/AGENTS.md"],
            Classification(False, False, True, True),
        ),
        (
            "canonical_schema",
            ["thoth-api/src/schema.rs"],
            Classification(False, True, True, True),
        ),
        (
            "agents_not_docs_only",
            ["AGENTS.md", "docs/example.md"],
            Classification(False, False, True, True),
        ),
    ]

    for name, paths, expected in cases:
        actual = classify_paths(paths)
        if actual != expected:
            raise AssertionError(f"{name}: expected {expected}, got {actual}")
        print(f"PASS {name}: {json.dumps(actual.as_outputs(), sort_keys=True)}")

    manual = classify_event("workflow_dispatch", {}, None)
    if manual != Classification.heavy():
        raise AssertionError(f"manual_dispatch: expected heavy, got {manual}")
    print(
        "PASS manual_dispatch: "
        f"{json.dumps(manual.as_outputs(), sort_keys=True)}"
    )

    try:
        classify_paths([])
    except ClassificationError:
        empty_result = Classification.heavy()
    else:
        raise AssertionError("empty_range: empty paths did not fail closed")
    print(
        "PASS empty_range_fail_closed: "
        f"{json.dumps(empty_result.as_outputs(), sort_keys=True)}"
    )

    try:
        changed_paths(ALL_ZERO_SHA, ALL_ZERO_SHA)
    except ClassificationError:
        invalid_range_result = Classification.heavy()
    else:
        raise AssertionError("invalid_range: all-zero range did not fail closed")
    print(
        "PASS invalid_range_fail_closed: "
        f"{json.dumps(invalid_range_result.as_outputs(), sort_keys=True)}"
    )

    with tempfile.TemporaryDirectory(prefix="ci-docs-classifier-") as temp_dir:
        repo = Path(temp_dir)
        git(repo, "init", "-q")
        git(repo, "config", "user.name", "CI classifier self-test")
        git(repo, "config", "user.email", "ci-classifier@example.invalid")

        (repo / "README.md").write_text("base\n", encoding="utf-8")
        git(repo, "add", "README.md")
        git(repo, "commit", "-qm", "base")
        base_sha = git(repo, "rev-parse", "HEAD")

        source = repo / "thoth-api" / "src"
        source.mkdir(parents=True)
        (source / "lib.rs").write_text("pub fn example() {}\n", encoding="utf-8")
        git(repo, "add", "thoth-api/src/lib.rs")
        git(repo, "commit", "-qm", "source")

        docs = repo / "docs"
        docs.mkdir()
        (docs / "example.md").write_text("# Example\n", encoding="utf-8")
        git(repo, "add", "docs/example.md")
        git(repo, "commit", "-qm", "docs")
        head_sha = git(repo, "rev-parse", "HEAD")

        paths = changed_paths(base_sha, head_sha, cwd=repo, merge_base=True)
        full_range = classify_paths(paths)
        expected = Classification(False, True, False, True)
        if full_range != expected:
            raise AssertionError(
                f"full_pr_diff: expected {expected}, got {full_range}"
            )
        print(
            "PASS full_pr_diff: "
            f"paths={json.dumps(paths)} "
            f"outputs={json.dumps(full_range.as_outputs(), sort_keys=True)}"
        )

    print(f"PASS all_self_tests: {len(cases) + 4} cases")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--self-test",
        action="store_true",
        help="run deterministic classifier and full-range tests",
    )
    mode.add_argument(
        "--paths",
        nargs="+",
        help="classify an explicit complete changed-file set",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        run_self_tests()
        return 0

    if args.paths is not None:
        try:
            classification = classify_paths(args.paths)
        except ClassificationError as error:
            print(f"FAIL CLOSED: {error}", file=sys.stderr)
            classification = Classification.heavy()
        emit_outputs(classification, None)
        return 0

    try:
        event = load_event(os.environ.get("GITHUB_EVENT_PATH", ""))
        classification = classify_event(
            os.environ.get("GITHUB_EVENT_NAME", ""),
            event,
            os.environ.get("GITHUB_SHA"),
        )
    except ClassificationError as error:
        print(f"FAIL CLOSED: {error}", file=sys.stderr)
        classification = Classification.heavy()

    emit_outputs(classification, os.environ.get("GITHUB_OUTPUT"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
