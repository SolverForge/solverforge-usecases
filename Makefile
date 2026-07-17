# SolverForge Use-Case Bundle Makefile
# Repo-root orchestration for the official uc-* publication apps.

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
VERSION := bundle
RUST_VERSION := 1.95+
OPEN_SOURCE_APPS := uc-deliveries uc-fsr uc-hospital uc-lessons
APPS ?= $(OPEN_SOURCE_APPS)
APP ?= uc-lessons
PORT ?= 7860
USECASE_SOURCE_ROOT ?= ../use-cases
RELEASE_AS ?=
RELEASE_VERSION ?=
PREPARED ?=
TAG ?=
PUBLISH_BRANCH ?= main
PUBLISH_REMOTE ?=

# ============== Phony Targets ==============
.PHONY: banner help list doctor require-node require-docker install-e2e browser-setup \
        verify-metadata verify-imports import-usecases build build-release run run-release \
        test test-all test-rust test-frontend-syntax test-e2e test-one test-release-tools lint fmt fmt-check \
        clippy check ci-local space-ci space-build space-run docker-build docker-run \
        pre-release release-usecase release-usecase-dry-run release-ci verify-release-tag \
        publish-usecase publish-usecase-dry-run publish-usecases publish-usecases-dry-run \
        version clean app-target

# ============== Default Target ==============
.DEFAULT_GOAL := help

# ============== Banner ==============
banner:
	@printf "$(EMERALD)$(BOLD)  ____        _                _____\n"
	@printf " / ___|  ___ | |_   _____ _ __|  ___|__  _ __ __ _  ___\n"
	@printf " \\___ \\\\ / _ \\\\| \\\\ \\\\ / / _ \\\\ '__| |_ / _ \\\\| '__/ _\` |/ _ \\\\\n"
	@printf "  ___) | (_) | |\\\\ V /  __/ |  |  _| (_) | | | (_| |  __/\n"
	@printf " |____/ \\\\___/|_| \\_/ \\___|_|  |_|  \\___/|_|  \\__, |\\___|\n"
	@printf "                                             |___/$(RESET)\n"
	@printf "  $(GRAY)$(VERSION)$(RESET) $(EMERALD)Use-Case Bundle Build System$(RESET)\n\n"

# ============== Environment Checks ==============
require-node:
	@command -v node >/dev/null 2>&1 || (printf "$(RED)$(CROSS) node is required for frontend/browser validation$(RESET)\n" && exit 1)

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
		printf "$(YELLOW)! node not found; frontend/browser targets will be unavailable$(RESET)\n"; \
	fi; \
	if command -v docker >/dev/null 2>&1; then \
		printf "$(GREEN)$(CHECK) docker: $$(docker --version)$(RESET)\n"; \
	else \
		printf "$(YELLOW)! docker not found; Space/Docker targets will be unavailable$(RESET)\n"; \
	fi; \
	if [ -d "$(USECASE_SOURCE_ROOT)" ]; then \
		printf "$(GREEN)$(CHECK) import source root: $(USECASE_SOURCE_ROOT)$(RESET)\n"; \
	else \
		printf "$(YELLOW)! import source root not found: $(USECASE_SOURCE_ROOT)$(RESET)\n"; \
		printf "$(GRAY)  set USECASE_SOURCE_ROOT=/path/to/use-cases for source-backed import drift checks$(RESET)\n"; \
	fi; \
	printf "$(GRAY)Official apps: $(OPEN_SOURCE_APPS)$(RESET)\n"; \
	printf "$(GRAY)Default APP: $(APP)$(RESET)\n"; \
	printf "$(GRAY)Default port: $(PORT)$(RESET)\n"; \
	if [ $$missing -ne 0 ]; then exit 1; fi
	@printf "\n"

install-e2e: require-node
	@printf "$(PROGRESS) Installing root Playwright dependencies...\n"
	@npm ci
	@printf "$(PROGRESS) Installing Chromium for browser checks...\n"
	@npx playwright install chromium
	@printf "$(GREEN)$(CHECK) Browser test dependencies installed$(RESET)\n"

browser-setup: install-e2e

