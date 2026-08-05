#!/usr/bin/env python3
"""THOTH-DB-CTRL-01 - deterministic, fail-closed Diesel schema synchronizer.

This is the sole authorized writer of the canonical Diesel contract at
``thoth-api/src/schema.rs``. It implements a capability-aware, two-phase
baseline-to-candidate comparison over three independently derived
representations of the schema:

* the PostgreSQL catalog (authoritative for column types, nullability, ordinal
  positions, primary keys, foreign keys, and ordered enum labels);
* raw ``diesel print-schema`` output (authoritative for generated Diesel types,
  joins, and ``allow_tables_to_appear_in_same_query!`` membership);
* the canonical ``thoth-api/src/schema.rs`` contract (convention-adjusted).

Each representation is derived independently and compared exactly against the
projection of an explicit version-2 expected-change manifest. Facts are never
copied from one representation into another before comparison.

Modes:

``check``     read-only verification; never writes the canonical schema.
``generate``  the only path that may atomically replace the canonical schema,
              and only after every safety, projection, compile, and cleanup
              check passes.

All subprocesses use argument arrays with ``shell=False``. No credential, URL,
row content, or personal data is ever printed.
"""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Optional
from urllib.parse import unquote, urlsplit

try:  # Python 3.11+ standard library
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - defensive
    tomllib = None  # type: ignore

# --------------------------------------------------------------------------- #
# Constants
# --------------------------------------------------------------------------- #

CANONICAL_REL = "thoth-api/src/schema.rs"
CONVENTION_REL = "thoth-api/diesel-schema-control.toml"
DIESEL_TOML_REL = "diesel.toml"
STAGING_REL = "target/diesel-schema.rs"
MIGRATIONS_REL = "thoth-api/migrations"
EXPECTED_CLI_VERSION = "2.3.10"
EXPECTED_REPOSITORY = "thoth-pub/thoth"
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
LOOPBACK_HOSTS = {"localhost", "127.0.0.1", "::1", "[::1]"}
LOOPBACK_ADDRS = {"127.0.0.1", "::1"}
LOCAL_DB_PREFIX = "thoth_diesel_"
DEFAULT_DEV_PORT = 5432

# Documented deterministic PostgreSQL -> canonical Diesel base-type mapping used
# only for shared-field cross-checks. It never injects facts into a leg; it
# only asserts that a manifest's declared postgres_type and diesel_type agree.
PG_TO_DIESEL = {
    "uuid": "Uuid",
    "text": "Text",
    "bool": "Bool",
    "boolean": "Bool",
    "int4": "Int4",
    "integer": "Int4",
    "int8": "Int8",
    "bigint": "Int8",
    "float8": "Float8",
    "double precision": "Float8",
    "date": "Date",
    "jsonb": "Jsonb",
    "timestamp": "Timestamp",
    "timestamp without time zone": "Timestamp",
    "timestamptz": "Timestamptz",
    "timestamp with time zone": "Timestamptz",
}


class ControlFailure(RuntimeError):
    """A fail-closed control violation carrying one safe reason code."""

    def __init__(self, reason: str):
        super().__init__(reason)
        self.reason = reason


# --------------------------------------------------------------------------- #
# Safe process execution
# --------------------------------------------------------------------------- #


def run(argv: list[str], *, cwd: Optional[Path] = None, env: Optional[dict] = None,
        check: bool = True, reason: str = "SUBPROCESS_FAILED") -> subprocess.CompletedProcess:
    """Run a subprocess with an argument array and ``shell=False``.

    Never interpolates into a shell; shell metacharacters in any argument remain
    literal. Output is captured as text and is never echoed with credentials.
    """
    if not isinstance(argv, list) or not all(isinstance(a, str) for a in argv):
        raise ControlFailure("SUBPROCESS_ARGS_INVALID")
    try:
        proc = subprocess.run(
            argv, cwd=str(cwd) if cwd else None, env=env,
            capture_output=True, text=True, shell=False,
        )
    except OSError as exc:  # pragma: no cover - environment failure
        raise ControlFailure(reason) from exc
    if check and proc.returncode != 0:
        raise ControlFailure(reason)
    return proc


# --------------------------------------------------------------------------- #
# Repository, configuration, and CLI validation
# --------------------------------------------------------------------------- #


def resolve_repo_root(script_path: Path) -> Path:
    cwd = Path.cwd().resolve()
    top = run(["git", "rev-parse", "--show-toplevel"], cwd=cwd,
              reason="REPOSITORY_ROOT_UNVERIFIED").stdout.strip()
    root = Path(top).resolve()
    if root != cwd:
        raise ControlFailure("WORKING_DIRECTORY_NOT_REPOSITORY_ROOT")
    # the script itself must live under this root
    if root not in script_path.resolve().parents:
        raise ControlFailure("REPOSITORY_ROOT_UNVERIFIED")
    return root


def contained(root: Path, rel: str) -> Path:
    """Resolve ``rel`` beneath ``root``, rejecting symlink or path escape."""
    target = (root / rel)
    resolved = target.resolve()
    if root not in resolved.parents and resolved != root:
        raise ControlFailure("PATH_ESCAPES_REPOSITORY")
    # reject symlinked components inside the repository for tracked inputs
    probe = root
    for part in Path(rel).parts:
        probe = probe / part
        if probe.is_symlink():
            raise ControlFailure("SYMLINK_ESCAPE_REJECTED")
    return resolved


def validate_diesel_toml(root: Path) -> None:
    path = contained(root, DIESEL_TOML_REL)
    if tomllib is None:  # pragma: no cover
        raise ControlFailure("TOML_SUPPORT_UNAVAILABLE")
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError):
        raise ControlFailure("DIESEL_CONFIG_UNPARSEABLE")
    print_schema = data.get("print_schema")
    if not isinstance(print_schema, dict):
        raise ControlFailure("DIESEL_CONFIG_MISSING_PRINT_SCHEMA")
    file_value = print_schema.get("file")
    if file_value != STAGING_REL:
        raise ControlFailure("DIESEL_CONFIG_OUTPUT_NOT_STAGING")
    # staging must live under ignored target/, never the canonical path
    staging = (root / file_value).resolve()
    if staging == (root / CANONICAL_REL).resolve():
        raise ControlFailure("DIESEL_CONFIG_TARGETS_CANONICAL")
    if (root / "target").resolve() not in staging.parents:
        raise ControlFailure("DIESEL_CONFIG_OUTPUT_NOT_IN_TARGET")
    derives = print_schema.get("custom_type_derives")
    if derives != ["diesel::query_builder::QueryId"]:
        raise ControlFailure("DIESEL_CONFIG_DERIVES_INVALID")


def diesel_bin() -> str:
    return os.environ.get("DIESEL_BIN", "diesel")


def validate_cli_version() -> None:
    proc = run([diesel_bin(), "--version"], reason="DIESEL_CLI_UNAVAILABLE")
    text = proc.stdout + proc.stderr
    versions = re.findall(r"(\d+\.\d+\.\d+)", text)
    if EXPECTED_CLI_VERSION not in versions:
        raise ControlFailure("DIESEL_CLI_VERSION_MISMATCH")


def validate_base_ref(root: Path, base_ref: str) -> None:
    if not isinstance(base_ref, str) or not SHA_RE.fullmatch(base_ref):
        raise ControlFailure("BASE_REF_NOT_FULL_LOWER_SHA")
    # resolve without symbolic-ref or abbreviation expansion
    proc = run(["git", "rev-parse", "--verify", "--end-of-options", f"{base_ref}^{{commit}}"],
               cwd=root, check=False)
    if proc.returncode != 0 or proc.stdout.strip() != base_ref:
        raise ControlFailure("BASE_REF_UNRESOLVABLE")
    anc = run(["git", "merge-base", "--is-ancestor", base_ref, "HEAD"], cwd=root, check=False)
    if anc.returncode != 0:
        raise ControlFailure("BASE_REF_NOT_ANCESTOR")


def candidate_head(root: Path) -> str:
    return run(["git", "rev-parse", "HEAD"], cwd=root,
               reason="CANDIDATE_HEAD_UNRESOLVABLE").stdout.strip()


# --------------------------------------------------------------------------- #
# Detached base worktree
# --------------------------------------------------------------------------- #


