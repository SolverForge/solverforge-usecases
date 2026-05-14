# SolverForge Lessons Makefile v0.1.0
# Rust + frontend + Space-oriented local build system.

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
APP_NAME := solverforge-lessons
PACKAGE_NAME := solverforge-lessons
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
RUST_VERSION := 1.95+
PORT ?= 7860
E2E_PORT ?= 7960
DOCKER_IMAGE ?= $(PACKAGE_NAME)
DOCKER_CONTEXT ?= .
DOCKERFILE_PATH := Dockerfile

# ============== Phony Targets ==============
.PHONY: banner help doctor build build-release run run-release test test-rust \
        test-frontend-syntax test-frontend test-e2e test-slow test-one lint fmt \
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
	@printf "  $(GRAY)v$(VERSION)$(RESET) $(EMERALD)Lessons demo build system$(RESET)\n\n"

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
	printf "$(GRAY)Browser smoke port: $(E2E_PORT)$(RESET)\n"; \
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
	@find static -name '*.js' -print0 | xargs -0 -n1 node --check && \
		printf "$(GREEN)$(CHECK) Frontend syntax checks passed$(RESET)\n" || \
		(printf "$(RED)$(CROSS) Frontend syntax checks failed$(RESET)\n" && exit 1)

test-frontend: test-frontend-syntax
	@printf "$(GREEN)$(CHECK) Frontend validation passed$(RESET)\n"

test-e2e: build-release require-node
	@printf "$(PROGRESS) Running browser smoke test on port $(E2E_PORT)...\n"
	@log=$$(mktemp); \
	PORT=$(E2E_PORT) target/release/$(APP_NAME) >"$$log" 2>&1 & \
	pid=$$!; \
	trap 'kill $$pid >/dev/null 2>&1 || true; rm -f "$$log"' EXIT INT TERM; \
	i=0; \
	until curl -fsS "http://127.0.0.1:$(E2E_PORT)/" >/dev/null 2>&1; do \
		i=$$((i + 1)); \
		if [ $$i -ge 60 ]; then \
			printf "$(RED)$(CROSS) app did not become ready$(RESET)\n"; \
			cat "$$log"; \
			exit 1; \
		fi; \
		sleep 0.2; \
	done; \
	node --input-type=module -e 'import { chromium } from "playwright"; const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || "/usr/bin/chromium", headless: true }); const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, colorScheme: "light" }); await page.goto("http://127.0.0.1:$(E2E_PORT)/", { waitUntil: "networkidle", timeout: 30000 }); const body = await page.locator("body").innerText(); if (!body.includes("SolverForge Lessons")) throw new Error("missing product title"); if (!body.includes("Cohort Timetables")) throw new Error("missing cohort timetable view"); if (body.includes("1W") || body.includes("2W") || body.includes("4W")) throw new Error("fixed lesson timetable should not render zoom presets"); if (body.includes("Room null") || body.includes("| null")) throw new Error("rendered null assignment text"); const nullAttrs = await page.locator("[role=\"null\"], [aria-label=\"null\"], [tabindex=\"null\"]").count(); if (nullAttrs !== 0) throw new Error(`found ${nullAttrs} null accessibility attributes`); await browser.close();'; \
	kill $$pid >/dev/null 2>&1 || true; \
	printf "$(GREEN)$(CHECK) Browser smoke test passed$(RESET)\n"

test-slow: banner
	@printf "$(ARROW) $(BOLD)Running large demo acceptance solve...$(RESET)\n"
	@cargo test large_demo_solves_to_feasible_progressing_schedule -- --ignored --nocapture && \
		printf "\n$(GREEN)$(CHECK) Slow acceptance solve passed$(RESET)\n\n" || \
		(printf "\n$(RED)$(CROSS) Slow acceptance solve failed$(RESET)\n\n" && exit 1)

test-one:
	@test -n "$(TEST)" || (printf "$(RED)$(CROSS) TEST=name is required$(RESET)\n" && exit 1)
	@printf "$(PROGRESS) Running test: $(YELLOW)$(TEST)$(RESET)\n"
	@RUST_LOG=info cargo test "$(TEST)" -- --nocapture

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

