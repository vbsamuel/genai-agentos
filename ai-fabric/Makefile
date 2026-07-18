SHELL := /usr/bin/env bash

.PHONY: fmt check lint test verify run clean

fmt:
	cargo fmt --all

check:
	cargo check --workspace --all-targets

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-targets

verify: fmt check lint test

run:
	cargo run -p ai-fabricd -- --bind 127.0.0.1:8787 --state ./var/ai-fabric.redb

clean:
	cargo clean
	rm -rf ./var
