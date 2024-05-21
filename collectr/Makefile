all: build

.PHONY: build
build:
	cargo clippy
	wrangler deploy --dry-run --outdir=dist

local:
	cargo clippy
	wrangler dev

deploy:
	cargo clippy
	wrangler deploy

test:
	cargo test
