# SolverForge FSR Makefile v1.0.1
# Rust + frontend syntax + Space-oriented local build system.

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

APP_NAME := solverforge_fsr
PACKAGE_NAME := solverforge-fsr
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
RUST_VERSION := 1.95+
PORT ?= 7860
DOCKER_IMAGE ?= $(PACKAGE_NAME)
DOCKER_CONTEXT ?= .
DOCKERFILE_PATH := Dockerfile
PLAYWRIGHT ?= ../node_modules/.bin/playwright

.PHONY: help doctor build build-release run run-release test test-rust \
        test-frontend-syntax test-e2e lint fmt fmt-check clippy check ci-local \
        space-ci space-build space-run docker-build docker-run pre-release \
        version clean watch require-node require-docker

.DEFAULT_GOAL := help

require-node:
	@command -v node >/dev/null 2>&1 || (printf "$(RED)$(CROSS) node is required for frontend validation$(RESET)\n" && exit 1)

require-docker:
	@command -v docker >/dev/null 2>&1 || (printf "$(RED)$(CROSS) docker is required for Space/Docker targets$(RESET)\n" && exit 1)

doctor:
	@printf "$(CYAN)$(BOLD)Environment Check$(RESET)\n\n"
	@missing=0; \
	if command -v cargo >/dev/null 2>&1; then \
		printf "$(GREEN)$(CHECK) cargo: $$(cargo --version)$(RESET)\n"; \
	else \
		printf "$(RED)$(CROSS) cargo not found$(RESET)\n"; missing=1; \
	fi; \
	if command -v rustc >/dev/null 2>&1; then \
		printf "$(GREEN)$(CHECK) rustc: $$(rustc --version)$(RESET)\n"; \
	else \
		printf "$(RED)$(CROSS) rustc not found$(RESET)\n"; missing=1; \
	fi; \
	if command -v node >/dev/null 2>&1; then \
		printf "$(GREEN)$(CHECK) node: $$(node --version)$(RESET)\n"; \
	else \
		printf "$(YELLOW)! node not found; frontend syntax validation will be unavailable$(RESET)\n"; \
	fi; \
	if command -v docker >/dev/null 2>&1; then \
		printf "$(GREEN)$(CHECK) docker: $$(docker --version)$(RESET)\n"; \
	else \
		printf "$(YELLOW)! docker not found; Space/Docker targets will be unavailable$(RESET)\n"; \
	fi; \
	printf "$(GRAY)Docker build context: $(DOCKER_CONTEXT)$(RESET)\n"; \
	printf "$(GRAY)Default app port: $(PORT)$(RESET)\n"; \
	if [ $$missing -ne 0 ]; then exit 1; fi

build:
	@printf "$(ARROW) Building $(PACKAGE_NAME)...\n"
	@cargo build --bin $(APP_NAME)

build-release:
	@printf "$(ARROW) Building release binary...\n"
	@cargo build --release --bin $(APP_NAME)

run:
	@printf "$(ARROW) Running $(PACKAGE_NAME) on port $(PORT)...\n"
	@PORT=$(PORT) cargo run --bin $(APP_NAME)

run-release:
	@printf "$(ARROW) Running release build on port $(PORT)...\n"
	@PORT=$(PORT) cargo run --release --bin $(APP_NAME)

test: test-rust test-frontend-syntax test-e2e
	@printf "\n$(GREEN)$(BOLD)$(CHECK) Standard validation passed$(RESET)\n\n"

test-rust:
	@printf "$(PROGRESS) Running cargo test --quiet...\n"
	@cargo test --quiet

test-frontend-syntax: require-node
	@printf "$(PROGRESS) Checking frontend module syntax...\n"
	@find static -name '*.js' -print0 | xargs -0 -n1 node --check

test-e2e: build-release require-node
	@printf "$(PROGRESS) Running Playwright browser tests...\n"
	@$(PLAYWRIGHT) test --config tests/e2e/playwright.config.js

fmt:
	@printf "$(PROGRESS) Formatting Rust code...\n"
	@cargo fmt

fmt-check:
	@printf "$(PROGRESS) Checking Rust formatting...\n"
	@cargo fmt --check

clippy:
	@printf "$(PROGRESS) Running clippy...\n"
	@cargo clippy --all-targets -- -D warnings

lint: fmt-check clippy test-frontend-syntax
	@printf "\n$(GREEN)$(BOLD)$(CHECK) Lint checks passed$(RESET)\n\n"

check: lint test

docker-build: require-docker
	@printf "$(PROGRESS) Building Docker image $(DOCKER_IMAGE)...\n"
	@docker build -f "$(DOCKERFILE_PATH)" -t "$(DOCKER_IMAGE)" "$(DOCKER_CONTEXT)"

docker-run: require-docker
	@printf "$(ARROW) Running $(DOCKER_IMAGE) on port $(PORT)...\n"
	@docker run --rm -it -e PORT=$(PORT) -p $(PORT):$(PORT) "$(DOCKER_IMAGE)"

