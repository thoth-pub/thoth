.PHONY: \
	build-graphql-api \
	build-export-api \
	run-graphql-api \
	run-export-api \
	docker-dev \
	docker-dev-build \
	docker-dev-run \
	docker-dev-db \
	docker-dev-redis \
	build \
	test \
	clippy \
	format \
	check-format \
	check \
	check-all \

all: build-graphql-api build-export-api
check-all: test check clippy check-format

run-graphql-api: build-graphql-api
	RUST_BACKTRACE=1 cargo run init

run-export-api: build-export-api
	RUST_BACKTRACE=1 cargo run start export-api

docker-dev: docker-dev-build docker-dev-run

docker-dev-build:
	docker compose -f docker-compose.dev.yml build

docker-dev-run:
	docker compose -f docker-compose.dev.yml up

docker-dev-db:
	docker compose -f docker-compose.dev.yml up db

docker-dev-redis:
	docker compose -f docker-compose.dev.yml up redis

build:
	cargo build -vv

build-graphql-api: build

build-export-api: build

test:
	cargo test --workspace

clippy:
	cargo clippy --all --all-targets --all-features -- -D warnings

format:
	cargo fmt --all --

check-format:
	cargo fmt --all -- --check

check:
	cargo check --workspace
