.PHONY: \
	help \
	run-db \
	run-zitadel-db \
	run-redis \
	run-zitadel \
	run-graphql-api \
	run-export-api \
	build \
	test \
	check \
	clippy \
	format \
	check-format \
	check-all \
	migration \
	coverage \
	check-diesel-schema \
	generate-diesel-schema \
	test-diesel-schema

CARGO_VERSION := $(shell grep '^version' Cargo.toml | sed -E 's/version *= *"([^"]+)"/\1/')
MAJOR        := $(word 1,$(subst ., ,$(CARGO_VERSION)))
MINOR        := $(word 2,$(subst ., ,$(CARGO_VERSION)))

DATE = $(shell date +"%Y%m%d")

help:
	@echo "Available targets:"
	@echo "  help              Show this help"
	@echo "  run-db            Start PostgreSQL (docker)"
	@echo "  run-zitadel-db    Start Zitadel PostgreSQL (docker)"
	@echo "  run-redis         Start Redis (docker)"
	@echo "  run-zitadel       Start Zitadel (docker)"
	@echo "  run-graphql-api   Run GraphQL API (cargo)"
	@echo "  run-export-api    Run export API (cargo)"
	@echo "  build             Build the workspace"
	@echo "  test              Run tests"
	@echo "  coverage          Run test coverage (cargo llvm-cov)"
	@echo "  check             Run cargo check"
	@echo "  clippy            Lint with cargo clippy"
	@echo "  format            Format code with cargo fmt"
	@echo "  check-format      Check formatting"
	@echo "  check-all         Run tests, clippy, and formatting checks"
	@echo "  migration         Create a database migration"
	@echo "  check-diesel-schema     Verify the canonical Diesel schema (THOTH-DB-CTRL-01)"
	@echo "  generate-diesel-schema  Regenerate the canonical Diesel schema (synchronizer-only writer)"
	@echo "  test-diesel-schema      Run the Diesel schema synchronizer test suite"

run-db:
	docker compose up db

run-zitadel-db:
	docker compose up zitadel-db

run-redis:
	docker compose up redis

run-zitadel:
	docker compose up zitadel

run-graphql-api: build
	RUST_BACKTRACE=1 cargo run init

run-export-api: build
	RUST_BACKTRACE=1 cargo run start export-api

build:
	cargo build -vv

test:
	cargo test --workspace

coverage:
	cargo llvm-cov --workspace --all-features --html --output-dir ./coverage

check:
	cargo check --workspace

clippy:
	cargo clippy --all --all-targets --all-features -- -D warnings

format:
	cargo fmt --all --

check-format:
	cargo fmt --all -- --check

check-all: test check clippy check-format

migration:
	@new_minor=$$(expr $(MINOR) + 1); \
	new_version="$(MAJOR).$$new_minor.0"; \
	dir="thoth-api/migrations/$(DATE)_v$$new_version"; \
	mkdir -p $$dir; \
	touch $$dir/up.sql; \
	touch $$dir/down.sql;

# --------------------------------------------------------------------------- #
# THOTH-DB-CTRL-01 - canonical Diesel schema control.
#
# .github/scripts/diesel_schema.py is the SOLE authorized writer of the
# canonical contract at thoth-api/src/schema.rs. Local and CI invocations use
# the exact same synchronizer. The exact Diesel CLI 2.3.10 must be supplied
# through DIESEL_BIN; these targets never guess a global installation. Each
# local target provisions a disposable PostgreSQL 17 container on an ephemeral
# loopback port with a `thoth_diesel_` database and removes it (and its
# anonymous storage) on exit. DATABASE_URL is never printed. check is
# read-only; generation requires an explicit expected-change file and the exact
# authorized full base SHA in THOTH_DIESEL_BASE_REF.
# --------------------------------------------------------------------------- #

DIESEL_SCHEMA_SCRIPT := .github/scripts/diesel_schema.py

define _diesel_require_bin
	if [ -z "$(DIESEL_BIN)" ]; then \
		echo "DIESEL_BIN must point at the exact Diesel CLI 2.3.10 binary."; \
		echo "Install it into a temporary root outside the repository, e.g.:"; \
		echo "  cargo install diesel_cli --version 2.3.10 \\"; \
		echo "    --root \"\$$THOTH_DIESEL_CLI_ROOT\" --no-default-features --features postgres --locked"; \
		exit 1; \
	fi
