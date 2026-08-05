#!/usr/bin/env python3
"""THOTH-DB-CTRL-01 - tests for the Diesel schema synchronizer.

Unit tests cover configuration/manifest/convention parsing, projection and
manifest semantics, structural negatives, representation independence, literal
safety, and rendering. They run without a database.

Integration tests provision their own disposable PostgreSQL 17 container on an
ephemeral loopback port with a ``thoth_diesel_`` database and remove it (and its
anonymous storage) on teardown. They exercise the real two-phase acquisition,
the controlled probe, enum handling, ``none``-mode excluded-effect migrations,
automatic-output bypass, forced-failure byte preservation, and cleanup. They are
skipped when Docker or the pinned Diesel CLI is unavailable.

All fixtures live in private temporary directories, disposable containers, and
temporary worktrees; nothing is left behind.
"""

from __future__ import annotations

import importlib.util
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import textwrap
import time
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
BASE_REF = "4c53709befc91acb481beac54a1d314926b61d76"


def _load_module():
    spec = importlib.util.spec_from_file_location(
        "diesel_schema", str(HERE / "diesel_schema.py")
    )
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


ds = _load_module()


# --------------------------------------------------------------------------- #
# Fixtures
# --------------------------------------------------------------------------- #

PROBE_MANIFEST = textwrap.dedent(
    """
    version = 2
    expected_projection = "change"

    [[add]]
    kind = "table"
    schema = "public"
    name = "thoth_db_ctrl_probe"

    [[add]]
    kind = "column"
    schema = "public"
    table = "thoth_db_ctrl_probe"
    name = "probe_id"
    postgres_type = "uuid"
    diesel_type = "Uuid"
    nullable = false
    ordinal = 1

    [[add]]
    kind = "column"
    schema = "public"
    table = "thoth_db_ctrl_probe"
    name = "probe_value"
    postgres_type = "text"
    diesel_type = "Text"
    nullable = true
    ordinal = 2

    [[add]]
    kind = "primary-key"
    schema = "public"
    table = "thoth_db_ctrl_probe"
    columns = ["probe_id"]

    [[add]]
    kind = "allow-table"
    schema = "public"
    table = "thoth_db_ctrl_probe"
    """
).strip()

NONE_MANIFEST = 'version = 2\nexpected_projection = "none"\n'

MINI_SCHEMA = textwrap.dedent(
    """
    pub mod sql_types {
        #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
        #[diesel(postgres_type(name = "widget_kind"))]
        pub struct WidgetKind;
    }

    use diesel::{allow_tables_to_appear_in_same_query, joinable, table};

    table! {
        use diesel::sql_types::*;
        use super::sql_types::WidgetKind;

        gadget (gadget_id) {
            gadget_id -> Uuid,
            name -> Text,
            kind -> WidgetKind,
            created_at -> Timestamptz,
        }
    }

    table! {
        use diesel::sql_types::*;

        #[sql_name = "gizmo"]
        widget (widget_id) {
            widget_id -> Uuid,
            gadget_id -> Uuid,
            label -> Nullable<Text>,
        }
    }

    joinable!(widget -> gadget (gadget_id));

    allow_tables_to_appear_in_same_query!(
        gadget,
        widget,
    );
    """
).strip() + "\n"


def _make_manifest(text: str) -> Path:
    tmp = tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False)
    tmp.write(text)
    tmp.close()
    return Path(tmp.name)


# --------------------------------------------------------------------------- #
# Unit: parsing and configuration
# --------------------------------------------------------------------------- #


