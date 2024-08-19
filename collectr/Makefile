all: build

.PHONY: clippy
clippy:
	cargo clippy

.PHONY: build
build: clippy
	wrangler deploy --dry-run --outdir=dist

local:
	cargo clippy
	wrangler dev

deploy:
	cargo clippy
	wrangler deploy

test:
	cargo test
