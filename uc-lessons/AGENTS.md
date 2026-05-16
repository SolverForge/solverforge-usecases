# Repository Guidelines

## Project Structure And Naming

`solverforge-lessons` is a Rust 1.95 SolverForge lesson-timetabling app with an
Axum server and static browser workspace. Keep the repo-local directory name
`uc-lessons`; product copy, metadata, and UI labels should use
`solverforge-lessons` or SolverForge Lessons.

- `src/domain/mod.rs` owns the `solverforge::planning_model!` manifest.
- `src/domain/plan.rs` owns the `Plan` planning solution.
- `src/domain/lesson.rs` owns the `Lesson` planning entity and its
  `timeslot_idx` and `room_idx` scalar variables.
- `src/domain/{timeslot,teacher,group,room}.rs` own the problem facts.
- `src/constraints/` owns one timetable scoring rule per file plus `mod.rs`
  assembly.
- `src/data/data_seed/` owns deterministic `LARGE` demo generation.
- `src/api/` owns REST, DTO, and SSE surfaces.
- `src/solver/` owns retained-job runtime orchestration.
- `static/` owns the browser shell and generated view model.
- `Dockerfile`, `Makefile`, `solver.toml`, and `solverforge.app.toml` define
  the deployment and runtime contract.

Keep the canonical solution name `Plan` and the public demo id `LARGE`.

## Build And Validation Commands

- `make help` shows the supported command surface.
- `make run-release` runs the app locally on `:7860`.
- `make test` runs Rust tests, frontend syntax checks, and the Playwright smoke.
- `make test-e2e` runs the real browser smoke.
- `make ci-local` runs formatting, clippy, release build, standard tests, and
  the Space Docker image build.
- `make test-slow` runs the ignored large-demo acceptance solve.
- `make pre-release` runs `ci-local` plus the slow acceptance solve.
- `cargo test` runs Rust unit tests.
- `PORT=7861 cargo run --bin solverforge-lessons` runs the app on an alternate
  port when `7860` is already occupied.

Use the Makefile as the authoritative local workflow.

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

The standard to aim for is: a new reader should understand why a piece of code
exists before they need to understand every line of how it works.

## Testing Guidance

Add Rust tests next to the behavior they protect. The current browser smoke is
the inline `make test-e2e` Playwright check; add a `tests/e2e/` tree only when
the browser flow grows beyond that single smoke. If you change solver behavior,
run `cargo test` and the ignored large-demo solve. If you change UI structure,
run the frontend syntax check and Playwright smoke.

## Runtime Notes

`solver.toml` is embedded by `Plan` through the planning-solution macro. Treat
it as the solver policy source of truth.

The app serves stock `solverforge-ui` assets, local static app modules, and
Axum API routes from one process. Retained solver jobs are controlled through
REST and observed through SSE.
