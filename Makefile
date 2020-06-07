CRATE_NAME:=sbak

.PHONY: check
check: soft-clean
	cargo fmt -- --check
	cargo test
	cargo clippy -- -D warnings

.PHONY: soft-clean
soft-clean:
	cargo clean -p $(CRATE_NAME)