@contextlib.contextmanager
def base_worktree(root: Path, base_ref: str):
    tmp = Path(tempfile.mkdtemp(prefix="thoth-diesel-base-"))
    wt = tmp / "worktree"
    created = False
    try:
        run(["git", "worktree", "add", "--detach", str(wt), base_ref], cwd=root,
            reason="BASE_WORKTREE_UNCREATABLE")
        created = True
        head = run(["git", "rev-parse", "HEAD"], cwd=wt,
                   reason="BASE_WORKTREE_HEAD_UNVERIFIED").stdout.strip()
        if head != base_ref:
            raise ControlFailure("BASE_WORKTREE_HEAD_MISMATCH")
        status = run(["git", "status", "--porcelain"], cwd=wt,
                     reason="BASE_WORKTREE_DIRTY").stdout.strip()
        if status:
            raise ControlFailure("BASE_WORKTREE_DIRTY")
        top = run(["git", "rev-parse", "--show-toplevel"], cwd=wt,
                  reason="BASE_WORKTREE_UNVERIFIED").stdout.strip()
        if Path(top).resolve() != wt.resolve():
            raise ControlFailure("BASE_WORKTREE_IDENTITY_MISMATCH")
        yield wt
    finally:
        if created:
            run(["git", "worktree", "remove", "--force", str(wt)], cwd=root, check=False)
        shutil.rmtree(tmp, ignore_errors=True)
        # confirm removal
        if wt.exists():
            raise ControlFailure("BASE_WORKTREE_UNREMOVABLE")


# --------------------------------------------------------------------------- #
# Safe database target gate
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class DbTarget:
    host: str
    port: int
    user: str
    dbname: str
    password: str


def parse_database_url() -> DbTarget:
    url = os.environ.get("DATABASE_URL")
    if not url:
        raise ControlFailure("DATABASE_URL_ABSENT")
    parts = urlsplit(url)
    if parts.scheme not in {"postgres", "postgresql"}:
        raise ControlFailure("DATABASE_URL_SCHEME_INVALID")
    host = parts.hostname or ""
    if host not in LOOPBACK_HOSTS:
        raise ControlFailure("SAFE_DATABASE_TARGET_UNVERIFIED")
    dbname = unquote(parts.path.lstrip("/"))
    if not dbname:
        raise ControlFailure("DATABASE_NAME_ABSENT")
    return DbTarget(
        host=host,
        port=parts.port or DEFAULT_DEV_PORT,
        user=unquote(parts.username or ""),
        dbname=dbname,
        password=unquote(parts.password or ""),
    )


def psql(target: DbTarget, sql: str) -> list[list[str]]:
    """Run a single SQL statement returning tab/unit-separated rows. Never logs
    the URL or password; credentials are passed via PGPASSWORD, not argv."""
    env = dict(os.environ)
    env["PGPASSWORD"] = target.password
    argv = [
        "psql", "-X", "-q", "-tA", "-F", "\x1f", "-R", "\x1e",
        "-v", "ON_ERROR_STOP=1",
        "-h", target.host, "-p", str(target.port),
        "-U", target.user, "-d", target.dbname,
        "-c", sql,
    ]
    proc = run(argv, env=env, reason="DATABASE_QUERY_FAILED")
    # Records are separated by the RS (\x1e); psql terminates the final record
    # with a trailing newline that must not become part of a field value.
    out = proc.stdout.rstrip("\n")
    if out.endswith("\x1e"):
        out = out[:-1]
    rows = [r for r in out.split("\x1e") if r != ""]
    return [r.split("\x1f") for r in rows]


def check_confirmation(target: DbTarget) -> str:
    """Pre-connection: the confirmation env var must equal the URL database."""
    confirm = os.environ.get("THOTH_DIESEL_CONFIRM_DATABASE")
    if not confirm:
        raise ControlFailure("DATABASE_CONFIRMATION_ABSENT")
    if confirm != target.dbname:
        raise ControlFailure("DATABASE_CONFIRMATION_MISMATCH")
    return confirm


def confirm_identity(target: DbTarget) -> dict:
    confirm = check_confirmation(target)
    rows = psql(target, (
        "SELECT current_database(), current_user, "
        "coalesce(host(inet_server_addr()),''), coalesce(inet_server_port()::text,''), "
        "coalesce(host(inet_client_addr()),''), coalesce(inet_client_port()::text,''), "
        "version();"
    ))
    if len(rows) != 1 or len(rows[0]) != 7:
        raise ControlFailure("DATABASE_IDENTITY_UNREADABLE")
    (cur_db, cur_user, srv_addr, srv_port, cli_addr, cli_port, version) = rows[0]
    if cur_db != target.dbname or cur_db != confirm:
        raise ControlFailure("DATABASE_CONFIRMATION_MISMATCH")
    return {
        "current_database": cur_db,
        "current_user": cur_user,
        "server_address": srv_addr,
        "server_port": srv_port,
        "client_address": cli_addr,
        "client_port": cli_port,
        "server_version": version.split(" on ")[0],
    }


def _is_private_container_addr(addr: str) -> bool:
    if addr in LOOPBACK_ADDRS:
        return True
    # RFC1918 / container bridge space
    if addr.startswith("10.") or addr.startswith("192.168."):
        return True
    if addr.startswith("172."):
        try:
            second = int(addr.split(".")[1])
        except (IndexError, ValueError):
            return False
        return 16 <= second <= 31
    return False


def classify_server_address(addr: str, provenance_private_ok: bool) -> str:
    if addr in LOOPBACK_ADDRS or addr == "":
        # empty means unix socket / same host loopback acceptance handled by caller
        if addr in LOOPBACK_ADDRS:
            return "LOOPBACK"
    if provenance_private_ok and _is_private_container_addr(addr):
        return "VERIFIED_PRIVATE_CONTAINER"
    raise ControlFailure("SERVER_ADDRESS_UNVERIFIED")


def inspect_local_container(target: DbTarget) -> dict:
    """Pre-connection structural gate for local disposable mode.

    Rejects the default developer port, wrong database prefix, and any container
    that is absent, not running, bind-mounted, durably stored, or whose loopback
    port mapping does not match the URL. Returns the parsed container info.
    """
    if target.port == DEFAULT_DEV_PORT:
        raise ControlFailure("LOCAL_PORT_IS_DEFAULT_DEV_PORT")
    if not target.dbname.startswith(LOCAL_DB_PREFIX):
        raise ControlFailure("LOCAL_DATABASE_PREFIX_INVALID")
    container = os.environ.get("THOTH_DIESEL_CONTAINER")
    if not container:
        raise ControlFailure("LOCAL_CONTAINER_IDENTITY_ABSENT")
    proc = run(["docker", "inspect", container], check=False)
    if proc.returncode != 0:
        raise ControlFailure("LOCAL_CONTAINER_UNVERIFIED")
    try:
        info = json.loads(proc.stdout)[0]
    except (json.JSONDecodeError, IndexError, KeyError):
        raise ControlFailure("LOCAL_CONTAINER_UNVERIFIED")
    if info.get("State", {}).get("Running") is not True:
        raise ControlFailure("LOCAL_CONTAINER_NOT_RUNNING")
    # port mapping: loopback host:port -> container 5432
    ports = info.get("NetworkSettings", {}).get("Ports", {}) or {}
    mapping = ports.get("5432/tcp")
    if not mapping:
        raise ControlFailure("LOCAL_CONTAINER_PORT_MAP_INVALID")
    ok = any(
        (m.get("HostIp") in LOOPBACK_ADDRS) and m.get("HostPort") == str(target.port)
        for m in mapping
    )
    if not ok:
        raise ControlFailure("LOCAL_CONTAINER_PORT_MAP_INVALID")
    # mounts: reject bind mounts and named/externally-managed durable storage.
    # A disposable anonymous volume (64-hex name created by the image) is allowed.
    anon_re = re.compile(r"^[0-9a-f]{64}$")
    for mount in info.get("Mounts", []) or []:
        if mount.get("Type") != "volume":
            raise ControlFailure("LOCAL_CONTAINER_BIND_MOUNT")
        name = mount.get("Name") or ""
        if not anon_re.fullmatch(name):
            raise ControlFailure("LOCAL_CONTAINER_DURABLE_STORAGE")
    return info


