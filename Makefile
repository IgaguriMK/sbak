CRATE_NAME:=sbak

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
	cargo clippy -- -D warnings

.PHONY: release
release: check pack

.PHONY: pack
pack: release-build
	mkdir $(CRATE_NAME)
	cp target/release/$(CRATE_NAME) ./$(CRATE_NAME)/
	tar czf $(CRATE_NAME).tar.gz ./$(CRATE_NAME)
	rm -r $(CRATE_NAME)

.PHONY: soft-clean
soft-clean:
	cargo clean -p $(CRATE_NAME)

.PHONY: clean
clean:
	cargo clean
	- rm $(CRATE_NAME).tar.gz
