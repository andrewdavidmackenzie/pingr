all: build

.PHONY: build
build:
	cargo clippy
	npx wrangler deploy --dry-run --outdir=dist

local:
	cargo clippy
	npx wrangler dev

deploy:
	cargo clippy
	npx wrangler deploy

test:
	cargo test