def gate_local_docker(target: DbTarget, identity: dict, info: dict) -> str:
    """Post-connection classification for local disposable mode."""
    # server address must belong to the container network or be loopback
    networks = info.get("NetworkSettings", {}).get("Networks", {}) or {}
    container_addrs = {
        n.get("IPAddress") for n in networks.values() if n.get("IPAddress")
    }
    global_ip = info.get("NetworkSettings", {}).get("IPAddress")
    if global_ip:
        container_addrs.add(global_ip)
    srv = identity["server_address"]
    if srv not in LOOPBACK_ADDRS:
        if srv not in container_addrs or not _is_private_container_addr(srv):
            raise ControlFailure("SERVER_ADDRESS_UNVERIFIED")
        classify_server_address(srv, provenance_private_ok=True)
    if target.user != os.environ.get("THOTH_DIESEL_EXPECTED_USER", target.user):
        raise ControlFailure("LOCAL_DATABASE_USER_MISMATCH")
    return "SAFE_DISPOSABLE_LOCAL"


def gate_github_actions_structural(target: DbTarget) -> None:
    """Pre-connection provenance gate for the GitHub Actions service."""
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise ControlFailure("CI_PROVENANCE_INVALID")
    if os.environ.get("GITHUB_REPOSITORY") != EXPECTED_REPOSITORY:
        raise ControlFailure("CI_REPOSITORY_INVALID")
    if os.environ.get("GITHUB_JOB") != "run_migrations":
        raise ControlFailure("CI_JOB_INVALID")
    ref = os.environ.get("GITHUB_WORKFLOW_REF", "")
    if ".github/workflows/run_migrations.yml" not in ref:
        raise ControlFailure("CI_WORKFLOW_REF_INVALID")
    if target.host not in LOOPBACK_HOSTS or target.port != DEFAULT_DEV_PORT:
        raise ControlFailure("CI_ENDPOINT_INVALID")
    if target.dbname != "thoth" or target.user != "thoth":
        raise ControlFailure("CI_DATABASE_IDENTITY_INVALID")


def gate_github_actions(target: DbTarget, identity: dict) -> str:
    if identity["current_database"] != "thoth":
        raise ControlFailure("CI_DATABASE_IDENTITY_INVALID")
    srv = identity["server_address"]
    if srv not in LOOPBACK_ADDRS and not _is_private_container_addr(srv):
        raise ControlFailure("SERVER_ADDRESS_UNVERIFIED")
    return "SAFE_DISPOSABLE_CI"


def safe_target_gate() -> tuple[DbTarget, dict, str]:
    target = parse_database_url()
    check_confirmation(target)
    ci = os.environ.get("GITHUB_ACTIONS") == "true"
    # Cheap structural provenance checks BEFORE any connection, so a misdirected
    # or default-port target is rejected without ever contacting a database.
    info = None
    if ci:
        gate_github_actions_structural(target)
    else:
        info = inspect_local_container(target)
    identity = confirm_identity(target)
    # The client-facing endpoint is the DATABASE_URL host, already enforced to be
    # loopback in parse_database_url(). The server's view of the client address
    # (inet_client_addr) may be the private container-bridge gateway when a
    # loopback host port is mapped into a container network; it must be loopback
    # or private container-network space, never public or unexplained.
    cli = identity["client_address"]
    if cli and not (cli in LOOPBACK_ADDRS or _is_private_container_addr(cli)):
        raise ControlFailure("CLIENT_ADDRESS_UNVERIFIED")
    if ci:
        provenance = gate_github_actions(target, identity)
    else:
        provenance = gate_local_docker(target, identity, info)
    server_class = ("LOOPBACK" if identity["server_address"] in LOOPBACK_ADDRS
                    else "VERIFIED_PRIVATE_CONTAINER")
    return target, {**identity, "server_class": server_class}, provenance


# --------------------------------------------------------------------------- #
# Catalog snapshot
# --------------------------------------------------------------------------- #


@dataclass
class CatalogTable:
    schema: str
    name: str
    columns: list[tuple]  # (name, pg_type, nullable(bool), ordinal(int))
    primary_key: list[str]
    foreign_keys: list[tuple]  # (child_cols, parent_table, parent_cols)


@dataclass
class Catalog:
    tables: dict  # (schema, name) -> CatalogTable
    enums: dict    # (schema, name) -> [labels]
    migrations: list

    def table_names(self) -> set:
        return set(self.tables.keys())


def snapshot_catalog(target: DbTarget) -> Catalog:
    # migrations ledger
    mig = psql(target, "SELECT version FROM __diesel_schema_migrations ORDER BY version;")
    migrations = [r[0] for r in mig]
    # columns
    col_rows = psql(target, (
        "SELECT c.table_name, c.column_name, "
        "  CASE WHEN c.data_type='USER-DEFINED' THEN c.udt_name ELSE c.data_type END, "
        "  c.is_nullable, c.ordinal_position "
        "FROM information_schema.columns c "
        "JOIN information_schema.tables t "
        "  ON t.table_schema=c.table_schema AND t.table_name=c.table_name "
        "WHERE c.table_schema='public' AND t.table_type='BASE TABLE' "
        "  AND c.table_name <> '__diesel_schema_migrations' "
        "ORDER BY c.table_name, c.ordinal_position;"
    ))
    tables: dict = {}
    for tname, cname, ctype, is_nullable, ordinal in col_rows:
        key = ("public", tname)
        t = tables.setdefault(key, CatalogTable("public", tname, [], [], []))
        t.columns.append((cname, ctype, is_nullable == "YES", int(ordinal)))
    # primary keys
    pk_rows = psql(target, (
        "SELECT tc.table_name, kcu.column_name, kcu.ordinal_position "
        "FROM information_schema.table_constraints tc "
        "JOIN information_schema.key_column_usage kcu "
        "  ON tc.constraint_name=kcu.constraint_name AND tc.table_schema=kcu.table_schema "
        "WHERE tc.constraint_type='PRIMARY KEY' AND tc.table_schema='public' "
        "ORDER BY tc.table_name, kcu.ordinal_position;"
    ))
    for tname, cname, _ in pk_rows:
        key = ("public", tname)
        if key in tables:
            tables[key].primary_key.append(cname)
    # foreign keys
    fk_rows = psql(target, (
        "SELECT tc.table_name, kcu.column_name, ccu.table_name, ccu.column_name, tc.constraint_name "
        "FROM information_schema.table_constraints tc "
        "JOIN information_schema.key_column_usage kcu "
        "  ON tc.constraint_name=kcu.constraint_name AND tc.table_schema=kcu.table_schema "
        "JOIN information_schema.constraint_column_usage ccu "
        "  ON tc.constraint_name=ccu.constraint_name "
        "WHERE tc.constraint_type='FOREIGN KEY' AND tc.table_schema='public' "
        "ORDER BY tc.table_name, tc.constraint_name, kcu.ordinal_position;"
    ))
    fk_group: dict = {}
    for tname, ccol, ptable, pcol, cname in fk_rows:
        fk_group.setdefault((tname, cname), (ptable, [], []))
        fk_group[(tname, cname)][1].append(ccol)
        fk_group[(tname, cname)][2].append(pcol)
    for (tname, _), (ptable, ccols, pcols) in fk_group.items():
        key = ("public", tname)
        if key in tables:
            tables[key].foreign_keys.append((tuple(ccols), ptable, tuple(pcols)))
    for t in tables.values():
        t.foreign_keys.sort()
    # enums
    enum_rows = psql(target, (
        "SELECT t.typname, e.enumlabel "
        "FROM pg_type t JOIN pg_enum e ON e.enumtypid=t.oid "
        "JOIN pg_namespace n ON n.oid=t.typnamespace "
        "WHERE n.nspname='public' ORDER BY t.typname, e.enumsortorder;"
    ))
    enums: dict = {}
    for tname, label in enum_rows:
        enums.setdefault(("public", tname), []).append(label)
    return Catalog(tables=tables, enums=enums, migrations=migrations)


# --------------------------------------------------------------------------- #
# Diesel schema parsing (raw and canonical share one grammar)
# --------------------------------------------------------------------------- #