# ============== Bundle Metadata ==============
list: banner
	@printf "$(CYAN)$(BOLD)Official Use Cases$(RESET)\n\n"
	@for app in $(OPEN_SOURCE_APPS); do printf "$(PROGRESS) %s\n" "$$app"; done
	@printf "\n"

verify-metadata: banner
	@printf "$(PROGRESS) Verifying bundle metadata and required app surfaces...\n"
	@bash scripts/verify-metadata.sh
	@printf "$(GREEN)$(CHECK) Metadata verified$(RESET)\n"

verify-imports: banner
	@printf "$(PROGRESS) Comparing imported apps against $(USECASE_SOURCE_ROOT) sources...\n"
	@USECASE_SOURCE_ROOT="$(USECASE_SOURCE_ROOT)" bash scripts/verify-imports.sh
	@printf "$(GREEN)$(CHECK) Import drift check completed$(RESET)\n"

import-usecases: banner
	@printf "$(PROGRESS) Refreshing official app directories from $(USECASE_SOURCE_ROOT)...\n"
	@USECASE_SOURCE_ROOT="$(USECASE_SOURCE_ROOT)" bash scripts/import-usecases.sh
	@printf "$(GREEN)$(CHECK) Imports refreshed$(RESET)\n"

# ============== Build & Run ==============
build: banner
	@$(MAKE) app-target TARGET=build --no-print-directory

build-release: banner
	@$(MAKE) app-target TARGET=build-release --no-print-directory

run:
	@test -d "$(APP)" || (printf "$(RED)$(CROSS) Unknown APP=$(APP)$(RESET)\n" && exit 1)
	@test -f "$(APP)/Makefile" || (printf "$(RED)$(CROSS) $(APP) is missing Makefile$(RESET)\n" && exit 1)
	@$(MAKE) -C "$(APP)" run PORT="$(PORT)" --no-print-directory

run-release:
	@test -d "$(APP)" || (printf "$(RED)$(CROSS) Unknown APP=$(APP)$(RESET)\n" && exit 1)
	@test -f "$(APP)/Makefile" || (printf "$(RED)$(CROSS) $(APP) is missing Makefile$(RESET)\n" && exit 1)
	@$(MAKE) -C "$(APP)" run-release PORT="$(PORT)" --no-print-directory

# ============== Test Targets ==============
test: test-all

test-all: banner
	@$(MAKE) app-target TARGET=test --no-print-directory

test-rust: banner
	@$(MAKE) app-target TARGET=test-rust --no-print-directory

test-frontend-syntax: banner require-node
	@$(MAKE) app-target TARGET=test-frontend-syntax --no-print-directory

test-e2e: banner require-node
	@$(MAKE) app-target TARGET=test-e2e --no-print-directory

test-release-tools: require-node
	@npm run test:release-tools

test-one:
	@test -n "$(TEST)" || (printf "$(RED)$(CROSS) TEST=name is required$(RESET)\n" && exit 1)
	@test -d "$(APP)" || (printf "$(RED)$(CROSS) Unknown APP=$(APP)$(RESET)\n" && exit 1)
	@if [ -f "$(APP)/Makefile" ]; then \
		$(MAKE) -C "$(APP)" test-one TEST="$(TEST)" --no-print-directory; \
	else \
		printf "$(PROGRESS) Running $(TEST) in $(APP)...\n"; \
		(cd "$(APP)" && RUST_LOG=info cargo test "$(TEST)" -- --nocapture); \
	fi

# ============== Lint & Format ==============
lint: banner verify-metadata verify-imports fmt-check clippy test-frontend-syntax
	@printf "\n$(GREEN)$(BOLD)$(CHECK) Bundle lint checks passed$(RESET)\n\n"

fmt: banner
	@$(MAKE) app-target TARGET=fmt --no-print-directory

fmt-check: banner
	@$(MAKE) app-target TARGET=fmt-check --no-print-directory

clippy: banner
	@$(MAKE) app-target TARGET=clippy --no-print-directory

check: lint test

