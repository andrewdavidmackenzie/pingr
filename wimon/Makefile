all: release

build: debug

debug:
	cargo clippy
	cargo test
	cargo build

release:
	cargo clippy
	cargo test
	cargo build --release

run:
	cargo run

test:
	cargo test