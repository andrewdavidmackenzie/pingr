all:

deploy:
	trunk build --release
	npx wrangler pages deploy dist

local:
	trunk build
	npx wrangler pages dev dist --compatibility-date=2024-01-24

local_db:
	npx wrangler kv:key put --local  --binding=DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"
	npx wrangler kv:key put --local  --binding=DEVICE_STATUS "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "Stopped"
	npx wrangler kv:key put --local  --binding=DEVICE_STATUS "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "NotReporting"

test:
	cargo test

format:
	leptosfmt ./**/*.rs

clean:
	rm -f dist/*