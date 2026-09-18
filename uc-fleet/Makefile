# SolverForge Fleet Makefile
# Rust + frontend + repository-local Docker build system.

SHELL := /bin/sh
.SHELLFLAGS := -eu -c
unexport BASH_FUNC_mc%%

GREEN := \033[92m
CYAN := \033[96m
YELLOW := \033[93m
RED := \033[91m
GRAY := \033[90m
BOLD := \033[1m
RESET := \033[0m

CHECK := OK
CROSS := FAIL
ARROW := =>
PROGRESS := ..

APP_NAME := solverforge-fleet
PACKAGE_NAME := solverforge-fleet
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
RELEASE_TAG := $(PACKAGE_NAME)@$(VERSION)
RUST_VERSION := 1.95+
PORT ?= 7860
E2E_PORT ?= 7961
DOCKER_IMAGE ?= $(PACKAGE_NAME)
DOCKER_CONTEXT ?= .
DOCKERFILE_PATH := Dockerfile

.PHONY: banner help doctor build build-release run run-release test test-rust \
        test-frontend-syntax test-frontend test-e2e test-slow test-one lint fmt \
        fmt-check clippy check ci-local docker-build docker-run pre-release \
        release-ci release-info version clean watch require-node require-docker

.DEFAULT_GOAL := help

banner:
	@printf "$(CYAN)$(BOLD)SolverForge Fleet$(RESET) $(GRAY)v$(VERSION)$(RESET)\n\n"

require-node:
	@command -v node >/dev/null 2>&1 || (printf "$(RED)$(CROSS) node is required for frontend validation$(RESET)\n" && exit 1)

require-docker:
	@command -v docker >/dev/null 2>&1 || (printf "$(RED)$(CROSS) docker is required for Docker targets$(RESET)\n" && exit 1)

doctor: banner
	@missing=0; \
	for tool in cargo rustc node; do \
		if command -v $$tool >/dev/null 2>&1; then \
			printf "$(GREEN)$(CHECK) %s: %s$(RESET)\n" "$$tool" "$$($$tool --version)"; \
		else \
			printf "$(RED)$(CROSS) %s not found$(RESET)\n" "$$tool"; missing=1; \
		fi; \
	done; \
	if command -v docker >/dev/null 2>&1; then \
		printf "$(GREEN)$(CHECK) docker: $$(docker --version)$(RESET)\n"; \
	else \
		printf "$(YELLOW)! docker not found; Docker targets are unavailable$(RESET)\n"; \
	fi; \
	if [ $$missing -ne 0 ]; then exit 1; fi

build: banner
	@cargo build --bin $(APP_NAME)
	@printf "$(GREEN)$(CHECK) Debug build passed$(RESET)\n"

build-release: banner
	@cargo build --release --bin $(APP_NAME)
	@printf "$(GREEN)$(CHECK) Release build passed$(RESET)\n"

run:
	@PORT=$(PORT) cargo run --bin $(APP_NAME)

run-release:
	@PORT=$(PORT) cargo run --release --bin $(APP_NAME)

test: test-rust test-frontend test-e2e
	@printf "\n$(GREEN)$(BOLD)$(CHECK) Standard validation passed$(RESET)\n"

test-rust: banner
	@cargo test --quiet
	@printf "$(GREEN)$(CHECK) Rust tests passed$(RESET)\n"

test-frontend-syntax: require-node
	@printf "$(PROGRESS) Checking frontend script syntax...\n"
	@for file in static/app.js static/fleet/*.js; do node --check "$$file"; done
	@printf "$(GREEN)$(CHECK) Frontend syntax passed$(RESET)\n"

test-frontend: test-frontend-syntax
	@printf "$(PROGRESS) Running frontend Node tests...\n"
	@node --test tests/frontend/*.test.js
	@printf "$(GREEN)$(CHECK) Frontend Node tests passed$(RESET)\n"

test-e2e: build-release require-node
	@printf "$(PROGRESS) Running browser smoke on port $(E2E_PORT)...\n"
	@log=$$(mktemp); \
	PORT=$(E2E_PORT) target/release/$(APP_NAME) >"$$log" 2>&1 & \
	pid=$$!; \
	trap 'kill $$pid >/dev/null 2>&1 || true; rm -f "$$log"' EXIT INT TERM; \
	i=0; \
	until curl -fsS "http://127.0.0.1:$(E2E_PORT)/health" >/dev/null 2>&1; do \
		i=$$((i + 1)); \
		if [ $$i -ge 100 ]; then cat "$$log"; exit 1; fi; \
		sleep 0.2; \
	done; \
	node --input-type=module -e 'import { chromium } from "playwright"; const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || "/usr/bin/chromium", headless: true }); const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, colorScheme: "light" }); const errors = []; page.on("pageerror", error => errors.push(error.message)); page.on("console", message => { if (message.type() === "error") errors.push(message.text()); }); await page.goto("http://127.0.0.1:$(E2E_PORT)/", { waitUntil: "networkidle", timeout: 30000 }); await page.locator("#sf-app").waitFor({ state: "visible", timeout: 30000 }); 	await page.getByText("Vessel schedule", { exact: true }).waitFor({ timeout: 30000 }); const terms = ["solverforge fleet", "vessel schedule", "dock occupancy", "unassigned", "disruptions"]; try { await page.waitForFunction(expected => expected.every(text => document.body.textContent.toLowerCase().includes(text)), terms, { timeout: 30000 }); } catch { const visible = (await page.locator("body").innerText()).replaceAll("\n", " ").slice(0, 500); throw new Error("workspace did not render: " + errors.join(" | ") + " visible=" + visible); } const body = (await page.locator("body").innerText()).toLowerCase(); if (body.includes("undefined")) throw new Error("rendered undefined text"); const nullAttrs = await page.locator("[role=\"null\"], [aria-label=\"null\"], [tabindex=\"null\"]").count(); if (nullAttrs !== 0) throw new Error("found " + nullAttrs + " null accessibility attributes"); await browser.close();'; \
	kill $$pid >/dev/null 2>&1 || true
	@printf "$(GREEN)$(CHECK) Browser smoke passed$(RESET)\n"

test-slow: banner
	@printf "$(ARROW) Running full BASELINE_FULL acceptance solve...\n"
	@cargo test --release baseline_full_solves_to_hard_feasible_terminal_state -- --ignored --nocapture

test-one:
	@test -n "$(TEST)" || (printf "$(RED)$(CROSS) TEST=name is required$(RESET)\n" && exit 1)
	@RUST_LOG=info cargo test "$(TEST)" -- --nocapture

fmt:
	@cargo fmt

fmt-check:
	@cargo fmt --check

clippy:
	@cargo clippy --all-targets -- -D warnings

lint: fmt-check clippy test-frontend-syntax

check: lint test

docker-build: require-docker
	@docker build -f "$(DOCKERFILE_PATH)" -t "$(DOCKER_IMAGE)" "$(DOCKER_CONTEXT)"
	@printf "$(GREEN)$(CHECK) Docker image built$(RESET)\n"

docker-run: require-docker
	@docker run --rm -it -e PORT=$(PORT) -p $(PORT):$(PORT) "$(DOCKER_IMAGE)"

ci-local: banner
	@printf "$(PROGRESS) 1/5 format\n"
	@$(MAKE) fmt-check --no-print-directory
	@printf "$(PROGRESS) 2/5 clippy\n"
	@$(MAKE) clippy --no-print-directory
	@printf "$(PROGRESS) 3/5 release build\n"
	@$(MAKE) build-release --no-print-directory
	@printf "$(PROGRESS) 4/5 standard tests\n"
	@$(MAKE) test --no-print-directory
	@printf "$(PROGRESS) 5/5 Docker image\n"
	@$(MAKE) docker-build --no-print-directory
	@printf "$(GREEN)$(BOLD)$(CHECK) Full local validation passed$(RESET)\n"

pre-release: ci-local test-slow
	@printf "$(GREEN)$(BOLD)$(CHECK) Pre-release validation passed$(RESET)\n"

release-ci: ci-local

release-info:
	@printf "Package: $(PACKAGE_NAME)\nVersion: $(VERSION)\nRelease tag: $(RELEASE_TAG)\n"

version:
	@printf "$(VERSION)\n"

clean:
	@cargo clean

watch:
	@cargo watch --version >/dev/null 2>&1 || (printf "$(RED)$(CROSS) cargo-watch is required$(RESET)\n" && exit 1)
	@PORT=$(PORT) cargo watch -x "run --bin $(APP_NAME)"

help: banner
	@printf "$(CYAN)Environment$(RESET)\n  make doctor\n"
	@printf "$(CYAN)Build and run$(RESET)\n  make build\n  make build-release\n  make run\n  make run-release\n"
	@printf "$(CYAN)Validation$(RESET)\n  make test\n  make test-rust\n  make test-frontend\n  make test-e2e\n  make test-slow\n  make test-one TEST=name\n  make lint\n  make ci-local\n  make pre-release\n"
	@printf "$(CYAN)Docker$(RESET)\n  make docker-build\n  make docker-run\n"
	@printf "$(CYAN)Metadata$(RESET)\n  make release-info\n  make version\n"
	@printf "$(GRAY)Rust $(RUST_VERSION); default port $(PORT); browser port $(E2E_PORT)$(RESET)\n"
