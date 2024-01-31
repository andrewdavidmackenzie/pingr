all:

deploy:
	trunk build --release
	npx wrangler pages deploy dist

local:
	trunk build
	npx wrangler pages dev dist --compatibility-date=2024-01-24

test:
	cargo test

format:
	leptosfmt ./**/*.rs

clean:
	rm -f dist/*