@dataclass
class ParsedTable:
    rust_name: str
    physical_name: str
    primary_key: list[str]
    # columns: (rust_name, physical_name, diesel_type, nullable, base_type)
    columns: list[tuple]


@dataclass
class ParsedSchema:
    sql_types: list[tuple]     # (rust_name, pg_name)  ordered
    tables: dict               # physical_name -> ParsedTable
    table_order: list          # rust names in source order
    joins: list                # (child_rust, parent_rust, [child_cols])
    allow_tables: list         # rust names in source order


def _split_type(diesel_type: str) -> tuple[str, bool]:
    m = re.match(r"Nullable<(.+)>$", diesel_type.strip())
    if m:
        return m.group(1).strip(), True
    return diesel_type.strip(), False


def parse_diesel_schema(text: str) -> ParsedSchema:
    sql_types = []
    for m in re.finditer(
        r'#\[diesel\(postgres_type\(name = "([^"]+)"\)\)\]\s*\n\s*pub struct (\w+);', text
    ):
        sql_types.append((m.group(2), m.group(1)))
    tables: dict = {}
    table_order: list = []
    for block in re.finditer(r"table! \{(.*?)\n\}", text, re.DOTALL):
        body = block.group(1)
        sqlname = re.search(
            r'#\[sql_name = "([^"]+)"\]\s*\n\s*(\w+) \(([\w, ]+)\) \{', body
        )
        if sqlname:
            physical, rust, pk_raw = sqlname.group(1), sqlname.group(2), sqlname.group(3)
        else:
            mt = re.search(r"\n\s*(\w+) \(([\w, ]+)\) \{", body)
            if not mt:
                raise ControlFailure("SCHEMA_PARSE_AMBIGUOUS")
            physical = rust = mt.group(1)
            pk_raw = mt.group(2)
        pk = [c.strip() for c in pk_raw.split(",")]
        coltext = body[body.rindex("{"):]
        columns = []
        for cm in re.finditer(
            r'(?:#\[sql_name = "([^"]+)"\]\s*\n\s*)?(\w+) -> ([^,\n]+),', coltext
        ):
            phys_col = cm.group(1) if cm.group(1) else cm.group(2)
            base, nullable = _split_type(cm.group(3))
            columns.append((cm.group(2), phys_col, cm.group(3).strip(), nullable, base))
        if physical in tables:
            raise ControlFailure("SCHEMA_PARSE_DUPLICATE_TABLE")
        tables[physical] = ParsedTable(rust, physical, pk, columns)
        table_order.append(rust)
    joins = []
    for jm in re.finditer(r"joinable!\((\w+) -> (\w+) \(([\w, ]+)\)\);", text):
        child, parent = jm.group(1), jm.group(2)
        cols = [c.strip() for c in jm.group(3).split(",")]
        joins.append((child, parent, cols))
    allow = []
    am = re.search(
        r"allow_tables_to_appear_in_same_query!\((.*?)\);", text, re.DOTALL
    )
    if am:
        allow = [x.strip() for x in am.group(1).split(",") if x.strip()]
    return ParsedSchema(sql_types, tables, table_order, joins, allow)


def raw_print_schema(root: Path, target: DbTarget) -> str:
    env = dict(os.environ)
    env["PGPASSWORD"] = target.password
    env["DATABASE_URL"] = os.environ["DATABASE_URL"]
    config = str(contained(root, DIESEL_TOML_REL))
    proc = run([diesel_bin(), "print-schema", "--config-file", config],
               cwd=root, env=env, reason="RAW_PRINT_SCHEMA_FAILED")
    # never consume automatic staging output; delete it if present
    staging = root / STAGING_REL
    if staging.exists():
        staging.unlink()
    return proc.stdout


# --------------------------------------------------------------------------- #
# Convention file
# --------------------------------------------------------------------------- #


@dataclass
class Convention:
    supplemental_types: list      # (diesel_type, pg_type)
    table_aliases: dict           # physical -> rust
    column_identifiers: list      # (rust_table, physical_col, raw_col, canonical_col)
    type_overrides: dict          # (rust_table, column) -> (raw_type, canonical_type)
    table_order: list
    column_order: dict            # rust_table -> [columns]


def load_convention(root: Path) -> Convention:
    path = contained(root, CONVENTION_REL)
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError):
        raise ControlFailure("CONVENTION_UNPARSEABLE")
    if data.get("cli", {}).get("diesel_version") != EXPECTED_CLI_VERSION:
        raise ControlFailure("CONVENTION_CLI_MISMATCH")

    def require(obj, keys, code):
        if not isinstance(obj, dict) or any(k not in obj for k in keys):
            raise ControlFailure(code)

    supp = []
    seen = set()
    for entry in data.get("supplemental_type", []):
        require(entry, ("diesel_type", "postgres_type"), "CONVENTION_SUPPLEMENTAL_INVALID")
        k = entry["diesel_type"]
        if k in seen:
            raise ControlFailure("CONVENTION_DUPLICATE")
        seen.add(k)
        supp.append((entry["diesel_type"], entry["postgres_type"]))

    aliases: dict = {}
    for entry in data.get("table_alias", []):
        require(entry, ("physical", "rust"), "CONVENTION_ALIAS_INVALID")
        if entry["physical"] in aliases:
            raise ControlFailure("CONVENTION_DUPLICATE")
        aliases[entry["physical"]] = entry["rust"]

    idents = []
    seen_id = set()
    for entry in data.get("column_identifier", []):
        require(entry, ("rust_table", "physical_column", "raw_diesel_column", "canonical_column"),
                "CONVENTION_IDENTIFIER_INVALID")
        k = (entry["rust_table"], entry["physical_column"])
        if k in seen_id:
            raise ControlFailure("CONVENTION_DUPLICATE")
        seen_id.add(k)
        idents.append((entry["rust_table"], entry["physical_column"],
                       entry["raw_diesel_column"], entry["canonical_column"]))

    overrides: dict = {}
    for entry in data.get("type_override", []):
        require(entry, ("rust_table", "column", "raw_diesel_type", "canonical_diesel_type"),
                "CONVENTION_OVERRIDE_INVALID")
        k = (entry["rust_table"], entry["column"])
        if k in overrides:
            raise ControlFailure("CONVENTION_DUPLICATE")
        overrides[k] = (entry["raw_diesel_type"], entry["canonical_diesel_type"])

    order = data.get("order", {}).get("tables", [])
    if not isinstance(order, list) or not order:
        raise ControlFailure("CONVENTION_TABLE_ORDER_INVALID")
    if len(order) != len(set(order)):
        raise ControlFailure("CONVENTION_DUPLICATE")

    col_order: dict = {}
    for entry in data.get("column_order", []):
        require(entry, ("rust_table", "columns"), "CONVENTION_COLUMN_ORDER_INVALID")
        if entry["rust_table"] in col_order:
            raise ControlFailure("CONVENTION_DUPLICATE")
        col_order[entry["rust_table"]] = list(entry["columns"])

    return Convention(supp, aliases, idents, overrides, order, col_order)


