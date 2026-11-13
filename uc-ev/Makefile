SHELL := /bin/sh
.SHELLFLAGS := -eu -c

APP_NAME := solverforge_ev
PORT ?= 7860
# Browser-test tooling. Playwright resolves from this app's own node_modules
# (`make install-e2e`), then from the bundle root's node_modules when the app
# sits inside the solverforge-usecases repository.
PLAYWRIGHT ?= $(firstword $(wildcard node_modules/.bin/playwright ../node_modules/.bin/playwright))
# Chromium: an explicit PLAYWRIGHT_CHROMIUM_EXECUTABLE wins, then the system
# /usr/bin/chromium; an empty value selects Playwright's bundled Chromium.
PLAYWRIGHT_CHROMIUM_EXECUTABLE ?= $(wildcard /usr/bin/chromium)
export PLAYWRIGHT_CHROMIUM_EXECUTABLE

.PHONY: help install-e2e require-node require-playwright build build-release run run-release \
	test test-rust test-frontend-syntax test-frontend test-e2e test-live-e2e test-slow test-one \
	fmt fmt-check clippy lint check ci-local release-ci docker-build docker-run pre-release clean

help:
	@echo "make run-release   run the app on :$(PORT)"
	@echo "make test          Rust, frontend, and browser tests (no road-network download)"
	@echo "make test-slow     full STANDARD solve on the live road network (release)"
	@echo "make test-live-e2e browser solve lifecycle on the live road network"
	@echo "make ci-local      fmt, clippy, release build, tests, Docker image"
	@echo "make pre-release   ci-local plus both live checks"
	@echo "make install-e2e   install the pinned Playwright"

build:
	cargo build --bin $(APP_NAME)

build-release:
	cargo build --release --bin $(APP_NAME)

run:
	PORT=$(PORT) cargo run --bin $(APP_NAME)

run-release:
	PORT=$(PORT) cargo run --release --bin $(APP_NAME)

test: test-rust test-frontend test-e2e

test-rust:
	cargo test --quiet

test-frontend-syntax:
	node --check static/app.js
	for file in static/app/*.mjs; do node --check "$$file"; done

test-frontend: test-frontend-syntax
	node --test tests/frontend/*.test.mjs

install-e2e: require-node
	npm ci --no-audit --no-fund
	@if [ -n "$(PLAYWRIGHT_CHROMIUM_EXECUTABLE)" ]; then \
		echo "Browser tests use Chromium at $(PLAYWRIGHT_CHROMIUM_EXECUTABLE)"; \
	else \
		echo "No system Chromium found; installing Playwright's bundled Chromium"; \
		node_modules/.bin/playwright install chromium; \
	fi

require-node:
	@command -v node >/dev/null 2>&1 || { echo "node is required for frontend and browser validation"; exit 1; }

require-playwright: require-node
	@if [ -z "$(PLAYWRIGHT)" ]; then \
		echo "Playwright is not installed: run make install-e2e first"; \
		exit 1; \
	fi

test-e2e: require-playwright build-release
	$(PLAYWRIGHT) test --config tests/e2e/playwright.config.js tests/e2e/app.spec.js

test-live-e2e: require-playwright build-release
	SOLVERFORGE_RUN_LIVE_TESTS=1 $(PLAYWRIGHT) test --config tests/e2e/playwright.config.js tests/e2e/app.live.spec.js

test-slow:
	cargo test --release --test slow_solve -- --ignored --nocapture

test-one:
	cargo test $(TEST) -- --nocapture

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets -- -D warnings

lint: fmt-check clippy test-frontend-syntax

check: lint test

docker-build:
	docker build -f Dockerfile -t solverforge-ev .

docker-run:
	docker run --rm -e PORT=$(PORT) -p $(PORT):$(PORT) solverforge-ev

ci-local: fmt-check clippy build-release test docker-build

pre-release: ci-local test-slow test-live-e2e

release-ci: ci-local

clean:
	cargo clean
	rm -rf test-results playwright-report
