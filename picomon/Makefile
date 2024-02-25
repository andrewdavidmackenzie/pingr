all: run

run: build
	cargo run --release

build:
	cargo build --release
	@cd .. && cargo run -p configr -- target/thumbv6m-none-eabi/release/picomon