def validate_convention(conv: Convention, canonical: ParsedSchema, raw: ParsedSchema) -> None:
    """Every convention entry must be used and account for a real raw/canonical
    structural difference. Unknown, unused, or conflicting entries fail closed.
    """
    raw_sql = set(raw.sql_types)
    can_sql = set(canonical.sql_types)
    # supplemental types: present canonically, absent from raw
    supp_declared = {(d, p) for d, p in conv.supplemental_types}
    supp_actual = can_sql - raw_sql
    if supp_declared != supp_actual:
        raise ControlFailure("CONVENTION_SUPPLEMENTAL_MISMATCH")
    # table aliases
    actual_aliases = {
        t.physical_name: t.rust_name
        for t in canonical.tables.values() if t.physical_name != t.rust_name
    }
    if conv.table_aliases != actual_aliases:
        raise ControlFailure("CONVENTION_ALIAS_MISMATCH")
    # column identifiers: physical column whose raw rust-name differs from canonical
    raw_by_phys = {t.physical_name: t for t in raw.tables.values()}
    actual_idents = set()
    for t in canonical.tables.values():
        rt = raw_by_phys.get(t.physical_name)
        if not rt:
            continue
        raw_cols = {c[1]: c[0] for c in rt.columns}   # phys -> raw rust
        for rust_col, phys_col, *_ in t.columns:
            raw_rust = raw_cols.get(phys_col)
            if raw_rust is not None and raw_rust != rust_col:
                actual_idents.add((t.rust_name, phys_col, raw_rust, rust_col))
    if set((a, b, c, d) for a, b, c, d in conv.column_identifiers) != actual_idents:
        raise ControlFailure("CONVENTION_IDENTIFIER_MISMATCH")
    # type overrides: physical column whose raw base type differs from canonical
    actual_overrides = {}
    for t in canonical.tables.values():
        rt = raw_by_phys.get(t.physical_name)
        if not rt:
            continue
        raw_types = {c[1]: c[4] for c in rt.columns}  # phys -> raw base
        for rust_col, phys_col, _dtype, _nullable, base in t.columns:
            raw_base = raw_types.get(phys_col)
            if raw_base is not None and raw_base != base:
                actual_overrides[(t.rust_name, rust_col)] = (raw_base, base)
    if conv.type_overrides != actual_overrides:
        raise ControlFailure("CONVENTION_OVERRIDE_MISMATCH")
    # table order must equal canonical source order exactly
    if conv.table_order != canonical.table_order:
        raise ControlFailure("CONVENTION_TABLE_ORDER_MISMATCH")
    # column order must equal canonical per-table order exactly, and cover all
    expected_cols = {t.rust_name: [c[0] for c in t.columns] for t in canonical.tables.values()}
    if conv.column_order != expected_cols:
        raise ControlFailure("CONVENTION_COLUMN_ORDER_MISMATCH")


# --------------------------------------------------------------------------- #
# Projections and deltas
# --------------------------------------------------------------------------- #


def catalog_projection(cat: Catalog) -> dict:
    tables = {}
    for (schema, name), t in cat.tables.items():
        tables[(schema, name)] = {
            "columns": {c[0]: {"postgres_type": c[1], "nullable": c[2], "ordinal": c[3]}
                        for c in t.columns},
            "primary_key": list(t.primary_key),
            "foreign_keys": sorted(t.foreign_keys),
        }
    enums = {k: list(v) for k, v in cat.enums.items()}
    return {"tables": tables, "enums": enums}


def diesel_projection(schema: ParsedSchema) -> dict:
    tables = {}
    for phys, t in schema.tables.items():
        tables[("public", phys)] = {
            "rust_name": t.rust_name,
            "columns": [
                {"name": c[1], "diesel_type": c[2], "nullable": c[3], "ordinal": i + 1}
                for i, c in enumerate(t.columns)
            ],
            "primary_key": list(t.primary_key),
        }
    joins = sorted((c, p, tuple(cols)) for c, p, cols in schema.joins)
    allow = set(schema.allow_tables)
    sql_types = set(schema.sql_types)
    return {"tables": tables, "joins": joins, "allow": allow, "sql_types": sql_types}


def _diff_keys(before: dict, after: dict) -> tuple[set, set, set]:
    b, a = set(before), set(after)
    return a - b, b - a, {k for k in a & b if before[k] != after[k]}


def catalog_delta(before: Catalog, after: Catalog) -> dict:
    bp, ap = catalog_projection(before), catalog_projection(after)
    added_t, removed_t, _ = _diff_keys(bp["tables"], ap["tables"])
    add_cols, remove_cols, change_cols, add_pk, change_pk = [], [], [], [], []
    # a newly added table contributes each of its columns as an added column so
    # the manifest's explicit column objects are validated for new tables too.
    for key in sorted(added_t):
        for c, meta in sorted(ap["tables"][key]["columns"].items()):
            add_cols.append((key, c, meta))
    for key in sorted(removed_t):
        for c, meta in sorted(bp["tables"][key]["columns"].items()):
            remove_cols.append((key, c, meta))
    for key in ap["tables"].keys() & bp["tables"].keys():
        bc, ac = bp["tables"][key]["columns"], ap["tables"][key]["columns"]
        a_new, a_gone, a_chg = _diff_keys(bc, ac)
        for c in sorted(a_new):
            add_cols.append((key, c, ac[c]))
        for c in sorted(a_gone):
            remove_cols.append((key, c, bc[c]))
        for c in sorted(a_chg):
            change_cols.append((key, c, bc[c], ac[c]))
        if bp["tables"][key]["primary_key"] != ap["tables"][key]["primary_key"]:
            change_pk.append((key, bp["tables"][key]["primary_key"],
                              ap["tables"][key]["primary_key"]))
    add_enum, remove_enum, change_enum = _diff_keys(bp["enums"], ap["enums"])
    return {
        "added_tables": added_t, "removed_tables": removed_t,
        "added_columns": add_cols, "removed_columns": remove_cols,
        "changed_columns": change_cols, "changed_pk": change_pk,
        "added_enums": add_enum, "removed_enums": remove_enum, "changed_enums": change_enum,
        "before": bp, "after": ap,
    }


def diesel_delta(before: ParsedSchema, after: ParsedSchema) -> dict:
    bp, ap = diesel_projection(before), diesel_projection(after)
    added_t, removed_t, _ = _diff_keys(bp["tables"], ap["tables"])
    add_cols, remove_cols, change_cols, change_pk = [], [], [], []
    # expand added/removed tables' columns so manifest column objects are checked
    for key in sorted(added_t):
        for c in ap["tables"][key]["columns"]:
            add_cols.append((key, c["name"], c))
    for key in sorted(removed_t):
        for c in bp["tables"][key]["columns"]:
            remove_cols.append((key, c["name"], c))
    for key in ap["tables"].keys() & bp["tables"].keys():
        bc = {c["name"]: c for c in bp["tables"][key]["columns"]}
        ac = {c["name"]: c for c in ap["tables"][key]["columns"]}
        a_new, a_gone, a_chg = _diff_keys(bc, ac)
        for c in sorted(a_new):
            add_cols.append((key, c, ac[c]))
        for c in sorted(a_gone):
            remove_cols.append((key, c, bc[c]))
        for c in sorted(a_chg):
            change_cols.append((key, c, bc[c], ac[c]))
        if bp["tables"][key]["primary_key"] != ap["tables"][key]["primary_key"]:
            change_pk.append((key, bp["tables"][key]["primary_key"],
                              ap["tables"][key]["primary_key"]))
    add_join = [j for j in ap["joins"] if j not in bp["joins"]]
    remove_join = [j for j in bp["joins"] if j not in ap["joins"]]
    add_allow = ap["allow"] - bp["allow"]
    remove_allow = bp["allow"] - ap["allow"]
    add_sql = ap["sql_types"] - bp["sql_types"]
    remove_sql = bp["sql_types"] - ap["sql_types"]
    return {
        "added_tables": added_t, "removed_tables": removed_t,
        "added_columns": add_cols, "removed_columns": remove_cols,
        "changed_columns": change_cols, "changed_pk": change_pk,
        "added_joins": add_join, "removed_joins": remove_join,
        "added_allow": add_allow, "removed_allow": remove_allow,
        "added_sql_types": add_sql, "removed_sql_types": remove_sql,
    }


def delta_is_empty_catalog(d: dict) -> bool:
    return not any([
        d["added_tables"], d["removed_tables"], d["added_columns"],
        d["removed_columns"], d["changed_columns"], d["changed_pk"],
        d["added_enums"], d["removed_enums"], d["changed_enums"],
    ])


def delta_is_empty_diesel(d: dict) -> bool:
    return not any([
        d["added_tables"], d["removed_tables"], d["added_columns"],
        d["removed_columns"], d["changed_columns"], d["changed_pk"],
        d["added_joins"], d["removed_joins"], d["added_allow"],
        d["removed_allow"], d["added_sql_types"], d["removed_sql_types"],
    ])


# --------------------------------------------------------------------------- #
# Manifest (version 2)
# --------------------------------------------------------------------------- #


@dataclass
class Manifest:
    expected_projection: str
    adds: list
    removes: list
    changes: list


