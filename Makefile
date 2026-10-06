.PHONY: all build test fmt check clean

all: check test build

build:
	cargo build --target wasm32-unknown-unknown --release

test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt --all -- --check

clean:
	cargo clean