# ============== CI & Space Validation ==============
ci-local: banner
	@printf "$(CYAN)$(BOLD)Local CI Simulation$(RESET)\n\n"
	@printf "$(PROGRESS) Step 1/7: Metadata verifier...\n"
	@$(MAKE) verify-metadata --no-print-directory
	@printf "$(PROGRESS) Step 2/7: Import drift verifier...\n"
	@$(MAKE) verify-imports --no-print-directory
	@printf "$(PROGRESS) Step 3/7: Format check...\n"
	@$(MAKE) fmt-check --no-print-directory
	@printf "$(PROGRESS) Step 4/7: Clippy...\n"
	@$(MAKE) clippy --no-print-directory
	@printf "$(PROGRESS) Step 5/7: Frontend syntax...\n"
	@$(MAKE) test-frontend-syntax --no-print-directory
	@printf "$(PROGRESS) Step 6/7: Release tooling tests...\n"
	@$(MAKE) test-release-tools --no-print-directory
	@printf "$(PROGRESS) Step 7/7: Standard app tests...\n"
	@$(MAKE) test-all --no-print-directory
	@printf "\n$(GREEN)$(BOLD)$(CHECK) LOCAL CI SIMULATION PASSED$(RESET)\n\n"

space-ci: banner
	@$(MAKE) app-target TARGET=ci-local --no-print-directory

space-build: docker-build

space-run: docker-run

docker-build: banner require-docker
	@$(MAKE) app-target TARGET=docker-build --no-print-directory

docker-run: require-docker
	@test -d "$(APP)" || (printf "$(RED)$(CROSS) Unknown APP=$(APP)$(RESET)\n" && exit 1)
	@test -f "$(APP)/Makefile" || (printf "$(RED)$(CROSS) $(APP) is missing Makefile$(RESET)\n" && exit 1)
	@$(MAKE) -C "$(APP)" docker-run PORT="$(PORT)" --no-print-directory

pre-release: banner
	@printf "$(CYAN)$(BOLD)Pre-Release Validation$(RESET)\n\n"
	@$(MAKE) ci-local --no-print-directory
	@printf "$(PROGRESS) Building all Space images...\n"
	@$(MAKE) docker-build --no-print-directory
	@printf "\n$(GREEN)$(BOLD)$(CHECK) Ready for use-case publication$(RESET)\n\n"

# ============== App Release Flow ==============
release-usecase: banner require-node
	@test -d "$(APP)" || (printf "$(RED)$(CROSS) Unknown APP=$(APP)$(RESET)\n" && exit 1)
	@args="--app $(APP)"; \
	if [ -n "$(RELEASE_AS)" ]; then args="$$args --release-as $(RELEASE_AS)"; fi; \
	if [ -n "$(PREPARED)" ]; then args="$$args --prepared"; fi; \
	printf "$(PROGRESS) Cutting app-scoped release for $(APP)...\n"; \
	npm run release:usecase -- $$args

release-usecase-dry-run: banner require-node
	@test -d "$(APP)" || (printf "$(RED)$(CROSS) Unknown APP=$(APP)$(RESET)\n" && exit 1)
	@args="--app $(APP) --dry-run"; \
	if [ -n "$(RELEASE_AS)" ]; then args="$$args --release-as $(RELEASE_AS)"; fi; \
	if [ -n "$(PREPARED)" ]; then args="$$args --prepared"; fi; \
	printf "$(PROGRESS) Previewing app-scoped release for $(APP)...\n"; \
	npm run release:usecase -- $$args

verify-release-tag: require-node
	@test -n "$(TAG)" || (printf "$(RED)$(CROSS) TAG=solverforge-app@x.y.z is required$(RESET)\n" && exit 1)
	@npm run verify:release-tag -- "$(TAG)"

release-ci: require-node
	@test -d "$(APP)" || (printf "$(RED)$(CROSS) Unknown APP=$(APP)$(RESET)\n" && exit 1)
	@if [ -n "$(RELEASE_VERSION)" ]; then \
		package=$$(printf '%s\n' "$(APP)" | sed 's/^uc-/solverforge-/'); \
		npm run verify:release-tag -- "$${package}@$(RELEASE_VERSION)"; \
	else \
		printf "$(YELLOW)! RELEASE_VERSION not set; skipping tag/Cargo/changelog consistency check$(RESET)\n"; \
	fi
	@$(MAKE) -C "$(APP)" release-ci --no-print-directory

