all:

deploy:
	npx wrangler pages deploy dist

local:
	npx wrangler pages dev dist

local_db:
	npx wrangler kv:key put --local  --binding=DEVICE_STATUS "6f9a4b82ea3977ade1c7a0728713dccbc40
    09f140b934d86d3e7671e1b632a89" "Reporting"