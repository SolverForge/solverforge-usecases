SHELL := /bin/sh
.SHELLFLAGS := -eu -c

APP_NAME := solverforge-orders
PORT ?= 7860
PLAYWRIGHT ?= ../node_modules/.bin/playwright

.PHONY: build build-release run run-release test test-rust test-frontend-syntax test-frontend test-e2e test-slow test-one fmt fmt-check clippy lint check ci-local release-ci docker-build docker-run pre-release

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
	node --check static/app/main.mjs
	node --check static/app/orders-model.mjs

test-frontend: test-frontend-syntax
	node --test tests/frontend/*.test.mjs

test-e2e: build-release
	$(PLAYWRIGHT) test --config tests/e2e/playwright.config.js

test-slow:
	cargo test standard_demo_solves_to_zero_hard_with_every_step_assigned_once -- --ignored --nocapture

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
