# Etchwise developer targets. Run `make help` for the list.
# Run `make check` locally before pushing; there is no CI on every push.

SHELL := /bin/bash
export PATH := /opt/homebrew/opt/rustup/bin:$(HOME)/.cargo/bin:$(HOME)/source/reactvenv/bin:$(PATH)
# macOS make 3.81 runs simple recipes without a shell and looks them up with
# its own PATH, so resolve the tools once here.
CARGO := $(shell PATH='$(PATH)' command -v cargo || echo cargo)
BUN := $(shell PATH='$(PATH)' command -v bun || echo bun)

DESKTOP := apps/desktop
CORE := imager-core
# Hardware and Tauri wiring, exercised by the integration and e2e tests.
COVERAGE_IGNORE := (src/main\.rs|src-tauri/src/lib\.rs|imager-platform/src/macos/.*)$$

.DEFAULT_GOAL := help
.PHONY: help install dev build dmg test test-rust test-ts lint lint-rust lint-ts format fmt-check \
	coverage coverage-rust coverage-ts check clean

help:            ## List targets
	@grep -E '^[a-z0-9-]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*## "} {printf "  %-14s %s\n", $$1, $$2}'

install:         ## Install JS dependencies and the git hooks
	$(BUN) install

# Optional custom OS list for `make dev`, e.g. REPO=repos/intel-nuc.json.
REPO ?=
DEV_ARGS := $(if $(REPO),-- -- --repo $(abspath $(REPO)),)

dev:             ## Run the desktop app with hot reload (REPO=file.json for a custom OS list)
	cd $(DESKTOP) && $(BUN) run tauri dev $(DEV_ARGS)

build:           ## Build the release .app
	cd $(DESKTOP) && $(BUN) run tauri build --bundles app

dmg:             ## Build the release .app and .dmg
	cd $(DESKTOP) && $(BUN) run tauri build

test: test-rust test-ts ## Run every test suite

test-rust:       ## Rust unit and integration tests
	$(CARGO) test --workspace

test-ts:         ## UI unit tests (vitest)
	cd $(DESKTOP) && $(BUN) run test

lint: lint-rust lint-ts ## Lint everything

lint-rust:       ## clippy with warnings as errors
	$(CARGO) clippy --workspace --all-targets -- -D warnings

lint-ts:         ## Type-check, eslint and prettier
	cd $(DESKTOP) && $(BUN) run typecheck && $(BUN) run lint

format:          ## Format Rust and TS sources
	$(CARGO) fmt --all
	cd $(DESKTOP) && $(BUN) run format

fmt-check:       ## Check formatting without writing
	$(CARGO) fmt --all -- --check
	cd $(DESKTOP) && $(BUN) x prettier --check .

coverage: coverage-rust coverage-ts ## Coverage gates (90%)

coverage-rust:   ## imager-core line coverage, fails under 90%
	$(CARGO) llvm-cov --workspace --ignore-filename-regex '$(COVERAGE_IGNORE)' --fail-under-lines 90

coverage-ts:     ## UI coverage, fails under 90%
	cd $(DESKTOP) && $(BUN) run coverage

check: fmt-check lint test coverage ## Everything CI would run

clean:           ## Remove Rust and JS build output (the disk is tight)
	$(CARGO) clean
	rm -rf $(DESKTOP)/dist $(DESKTOP)/coverage
