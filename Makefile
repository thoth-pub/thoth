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
	migration

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
	@echo "  check             Run cargo check"
	@echo "  clippy            Lint with cargo clippy"
	@echo "  format            Format code with cargo fmt"
	@echo "  check-format      Check formatting"
	@echo "  check-all         Run tests, clippy, and formatting checks"
	@echo "  migration         Create a database migration"

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