# ============== Use-Case Publication ==============
publish-usecase-dry-run: banner require-node
	@test -n "$(TAG)" || (printf "$(RED)$(CROSS) TAG=solverforge-app@x.y.z is required$(RESET)\n" && exit 1)
	@npm run publish:usecases -- --tag "$(TAG)" --branch "$(PUBLISH_BRANCH)" $(if $(PUBLISH_REMOTE),--remote "$(PUBLISH_REMOTE)") --dry-run

publish-usecase: banner require-node
	@test -n "$(TAG)" || (printf "$(RED)$(CROSS) TAG=solverforge-app@x.y.z is required$(RESET)\n" && exit 1)
	@npm run publish:usecases -- --tag "$(TAG)" --branch "$(PUBLISH_BRANCH)" $(if $(PUBLISH_REMOTE),--remote "$(PUBLISH_REMOTE)")

publish-usecases-dry-run: banner require-node
	@npm run publish:usecases -- --all --branch "$(PUBLISH_BRANCH)" $(if $(PUBLISH_REMOTE),--remote "$(PUBLISH_REMOTE)") --dry-run

publish-usecases: banner require-node
	@npm run publish:usecases -- --all --branch "$(PUBLISH_BRANCH)" $(if $(PUBLISH_REMOTE),--remote "$(PUBLISH_REMOTE)")

# ============== Metadata & Cleanup ==============
version: banner
	@printf "$(CYAN)Bundle version:$(RESET) $(YELLOW)$(BOLD)$(VERSION)$(RESET)\n"
	@printf "$(CYAN)Rust version required:$(RESET) $(YELLOW)$(BOLD)$(RUST_VERSION)$(RESET)\n"
	@printf "$(CYAN)Official apps:$(RESET) $(YELLOW)$(BOLD)$(OPEN_SOURCE_APPS)$(RESET)\n"
	@printf "$(CYAN)Import source root:$(RESET) $(YELLOW)$(BOLD)$(USECASE_SOURCE_ROOT)$(RESET)\n"
	@printf "$(CYAN)Release tag format:$(RESET) $(YELLOW)$(BOLD)solverforge-<app>@<version>$(RESET)\n"

clean: banner
	@$(MAKE) app-target TARGET=clean --no-print-directory
	@rm -rf test-results
	@printf "$(GREEN)$(CHECK) Root test artifacts cleaned$(RESET)\n"

# ============== App Dispatch ==============
app-target:
	@test -n "$(TARGET)" || (printf "$(RED)$(CROSS) TARGET is required$(RESET)\n" && exit 1)
	@for app in $(APPS); do \
		test -d "$$app" || (printf "$(RED)$(CROSS) Missing app directory: $$app$(RESET)\n" && exit 1); \
		test -f "$$app/Makefile" || (printf "$(RED)$(CROSS) $$app is missing Makefile$(RESET)\n" && exit 1); \
		printf "\n$(CYAN)$(BOLD)==> $$app: $(TARGET)$(RESET)\n"; \
		$(MAKE) -C "$$app" "$(TARGET)" --no-print-directory; \
	done
	@printf "\n$(GREEN)$(CHECK) $(TARGET) completed for $(APPS)$(RESET)\n\n"

