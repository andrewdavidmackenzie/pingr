all: picomon collectr configr viewr wimon

.PHONY: deploy
deploy:
	@cd collectr && $(MAKE) deploy
	@cd viewr && $(MAKE) deploy

.PHONY: picomon
picomon:
	@cd picomon && $(MAKE) build

.PHONY: collectr
collectr:
	@cd collectr && $(MAKE) build

.PHONY: configr
configr:
	@cd configr && cargo build

.PHONY: viewr
viewr:
	@cd viewr && $(MAKE) build

.PHONY: wimon
wimon:
	@cd wimon && $(MAKE) build