pre-release: banner
	@printf "$(CYAN)$(BOLD)Pre-Release Validation v$(VERSION)$(RESET)\n\n"
	@$(MAKE) ci-local --no-print-directory
	@printf "$(PROGRESS) Final step: slow acceptance solve...\n"
	@$(MAKE) test-slow --no-print-directory
	@printf "$(GREEN)$(BOLD)$(CHECK) Ready for publication or Space update$(RESET)\n\n"

# ============== Metadata & Cleanup ==============
version:
	@printf "$(CYAN)Current version:$(RESET) $(YELLOW)$(BOLD)$(VERSION)$(RESET)\n"
	@printf "$(CYAN)Default port:$(RESET) $(YELLOW)$(BOLD)$(PORT)$(RESET)\n"
	@printf "$(CYAN)Browser smoke port:$(RESET) $(YELLOW)$(BOLD)$(E2E_PORT)$(RESET)\n"

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
	@printf "$(CYAN)$(BOLD)Environment:$(RESET)\n"
	@printf "  $(GREEN)make doctor$(RESET)         - Check local cargo/rustc/node readiness\n"
	@printf "\n$(CYAN)$(BOLD)Build & Run:$(RESET)\n"
	@printf "  $(GREEN)make build$(RESET)          - Build the app in debug mode\n"
	@printf "  $(GREEN)make build-release$(RESET)  - Build the app in release mode\n"
	@printf "  $(GREEN)make run$(RESET)            - Run locally on port $(PORT)\n"
	@printf "  $(GREEN)make run-release$(RESET)    - Run the release build on port $(PORT)\n"
	@printf "\n$(CYAN)$(BOLD)Tests & Validation:$(RESET)\n"
	@printf "  $(GREEN)make test$(RESET)           - Run Rust, frontend syntax, and browser smoke checks\n"
	@printf "  $(GREEN)make test-rust$(RESET)      - Run Rust tests only\n"
	@printf "  $(GREEN)make test-frontend$(RESET)  - Run frontend syntax checks\n"
	@printf "  $(GREEN)make test-e2e$(RESET)       - Run a Playwright browser smoke check\n"
	@printf "  $(GREEN)make test-slow$(RESET)      - Run the ignored large-demo acceptance solve\n"
	@printf "  $(GREEN)make test-one TEST=name$(RESET) - Run a specific Rust test with output\n"
	@printf "  $(GREEN)make lint$(RESET)           - Run fmt-check, clippy, and frontend syntax checks\n"
	@printf "  $(GREEN)make check$(RESET)          - Run lint plus standard tests\n"
	@printf "  $(GREEN)make ci-local$(RESET)       - Run local Space validation pipeline\n"
	@printf "  $(GREEN)make pre-release$(RESET)    - Run ci-local plus slow acceptance solve\n"
	@printf "\n$(CYAN)$(BOLD)Space & Docker:$(RESET)\n"
	@printf "  $(GREEN)make space-build$(RESET)    - Build the Docker image used for Space deployment\n"
	@printf "  $(GREEN)make space-run$(RESET)      - Build and run that image locally on port $(PORT)\n"
	@printf "  $(GREEN)make docker-build$(RESET)   - Build the Docker image directly\n"
	@printf "  $(GREEN)make docker-run$(RESET)     - Run the Docker image directly\n"
	@printf "\n$(CYAN)$(BOLD)Other:$(RESET)\n"
	@printf "  $(GREEN)make fmt$(RESET)            - Format Rust code\n"
	@printf "  $(GREEN)make version$(RESET)        - Show version and default ports\n"
	@printf "  $(GREEN)make clean$(RESET)          - Clean build artifacts\n"
	@printf "  $(GREEN)make watch$(RESET)          - Watch source files and rerun the app\n"
	@printf "  $(GREEN)make help$(RESET)           - Show this help message\n"
	@printf "\n$(GRAY)Rust version required: $(RUST_VERSION)$(RESET)\n"
	@printf "$(GRAY)Current version: v$(VERSION)$(RESET)\n"
