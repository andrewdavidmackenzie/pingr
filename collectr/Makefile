all:
	cargo clippy
	npx wrangler deploy --dry-run --outdir=dist

local:
	npx wrangler dev

deploy:
	npx wrangler deploy

test:
	cargo test
