# Repository Guidelines

## Project Structure And Naming

`solverforge-fsr` is a Rust 1.95 SolverForge field-service routing app with an
Axum server and static browser workspace. The app package version is declared
in `Cargo.toml`, and the release binary is `solverforge_fsr`.

- `src/domain/mod.rs` owns the `solverforge::planning_model!` manifest.
- `src/domain/field_service_plan.rs` owns the `FieldServicePlan` solution,
  transient visit-index normalization, and route shadow refresh hook.
- `src/domain/location.rs`, `service_visit.rs`, and `travel_leg.rs` own the
  problem facts.
- `src/domain/technician_route.rs` owns the planning entity and its `visits`
  list variable.
- `src/domain/route_metrics.rs` owns route shadow measurement.
- `src/constraints/` owns SolverForge scoring rules, one business rule per file.
  Prefer stock `ConstraintFactory` streams; `assigned_visits.rs` keeps the
  duplicate-assignment check as a small custom `IncrementalConstraint` because
  a grouped stream would count singleton groups as analysis matches.
- `src/data/data_seed.rs` owns `STANDARD` demo assembly and road-matrix
  preparation; `src/data/bergamo_*.rs` owns the static locations, visit
  profiles, technicians, and shared catalog types.
- `src/api/` owns REST, DTO, route geometry, and SSE surfaces.
- `src/solver/` owns retained-job runtime orchestration.
- `static/` owns the browser workspace, split by responsibility
  (`app-route-state.js`, `app-render-routes.js`, etc.).
- `Dockerfile`, `Makefile`, `solver.toml`, and `solverforge.app.toml` define
  the deployment and runtime contract.

Keep handwritten source, docs, and deployment files under 300 lines; split by
module or responsibility when a file approaches that size.

## Build, Test, and Development Commands

- `make doctor` checks local `cargo`, `rustc`, `node`, and `docker` readiness.
- `make run` runs the debug server on `PORT` (default `7860`).
- `make build-release` builds `solverforge_fsr` in release mode.
- `make test` runs Rust tests, frontend JavaScript syntax checks, and the
  Playwright browser smoke.
- `make lint` runs `cargo fmt --check`, clippy with warnings denied, and JS syntax checks.
- `make ci-local` runs the full Hugging Face Space validation path, including Docker image build.
- `make space-run` builds and runs the Docker Space image locally.

## Coding Style & Naming Conventions

Use idiomatic Rust 2021 with `cargo fmt` formatting and clippy under
`-D warnings`. Rust modules and files use `snake_case`; types use `PascalCase`;
functions, fields, and variables use `snake_case`. Keep API DTOs explicit and
snapshot-scoped. Frontend files should stay plain JavaScript modules with clear
ownership boundaries rather than large shared scripts.

## Testing Guidelines

Place Rust unit tests near the code they cover, using descriptive names such as
`reports_unreachable_route_segments`. Run `make test` before handing off normal
changes and `make ci-local` before deployment, dependency, Docker, or Space
changes. Frontend validation includes `node --check` over `static/*.js`; served
browser behavior is covered by `make test-e2e`.

## Documentation And Commenting Policy

Assume a reader who is new to Rust and new to planning optimization.

- Keep `README.md`, `WIREFRAME.md`, this file, `solver.toml`,
  `solverforge.app.toml`, `static/sf-config.json`, and the visible browser API
  guide aligned.
- Keep `docs/screenshot.png` current whenever the visible browser shell changes.
- Add module or function comments where code coordinates SolverForge concepts:
  facts, planning entities, variables, retained jobs, road matrices, route
  geometry, or score math.
- Explain domain meaning and solver consequences. Do not keep scaffold
  placeholders, future-tense planning prose, or comments that merely restate
  syntax.
- When docs mention versions, counts, routes, demo IDs, solver policy, or
  validation expectations, verify those facts against current code in the same
  patch.

## Commit & Pull Request Guidelines

History uses conventional commits such as `feat(fsr): ...`, `fix(ui): ...`,
and `chore: ...`. Keep each commit focused on one revertable
intent and include a full body when the change spans behavior, deployment, or
dependencies. PRs should describe the user-visible effect, linked issue or
review comment, validation commands run, and include screenshots for visible UI
changes.

## Security & Configuration Tips

Do not commit credentials, local Hugging Face tokens, generated desktop bundles,
or build output. Keep Docker/Space builds registry-backed through the declared
crates.io dependency line unless the build context explicitly vendors local
crates.
