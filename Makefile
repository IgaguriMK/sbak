CRATE_NAME:=sbak

DOC_OPTION:=--no-deps

export GIT_BRANCH:=$(shell git rev-parse --abbrev-ref HEAD)
export GIT_HASH:=$(shell git rev-parse HEAD)
export GIT_DIFF:=$(shell git diff HEAD | wc -l)
export GIT_UNTRACKED:=$(shell git status | grep 'Untracked' | wc -l)

$(info ===========================================================)
$(info GIT_BRANCH     $(GIT_BRANCH))
$(info GIT_HASH       $(GIT_HASH))
$(info GIT_DIFF=      $(GIT_DIFF))
$(info GIT_UNTRACKED  $(GIT_UNTRACKED))
$(info ===========================================================)


.PHONY: all
all: build check

.PHONY: build
build: soft-clean
	cargo build

.PHONY: release-build
release-build:
	cargo build --release

.PHONY: check
check: soft-clean
	cargo test
	cargo fmt -- --check
	cargo clippy -- -D warnings

.PHONY: doc
doc:
	cargo doc $(DOC_OPTION)

.PHONY: doc-open
doc-open:
	cargo doc $(DOC_OPTION) --open

.PHONY: release
release: check release-build

.PHONY: soft-clean
soft-clean:
	cargo clean -p $(CRATE_NAME)

.PHONY: clean
clean:
	cargo clean
	- rm $(CRATE_NAME).tar.gz
