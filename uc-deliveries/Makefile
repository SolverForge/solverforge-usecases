# SolverForge Deliveries Makefile v1.0.1
# Rust + frontend + Space-oriented local build system.
#
# This app is validated for local development and Docker-based Hugging Face
# Space deployment. `ci-local` therefore includes a Docker image build.

SHELL := /bin/sh
.SHELLFLAGS := -eu -c
unexport BASH_FUNC_mc%%

# ============== Colors & Symbols ==============
GREEN := \033[92m
EMERALD := \033[38;2;16;185;129m
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

# ============== Project Metadata ==============
APP_NAME := solverforge_deliveries
PACKAGE_NAME := solverforge-deliveries
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
RUST_VERSION := 1.95+
PORT ?= 7860
DOCKER_IMAGE ?= $(PACKAGE_NAME)
DOCKER_CONTEXT ?= .
DOCKERFILE_PATH := Dockerfile
PLAYWRIGHT ?= ../node_modules/.bin/playwright

# ============== Phony Targets ==============
.PHONY: banner help doctor build build-release run run-release test test-rust \
        test-frontend-syntax test-frontend test-e2e test-live-road test-one lint fmt \
        fmt-check clippy check ci-local space-ci space-build space-run docker-build \
        docker-run pre-release version clean watch require-node require-docker

.DEFAULT_GOAL := help

# ============== Banner ==============
banner:
	@printf "$(EMERALD)$(BOLD)  ____        _                _____\n"
	@printf " / ___|  ___ | |_   _____ _ __|  ___|__  _ __ __ _  ___\n"
	@printf " \\___ \\\\ / _ \\\\| \\\\ \\\\ / / _ \\\\ '__| |_ / _ \\\\| '__/ _\` |/ _ \\\\\n"
	@printf "  ___) | (_) | |\\\\ V /  __/ |  |  _| (_) | | | (_| |  __/\n"
	@printf " |____/ \\\\___/|_| \\_/ \\___|_|  |_|  \\___/|_|  \\__, |\\___|\n"
	@printf "                                             |___/$(RESET)\n"
	@printf "  $(GRAY)v$(VERSION)$(RESET) $(EMERALD)Deliveries demo build system$(RESET)\n\n"

# ============== Environment Checks ==============
require-node:
	@command -v node >/dev/null 2>&1 || (printf "$(RED)$(CROSS) node is required for frontend validation$(RESET)\n" && exit 1)

require-docker:
	@command -v docker >/dev/null 2>&1 || (printf "$(RED)$(CROSS) docker is required for Space/Docker targets$(RESET)\n" && exit 1)

doctor: banner
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
		printf "$(RED)$(CROSS) node not found$(RESET)\n"; missing=1; \
	fi; \
	if command -v docker >/dev/null 2>&1; then \
		printf "$(GREEN)$(CHECK) docker: $$(docker --version)$(RESET)\n"; \
	else \
		printf "$(YELLOW)! docker not found; Space/Docker targets will be unavailable$(RESET)\n"; \
	fi; \
	printf "$(GRAY)Docker build context: $(DOCKER_CONTEXT)$(RESET)\n"; \
	printf "$(GRAY)Default app port: $(PORT)$(RESET)\n"; \
	if [ $$missing -ne 0 ]; then exit 1; fi
	@printf "\n"

# ============== Build & Run ==============
build: banner
	@printf "$(ARROW) $(BOLD)Building $(PACKAGE_NAME)...$(RESET)\n"
	@cargo build --bin $(APP_NAME) && \
		printf "$(GREEN)$(CHECK) Debug build successful$(RESET)\n\n" || \
		(printf "$(RED)$(CROSS) Debug build failed$(RESET)\n\n" && exit 1)

build-release: banner
	@printf "$(ARROW) $(BOLD)Building release binary...$(RESET)\n"
	@cargo build --release --bin $(APP_NAME) && \
		printf "$(GREEN)$(CHECK) Release build successful$(RESET)\n\n" || \
		(printf "$(RED)$(CROSS) Release build failed$(RESET)\n\n" && exit 1)

run:
	@printf "$(ARROW) Running $(PACKAGE_NAME) on port $(PORT)...\n"
	@PORT=$(PORT) cargo run --bin $(APP_NAME)

run-release:
	@printf "$(ARROW) Running release build on port $(PORT)...\n"
	@PORT=$(PORT) cargo run --release --bin $(APP_NAME)

# ============== Test Targets ==============
test: test-rust test-frontend test-e2e
	@printf "\n$(GREEN)$(BOLD)$(CHECK) Standard validation passed$(RESET)\n\n"

test-rust: banner
	@printf "$(ARROW) $(BOLD)Running cargo test --quiet...$(RESET)\n"
	@cargo test --quiet && \
		printf "\n$(GREEN)$(CHECK) Rust tests passed$(RESET)\n\n" || \
		(printf "\n$(RED)$(CROSS) Rust tests failed$(RESET)\n\n" && exit 1)

test-frontend-syntax: require-node
	@printf "$(PROGRESS) Checking frontend module syntax...\n"
	@find static/app -name '*.mjs' -print0 | xargs -0 -n1 node --check && \
		printf "$(GREEN)$(CHECK) Frontend syntax checks passed$(RESET)\n" || \
		(printf "$(RED)$(CROSS) Frontend syntax checks failed$(RESET)\n" && exit 1)

test-frontend: test-frontend-syntax
	@printf "$(PROGRESS) Running frontend tests...\n"
	@node --test tests/frontend_models.test.mjs && \
		printf "$(GREEN)$(CHECK) Frontend tests passed$(RESET)\n" || \
		(printf "$(RED)$(CROSS) Frontend tests failed$(RESET)\n" && exit 1)

test-e2e: build-release require-node
	@printf "$(PROGRESS) Running Playwright browser tests...\n"
	@$(PLAYWRIGHT) test --config tests/e2e/playwright.config.js && \
		printf "$(GREEN)$(CHECK) Playwright browser tests passed$(RESET)\n" || \
		(printf "$(RED)$(CROSS) Playwright browser tests failed$(RESET)\n" && exit 1)

test-live-road: banner
	@printf "$(ARROW) $(BOLD)Running live road-network route tests...$(RESET)\n"
	@SOLVERFORGE_RUN_LIVE_TESTS=1 cargo test live_demo_locations_are_mutually_reachable_when_enabled -- --nocapture && \
		SOLVERFORGE_RUN_LIVE_TESTS=1 cargo test road_network_job_routes_work_when_live_tests_are_enabled -- --nocapture && \
		printf "\n$(GREEN)$(CHECK) Live road-network test passed$(RESET)\n\n" || \
		(printf "\n$(RED)$(CROSS) Live road-network test failed$(RESET)\n\n" && exit 1)

test-one:
	@printf "$(PROGRESS) Running test: $(YELLOW)$(TEST)$(RESET)\n"
	@RUST_LOG=info cargo test $(TEST) -- --nocapture

# ============== Lint & Format ==============
fmt:
	@printf "$(PROGRESS) Formatting Rust code...\n"
	@cargo fmt
	@printf "$(GREEN)$(CHECK) Code formatted$(RESET)\n"

fmt-check:
	@printf "$(PROGRESS) Checking Rust formatting...\n"
	@cargo fmt --check && \
		printf "$(GREEN)$(CHECK) Formatting valid$(RESET)\n" || \
		(printf "$(RED)$(CROSS) Formatting issues found$(RESET)\n" && exit 1)

clippy:
	@printf "$(PROGRESS) Running clippy...\n"
	@cargo clippy --all-targets -- -D warnings && \
		printf "$(GREEN)$(CHECK) Clippy passed$(RESET)\n" || \
		(printf "$(RED)$(CROSS) Clippy warnings found$(RESET)\n" && exit 1)

lint: fmt-check clippy test-frontend-syntax
	@printf "\n$(GREEN)$(BOLD)$(CHECK) Lint checks passed$(RESET)\n\n"

check: lint test

# ============== Space & Docker ==============
docker-build: require-docker
	@printf "$(PROGRESS) Building Docker image $(DOCKER_IMAGE)...\n"
	@docker build -f "$(DOCKERFILE_PATH)" -t "$(DOCKER_IMAGE)" "$(DOCKER_CONTEXT)" && \
		printf "$(GREEN)$(CHECK) Docker image built$(RESET)\n" || \
		(printf "$(RED)$(CROSS) Docker build failed$(RESET)\n" && exit 1)

docker-run: require-docker
	@printf "$(ARROW) Running $(DOCKER_IMAGE) on port $(PORT)...\n"
	@docker run --rm -it -e PORT=$(PORT) -p $(PORT):$(PORT) "$(DOCKER_IMAGE)"

space-build: docker-build

space-run: space-build
	@printf "$(GREEN)$(CHECK) Starting local container that mirrors the Space image$(RESET)\n"
	@$(MAKE) docker-run --no-print-directory PORT=$(PORT) DOCKER_IMAGE=$(DOCKER_IMAGE)

space-ci: ci-local

# ============== CI & Release Validation ==============
ci-local: banner
	@printf "$(CYAN)$(BOLD)Local Validation Pipeline$(RESET)\n\n"
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

pre-release: banner
	@printf "$(CYAN)$(BOLD)Pre-Release Validation v$(VERSION)$(RESET)\n\n"
	@$(MAKE) ci-local --no-print-directory
	@printf "$(PROGRESS) Final step: live road-network smoke...\n"
	@$(MAKE) test-live-road --no-print-directory
	@printf "$(GREEN)$(BOLD)$(CHECK) Ready for publication or Space update$(RESET)\n\n"

# ============== Metadata & Cleanup ==============
version:
	@printf "$(CYAN)Current version:$(RESET) $(YELLOW)$(BOLD)$(VERSION)$(RESET)\n"
	@printf "$(CYAN)Default port:$(RESET) $(YELLOW)$(BOLD)$(PORT)$(RESET)\n"

clean:
	@printf "$(ARROW) Cleaning build artifacts...\n"
	@cargo clean
	@printf "$(GREEN)$(CHECK) Clean complete$(RESET)\n"

watch:
	@printf "$(ARROW) Watching and rerunning the app on port $(PORT)...\n"
	@cargo watch --version >/dev/null 2>&1 || \
		(printf "$(RED)$(CROSS) cargo-watch is required for make watch$(RESET)\n" && exit 1)
	@PORT=$(PORT) cargo watch -x "run --bin $(APP_NAME)"

# ============== Help ==============
help: banner
	@/bin/echo -e "$(CYAN)$(BOLD)Environment:$(RESET)"
	@/bin/echo -e "  $(GREEN)make doctor$(RESET)         - Check local cargo/rustc/node readiness"
	@/bin/echo -e ""
	@/bin/echo -e "$(CYAN)$(BOLD)Build & Run:$(RESET)"
	@/bin/echo -e "  $(GREEN)make build$(RESET)          - Build the app in debug mode"
	@/bin/echo -e "  $(GREEN)make build-release$(RESET)  - Build the app in release mode"
	@/bin/echo -e "  $(GREEN)make run$(RESET)            - Run locally on port $(PORT)"
	@/bin/echo -e "  $(GREEN)make run-release$(RESET)    - Run the release build on port $(PORT)"
	@/bin/echo -e ""
	@/bin/echo -e "$(CYAN)$(BOLD)Tests & Validation:$(RESET)"
	@/bin/echo -e "  $(GREEN)make test$(RESET)           - Run Rust, frontend, and Playwright tests"
	@/bin/echo -e "  $(GREEN)make test-rust$(RESET)      - Run Rust tests only"
	@/bin/echo -e "  $(GREEN)make test-frontend$(RESET)  - Run frontend syntax checks and tests"
	@/bin/echo -e "  $(GREEN)make test-e2e$(RESET)       - Run Playwright browser tests"
	@/bin/echo -e "  $(GREEN)make test-live-road$(RESET) - Run the live road-network smoke test"
	@/bin/echo -e "  $(GREEN)make test-one TEST=name$(RESET) - Run a specific Rust test with output"
	@/bin/echo -e "  $(GREEN)make lint$(RESET)           - Run fmt-check, clippy, and frontend syntax checks"
	@/bin/echo -e "  $(GREEN)make check$(RESET)          - Run lint plus standard tests"
	@/bin/echo -e "  $(GREEN)make ci-local$(RESET)       - Run local Space validation pipeline"
	@/bin/echo -e "  $(GREEN)make pre-release$(RESET)    - Run ci-local plus live road-network smoke"
	@/bin/echo -e ""
	@/bin/echo -e "$(CYAN)$(BOLD)Space & Docker:$(RESET)"
	@/bin/echo -e "  $(GREEN)make space-build$(RESET)    - Build the Docker image used for Space deployment"
	@/bin/echo -e "  $(GREEN)make space-run$(RESET)      - Build and run that image locally on port $(PORT)"
	@/bin/echo -e "  $(GREEN)make docker-build$(RESET)   - Build the Docker image directly"
	@/bin/echo -e "  $(GREEN)make docker-run$(RESET)     - Run the Docker image directly"
	@/bin/echo -e ""
	@/bin/echo -e "$(CYAN)$(BOLD)Other:$(RESET)"
	@/bin/echo -e "  $(GREEN)make fmt$(RESET)            - Format Rust code"
	@/bin/echo -e "  $(GREEN)make version$(RESET)        - Show version and default port"
	@/bin/echo -e "  $(GREEN)make clean$(RESET)          - Clean build artifacts"
	@/bin/echo -e "  $(GREEN)make watch$(RESET)          - Watch source files and rerun the app"
	@/bin/echo -e "  $(GREEN)make help$(RESET)           - Show this help message"
	@/bin/echo -e ""
	@/bin/echo -e "$(GRAY)Rust version required: $(RUST_VERSION)$(RESET)"
	@/bin/echo -e "$(GRAY)Current version: v$(VERSION)$(RESET)"
	@/bin/echo -e "$(GRAY)Default port: $(PORT)$(RESET)"
	@/bin/echo -e ""