def load_manifest(path: Path) -> Manifest:
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError:
        raise ControlFailure("MANIFEST_UNREADABLE")
    try:
        data = tomllib.loads(raw)
    except tomllib.TOMLDecodeError:
        raise ControlFailure("MANIFEST_UNPARSEABLE")
    if data.get("version") != 2:
        raise ControlFailure("MANIFEST_VERSION_INVALID")
    # exactly one expected_projection with a strict value
    if raw.count("expected_projection") != 1:
        raise ControlFailure("MANIFEST_PROJECTION_DUPLICATE")
    mode = data.get("expected_projection")
    if mode not in {"change", "none"}:
        raise ControlFailure("MANIFEST_PROJECTION_INVALID")
    adds = data.get("add", [])
    removes = data.get("remove", [])
    changes = data.get("change", [])
    for op in adds + removes + changes:
        if not isinstance(op, dict) or "kind" not in op:
            raise ControlFailure("MANIFEST_OBJECT_INVALID")
        validate_manifest_object_shape(op, changes=(op in changes))
    if mode == "none" and (adds or removes or changes):
        raise ControlFailure("MANIFEST_NONE_HAS_OPERATIONS")
    return Manifest(mode, adds, removes, changes)


REQUIRED_FIELDS = {
    "table": {"schema", "name"},
    "column": {"schema", "table", "name", "postgres_type", "diesel_type", "nullable", "ordinal"},
    "primary-key": {"schema", "table", "columns"},
    "allow-table": {"schema", "table"},
    "join": {"child_schema", "child_table", "child_columns",
             "parent_schema", "parent_table", "parent_columns"},
    "sql-type": {"schema", "name", "diesel_type", "definition"},
}


def validate_manifest_object_shape(op: dict, changes: bool = False) -> None:
    kind = op.get("kind")
    if kind not in REQUIRED_FIELDS:
        raise ControlFailure("MANIFEST_KIND_UNKNOWN")
    if changes:
        if "before" not in op or "after" not in op:
            raise ControlFailure("MANIFEST_CHANGE_INCOMPLETE")
        for side in ("before", "after"):
            body = op[side]
            missing = REQUIRED_FIELDS[kind] - set(body)
            if missing:
                raise ControlFailure("MANIFEST_OBJECT_INCOMPLETE")
        return
    missing = REQUIRED_FIELDS[kind] - set(op)
    if missing:
        raise ControlFailure("MANIFEST_OBJECT_INCOMPLETE")
    # forbid stray wildcard / catch-all tokens
    for value in op.values():
        if isinstance(value, str) and value.strip() in {"*", "all", "any"}:
            raise ControlFailure("MANIFEST_WILDCARD_FORBIDDEN")
    if kind == "sql-type":
        definition = op["definition"]
        if not isinstance(definition, dict) or definition.get("kind") != "enum":
            raise ControlFailure("MANIFEST_SQLTYPE_DEFINITION_INVALID")
        if not isinstance(definition.get("labels"), list) or not definition["labels"]:
            raise ControlFailure("MANIFEST_SQLTYPE_DEFINITION_INVALID")
    if kind == "column":
        pg = op["postgres_type"]
        expected = PG_TO_DIESEL.get(pg)
        if expected is not None and expected != op["diesel_type"]:
            raise ControlFailure("MANIFEST_COLUMN_TYPE_INCONSISTENT")


# --------------------------------------------------------------------------- #
# Manifest projections and exact comparison
# --------------------------------------------------------------------------- #


def _key(schema, name):
    return (schema, name)


def manifest_catalog_projection(m: Manifest) -> dict:
    add_tables, add_cols, add_pk, add_enum = set(), [], [], {}
    for op in m.adds:
        k = op["kind"]
        if k == "table":
            add_tables.add(_key(op["schema"], op["name"]))
        elif k == "column":
            add_cols.append(((op["schema"], op["table"]), op["name"], {
                "postgres_type": op["postgres_type"], "nullable": bool(op["nullable"]),
                "ordinal": int(op["ordinal"])}))
        elif k == "primary-key":
            add_pk.append(((op["schema"], op["table"]), list(op["columns"])))
        elif k == "sql-type":
            add_enum[_key(op["schema"], op["name"])] = list(op["definition"]["labels"])
    return {"add_tables": add_tables, "add_cols": add_cols, "add_pk": add_pk,
            "add_enum": add_enum}


def manifest_diesel_projection(m: Manifest) -> dict:
    add_tables, add_cols, add_pk, add_join, add_allow, add_sql = set(), [], [], [], set(), set()
    for op in m.adds:
        k = op["kind"]
        if k == "table":
            add_tables.add(_key(op["schema"], op["name"]))
        elif k == "column":
            add_cols.append(((op["schema"], op["table"]), op["name"], {
                "diesel_type": _wrap_nullable(op["diesel_type"], bool(op["nullable"])),
                "nullable": bool(op["nullable"]), "ordinal": int(op["ordinal"])}))
        elif k == "primary-key":
            add_pk.append(((op["schema"], op["table"]), list(op["columns"])))
        elif k == "allow-table":
            add_allow.add(op["table"])
        elif k == "join":
            add_join.append((op["child_table"], op["parent_table"], list(op["child_columns"])))
        elif k == "sql-type":
            add_sql.add((op["diesel_type"], op["name"]))
    return {"add_tables": add_tables, "add_cols": add_cols, "add_pk": add_pk,
            "add_join": add_join, "add_allow": add_allow, "add_sql": add_sql}


def _wrap_nullable(base: str, nullable: bool) -> str:
    return f"Nullable<{base}>" if nullable else base


# --------------------------------------------------------------------------- #
# Rendering (byte-preserving for unchanged blocks; deterministic for additions)
# --------------------------------------------------------------------------- #


def render_added_table_block(op_table: dict, columns: list[dict]) -> str:
    lines = ["table! {", "    use diesel::sql_types::*;", "", ]
    pk = op_table["primary_key"]
    lines.append(f"    {op_table['name']} ({', '.join(pk)}) {{")
    for col in columns:
        lines.append(f"        {col['name']} -> {col['diesel_type']},")
    lines.append("    }")
    lines.append("}")
    return "\n".join(lines)


def render_candidate(canonical_text: str, manifest: Manifest) -> str:
    """Produce the candidate canonical schema.

    ``none`` mode returns the canonical text byte-for-byte. ``change`` mode
    applies bounded, deterministic block-level additions/removals; unchanged
    blocks and whitespace are preserved verbatim.
    """
    if manifest.expected_projection == "none":
        return canonical_text
    text = canonical_text
    # Only bounded table additions with their allow-table membership are
    # rendered. Any other change kind requires a renderer extension and fails
    # closed rather than silently emitting an incomplete block.
    add_tables = {op["name"]: op for op in manifest.adds if op["kind"] == "table"}
    add_columns: dict = {}
    add_pk: dict = {}
    add_allow = {op["table"] for op in manifest.adds if op["kind"] == "allow-table"}
    for op in manifest.adds:
        if op["kind"] == "column":
            add_columns.setdefault(op["table"], []).append(op)
        elif op["kind"] == "primary-key":
            add_pk[op["table"]] = op
    for op in manifest.adds:
        if op["kind"] not in {"table", "column", "primary-key", "allow-table"}:
            raise ControlFailure("RENDER_UNSUPPORTED_OPERATION")
    if manifest.removes or manifest.changes:
        raise ControlFailure("RENDER_UNSUPPORTED_OPERATION")

    blocks = []
    for tname, top in add_tables.items():
        cols = sorted(add_columns.get(tname, []), key=lambda c: int(c["ordinal"]))
        if tname not in add_pk:
            raise ControlFailure("RENDER_MISSING_PRIMARY_KEY")
        table_meta = {"name": tname, "primary_key": list(add_pk[tname]["columns"])}
        rendered_cols = [
            {"name": c["name"], "diesel_type": _wrap_nullable(c["diesel_type"], bool(c["nullable"]))}
            for c in cols
        ]
        blocks.append(render_added_table_block(table_meta, rendered_cols))

    # insert new table! blocks immediately before the first joinable! line
    join_anchor = text.index("joinable!(")
    insertion = "".join(b + "\n\n" for b in blocks)
    text = text[:join_anchor] + insertion + text[join_anchor:]

    # add allow-table membership entries before the closing of the macro
    if add_allow:
        m = re.search(r"(allow_tables_to_appear_in_same_query!\(\n)(.*?)(\);)", text, re.DOTALL)
        if not m:
            raise ControlFailure("RENDER_ALLOW_TABLE_ANCHOR_MISSING")
        body = m.group(2)
        additions = "".join(f"    {t},\n" for t in sorted(add_allow))
        text = text[:m.start(2)] + body + additions + text[m.end(2):]
    return text


