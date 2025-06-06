.PHONY: \
	build-graphql-api \
	build-export-api \
	build-app \
	run-app \
	run-graphql-api \
	run-export-api \
	watch-app \
	docker-dev \
	docker-dev-build \
	docker-dev-run \
	docker-dev-db \
	docker-dev-redis \
	docker-dev-zitadel \
	docker-dev-zitadel-db \
	build \
	test \
	clippy \
	format \
	check-format \
	check \
	check-all \

all: build-graphql-api build-export-api build-app
check-all: test check clippy check-format

run-app: build-app
	RUST_BACKTRACE=1 cargo run start app

run-graphql-api: build-graphql-api
	RUST_BACKTRACE=1 cargo run init

run-export-api: build-export-api
	RUST_BACKTRACE=1 cargo run start export-api

watch-app:
	trunk serve thoth-app/index.html

docker-dev: docker-dev-build docker-dev-run

docker-dev-build:
	docker compose -f docker-compose.dev.yml build

docker-dev-run:
	docker compose -f docker-compose.dev.yml up

docker-dev-db:
	docker compose -f docker-compose.dev.yml up db

docker-dev-redis:
	docker compose -f docker-compose.dev.yml up redis

docker-dev-zitadel:
	docker compose -f docker-compose.dev.yml up zitadel

docker-dev-zitadel-db:
	docker compose -f docker-compose.dev.yml up zitadel-db

build:
	cargo build -vv

build-graphql-api: build

build-export-api: build

build-app: build

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
