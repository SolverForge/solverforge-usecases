# Repository Guidelines

## Project Structure And Naming

`solverforge-fleet` is a Rust 1.95 SolverForge synthetic fleet-readiness
planning app with an Axum server and static browser workspace. Keep the
repo-local directory name `uc-fleet`; product copy and app metadata use
`solverforge-fleet` or SolverForge Fleet.

- `src/domain/plan.rs` owns `Plan`, its fact/entity collections, and embedded solver policy.
- `src/domain/work_package.rs` owns scalar dock and start-day decisions.
- `src/domain/inspection_assignment.rs` and `training_assignment.rs` own scalar day decisions.
- `src/constraints/` owns the 15 hard and soft score rules.
- `src/constraints/support/` owns shared scoring, readiness, repair, metrics, and schedule calculations.
- `src/data/data_seed/` owns the deterministic 24-vessel, 56-day scenarios.
- `src/solver/` owns retained-job orchestration and SSE payloads.
- `src/api/` owns REST, SSE, plan-session, scenario, repair, and comparison routes.
- `static/` owns the fleet planner workspace built from `solverforge-ui` globals.
- `Dockerfile`, `Makefile`, `solver.toml`, and `solverforge.app.toml` define deployment and runtime contracts.

Keep the solution name `Plan`. Public demo IDs are `BASELINE`,
`TECHNICIAN_SHORTAGE`, and `BASELINE_FULL`; the default is `BASELINE_FULL`.

## Build And Validation Commands

- `make help` shows the supported command surface.
- `make run-release` runs the app on `:7860`.
- `make test` runs Rust tests, frontend Node syntax/transform tests, and Playwright smoke.
- `make test-e2e` runs the browser smoke against the release binary.
- `make test-slow` runs the ignored full `BASELINE_FULL` acceptance solve.
- `make ci-local` runs formatting, clippy, release build, standard tests, and Docker build.
- `make pre-release` runs `ci-local` plus the slow acceptance solve.
- `PORT=7861 cargo run --bin solverforge-fleet` uses an alternate port.

Use the Makefile as the authoritative app-local workflow.

## Model Contracts

All decisions are scalar and allow unassigned values:

- `WorkPackage.dock_idx` indexes `Plan.docks`.
- `WorkPackage.start_day_idx` indexes `Plan.days`.
- `InspectionAssignment.day_idx` and `TrainingAssignment.day_idx` index `Plan.days`.

Candidate index vectors and nearby-distance functions are part of each variable
contract. Changes must preserve DTO validation, candidate population, scoring,
repair helpers, and frontend schedule serialization together.

Despite its name, `weekly_readiness_floor` enforces readiness on each modeled
day. Do not describe it as weekly-only or change `week_days(plan)` to weekly
sampling. `weekly_readiness` is a separate presentation summary that samples
the first day in each display week.

Plan-session repair is a derived re-solve with lineage. It reads a selected
snapshot, applies an event, starts a new retained job, and records a comparison
link. Do not call it checkpoint continuation unless the implementation changes
and is verified end to end.

## Coding And Documentation

Use Rust 2021 style and `cargo fmt`. Keep deterministic demo generation and the
fixed solver seed unless a source-backed requirement changes them. Comments
should explain domain meaning, invariants, or runtime consequences rather than
repeat syntax.

Keep `README.md`, `WIREFRAME.md`, this file, `solver.toml`,
`solverforge.app.toml`, and `static/sf-config.json` aligned when their owning
behavior changes. Verify every documented version, count, route, demo ID, and
solver-policy value against source in the same patch.

This use case is repository-only. Do not add it to Hugging Face synchronization
without a real target Space and the required root publication changes.

## Testing Guidance

Add focused Rust tests beside local behavior. Use `tests/project_shape.rs` for
cross-file package, source-tree, API, and static-shell contracts. Keep pure
frontend transforms in `tests/frontend/` and browser behavior in the inline
`make test-e2e` Playwright smoke unless it grows beyond a single workflow.

Run `make test-slow` after solver-policy, candidate-domain, hard-constraint, or
demo-data changes. The acceptance test starts from the fully unassigned
`BASELINE_FULL` plan and requires a completed terminal solve with zero hard
score.