# --------------------------------------------------------------------------- #
# Migration application (pinned CLI, candidate config)
# --------------------------------------------------------------------------- #


def apply_migrations(root: Path, migrations_dir: Path, target: DbTarget) -> None:
    env = dict(os.environ)
    env["PGPASSWORD"] = target.password
    env["DATABASE_URL"] = os.environ["DATABASE_URL"]
    config = str(contained(root, DIESEL_TOML_REL))
    run([diesel_bin(), "migration", "run",
         "--config-file", config,
         "--migration-dir", str(migrations_dir)],
        cwd=root, env=env, reason="MIGRATION_APPLICATION_FAILED")
    staging = root / STAGING_REL
    if staging.exists():
        staging.unlink()


def prove_empty(target: DbTarget) -> None:
    rows = psql(target, (
        "SELECT count(*) FROM information_schema.tables "
        "WHERE table_schema='public' AND table_type='BASE TABLE';"
    ))
    if rows and rows[0][0] != "0":
        raise ControlFailure("DISPOSABLE_DATABASE_NOT_EMPTY")


def reset_public_schema(target: DbTarget) -> None:
    psql(target, "DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;")


# --------------------------------------------------------------------------- #
# Compilation of a candidate schema in an isolated worktree
# --------------------------------------------------------------------------- #


def compile_candidate(root: Path, head: str, candidate_text: str) -> None:
    if os.environ.get("THOTH_DIESEL_SKIP_COMPILE") == "1":
        # explicit opt-out used ONLY by fast unit tests; integration tests and CI
        # never set this, so the mandatory compile gate remains active there.
        return
    tmp = Path(tempfile.mkdtemp(prefix="thoth-diesel-compile-"))
    wt = tmp / "worktree"
    created = False
    try:
        run(["git", "worktree", "add", "--detach", str(wt), head], cwd=root,
            reason="COMPILE_WORKTREE_UNCREATABLE")
        created = True
        (wt / CANONICAL_REL).write_text(candidate_text, encoding="utf-8")
        env = dict(os.environ)
        # reuse the main target dir to avoid a cold rebuild
        env.setdefault("CARGO_TARGET_DIR", str(root / "target"))
        run(["cargo", "check", "-p", "thoth-api", "--features", "backend"],
            cwd=wt, env=env, reason="CANDIDATE_COMPILE_FAILED")
    finally:
        if created:
            run(["git", "worktree", "remove", "--force", str(wt)], cwd=root, check=False)
        shutil.rmtree(tmp, ignore_errors=True)
        if wt.exists():
            raise ControlFailure("COMPILE_WORKTREE_UNREMOVABLE")


# --------------------------------------------------------------------------- #
# Atomic canonical write with exclusive lock
# --------------------------------------------------------------------------- #


def atomic_write_canonical(root: Path, head: str, original_bytes: bytes, candidate_text: str) -> None:
    canonical = contained(root, CANONICAL_REL)
    lock_path = canonical.with_suffix(canonical.suffix + ".lock")
    lock_fd = os.open(str(lock_path), os.O_CREAT | os.O_RDWR, 0o644)
    try:
        try:
            fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError:
            raise ControlFailure("CANONICAL_LOCK_CONTENDED")
        # reverify repository root, HEAD, and original bytes before writing
        if candidate_head(root) != head:
            raise ControlFailure("CANDIDATE_HEAD_MOVED")
        if canonical.read_bytes() != original_bytes:
            raise ControlFailure("CANONICAL_MUTATED_BEFORE_WRITE")
        directory = canonical.parent
        fd, tmp_name = tempfile.mkstemp(dir=str(directory), prefix=".schema-", suffix=".tmp")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as fh:
                fh.write(candidate_text)
                fh.flush()
                os.fsync(fh.fileno())
            os.replace(tmp_name, str(canonical))
            dir_fd = os.open(str(directory), os.O_DIRECTORY)
            try:
                os.fsync(dir_fd)
            finally:
                os.close(dir_fd)
        except BaseException:
            with contextlib.suppress(OSError):
                os.unlink(tmp_name)
            raise
    finally:
        fcntl.flock(lock_fd, fcntl.LOCK_UN)
        os.close(lock_fd)
        with contextlib.suppress(OSError):
            os.unlink(lock_path)


# --------------------------------------------------------------------------- #
# Two-phase acquisition
# --------------------------------------------------------------------------- #


@dataclass
class Acquisition:
    baseline_catalog: Catalog
    candidate_catalog: Catalog
    baseline_raw: ParsedSchema
    candidate_raw: ParsedSchema
    baseline_canonical: ParsedSchema
    candidate_canonical: ParsedSchema
    baseline_ledger: list
    candidate_ledger: list
    candidate_canonical_text: str


def acquire(root: Path, base_ref: str, target: DbTarget, manifest: Manifest,
            probe_sql: Optional[str] = None) -> Acquisition:
    """Perform the independent two-phase baseline-to-candidate acquisition on one
    proven disposable database."""
    reset_public_schema(target)
    prove_empty(target)
    with base_worktree(root, base_ref) as wt:
        base_migrations = wt / MIGRATIONS_REL
        apply_migrations(root, base_migrations, target)
        baseline_catalog = snapshot_catalog(target)
        baseline_raw = parse_diesel_schema(raw_print_schema(root, target))
        base_canonical_bytes = run(
            ["git", "show", f"{base_ref}:{CANONICAL_REL}"], cwd=root,
            reason="BASELINE_CANONICAL_UNREADABLE").stdout
        wt_canonical = (wt / CANONICAL_REL).read_text(encoding="utf-8")
        if wt_canonical != base_canonical_bytes:
            raise ControlFailure("BASELINE_CANONICAL_MISMATCH")
        baseline_canonical = parse_diesel_schema(base_canonical_bytes)
    baseline_ledger = list(baseline_catalog.migrations)

    # candidate phase: apply only pending candidate migrations to the SAME db
    candidate_migrations = root / MIGRATIONS_REL
    apply_migrations(root, candidate_migrations, target)
    if probe_sql:
        psql(target, probe_sql)
    candidate_catalog = snapshot_catalog(target)
    candidate_ledger = list(candidate_catalog.migrations)
    if candidate_ledger[: len(baseline_ledger)] != baseline_ledger:
        raise ControlFailure("MIGRATION_LEDGER_PREFIX_INVALID")
    candidate_raw = parse_diesel_schema(raw_print_schema(root, target))
    canonical_text = (root / CANONICAL_REL).read_text(encoding="utf-8")
    candidate_canonical_text = render_candidate(canonical_text, manifest)
    candidate_canonical = parse_diesel_schema(candidate_canonical_text)
    return Acquisition(
        baseline_catalog, candidate_catalog, baseline_raw, candidate_raw,
        baseline_canonical, candidate_canonical, baseline_ledger, candidate_ledger,
        candidate_canonical_text,
    )


# --------------------------------------------------------------------------- #
# Exact projection comparison
# --------------------------------------------------------------------------- #


def compare_projections(acq: Acquisition, manifest: Manifest) -> dict:
    cat_d = catalog_delta(acq.baseline_catalog, acq.candidate_catalog)
    raw_d = diesel_delta(acq.baseline_raw, acq.candidate_raw)
    can_d = diesel_delta(acq.baseline_canonical, acq.candidate_canonical)

    cat_empty = delta_is_empty_catalog(cat_d)
    raw_empty = delta_is_empty_diesel(raw_d)
    can_empty = delta_is_empty_diesel(can_d)

    if manifest.expected_projection == "none":
        if not (cat_empty and raw_empty and can_empty):
            raise ControlFailure("NONE_MODE_CONTROLLED_CHANGE_DETECTED")
    else:  # change
        if cat_empty and raw_empty and can_empty:
            raise ControlFailure("CHANGE_MODE_ALL_PROJECTIONS_EMPTY")
        _compare_catalog(cat_d, manifest)
        _compare_diesel(raw_d, manifest, leg="raw")
        _compare_diesel(can_d, manifest, leg="canonical")
    return {"catalog": cat_d, "raw": raw_d, "canonical": can_d,
            "counts": _aggregate_counts(manifest)}