# ============== Help ==============
help: banner
	@printf "$(CYAN)$(BOLD)Bundle Commands:$(RESET)\n"
	@printf "  $(GREEN)make list$(RESET)                  - List official uc-* app directories\n"
	@printf "  $(GREEN)make doctor$(RESET)                - Check local cargo/rustc/node/docker readiness\n"
	@printf "  $(GREEN)make verify-metadata$(RESET)       - Validate bundle metadata and required app surfaces\n"
	@printf "  $(GREEN)make verify-imports$(RESET)        - Compare uc-* imports against USECASE_SOURCE_ROOT\n"
	@printf "  $(GREEN)make import-usecases$(RESET)       - Refresh official app directories from USECASE_SOURCE_ROOT\n"
	@printf "\n$(CYAN)$(BOLD)Build & Run:$(RESET)\n"
	@printf "  $(GREEN)make build$(RESET)                 - Build all official apps\n"
	@printf "  $(GREEN)make build-release$(RESET)         - Build all official apps in release mode\n"
	@printf "  $(GREEN)make run APP=uc-lessons$(RESET)    - Run one app locally on PORT=$(PORT)\n"
	@printf "  $(GREEN)make run-release APP=uc-lessons$(RESET) - Run one release build locally\n"
	@printf "\n$(CYAN)$(BOLD)Tests & Validation:$(RESET)\n"
	@printf "  $(GREEN)make test$(RESET)                  - Run the standard app test surface across all apps\n"
	@printf "  $(GREEN)make test-rust$(RESET)             - Run Rust tests across all apps\n"
	@printf "  $(GREEN)make test-frontend-syntax$(RESET)  - Check frontend JavaScript syntax across apps\n"
	@printf "  $(GREEN)make test-e2e$(RESET)              - Run each app's browser validation target\n"
	@printf "  $(GREEN)make test-one APP=uc-hospital TEST=name$(RESET) - Run one named test\n"
	@printf "\n$(CYAN)$(BOLD)Lint & Format:$(RESET)\n"
	@printf "  $(GREEN)make lint$(RESET)                  - Metadata/import checks, fmt, clippy, frontend syntax\n"
	@printf "  $(GREEN)make fmt$(RESET)                   - Format Rust code in every app\n"
	@printf "  $(GREEN)make fmt-check$(RESET)             - Check Rust formatting in every app\n"
	@printf "  $(GREEN)make clippy$(RESET)                - Run clippy in every app\n"
	@printf "\n$(CYAN)$(BOLD)CI & Space:$(RESET)\n"
	@printf "  $(GREEN)make ci-local$(RESET)              - Run the root local CI simulation\n"
	@printf "  $(GREEN)make space-ci$(RESET)              - Delegate each app's Space readiness pipeline\n"
	@printf "  $(GREEN)make docker-build$(RESET)          - Build all app Docker images\n"
	@printf "  $(GREEN)make docker-run APP=uc-hospital$(RESET) - Run one app Docker image locally\n"
	@printf "  $(GREEN)make pre-release$(RESET)           - Run CI simulation and build all Space images\n"
	@printf "\n$(CYAN)$(BOLD)Use-Case Releases:$(RESET)\n"
	@printf "  $(GREEN)make release-usecase-dry-run APP=uc-hospital$(RESET) - Preview app changelog/version/tag release\n"
	@printf "  $(GREEN)make release-usecase APP=uc-hospital RELEASE_AS=patch$(RESET) - Cut app changelog/version/tag release\n"
	@printf "  $(GREEN)make release-usecase APP=uc-hospital PREPARED=1$(RESET) - Tag an already-prepared app version without another bump\n"
	@printf "  $(GREEN)make verify-release-tag TAG=solverforge-hospital@x.y.z$(RESET) - Validate tag against Cargo.toml, Cargo.lock, and CHANGELOG.md\n"
	@printf "  $(GREEN)make release-ci APP=uc-hospital RELEASE_VERSION=x.y.z$(RESET) - Run tag-aware app CI gate\n"
	@printf "  $(GREEN)make publish-usecase-dry-run TAG=solverforge-hospital@x.y.z$(RESET) - Preview the guarded GitHub branch/tag push\n"
	@printf "  $(GREEN)make publish-usecase TAG=solverforge-hospital@x.y.z$(RESET) - Push main and one release tag to trigger its Space sync\n"
	@printf "  $(GREEN)make publish-usecases-dry-run$(RESET) - Preview publication of every app's current release tag\n"
	@printf "  $(GREEN)make publish-usecases$(RESET)        - Push main once and current app tags separately\n"
	@printf "\n$(CYAN)$(BOLD)Other:$(RESET)\n"
	@printf "  $(GREEN)make install-e2e$(RESET)           - Install root Playwright dependencies\n"
	@printf "  $(GREEN)make version$(RESET)               - Show bundle metadata\n"
	@printf "  $(GREEN)make clean$(RESET)                 - Clean app build artifacts and root test-results\n"
	@printf "\n$(GRAY)Rust version required: $(RUST_VERSION)$(RESET)\n"
	@printf "$(GRAY)Apps: $(OPEN_SOURCE_APPS)$(RESET)\n\n"
