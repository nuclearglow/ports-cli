.DEFAULT_GOAL := help
SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c

CARGO ?= cargo
RELEASE_BRANCH ?= main
REMOTE ?= origin

CRATE_VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
MSRV := $(shell sed -n 's/^rust-version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TAG := v$(CRATE_VERSION)

# Load CARGO_REGISTRY_TOKEN from .envrc via direnv if the shell hasn't already.
ifeq ($(CARGO_REGISTRY_TOKEN),)
ifneq ($(and $(wildcard .envrc),$(shell command -v direnv)),)
PUBLISH_CARGO := direnv exec . $(CARGO)
endif
endif
PUBLISH_CARGO ?= $(CARGO)

##@ CI

.PHONY: ci
ci: fmt-check clippy test msrv ## Run all CI checks

.PHONY: fmt
fmt: ## Format the code
	$(CARGO) fmt

.PHONY: fmt-check
fmt-check: ## Check formatting
	$(CARGO) fmt --check

.PHONY: clippy
clippy: ## Run clippy, warnings are errors
	$(CARGO) clippy --all-targets --locked -- -D warnings

.PHONY: test
test: ## Run all tests
	$(CARGO) test --locked

.PHONY: msrv
msrv: ## Check the build with the minimum supported Rust version
	rustup toolchain list | grep -q '^$(MSRV)' || rustup toolchain install $(MSRV) --profile minimal
	$(CARGO) +$(MSRV) check --all-targets --locked

##@ Build

.PHONY: build
build: ## Build an optimized binary (target/release/ports)
	$(CARGO) build --release --locked

.PHONY: install
install: ## Install the binary from this checkout
	$(CARGO) install --path . --locked

##@ Release

.PHONY: version
version: ## Print the current version
	@echo $(CRATE_VERSION)

.PHONY: bump-patch bump-minor bump-major
bump-patch: ## Release a patch version (x.y.Z)
	@$(MAKE) --no-print-directory release-version VERSION=$$(echo $(CRATE_VERSION) | awk -F. '{print $$1"."$$2"."$$3+1}')
bump-minor: ## Release a minor version (x.Y.0)
	@$(MAKE) --no-print-directory release-version VERSION=$$(echo $(CRATE_VERSION) | awk -F. '{print $$1"."$$2+1".0"}')
bump-major: ## Release a major version (X.0.0)
	@$(MAKE) --no-print-directory release-version VERSION=$$(echo $(CRATE_VERSION) | awk -F. '{print $$1+1".0.0"}')

.PHONY: release-version
release-version: ## Set VERSION=x.y.z, update the changelog, commit and tag
	@[[ "$(VERSION)" =~ ^[0-9]+\.[0-9]+\.[0-9]+$$ ]] || { echo "error: usage: make release-version VERSION=x.y.z" >&2; exit 1; }
	@[[ "$(VERSION)" != "$(CRATE_VERSION)" ]] || { echo "error: version is already $(VERSION), use 'make tag'" >&2; exit 1; }
	@$(MAKE) --no-print-directory check-clean
	@if grep -q '^## Unreleased' CHANGELOG.md; then \
		sed -i 's/^## Unreleased.*/## $(VERSION)/' CHANGELOG.md; \
	elif ! grep -q '^## $(VERSION)$$' CHANGELOG.md; then \
		echo "error: add a '## Unreleased' or '## $(VERSION)' section to CHANGELOG.md first" >&2; exit 1; \
	fi
	sed -i '0,/^version = ".*"/s//version = "$(VERSION)"/' Cargo.toml
	$(CARGO) update --workspace --offline --quiet
	git add Cargo.toml Cargo.lock CHANGELOG.md
	git commit -m "chore: release v$(VERSION)"
	git tag -a v$(VERSION) -m "v$(VERSION)"
	@echo "Released v$(VERSION) locally. Run 'make publish' to publish it."

.PHONY: tag
tag: check-clean ## Tag the current version (if it was bumped by hand)
	@git rev-parse -q --verify "refs/tags/$(TAG)" >/dev/null && { echo "error: tag $(TAG) already exists" >&2; exit 1; } || true
	@grep -q '^## $(CRATE_VERSION)$$' CHANGELOG.md || { echo "error: CHANGELOG.md has no '## $(CRATE_VERSION)' section" >&2; exit 1; }
	git tag -a $(TAG) -m "$(TAG)"

.PHONY: publish-dry-run
publish-dry-run: ## Package and verify the crate without uploading
	$(CARGO) publish --dry-run --locked

.PHONY: publish
publish: check-clean check-release ci publish-dry-run ## Run CI, publish to crates.io and push branch + tag
	$(PUBLISH_CARGO) publish --locked
	git push $(REMOTE) $(RELEASE_BRANCH) $(TAG)

.PHONY: check-clean
check-clean:
	@git diff --quiet && git diff --cached --quiet || { echo "error: working tree has uncommitted changes" >&2; exit 1; }

.PHONY: check-release
check-release:
	@[[ "$$(git rev-parse --abbrev-ref HEAD)" == "$(RELEASE_BRANCH)" ]] || { echo "error: releases are published from '$(RELEASE_BRANCH)' (override with RELEASE_BRANCH=...)" >&2; exit 1; }
	@[[ "$$(git rev-parse -q --verify "refs/tags/$(TAG)^{commit}" || true)" == "$$(git rev-parse HEAD)" ]] || { echo "error: HEAD is not tagged $(TAG), run 'make bump-*' or 'make tag'" >&2; exit 1; }

##@ Help

.PHONY: help
help: ## Show this help
	@awk 'BEGIN {FS = ":.*## "} \
		/^##@/ {printf "\n\033[1m%s\033[0m\n", substr($$0, 5)} \
		/^[a-zA-Z_-]+:.*## / {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
