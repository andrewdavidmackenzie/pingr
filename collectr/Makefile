all:
	cargo clippy
	npx wrangler build

run:
	npx wrangler dev

deploy:
	npx wrangler deploy

test:
	cargo test

local_db:
	# create the KV table for SSID_LIST
	# npx wrangler kv:namespace --local create SSID_LIST
	# Enter a number of SSIDs in the list - with fake json details for now
	@npx wrangler kv:key put --local  --binding=SSID_LIST "MOVISTAR_8A9E" "{}"
	@npx wrangler kv:key put --local  --binding=SSID_LIST "MOVISTAR_8A9E_PLUS" "{}"
	# create the KV table for DEVICE_STATUS
	# npx wrangler kv:namespace --local create DEVICE_STATUS
	# Enter a number of devices in DEVICE_STATUS KV table with different status
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "Stopped"
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "NotReporting"
	@npx wrangler kv:key put --local  --binding=DEVICE_STATUS "d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "Reporting"
	# create the KV table for DEVICE_ID_CONNECTION_MAPPING to map devices to the SSIDs connections they are using
	# npx wrangler kv:namespace --local create DEVICE_ID_CONNECTION_MAPPING
	# Map each device to a particular Connection
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "ssid=MOVISTAR_8A9E"
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "ssid=MOVISTAR_8A9E"
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "ssid=MOVISTAR_8A9E"
	@npx wrangler kv:key put --local  --binding=DEVICE_ID_CONNECTION_MAPPING "d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "ssid=MOVISTAR_8A9E_PLUS"
	# create the kv table for SSID_DEVICE_STATUS
	#
	# Enter a number of devices status into that table
	@npx wrangler kv:key put --local --binding SSID_DEVICE_STATUS "MOVISTAR_8A9E::6f9a4b82ea3977ade1c7a0728713dccbc4009f140b934d86d3e7671e1b632a89" "Reporting"
	@npx wrangler kv:key put --local --binding SSID_DEVICE_STATUS "MOVISTAR_8A9E::84b1f1308782a5e4abd6c4e1da685f6e2921c7bc1139412b12da3e0a567482a4" "Stopped"
	@npx wrangler kv:key put --local --binding SSID_DEVICE_STATUS "MOVISTAR_8A9E::fce9597011454eedca41d1b43a544a419126ee78cf66ba9c4666ab9dc2a09bda" "NotReporting"
	@npx wrangler kv:key put --local --binding SSID_DEVICE_STATUS "MOVISTAR_8A9E_PLUS::d77c8bb72c2b7cb98145d882533da5e64475189dd89180ef708d551d0406bcc0" "Reporting"
	# List what is present
	@echo "SSID_LIST"
	npx wrangler kv:key list --local --binding=SSID_LIST
	@echo "DEVICE_STATUS"
	npx wrangler kv:key list --local --binding=DEVICE_STATUS
	@echo "DEVICE_ID_CONNECTION_MAPPING"
	npx wrangler kv:key list --local --binding=DEVICE_ID_CONNECTION_MAPPING
	@echo "SSID_DEVICE_STATUS"
	npx wrangler kv:key list --local --binding=SSID_DEVICE_STATUS
