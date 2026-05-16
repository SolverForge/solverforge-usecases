# Repository Guidelines

## Project Structure And Naming

This repo follows the current `solverforge-cli` app shape. The app package
version is declared in `Cargo.toml`, and the release binary is
`solverforge_deliveries`.

- `src/domain/mod.rs` owns the `solverforge::planning_model!` manifest.
- `src/domain/plan.rs` owns the `Plan` planning solution.
- `src/domain/delivery.rs` owns the `Delivery` problem fact.
- `src/domain/vehicle.rs` owns the `Vehicle` planning entity and its
  `delivery_order` list variable.
- `src/domain/preview.rs` owns transport/view preview structs.
- `src/domain/route_metrics/` owns route preparation, the local CVRP feasibility
  hook, scoring preview, route geometry, and insertion ranking.
- `src/constraints/` owns one score rule per file plus `mod.rs` assembly.
- `src/data/data_seed/` owns deterministic city demo-data modules with grouped
  visit files for scaled delivery counts.
- `src/api/` owns REST, DTO, and SSE surfaces.
- `src/solver/` owns retained-job runtime orchestration.
- `static/app/models/` owns frontend plan modeling.
- `static/app/ui/` owns browser layout and rendering helpers.

Keep the canonical solution name `Plan`. Do not reintroduce `DeliveryPlan` or
`delivery_plan.rs`.

## File Size Rule

No source, test, frontend, config, or repo documentation file should reach 300
lines. Split by responsibility before that point. Large generated/cache output
under `target/` and `.osm_cache/` is outside this rule.

## Build And Validation Commands

- `make help` shows the supported command surface.
- `make run-release` runs the app locally on `:7860`.
- `make test` runs Rust, frontend, and Playwright browser tests.
- `make test-e2e` runs the real browser Playwright smoke.
- `make space-build` builds the Docker image used by Hugging Face Spaces.
- `make space-run` builds and runs that image locally.
- `make ci-local` runs formatting, clippy, release build, standard tests, and
  the Space Docker image build.
- `make test-live-road` runs the env-enabled road-network smoke test.
- `make pre-release` runs `ci-local` and the live road-network smoke.
- `cargo test` runs Rust unit and integration tests.
- `node --test tests/frontend_models.test.mjs` runs frontend model tests.

Use the Makefile as the authoritative local workflow, matching the
`solverforge-hospital` Space/Docker validation standard.

## No Suppression Policy

Do not add warning suppressions, fallback compatibility branches, or unused
helper modules. If a split creates unused code, restructure the modules so each
compiled item is used by its crate.

## Documentation Policy

Keep `Cargo.toml`, `Cargo.lock`, `Makefile`, `Dockerfile`, `README.md`,
`WIREFRAME.md`, `AGENTS.md`, `docs/screenshot.png`,
`solverforge.app.toml`, `solver.toml`, `static/sf-config.json`, and the visible
API guide in
`static/app/ui/api-guide.mjs` aligned.

When changing routes, solver policy, demo IDs, dependency sources, file layout,
or browser behavior, update the docs and screenshot in the same patch. Prefer
current-state documentation over planning language.

## Testing Guidance

Add Rust unit tests next to the behavior they protect. Add API integration
coverage under `tests/api_contract/` and shared integration helpers under
`tests/support/` only when every helper is used by the single
`tests/api_contract.rs` crate. Add frontend model tests in
`tests/frontend_models.test.mjs`, and browser-flow tests in `tests/e2e/`.

Road-network tests should stay env-gated unless the test uses only cached or
straight-line behavior. Use `SOLVERFORGE_RUN_LIVE_TESTS=1` through
`make test-live-road` when validating live map/routing paths.

## Runtime Notes

`solver.toml` is embedded by `Plan` through the planning-solution macro. Treat
it as the solver policy source of truth.

`Cargo.toml` currently uses Rust `1.95` and crates.io dependency declarations.
Keep every direct dependency declaration, `solverforge.app.toml`, `Cargo.lock`,
and docs truthful if dependency sources or versions change.

The app serves stock `solverforge-ui` assets, local static app modules, and
Axum API routes from one process. Retained solver jobs are controlled through
REST and observed through SSE.
