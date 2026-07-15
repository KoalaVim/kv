.PHONY: build install check clippy test fmt

build:
	cargo build --release

install:
	cargo install --path .

check:
	cargo check

clippy:
	cargo clippy -- -D warnings

test:
	cargo test

fmt:
	cargo fmt --check
