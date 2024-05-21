all: build

build:
	trunk build --release

deploy:
	trunk build --release
	wrangler pages deploy dist

local:
	trunk build
	wrangler pages dev dist --compatibility-date=2024-01-24

test:
	cargo test

format:
	leptosfmt ./**/*.rs

clean:
	rm -f dist/*

local_db:
	# create the KV table for DEVICE_STATUS
	# wrangler kv:namespace --local create DEVICE_STATUS
	# Enter a number of devices in DEVICE_STATUS KV table with different status
	@wrangler kv:key put --local  --binding=DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"
	@wrangler kv:key put --local  --binding=DEVICE_STATUS "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "Stopped"
	@wrangler kv:key put --local  --binding=DEVICE_STATUS "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "Offline"
	@wrangler kv:key put --local  --binding=DEVICE_STATUS "d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "Reporting"
	# Enter a number of devices status into CONNECTION_DEVICE_STATUS
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E::6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E::84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "Stopped"
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E::fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "Offline"
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E_PLUS::d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "Reporting"
	# Enter a number of devices status into DEVICE_DETAILS
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "{}"
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "{}"
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "{}"
	@wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "{friendly_name: Some(\"Nice Device\")}"
	# List what is present
	@echo "DEVICE_STATUS"
	wrangler kv:key list --local --binding=DEVICE_STATUS
	@echo "SSID_DEVICE_STATUS"
	wrangler kv:key list --local --binding=CONNECTION_DEVICE_STATUS
