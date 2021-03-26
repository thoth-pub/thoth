THOTH_API ?= http://localhost:8000

.PHONY: \
	build-api \
	build-app \
	run-app \
	run-api \

all: build-api build-app

run-app: build-app
	RUST_BACKTRACE=1 cargo run start app

run-api: build-api
	RUST_BACKTRACE=1 cargo run init

build-api:
	THOTH_API=$(THOTH_API) cargo build

build-app: build-wasm
	THOTH_API=$(THOTH_API) cargo build

build-wasm:
	wasm-pack build --debug thoth-app/ --target web && \
		rollup thoth-app/main.js --format iife --file thoth-app/pkg/thoth_app.js