space-build: docker-build

space-run: space-build
	@printf "$(GREEN)$(CHECK) Starting local container that mirrors the Space image$(RESET)\n"
	@$(MAKE) docker-run --no-print-directory PORT=$(PORT) DOCKER_IMAGE=$(DOCKER_IMAGE)

space-ci: ci-local

ci-local:
	@printf "$(CYAN)$(BOLD)Local Space Validation Pipeline$(RESET)\n\n"
	@printf "$(PROGRESS) Step 1/5: Format check...\n"
	@$(MAKE) fmt-check --no-print-directory
	@printf "$(PROGRESS) Step 2/5: Clippy...\n"
	@$(MAKE) clippy --no-print-directory
	@printf "$(PROGRESS) Step 3/5: Release build...\n"
	@$(MAKE) build-release --no-print-directory
	@printf "$(PROGRESS) Step 4/5: Standard test surface...\n"
	@$(MAKE) test --no-print-directory
	@printf "$(PROGRESS) Step 5/5: Docker/Space image build...\n"
	@$(MAKE) space-build --no-print-directory
	@printf "\n$(GREEN)$(BOLD)$(CHECK) LOCAL SPACE VALIDATION PASSED$(RESET)\n\n"

pre-release: ci-local
	@printf "$(GREEN)$(BOLD)$(CHECK) Ready for Hugging Face Space update$(RESET)\n\n"

version:
	@printf "$(CYAN)Current version:$(RESET) $(YELLOW)$(BOLD)$(VERSION)$(RESET)\n"
	@printf "$(CYAN)Default port:$(RESET) $(YELLOW)$(BOLD)$(PORT)$(RESET)\n"

clean:
	@printf "$(ARROW) Cleaning build artifacts...\n"
	@cargo clean

watch:
	@printf "$(ARROW) Watching and rerunning the app on port $(PORT)...\n"
	@cargo watch --version >/dev/null 2>&1 || \
		(printf "$(RED)$(CROSS) cargo-watch is required for make watch$(RESET)\n" && exit 1)
	@PORT=$(PORT) cargo watch -x "run --bin $(APP_NAME)"

help:
	@/bin/echo -e "$(CYAN)$(BOLD)Build & Run:$(RESET)"
	@/bin/echo -e "  $(GREEN)make build$(RESET)          - Build the app in debug mode"
	@/bin/echo -e "  $(GREEN)make build-release$(RESET)  - Build the app in release mode"
	@/bin/echo -e "  $(GREEN)make run$(RESET)            - Run locally on port $(PORT)"
	@/bin/echo -e "  $(GREEN)make run-release$(RESET)    - Run the release build on port $(PORT)"
	@/bin/echo -e ""
	@/bin/echo -e "$(CYAN)$(BOLD)Tests & Validation:$(RESET)"
	@/bin/echo -e "  $(GREEN)make test$(RESET)           - Run Rust, frontend syntax, and Playwright checks"
	@/bin/echo -e "  $(GREEN)make test-e2e$(RESET)       - Run Playwright browser tests"
	@/bin/echo -e "  $(GREEN)make lint$(RESET)           - Run fmt-check, clippy, and frontend syntax checks"
	@/bin/echo -e "  $(GREEN)make ci-local$(RESET)       - Run local Space validation pipeline"
	@/bin/echo -e "  $(GREEN)make pre-release$(RESET)    - Run all local Space readiness checks"
	@/bin/echo -e ""
	@/bin/echo -e "$(CYAN)$(BOLD)Space & Docker:$(RESET)"
	@/bin/echo -e "  $(GREEN)make space-build$(RESET)    - Build the Docker image used for Space deployment"
	@/bin/echo -e "  $(GREEN)make space-run$(RESET)      - Build and run that image locally on port $(PORT)"
	@/bin/echo -e "  $(GREEN)make docker-build$(RESET)   - Build the Docker image directly"
	@/bin/echo -e "  $(GREEN)make docker-run$(RESET)     - Run the Docker image directly"
	@/bin/echo -e ""
	@/bin/echo -e "$(CYAN)$(BOLD)Other:$(RESET)"
	@/bin/echo -e "  $(GREEN)make doctor$(RESET)         - Check local cargo/rustc/node readiness"
	@/bin/echo -e "  $(GREEN)make fmt$(RESET)            - Format Rust code"
	@/bin/echo -e "  $(GREEN)make version$(RESET)        - Show version and default port"
	@/bin/echo -e "  $(GREEN)make clean$(RESET)          - Clean build artifacts"
	@/bin/echo -e "  $(GREEN)make watch$(RESET)          - Watch source files and rerun the app"
	@/bin/echo -e ""
	@/bin/echo -e "$(GRAY)Rust version required: $(RUST_VERSION)$(RESET)"
	@/bin/echo -e "$(GRAY)Current version: v$(VERSION)$(RESET)"
