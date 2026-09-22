# Repository Guidelines

## Structure And Naming

`solverforge-orders` is a Rust 1.95 SolverForge web app. Keep the directory name
`uc-orders`; product labels, package metadata, binary names, and future tags use
SolverForge Orders and `solverforge-orders`.

- `src/domain/` owns immutable pick steps, warehouse distance, trolley entities,
  route shadows, and the `Plan` solution.
- `src/constraints/` owns one named hard or soft rule per file.
- `src/data/data_seed.rs` owns the deterministic source-faithful `STANDARD` data.
- `src/api/` and `src/solver/` retain REST, SSE, telemetry, and job lifecycle.
- `static/app/orders-model.mjs` owns pure browser transforms;
  `static/app/main.mjs` owns rendering and SolverForge lifecycle wiring.
- `tests/` protects score deltas, transforms, browser lifecycle, and full solve.

This app is repository-only. Do not add it to the Hugging Face sync allowlist
until a matching Space exists. Do not create `CHANGELOG.md` manually and do not
add `docs/screenshot.png` without a current live browser capture.

## Model Rules

`PickStep` indexes are list elements and `Trolley.step_order` is the sole
planning variable. Preserve globally unique stable IDs and exact-once list
ownership. Do not add scalar assignments, precomputed score matrices, greedy
application initializers, or post-solve repair.

Warehouse distance must stay behaviorally identical to the piecewise source
`warehouse.py` function, including closed return legs. Bucket demand is ceiling
division per order per trolley, not total trolley volume. The 1,000-point soft
incidence baseline is intentional. All values are integral, so `HardSoftScore`
is deliberately equivalent to the source `HardSoftDecimalScore` here.

## Commands

- `make run-release` runs the app on `:7860`.
- `make test` runs Rust, frontend, and Playwright tests.
- `make test-slow` requires zero hard score and exact-once assignment.
- `make ci-local` adds formatting, Clippy, release, and Docker validation.
- `solverforge check` validates managed model metadata and generated UI model.

Any constraint change requires an exact score test and the ignored acceptance
solve. Any warehouse-distance change requires branch tests in Rust and JS. Any
UI transform change requires Node coverage; lifecycle changes require Playwright.