endef

check-diesel-schema:
	@$(_diesel_require_bin)
	@if [ -z "$(THOTH_DIESEL_BASE_REF)" ]; then \
		echo "THOTH_DIESEL_BASE_REF must be the exact authorized full base SHA."; exit 1; fi
	@set -eu; \
	manifest=$$(mktemp); printf 'version = 2\nexpected_projection = "none"\n' > "$$manifest"; \
	expected="$${THOTH_DIESEL_EXPECTED_CHANGE_FILE:-$$manifest}"; \
	if [ "$${GITHUB_ACTIONS:-}" = "true" ]; then \
		cleanup() { rm -f "$$manifest"; }; trap cleanup EXIT; \
		DIESEL_BIN="$(DIESEL_BIN)" \
		python3 $(DIESEL_SCHEMA_SCRIPT) check \
			--base-ref "$(THOTH_DIESEL_BASE_REF)" \
			--expected-change "$$expected" \
			--output thoth-api/src/schema.rs; \
	else \
		name="thoth_diesel_local_$$$$"; \
		port=$$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()'); \
		cleanup() { docker rm -f -v "$$name" >/dev/null 2>&1 || true; rm -f "$$manifest"; }; \
		trap cleanup EXIT; \
		docker run -d --name "$$name" -e POSTGRES_PASSWORD=thoth -e POSTGRES_USER=thoth \
			-e POSTGRES_DB="$$name" -p 127.0.0.1:$$port:5432 postgres:17 >/dev/null; \
		for i in $$(seq 1 60); do docker exec "$$name" pg_isready -U thoth -d "$$name" >/dev/null 2>&1 && break; sleep 1; done; \
		DIESEL_BIN="$(DIESEL_BIN)" \
		DATABASE_URL="postgres://thoth:thoth@localhost:$$port/$$name" \
		THOTH_DIESEL_CONFIRM_DATABASE="$$name" \
		THOTH_DIESEL_CONTAINER="$$name" \
		python3 $(DIESEL_SCHEMA_SCRIPT) check \
			--base-ref "$(THOTH_DIESEL_BASE_REF)" \
			--expected-change "$$expected" \
			--output thoth-api/src/schema.rs; \
	fi

generate-diesel-schema:
	@$(_diesel_require_bin)
	@if [ -z "$(THOTH_DIESEL_BASE_REF)" ]; then \
		echo "THOTH_DIESEL_BASE_REF must be the exact authorized full base SHA."; exit 1; fi
	@if [ -z "$(THOTH_DIESEL_EXPECTED_CHANGE_FILE)" ]; then \
		echo "THOTH_DIESEL_EXPECTED_CHANGE_FILE must point at an explicit version-2 manifest."; exit 1; fi
	@set -eu; \
	name="thoth_diesel_local_$$$$"; \
	port=$$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()'); \
	cleanup() { docker rm -f -v "$$name" >/dev/null 2>&1 || true; }; \
	trap cleanup EXIT; \
	docker run -d --name "$$name" -e POSTGRES_PASSWORD=thoth -e POSTGRES_USER=thoth \
		-e POSTGRES_DB="$$name" -p 127.0.0.1:$$port:5432 postgres:17 >/dev/null; \
	for i in $$(seq 1 60); do docker exec "$$name" pg_isready -U thoth -d "$$name" >/dev/null 2>&1 && break; sleep 1; done; \
	DIESEL_BIN="$(DIESEL_BIN)" \
	DATABASE_URL="postgres://thoth:thoth@localhost:$$port/$$name" \
	THOTH_DIESEL_CONFIRM_DATABASE="$$name" \
	THOTH_DIESEL_CONTAINER="$$name" \
	python3 $(DIESEL_SCHEMA_SCRIPT) generate \
		--base-ref "$(THOTH_DIESEL_BASE_REF)" \
		--expected-change "$(THOTH_DIESEL_EXPECTED_CHANGE_FILE)" \
		--output thoth-api/src/schema.rs

test-diesel-schema:
	@$(_diesel_require_bin)
	@DIESEL_BIN="$(DIESEL_BIN)" THOTH_DIESEL_RUN_COMPILE=1 \
		python3 $(DIESEL_SCHEMA_SCRIPT:diesel_schema.py=test_diesel_schema.py)

