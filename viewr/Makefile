all:

build:
	trunk build --release

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

local_db:
	# create the KV table for CONNECTION_LIST
	# Enter a number of SSIDs in the list - with fake json details for now
	@npx wrangler kv:key put --local  --binding=CONNECTION_LIST "ssid=MOVISTAR_8A9E" "{}"
	@npx wrangler kv:key put --local  --binding=CONNECTION_LIST "ssid=MOVISTAR_8A9E_PLUS" "{}"
	# create the KV table for DEVICE_STATUS
	# npx wrangler kv:namespace --local create DEVICE_STATUS
	# Enter a number of devices in DEVICE_STATUS KV table with different status
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "Stopped"
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "Offline"
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "Reporting"
	# create the KV table for DEVICE_ID_CONNECTION_MAPPING to map devices to the SSIDs connections they are using
	# npx wrangler kv:namespace --local create DEVICE_ID_CONNECTION_MAPPING
	# Map each device to a particular Connection
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "ssid=MOVISTAR_8A9E"
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "ssid=MOVISTAR_8A9E"
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "ssid=MOVISTAR_8A9E"
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "ssid=MOVISTAR_8A9E_PLUS"
	# Enter a number of devices status into CONNECTION_DEVICE_STATUS
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E::6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E::84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "Stopped"
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E::fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "Offline"
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "ssid=MOVISTAR_8A9E_PLUS::d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "Reporting"
	# Enter a number of devices status into DEVICE_DETAILS
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "{}"
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "{}"
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "{}"
	@npx wrangler kv:key put --local --binding CONNECTION_DEVICE_STATUS "d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "{friendly_name: \"Nice Device\"}"
	# List what is present
	@echo "SSID_LIST"
	npx wrangler kv:key list --local --binding=CONNECTION_LIST
	@echo "DEVICE_STATUS"
	npx wrangler kv:key list --local --binding=DEVICE_STATUS
	@echo "DEVICE_ID_CONNECTION_MAPPING"
	npx wrangler kv:key list --local --binding=DEVICE_ID_CONNECTION_MAPPING
	@echo "SSID_DEVICE_STATUS"
	npx wrangler kv:key list --local --binding=CONNECTION_DEVICE_STATUS
