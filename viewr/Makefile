all:

deploy:
	npx wrangler pages deploy dist

local:
	trunk build
	npx wrangler pages dev dist --compatibility-date=2024-01-24

local_db:
	npx wrangler kv:key put --local  --binding=DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"

test:
	cargo test

format:
	leptosfmt ./**/*.rs

clean:
	rm -f dist/*