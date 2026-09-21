# Repository Guidelines

## Project Structure And Naming

`solverforge-flightcrew` is a Rust 1.95 SolverForge web app. Keep the local
directory name `uc-flightcrew`; product labels, package metadata, and release
tags use SolverForge Flight Crew and `solverforge-flightcrew`.

- `src/domain/` owns airports, employees, flights, crew-seat entities, and the
  `Plan` solution.
- `src/constraints/` owns one named score rule per file.
- `src/data/data_seed.rs` owns the deterministic `STANDARD` schedule.
- `src/api/` and `src/solver/` own retained REST, SSE, and job lifecycle logic.
- `static/flightcrew-model.js` owns pure timeline transforms;
  `static/app.js` owns shell and lifecycle wiring.
- `tests/constraints.rs`, `tests/frontend/`, `tests/e2e/`, and
  `tests/slow_solve.rs` protect the behavioral contract.

Keep the canonical demo ID `STANDARD` and the solution name `Plan`.

## Commands

- `make run-release` runs the app on `:7860`.
- `make test` runs Rust, frontend, and Playwright tests.
- `make test-slow` requires the standard demo to reach zero hard score.
- `make ci-local` runs formatting, Clippy, release build, tests, and Docker.
- `make pre-release` adds the slow acceptance solve.

## Model Rules

The source application's executable 10-hour long-haul threshold takes
precedence over its contradictory prose. Keep one planning entity per required
seat. Candidate providers may remove employees without the seat's skill, but
must not precompute constraint scores or repair solutions after solving.

Minute fields are integer offsets from the demo horizon. Duty starts 45 minutes
before departure and ends 20 minutes after arrival. Preserve those offsets in
all rest calculations and in documentation.

## Documentation And Testing

Keep `README.md`, `WIREFRAME.md`, this file, `solver.toml`,
`solverforge.app.toml`, `static/sf-config.json`, and `docs/screenshot.png`
aligned. Comments should explain aviation meaning, model invariants, or runtime
consequences rather than restating Rust syntax.

Any constraint change requires an exact score-delta test and the ignored real
solve. Any UI change requires Node transform tests and the Playwright smoke.
This app is repository-only; do not add it to the Hugging Face sync matrix until
the matching target exists.
