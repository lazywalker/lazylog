.PHONY: check test examples build-examples

check:
	cargo fmt --all -- --check
	cargo clippy --all-features -- -D warnings 

build:
	cargo build --all-features

test:
	cargo test --all-features
	cargo test

cov:
	cargo llvm-cov test -q --all-features

examples:
	cargo run --example basic --all-features
	cargo run --example json_logging --all-features
	cargo run --example rotation --all-features