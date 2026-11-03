SHELL := /bin/sh
.SHELLFLAGS := -eu -c

APP_NAME := solverforge-flightcrew
PORT ?= 7860
# Browser-test tooling. Playwright resolves from this app's own node_modules
# (`make install-e2e`), then from the bundle root's node_modules when the app
# sits inside the solverforge-usecases repository.
PLAYWRIGHT ?= $(firstword $(wildcard node_modules/.bin/playwright ../node_modules/.bin/playwright))
# Chromium: an explicit PLAYWRIGHT_CHROMIUM_EXECUTABLE wins, then the system
# /usr/bin/chromium; an empty value selects Playwright's bundled Chromium.
PLAYWRIGHT_CHROMIUM_EXECUTABLE ?= $(wildcard /usr/bin/chromium)
export PLAYWRIGHT_CHROMIUM_EXECUTABLE

.PHONY: install-e2e require-playwright build build-release run run-release test test-rust test-frontend-syntax test-frontend test-e2e test-slow test-one fmt fmt-check clippy lint check ci-local release-ci docker-build docker-run pre-release

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
	node --check static/flightcrew-model.js

test-frontend: test-frontend-syntax
	node --test tests/frontend/*.test.js

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
	$(PLAYWRIGHT) test --config tests/e2e/playwright.config.js

test-slow:
	cargo test --release standard_demo_solves_to_feasible_terminal_state -- --ignored --nocapture

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
	docker build -f Dockerfile -t $(APP_NAME) .

docker-run:
	docker run --rm -e PORT=$(PORT) -p $(PORT):$(PORT) $(APP_NAME)

ci-local: fmt-check clippy build-release test docker-build

pre-release: ci-local test-slow

release-ci: ci-local