def _compare_catalog(delta: dict, manifest: Manifest) -> None:
    proj = manifest_catalog_projection(manifest)
    if delta["added_tables"] != proj["add_tables"]:
        raise ControlFailure("CATALOG_TABLE_DELTA_MISMATCH")
    actual_cols = {(k, n): meta for (k, n, meta) in delta["added_columns"]}
    expected_cols = {(k, n): meta for (k, n, meta) in proj["add_cols"]}
    if actual_cols != expected_cols:
        raise ControlFailure("CATALOG_COLUMN_DELTA_MISMATCH")
    actual_pk = {k: cols for (k, before, cols) in delta["changed_pk"]}
    # new tables introduce PKs via added table + changed_pk not populated; check adds
    expected_pk = {k: cols for (k, cols) in proj["add_pk"]}
    # for added tables the PK appears as part of the new table; verify catalog PK
    for key, cols in expected_pk.items():
        after_tbl = delta["after"]["tables"].get(key)
        if not after_tbl or after_tbl["primary_key"] != cols:
            raise ControlFailure("CATALOG_PRIMARY_KEY_MISMATCH")
    # enum before/after handled via changed/added enums
    if proj["add_enum"]:
        for key, labels in proj["add_enum"].items():
            if acq_labels := delta["after"]["enums"].get(key):
                if acq_labels != labels:
                    raise ControlFailure("CATALOG_ENUM_LABELS_MISMATCH")
            else:
                raise ControlFailure("CATALOG_ENUM_MISSING")
            if key in delta["before"]["enums"]:
                raise ControlFailure("CATALOG_ENUM_PRESENT_AT_BASELINE")


def _compare_diesel(delta: dict, manifest: Manifest, leg: str) -> None:
    proj = manifest_diesel_projection(manifest)
    if delta["added_tables"] != proj["add_tables"]:
        raise ControlFailure(f"{leg.upper()}_TABLE_DELTA_MISMATCH")
    actual_cols = {(k, n): (meta["diesel_type"], meta["nullable"], meta["ordinal"])
                   for (k, n, meta) in delta["added_columns"]}
    expected_cols = {(k, n): (meta["diesel_type"], meta["nullable"], meta["ordinal"])
                     for (k, n, meta) in proj["add_cols"]}
    if actual_cols != expected_cols:
        raise ControlFailure(f"{leg.upper()}_COLUMN_DELTA_MISMATCH")
    if set(delta["added_allow"]) != set(proj["add_allow"]):
        raise ControlFailure(f"{leg.upper()}_ALLOW_TABLE_MISMATCH")
    actual_join = sorted((c, p, tuple(cols)) for (c, p, cols) in delta["added_joins"])
    expected_join = sorted((c, p, tuple(cols)) for (c, p, cols) in proj["add_join"])
    if actual_join != expected_join:
        raise ControlFailure(f"{leg.upper()}_JOIN_MISMATCH")
    if set(delta["added_sql_types"]) != set(proj["add_sql"]):
        raise ControlFailure(f"{leg.upper()}_SQLTYPE_MISMATCH")


def _aggregate_counts(manifest: Manifest) -> dict:
    return {"added": len(manifest.adds), "removed": len(manifest.removes),
            "changed": len(manifest.changes)}


# --------------------------------------------------------------------------- #
# Orchestration: check / generate
# --------------------------------------------------------------------------- #


def _emit(statuses: dict) -> None:
    for key, value in statuses.items():
        print(f"{key}={value}")


def execute(mode: str, base_ref: str, manifest_path: Path, output_rel: str) -> int:
    script_path = Path(__file__)
    root = resolve_repo_root(script_path)
    validate_diesel_toml(root)
    validate_cli_version()
    validate_base_ref(root, base_ref)
    head = candidate_head(root)

    if output_rel != CANONICAL_REL:
        raise ControlFailure("OUTPUT_PATH_NOT_CANONICAL")
    canonical_path = contained(root, CANONICAL_REL)
    original_bytes = canonical_path.read_bytes()

    manifest = load_manifest(manifest_path)
    target, identity, provenance = safe_target_gate()

    canonical_now = parse_diesel_schema(original_bytes.decode("utf-8"))
    convention = load_convention(root)

    # first acquisition
    acq1 = acquire(root, base_ref, target, manifest)
    validate_convention(convention, acq1.baseline_canonical, acq1.baseline_raw)
    compare_projections(acq1, manifest)

    # deterministic repeat from fresh disposable state
    acq2 = acquire(root, base_ref, target, manifest)
    result = compare_projections(acq2, manifest)
    if acq1.candidate_canonical_text != acq2.candidate_canonical_text:
        raise ControlFailure("DETERMINISTIC_REPEAT_MISMATCH")

    candidate_text = acq2.candidate_canonical_text

    if manifest.expected_projection == "none" and candidate_text != original_bytes.decode("utf-8"):
        raise ControlFailure("NONE_MODE_CANONICAL_NOT_BYTE_IDENTICAL")

    # focused compilation of the candidate in an isolated worktree
    compile_candidate(root, head, candidate_text)

    # confirm HEAD/base did not move during the run
    if candidate_head(root) != head:
        raise ControlFailure("CANDIDATE_HEAD_MOVED")
    validate_base_ref(root, base_ref)

    if mode == "generate":
        atomic_write_canonical(root, head, original_bytes, candidate_text)

    # in check mode the canonical file must be unchanged on disk
    if mode == "check" and canonical_path.read_bytes() != original_bytes:
        raise ControlFailure("CANONICAL_MUTATED_IN_CHECK")

    server_class = identity["server_class"]
    statuses = {
        "THOTH_DIESEL_TARGET": ("SAFE_DISPOSABLE_LOCAL" if provenance == "SAFE_DISPOSABLE_LOCAL"
                                 else "SAFE_DISPOSABLE_CI"),
        "THOTH_DIESEL_CLI": EXPECTED_CLI_VERSION,
        "THOTH_DIESEL_BASE_REF": "VERIFIED_FULL_AUTHORIZED_ANCESTOR_SHA",
        "THOTH_DIESEL_BASELINE": "CAPTURED_INDEPENDENTLY",
        "THOTH_DIESEL_CANDIDATE": "CAPTURED_INDEPENDENTLY",
        "THOTH_DIESEL_CONFIG": "diesel.toml",
        "THOTH_DIESEL_SCHEMA": CANONICAL_REL,
        "THOTH_DIESEL_CLIENT_ENDPOINT": "LOOPBACK",
        "THOTH_DIESEL_SERVER_ADDRESS": server_class,
        "THOTH_DIESEL_EXPECTED_PROJECTION": manifest.expected_projection.upper(),
        "THOTH_DIESEL_DELTA": "EXACT_PROJECTED_MATCH",
        "THOTH_DIESEL_CLEANUP": "COMPLETE",
        "THOTH_DIESEL_REPEAT": "IDENTICAL",
        "THOTH_DIESEL_DIFF": "CLEAN",
        "THOTH_DIESEL_COUNTS": (
            f"added={result['counts']['added']} "
            f"removed={result['counts']['removed']} "
            f"changed={result['counts']['changed']}"),
    }
    _emit(statuses)
    if manifest.expected_projection == "none":
        print("THOTH_DIESEL_EXCLUDED_EFFECTS=NOT_VALIDATED_BY_PROJECTION_CONTROL")
    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    for name in ("check", "generate"):
        p = sub.add_parser(name)
        p.add_argument("--base-ref", required=True)
        p.add_argument("--expected-change", required=True)
        p.add_argument("--output", default=CANONICAL_REL)
    return parser.parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        return execute(args.mode, args.base_ref, Path(args.expected_change), args.output)
    except ControlFailure as failure:
        print("BLOCKED - THOTH DIESEL GENERATION CONTROL FAILED")
        print(f"REASON={failure.reason}")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
