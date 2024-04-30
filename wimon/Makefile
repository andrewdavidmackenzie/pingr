PI_TARGET := pizero2w0.local

all: release

build: debug

.PHONY: clippy
clippy:
	cargo clippy --manifest-path Cargo.toml

debug: clippy
	cargo test
	cargo build

release: clippy
	cargo test --release
	cargo build --release

run:
	cargo run

test:
	cargo test

.PHONY: piclippy
piclippy:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross clippy --manifest-path Cargo.toml --release --target=aarch64-unknown-linux-gnu

.PHONY: pibuild
pibuild:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --manifest-path Cargo.toml --release --target=aarch64-unknown-linux-gnu

.PHONY: picopy
picopy: pibuild
	scp ../target/aarch64-unknown-linux-gnu/release/wimon andrew@$(PI_TARGET):~/wimon

.PHONY: ssh
ssh:
	ssh andrew@$(PI_TARGET)