# Repository Guidelines

## Project Structure And Naming

`solverforge-furnace` is a Rust 1.95 SolverForge heat-treatment scheduling app
with an Axum server and static browser workspace. Keep the repo-local directory
name `uc-furnace`; product copy, metadata, and UI labels should use
`solverforge-furnace` or SolverForge Furnace.

- `src/domain/mod.rs` owns the `solverforge::planning_model!` manifest.
- `src/domain/plan.rs` owns the `Plan` planning solution and wires the
  `solver_toml`, `scalar_groups`, and `conflict_repairs` attributes.
- `src/domain/resources.rs` owns the problem facts: `Furnace`, `WorkOrder`,
  `Operator`, `Shift`, and `ShiftCoverageDemand`.
- `src/domain/furnace.rs` owns `FurnaceAssignment` and its five scalar planning
  variables plus the furnace construction hooks.
- `src/domain/coverage.rs` owns `OperatorShiftAssignment` and its `shift_type`
  scalar variable plus the roster construction hooks.
- `src/domain/{enums,constants,time,workforce,changeover,support}.rs` own the
  value vocabularies, time model, changeover math, and shared helpers.
- `src/constraints/` owns the score rules, grouped by `furnace_hard`,
  `furnace_soft`, `staffing_hard`, `monitoring_hard`, `roster_hard`, and
  `roster_soft`, with assembly in `mod.rs`.
- `src/data/data_seed/` owns deterministic `STANDARD` demo generation.
- `src/solver/` owns retained-job runtime orchestration, the grouped scalar
  providers, and the conflict-repair providers.
- `src/api/` owns REST, DTO, analysis, and SSE surfaces.
- `static/` owns the browser shell and generated view model.
- `Dockerfile`, `Makefile`, `solver.toml`, and `solverforge.app.toml` define
  the deployment and runtime contract.

Keep the canonical solution name `Plan` and the public demo id `STANDARD`.

## Build And Validation Commands

- `make help` shows the supported command surface.
- `make run-release` runs the app locally on `:7860`.
- `make test` runs Rust tests, frontend syntax checks, and the Playwright smoke.
- `make test-e2e` runs the real browser smoke.
- `make ci-local` runs formatting, clippy, release build, standard tests, and
  the Space Docker image build.
- `make test-slow` runs the ignored `STANDARD` acceptance solve.
- `make pre-release` runs `ci-local` plus the slow acceptance solve.
- `cargo test` runs Rust unit tests.
- `PORT=7861 cargo run --bin solverforge-furnace` runs the app on an alternate
  port when `7860` is already occupied.

Use the Makefile as the authoritative local workflow.

## Coding Style And Naming Conventions

Use Rust 2021 style with `cargo fmt`; keep imports and formatting
rustfmt-compatible. Prefer small, explicit functions over clever indirection.
Rust module and file names are `snake_case`; types are `UpperCamelCase`; tests
should describe behavior plainly. Frontend modules are plain ES modules in
`snake_case` filenames. Keep demo generation deterministic: do not introduce
random behavior without a fixed seed and an explicit reason.

Value encodings are part of the model contract. `FurnaceAssignment.assignment`
is `furnace_idx * NUM_TIME_SLOTS + time_slot_idx`, and
`OperatorShiftAssignment.shift_type` is `0..=3` (Morning, Afternoon, Night,
Off). Any change to either encoding must update the providers, the DTO
validation, the constraints, and the tests in the same patch.

## Documentation And Commenting Policy

Assume a reader who is new to SolverForge and new to optimization modeling.

- Keep `README.md`, `WIREFRAME.md`, this file, `solver.toml`,
  `solverforge.app.toml`, `static/sf-config.json`, and `docs/screenshot.png`
  aligned.
- Add module-level docs or comments for modules that explain their role in the
  app and where they sit in the data flow.
- Add function comments when the function coordinates SolverForge concepts,
  rebuilds invariants, shapes demo data, converts between layers, or otherwise
  does something a beginner would not infer from the signature.
- Write comments that explain intent, domain meaning, invariants, and runtime
  consequences. Do not write comments that merely restate syntax.
- Keep comments present-tense and source-backed. If behavior changes, update or
  delete the stale comment in the same patch.
- When docs mention versions, counts, routes, demo IDs, solver policy, or
  validation expectations, verify those facts against current code in the same
  patch.
- Treat `solver.toml` provider names and `src/solver/` provider registrations
  as one contract. A rename in one place must be renamed in the other.

The standard to aim for is: a new reader should understand why a piece of code
exists before they need to understand every line of how it works.

## Testing Guidance

Add Rust tests next to the behavior they protect. The current browser smoke is
the inline `make test-e2e` Playwright check; add a `tests/e2e/` tree only when
the browser flow grows beyond that single smoke. If you change solver policy,
run `cargo test`, `make test-slow`, and the `solver.toml` contract test in
`src/solver/service/tests/config.rs`. If you change UI structure, run the
frontend syntax check and Playwright smoke.

The `tests/project_shape.rs` integration test enforces the canonical source
tree, the static shell wiring, and the "no seed or repair backdoor" policy.
Update it only when the canonical shape intentionally changes.

## Runtime Notes

`solver.toml` is embedded by `Plan` through the planning-solution macro. Treat
it as the solver policy source of truth.

The app serves stock `solverforge-ui` assets, local static app modules, and
Axum API routes from one process. Retained solver jobs are controlled through
REST and observed through SSE.