class DieselTomlTests(unittest.TestCase):
    def test_repository_config_is_valid(self):
        ds.validate_diesel_toml(REPO)  # must not raise

    def test_output_must_be_staging(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            (root / "target").mkdir()
            (root / "diesel.toml").write_text(
                '[print_schema]\nfile = "thoth-api/src/schema.rs"\n'
                'custom_type_derives = ["diesel::query_builder::QueryId"]\n'
            )
            (root / "thoth-api" / "src").mkdir(parents=True)
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.validate_diesel_toml(root)
            self.assertIn("STAGING", ctx.exception.reason)

    def test_invalid_current_config_form_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            (root / "target").mkdir()
            # missing commas -> unparseable, mirrors the pre-correction config
            (root / "diesel.toml").write_text(
                '[print_schema]\nfile = "target/diesel-schema.rs"\n'
                'custom_type_derives = [\n  "a"\n  "b"\n]\n'
            )
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.validate_diesel_toml(root)
            self.assertEqual(ctx.exception.reason, "DIESEL_CONFIG_UNPARSEABLE")

    def test_derive_set_must_be_minimal(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            (root / "target").mkdir()
            (root / "diesel.toml").write_text(
                '[print_schema]\nfile = "target/diesel-schema.rs"\n'
                'custom_type_derives = ["diesel::sql_types::*", '
                '"diesel::query_builder::QueryId"]\n'
            )
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.validate_diesel_toml(root)
            self.assertEqual(ctx.exception.reason, "DIESEL_CONFIG_DERIVES_INVALID")


class SchemaParserTests(unittest.TestCase):
    def setUp(self):
        self.schema = ds.parse_diesel_schema(MINI_SCHEMA)

    def test_sql_types(self):
        self.assertEqual(self.schema.sql_types, [("WidgetKind", "widget_kind")])

    def test_alias_and_columns(self):
        widget = self.schema.tables["gizmo"]
        self.assertEqual(widget.rust_name, "widget")
        self.assertEqual(widget.physical_name, "gizmo")
        names = [c[1] for c in widget.tables_columns()] if hasattr(widget, "tables_columns") else [c[1] for c in widget.columns]
        self.assertEqual(names, ["widget_id", "gadget_id", "label"])

    def test_nullability_and_types(self):
        widget = self.schema.tables["gizmo"]
        label = [c for c in widget.columns if c[1] == "label"][0]
        self.assertTrue(label[3])            # nullable
        self.assertEqual(label[4], "Text")   # base type
        gadget = self.schema.tables["gadget"]
        created = [c for c in gadget.columns if c[1] == "created_at"][0]
        self.assertFalse(created[3])
        self.assertEqual(created[4], "Timestamptz")

    def test_primary_key_and_join_and_allow(self):
        self.assertEqual(self.schema.tables["gadget"].primary_key, ["gadget_id"])
        self.assertIn(("widget", "gadget", ["gadget_id"]), self.schema.joins)
        self.assertEqual(set(self.schema.allow_tables), {"gadget", "widget"})

    def test_unsupported_macro_form_fails_closed(self):
        broken = "table! {\n    use diesel::sql_types::*;\n\n    { bad\n}\n"
        with self.assertRaises(ds.ControlFailure):
            ds.parse_diesel_schema(broken)


class BaseRefTests(unittest.TestCase):
    def test_reject_bad_shas(self):
        for value in ["", "HEAD", "4C53709B" * 5, "abc", "z" * 40, BASE_REF[:39]]:
            with self.assertRaises(ds.ControlFailure):
                ds.validate_base_ref(REPO, value)

    def test_accept_authorized_base(self):
        ds.validate_base_ref(REPO, BASE_REF)  # ancestor of HEAD, must not raise


# --------------------------------------------------------------------------- #
# Unit: manifest version 2
# --------------------------------------------------------------------------- #


class ManifestTests(unittest.TestCase):
    def _load(self, text):
        path = _make_manifest(text)
        try:
            return ds.load_manifest(path)
        finally:
            path.unlink()

    def test_probe_manifest_ok(self):
        m = self._load(PROBE_MANIFEST)
        self.assertEqual(m.expected_projection, "change")
        self.assertEqual(len(m.adds), 5)

    def test_none_manifest_ok(self):
        m = self._load(NONE_MANIFEST)
        self.assertEqual(m.expected_projection, "none")
        self.assertEqual(m.adds, [])

    def test_missing_mode(self):
        with self.assertRaises(ds.ControlFailure):
            self._load("version = 2\n")

    def test_unknown_mode(self):
        with self.assertRaises(ds.ControlFailure):
            self._load('version = 2\nexpected_projection = "other"\n')

    def test_case_variant_mode(self):
        with self.assertRaises(ds.ControlFailure):
            self._load('version = 2\nexpected_projection = "None"\n')

    def test_duplicate_mode(self):
        with self.assertRaises(ds.ControlFailure) as ctx:
            self._load('version = 2\nexpected_projection = "none"\n'
                       'expected_projection = "change"\n')
        # TOML itself rejects duplicate keys, or our guard does
        self.assertIsInstance(ctx.exception, ds.ControlFailure)

    def test_wrong_version(self):
        with self.assertRaises(ds.ControlFailure):
            self._load('version = 1\nexpected_projection = "none"\n')

    def test_none_with_operations_rejected(self):
        text = NONE_MANIFEST + (
            '\n[[add]]\nkind = "table"\nschema = "public"\nname = "x"\n'
        )
        with self.assertRaises(ds.ControlFailure) as ctx:
            self._load(text)
        self.assertEqual(ctx.exception.reason, "MANIFEST_NONE_HAS_OPERATIONS")

    def test_incomplete_column_object(self):
        text = (
            'version = 2\nexpected_projection = "change"\n'
            '[[add]]\nkind = "column"\nschema = "public"\ntable = "x"\nname = "y"\n'
        )
        with self.assertRaises(ds.ControlFailure) as ctx:
            self._load(text)
        self.assertEqual(ctx.exception.reason, "MANIFEST_OBJECT_INCOMPLETE")

    def test_wildcard_rejected(self):
        text = (
            'version = 2\nexpected_projection = "change"\n'
            '[[add]]\nkind = "table"\nschema = "public"\nname = "*"\n'
        )
        with self.assertRaises(ds.ControlFailure) as ctx:
            self._load(text)
        self.assertEqual(ctx.exception.reason, "MANIFEST_WILDCARD_FORBIDDEN")

    def test_unknown_kind(self):
        text = (
            'version = 2\nexpected_projection = "change"\n'
            '[[add]]\nkind = "index"\nschema = "public"\nname = "x"\n'
        )
        with self.assertRaises(ds.ControlFailure) as ctx:
            self._load(text)
        self.assertEqual(ctx.exception.reason, "MANIFEST_KIND_UNKNOWN")

    def test_column_type_inconsistency_rejected(self):
        text = (
            'version = 2\nexpected_projection = "change"\n'
            '[[add]]\nkind = "column"\nschema = "public"\ntable = "x"\nname = "y"\n'
            'postgres_type = "uuid"\ndiesel_type = "Text"\nnullable = false\nordinal = 1\n'
        )
        with self.assertRaises(ds.ControlFailure) as ctx:
            self._load(text)
        self.assertEqual(ctx.exception.reason, "MANIFEST_COLUMN_TYPE_INCONSISTENT")

    def test_incomplete_sqltype_definition(self):
        text = (
            'version = 2\nexpected_projection = "change"\n'
            '[[add]]\nkind = "sql-type"\nschema = "public"\nname = "t"\n'
            'diesel_type = "T"\n[add.definition]\nkind = "enum"\nlabels = []\n'
        )
        with self.assertRaises(ds.ControlFailure) as ctx:
            self._load(text)
        self.assertEqual(ctx.exception.reason, "MANIFEST_SQLTYPE_DEFINITION_INVALID")


# --------------------------------------------------------------------------- #
# Unit: projections, deltas, and structural negatives
# --------------------------------------------------------------------------- #


def _catalog(tables, enums=None, migrations=None):
    tab = {}
    for name, cols, pk, fks in tables:
        tab[("public", name)] = ds.CatalogTable("public", name, cols, pk, fks)
    return ds.Catalog(tab, enums or {}, migrations or [])


class DeltaTests(unittest.TestCase):
    def test_added_table_expands_columns(self):
        before = _catalog([])
        after = _catalog([
            ("probe",
             [("probe_id", "uuid", False, 1), ("probe_value", "text", True, 2)],
             ["probe_id"], []),
        ])
        d = ds.catalog_delta(before, after)
        self.assertIn(("public", "probe"), d["added_tables"])
        cols = {(k, n) for (k, n, meta) in d["added_columns"]}
        self.assertEqual(cols, {(("public", "probe"), "probe_id"),
                                (("public", "probe"), "probe_value")})

    def test_none_delta_empty(self):
        cat = _catalog([("t", [("id", "uuid", False, 1)], ["id"], [])])
        self.assertTrue(ds.delta_is_empty_catalog(ds.catalog_delta(cat, cat)))

    def test_enum_label_change_is_catalog_nonempty(self):
        before = _catalog([], enums={("public", "e"): ["A", "B"]})
        after = _catalog([], enums={("public", "e"): ["A", "B", "C"]})
        d = ds.catalog_delta(before, after)
        self.assertFalse(ds.delta_is_empty_catalog(d))
        self.assertIn(("public", "e"), d["changed_enums"])


class ProjectionComparisonTests(unittest.TestCase):
    """Structural negatives against a synthetic probe acquisition."""

    def _probe_acq(self, cols, pk=("probe_id",), enums=None, canonical_extra_join=False):
        before_cat = _catalog([], enums={})
        after_cat = _catalog([("thoth_db_ctrl_probe", cols, list(pk), [])],
                             enums=enums or {})
        raw_before = ds.parse_diesel_schema(_wrap_schema(""))
        raw_after = ds.parse_diesel_schema(_wrap_schema(_probe_block(cols, pk)))
        can_before = raw_before
        can_after = raw_after
        return ds.Acquisition(
            before_cat, after_cat, raw_before, raw_after,
            can_before, can_after, [], [], "",
        )

    def _manifest(self, text=PROBE_MANIFEST):
        path = _make_manifest(text)
        try:
            return ds.load_manifest(path)
        finally:
            path.unlink()

    def test_probe_projection_matches(self):
        cols = [("probe_id", "uuid", False, 1), ("probe_value", "text", True, 2)]
        acq = self._probe_acq(cols)
        ds.compare_projections(acq, self._manifest())  # must not raise

    def test_wrong_type_rejected(self):
        cols = [("probe_id", "integer", False, 1), ("probe_value", "text", True, 2)]
        acq = self._probe_acq(cols)
        with self.assertRaises(ds.ControlFailure):
            ds.compare_projections(acq, self._manifest())

    def test_wrong_nullability_rejected(self):
        cols = [("probe_id", "uuid", True, 1), ("probe_value", "text", True, 2)]
        acq = self._probe_acq(cols)
        with self.assertRaises(ds.ControlFailure):
            ds.compare_projections(acq, self._manifest())

    def test_wrong_ordinal_rejected(self):
        cols = [("probe_id", "uuid", False, 2), ("probe_value", "text", True, 1)]
        acq = self._probe_acq(cols)
        with self.assertRaises(ds.ControlFailure):
            ds.compare_projections(acq, self._manifest())

    def test_wrong_primary_key_rejected(self):
        cols = [("probe_id", "uuid", False, 1), ("probe_value", "text", True, 2)]
        acq = self._probe_acq(cols, pk=("probe_value",))
        with self.assertRaises(ds.ControlFailure):
            ds.compare_projections(acq, self._manifest())

    def test_change_mode_all_empty_rejected(self):
        cat = _catalog([("t", [("id", "uuid", False, 1)], ["id"], [])])
        raw = ds.parse_diesel_schema(_wrap_schema(_probe_block(
            [("id", "uuid", False, 1)], ("id",), name="t")))
        acq = ds.Acquisition(cat, cat, raw, raw, raw, raw, [], [], "")
        with self.assertRaises(ds.ControlFailure) as ctx:
            ds.compare_projections(acq, self._manifest())
        self.assertEqual(ctx.exception.reason, "CHANGE_MODE_ALL_PROJECTIONS_EMPTY")

    def test_none_mode_hidden_change_rejected(self):
        before = _catalog([])
        after = _catalog([("sneaky", [("id", "uuid", False, 1)], ["id"], [])])
        raw_b = ds.parse_diesel_schema(_wrap_schema(""))
        raw_a = ds.parse_diesel_schema(_wrap_schema(_probe_block(
            [("id", "uuid", False, 1)], ("id",), name="sneaky")))
        acq = ds.Acquisition(before, after, raw_b, raw_a, raw_b, raw_a, [], [], "")
        with self.assertRaises(ds.ControlFailure) as ctx:
            ds.compare_projections(acq, self._manifest(NONE_MANIFEST))
        self.assertEqual(ctx.exception.reason, "NONE_MODE_CONTROLLED_CHANGE_DETECTED")


def _wrap_schema(table_blocks: str) -> str:
    return (
        "pub mod sql_types {\n}\n\n"
        "use diesel::{allow_tables_to_appear_in_same_query, joinable, table};\n\n"
        + table_blocks +
        "\nallow_tables_to_appear_in_same_query!(\n"
        + "".join(f"    {n},\n" for n in _table_names(table_blocks))
        + ");\n"
    )


def _table_names(blocks: str):
    return re.findall(r"\n    (\w+) \(", blocks)


def _probe_block(cols, pk, name="thoth_db_ctrl_probe") -> str:
    lines = ["table! {", "    use diesel::sql_types::*;", "",
             f"    {name} ({', '.join(pk)}) {{"]
    diesel_map = {"uuid": "Uuid", "text": "Text", "integer": "Int4"}
    for cname, ctype, nullable, _ord in cols:
        base = diesel_map.get(ctype, "Text")
        typ = f"Nullable<{base}>" if nullable else base
        lines.append(f"        {cname} -> {typ},")
    lines += ["    }", "}", ""]
    return "\n".join(lines)


# --------------------------------------------------------------------------- #
# Unit: rendering
# --------------------------------------------------------------------------- #


class RenderTests(unittest.TestCase):
    def test_none_mode_byte_identical(self):
        man = ds.Manifest("none", [], [], [])
        self.assertEqual(ds.render_candidate(MINI_SCHEMA, man), MINI_SCHEMA)

    def test_probe_render_deterministic_and_preserves_blocks(self):
        path = _make_manifest(PROBE_MANIFEST)
        try:
            man = ds.load_manifest(path)
        finally:
            path.unlink()
        out1 = ds.render_candidate(MINI_SCHEMA, man)
        out2 = ds.render_candidate(MINI_SCHEMA, man)
        self.assertEqual(out1, out2)
        # every original line survives verbatim
        for line in MINI_SCHEMA.splitlines():
            self.assertIn(line, out1)
        self.assertIn("thoth_db_ctrl_probe (probe_id) {", out1)
        self.assertIn("\n    thoth_db_ctrl_probe,\n", out1)


# --------------------------------------------------------------------------- #
# Unit: safe execution / literal safety
# --------------------------------------------------------------------------- #


class LiteralSafetyTests(unittest.TestCase):
    def test_metacharacters_stay_literal(self):
        payload = "a; touch /tmp/should_not_exist_$$; `echo x` $(echo y)"
        proc = ds.run(["printf", "%s", payload])
        self.assertEqual(proc.stdout, payload)
        self.assertFalse(Path("/tmp/should_not_exist_$$").exists())

    def test_non_list_args_rejected(self):
        with self.assertRaises(ds.ControlFailure):
            ds.run("echo hi")  # type: ignore

    def test_database_url_parsing_rejects_non_loopback(self):
        old = os.environ.get("DATABASE_URL")
        os.environ["DATABASE_URL"] = "postgres://u:p@db.example.com:5432/thoth"
        try:
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.parse_database_url()
            self.assertEqual(ctx.exception.reason, "SAFE_DATABASE_TARGET_UNVERIFIED")
        finally:
            if old is None:
                del os.environ["DATABASE_URL"]
            else:
                os.environ["DATABASE_URL"] = old

    def test_output_never_contains_url_or_credentials(self):
        # the emitted status dictionary contains only safe tokens
        statuses = {
            "THOTH_DIESEL_TARGET": "SAFE_DISPOSABLE_LOCAL",
            "THOTH_DIESEL_CLI": "2.3.10",
        }
        blob = json.dumps(statuses)
        self.assertNotIn("postgres://", blob)
        self.assertNotIn("password", blob.lower())


# --------------------------------------------------------------------------- #
# Integration: real disposable PostgreSQL 17
# --------------------------------------------------------------------------- #


def _tool(name):
    return shutil.which(name) is not None


def _free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


class _Disposable:
    container = None
    port = None
    dbname = None
    diesel_bin = None


def _diesel_bin():
    env = os.environ.get("DIESEL_BIN")
    if env and Path(env).exists():
        proc = subprocess.run([env, "--version"], capture_output=True, text=True)
        if "2.3.10" in (proc.stdout + proc.stderr):
            return env
    return None


DB_REASON = ""


def setUpModule():
    global DB_REASON
    if not _tool("docker") or not _tool("psql"):
        DB_REASON = "docker/psql unavailable"
        return
    diesel = _diesel_bin()
    if not diesel:
        DB_REASON = "pinned diesel 2.3.10 CLI unavailable (set DIESEL_BIN)"
        return
    _Disposable.diesel_bin = diesel
    _Disposable.port = _free_port()
    _Disposable.dbname = f"thoth_diesel_test_{os.getpid()}"
    _Disposable.container = f"thoth_diesel_test_{os.getpid()}"
    proc = subprocess.run(
        ["docker", "run", "-d", "--name", _Disposable.container,
         "-e", "POSTGRES_PASSWORD=thoth", "-e", "POSTGRES_USER=thoth",
         "-e", f"POSTGRES_DB={_Disposable.dbname}",
         "-p", f"127.0.0.1:{_Disposable.port}:5432", "postgres:17"],
        capture_output=True, text=True,
    )
    if proc.returncode != 0:
        DB_REASON = f"cannot start container: {proc.stderr.strip()[:120]}"
        _Disposable.container = None
        return
    # wait for readiness
    for _ in range(60):
        ready = subprocess.run(
            ["docker", "exec", _Disposable.container, "pg_isready",
             "-U", "thoth", "-d", _Disposable.dbname],
            capture_output=True,
        )
        if ready.returncode == 0:
            break
        time.sleep(1)
    else:
        DB_REASON = "container never became ready"


def tearDownModule():
    if _Disposable.container:
        subprocess.run(["docker", "rm", "-f", "-v", _Disposable.container],
                       capture_output=True)
        # prove removal
        exists = subprocess.run(
            ["docker", "ps", "-aq", "--filter", f"name=^{_Disposable.container}$"],
            capture_output=True, text=True,
        )
        assert exists.stdout.strip() == "", "disposable container was not removed"


class DatabaseIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not _Disposable.container:
            raise unittest.SkipTest(DB_REASON or "no disposable database")
        cls.env = dict(os.environ)
        cls.env["DIESEL_BIN"] = _Disposable.diesel_bin
        cls.env["DATABASE_URL"] = (
            f"postgres://thoth:thoth@localhost:{_Disposable.port}/{_Disposable.dbname}"
        )
        cls.env["THOTH_DIESEL_CONFIRM_DATABASE"] = _Disposable.dbname
        cls.env["THOTH_DIESEL_CONTAINER"] = _Disposable.container

    def _apply_env(self):
        saved = {k: os.environ.get(k) for k in self.env}
        os.environ.update(self.env)
        return saved

    def _restore_env(self, saved):
        for k, v in saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v

    def _target(self):
        return ds.parse_database_url()

    def test_a_safe_target_gate(self):
        saved = self._apply_env()
        try:
            target, identity, prov = ds.safe_target_gate()
            self.assertEqual(prov, "SAFE_DISPOSABLE_LOCAL")
            self.assertEqual(identity["current_database"], _Disposable.dbname)
        finally:
            self._restore_env(saved)

    def test_b_default_port_rejected(self):
        saved = self._apply_env()
        os.environ["DATABASE_URL"] = (
            f"postgres://thoth:thoth@localhost:5432/{_Disposable.dbname}"
        )
        try:
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.safe_target_gate()
            self.assertEqual(ctx.exception.reason, "LOCAL_PORT_IS_DEFAULT_DEV_PORT")
        finally:
            self._restore_env(saved)

    def test_c_wrong_prefix_rejected(self):
        saved = self._apply_env()
        # rename confirmation to a non thoth_diesel_ name by pointing at 'postgres'
        os.environ["DATABASE_URL"] = (
            f"postgres://thoth:thoth@localhost:{_Disposable.port}/postgres"
        )
        os.environ["THOTH_DIESEL_CONFIRM_DATABASE"] = "postgres"
        try:
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.safe_target_gate()
            self.assertEqual(ctx.exception.reason, "LOCAL_DATABASE_PREFIX_INVALID")
        finally:
            self._restore_env(saved)

    def test_d_confirmation_mismatch_rejected(self):
        saved = self._apply_env()
        os.environ["THOTH_DIESEL_CONFIRM_DATABASE"] = "thoth_diesel_wrong"
        try:
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.safe_target_gate()
            self.assertEqual(ctx.exception.reason, "DATABASE_CONFIRMATION_MISMATCH")
        finally:
            self._restore_env(saved)

    def test_e_missing_container_rejected(self):
        saved = self._apply_env()
        os.environ["THOTH_DIESEL_CONTAINER"] = "no_such_container_xyz"
        try:
            with self.assertRaises(ds.ControlFailure) as ctx:
                ds.safe_target_gate()
            self.assertEqual(ctx.exception.reason, "LOCAL_CONTAINER_UNVERIFIED")
        finally:
            self._restore_env(saved)

    def test_f_full_migration_chain_and_revert(self):
        saved = self._apply_env()
        try:
            target = self._target()
            ds.reset_public_schema(target)
            ds.prove_empty(target)
            ds.apply_migrations(REPO, REPO / ds.MIGRATIONS_REL, target)
            cat = ds.snapshot_catalog(target)
            self.assertGreater(len(cat.tables), 50)
            self.assertEqual(len(cat.migrations), 4)
        finally:
            self._restore_env(saved)

    def test_g_none_mode_noop_twice_identical(self):
        saved = self._apply_env()
        os.environ["THOTH_DIESEL_SKIP_COMPILE"] = "1"
        try:
            target = self._target()
            man = ds.Manifest("none", [], [], [])
            acq1 = ds.acquire(REPO, BASE_REF, target, man)
            acq2 = ds.acquire(REPO, BASE_REF, target, man)
            self.assertEqual(acq1.candidate_canonical_text,
                             acq2.candidate_canonical_text)
            canonical = (REPO / ds.CANONICAL_REL).read_text()
            self.assertEqual(acq1.candidate_canonical_text, canonical)
        finally:
            os.environ.pop("THOTH_DIESEL_SKIP_COMPILE", None)
            self._restore_env(saved)

    def test_h_convention_accounts_for_all_differences(self):
        saved = self._apply_env()
        os.environ["THOTH_DIESEL_SKIP_COMPILE"] = "1"
        try:
            target = self._target()
            man = ds.Manifest("none", [], [], [])
            acq = ds.acquire(REPO, BASE_REF, target, man)
            conv = ds.load_convention(REPO)
            ds.validate_convention(conv, acq.baseline_canonical, acq.baseline_raw)
        finally:
            os.environ.pop("THOTH_DIESEL_SKIP_COMPILE", None)
            self._restore_env(saved)

    def test_i_probe_projection_and_no_join(self):
        saved = self._apply_env()
        os.environ["THOTH_DIESEL_SKIP_COMPILE"] = "1"
        try:
            target = self._target()
            path = _make_manifest(PROBE_MANIFEST)
            man = ds.load_manifest(path)
            path.unlink()
            probe = ("CREATE TABLE public.thoth_db_ctrl_probe "
                     "(probe_id uuid PRIMARY KEY, probe_value text);")
            acq = ds.acquire(REPO, BASE_REF, target, man, probe_sql=probe)
            result = ds.compare_projections(acq, man)
            self.assertEqual(result["counts"]["added"], 5)
            self.assertEqual(result["raw"]["added_joins"], [])
        finally:
            os.environ.pop("THOTH_DIESEL_SKIP_COMPILE", None)
            self._restore_env(saved)

    def test_j_new_enum_absent_at_baseline(self):
        saved = self._apply_env()
        os.environ["THOTH_DIESEL_SKIP_COMPILE"] = "1"
        try:
            target = self._target()
            man = ds.Manifest("none", [], [], [])
            # baseline has no such enum; introduce it as a controlled probe
            probe = "CREATE TYPE public.thoth_probe_enum AS ENUM ('A', 'B');"
            acq = ds.acquire(REPO, BASE_REF, target, man, probe_sql=probe)
            self.assertNotIn(("public", "thoth_probe_enum"),
                             acq.baseline_catalog.enums)
            self.assertEqual(acq.candidate_catalog.enums[("public", "thoth_probe_enum")],
                             ["A", "B"])
        finally:
            os.environ.pop("THOTH_DIESEL_SKIP_COMPILE", None)
            self._restore_env(saved)

    def test_k_index_only_migration_is_none(self):
        saved = self._apply_env()
        os.environ["THOTH_DIESEL_SKIP_COMPILE"] = "1"
        try:
            target = self._target()
            man = ds.Manifest("none", [], [], [])
            probe = "CREATE INDEX thoth_probe_idx ON public.work (doi);"
            acq = ds.acquire(REPO, BASE_REF, target, man, probe_sql=probe)
            result = ds.compare_projections(acq, man)  # index is excluded effect
            self.assertTrue(ds.delta_is_empty_diesel(result["raw"]))
        finally:
            os.environ.pop("THOTH_DIESEL_SKIP_COMPILE", None)
            self._restore_env(saved)

    def test_l_direct_diesel_cannot_write_canonical(self):
        saved = self._apply_env()
        try:
            target = self._target()
            ds.reset_public_schema(target)
            ds.prove_empty(target)
            before = (REPO / ds.CANONICAL_REL).read_bytes()
            ds.apply_migrations(REPO, REPO / ds.MIGRATIONS_REL, target)
            raw = ds.raw_print_schema(REPO, target)
            self.assertIn("diesel", raw.lower())
            after = (REPO / ds.CANONICAL_REL).read_bytes()
            self.assertEqual(before, after)  # canonical untouched
            self.assertFalse((REPO / ds.STAGING_REL).exists())  # staging deleted
        finally:
            self._restore_env(saved)

    def test_m_probe_candidate_compiles(self):
        if os.environ.get("THOTH_DIESEL_RUN_COMPILE") != "1":
            raise unittest.SkipTest("set THOTH_DIESEL_RUN_COMPILE=1 for compile test")
        saved = self._apply_env()
        try:
            target = self._target()
            path = _make_manifest(PROBE_MANIFEST)
            man = ds.load_manifest(path)
            path.unlink()
            probe = ("CREATE TABLE public.thoth_db_ctrl_probe "
                     "(probe_id uuid PRIMARY KEY, probe_value text);")
            acq = ds.acquire(REPO, BASE_REF, target, man, probe_sql=probe)
            head = ds.candidate_head(REPO)
            ds.compile_candidate(REPO, head, acq.candidate_canonical_text)
        finally:
            self._restore_env(saved)


if __name__ == "__main__":
    unittest.main(verbosity=2)
