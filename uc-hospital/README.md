---
title: SolverForge Hospital
emoji: 🏥
colorFrom: red
colorTo: pink
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: SolverForge hospital scheduling example
---

# SolverForge Hospital

![SolverForge Hospital screenshot](docs/screenshot.png)

`solverforge-hospital` is a beginner-friendly example of a real SolverForge app.
It answers one concrete question:

"Given a hospital workforce and a month of shifts, which employee should cover each shift?"

## Documentation Map

Each top-level document has a different job:

- `README.md`
  Quick start, concepts, API surface, and the shortest learning path.
- `WIREFRAME.md`
  Architecture and request/data flow across backend, runtime, and frontend.
- `docs/api-and-solver-policy.md`
  REST routes, payload shape, lifecycle semantics, and solver policy notes.
- `AGENTS.md`
  Contribution rules, validation commands, and the documentation standard for future edits.
- `Makefile`
  The beginner-friendly command surface for local development, validation, and
  Docker-based Hugging Face Space preparation.

If you are new to SolverForge, read this repo as three layers:

1. The planning-model manifest and model modules in `src/domain/`
2. The score rules in `src/constraints/`
3. The runtime and browser app in `src/api/`, `src/solver/`, and `static/app/`

## What SolverForge Is Doing Here

SolverForge is the optimization engine. In this app:

- `Employee` is a problem fact: input data the solver does not move
- `Shift` is the planning entity: the thing the solver assigns
- `Shift.employee_idx` is the planning variable: the actual choice the solver makes
- the constraints decide whether an assignment is legal and whether it is good
- `solver.toml` tells the runtime how to search for a better assignment

The app ships one serious demo instance rather than many toy presets:

- fixed random seed
- fixed 28-day schedule horizon
- 50 employees
- 688 shifts
- 8-hour shifts
- deterministic generator
- retained-job runtime with pause, resume, cancel, snapshot, and analysis

## Read The Code In This Order

If you want to learn the codebase, this order is the shortest path:

1. [src/domain/employee.rs](src/domain/employee.rs)
   `Employee` is the input fact model.
2. [src/domain/care_hub.rs](src/domain/care_hub.rs)
   `CareHub` explains how the app models service-line proximity.
3. [src/domain/mod.rs](src/domain/mod.rs)
   This is the `planning_model!` manifest that lists and exports the model modules.
4. [src/domain/plan.rs](src/domain/plan.rs)
   `Shift`, `Plan`, nearby search meters, and transport normalization live here.
5. [src/constraints/mod.rs](src/constraints/mod.rs)
   This lists every scoring rule.
6. [src/constraints/*.rs](src/constraints)
   Each file explains one real scheduling rule.
7. [src/data/data_seed/entrypoints.rs](src/data/data_seed/entrypoints.rs)
   This shows the public demo-data surface.
8. [src/data/data_seed/large.rs](src/data/data_seed/large.rs)
   This assembles the published benchmark instance.
9. [src/solver/service.rs](src/solver/service.rs)
   This is the retained-job runtime facade.
10. [src/api/routes.rs](src/api/routes.rs) and [src/api/sse.rs](src/api/sse.rs)
   These expose the HTTP and live-event contracts.
11. [static/app/main.mjs](static/app/main.mjs)
   This is the browser boot sequence.
12. [static/app/shell/](static/app/shell) and [static/app/schedule/](static/app/schedule)
   These adapt stock `solverforge-ui` pieces to the hospital example.

## Project Shape

- `src/domain/`
  `planning_model!` manifest, planning model, derived fields, nearby meters.
- `src/constraints/`
  Hard and soft scoring rules.
- `src/data/`
  Deterministic demo-data generator.
- `src/solver/`
  Retained-job orchestration over `SolverManager<Plan>`.
- `src/api/`
  REST routes, DTOs, and SSE endpoint.
- `static/app/`
  Browser code built as plain ES modules on top of `solverforge-ui`.
- `tests/frontend/`
  Browserless UI tests using the fake DOM in `tests/support/`.
- `tests/e2e/`
  Playwright smoke tests against the served browser app.

## Quick Start

This repo depends on the published SolverForge runtime and UI crates:

- `solverforge = 0.13.0`
- `solverforge-ui = 0.6.5`

The app package version is `1.0.2`, and the runtime score type is
`HardSoftDecimalScore`.

Run the app:

```sh
make run-release
```

Then open `http://localhost:7860`.

If you want to inspect the command surface first:

```sh
make help
```

## What You Will See

The browser UI does five things:

1. Loads `static/sf-config.json`
2. Loads `static/generated/ui-model.json`
3. Fetches the `LARGE` demo dataset from `/demo-data/LARGE`
4. Renders two schedule views:
   `By location` and `By employee`
5. Starts a retained solving job when you click Solve

The frontend is intentionally thin. It mostly uses stock `solverforge-ui` pieces:

- `SF.createBackend({ type: "axum" })`
- `SF.createHeader()`
- `SF.createStatusBar()`
- `SF.createSolver()`
- `SF.rail.createTimeline()`

The app also exposes a visible "REST API" guide inside the browser shell. See
[`docs/api-and-solver-policy.md`](docs/api-and-solver-policy.md) for the route
list, lifecycle semantics, payload shape, and solver policy notes that this
guide is expected to match.

## Run Validation

Standard local validation:

```sh
make test
```

Space-style local CI simulation:

```sh
make ci-local
```

Slow acceptance solve:

```sh
make test-slow
```

`make test` includes Rust tests, browserless frontend tests, and Playwright
browser tests. `make ci-local` is the main pre-push validation path for this
repo. It also checks formatting, runs clippy, builds the release binary, and
builds the Docker image that the Space-style deploy path expects.

## Demo Dataset Intent

The generator is not random filler data. It is designed to publish a problem
that is:

- hard-feasible
- deterministic
- narrow enough that candidate choice matters
- rich enough that soft-score improvements still exist after construction

The hidden witness roster in `src/data/data_seed/witness.rs` is especially
important: it gives the generator an internal feasible assignment that is used
to shape unavailability and preferences, but it is never shown to the solver.

The `LARGE` dataset is built once and cached in memory because it is immutable
and deterministic. Each request still receives an owned `Plan`, but the server
does not regenerate the same public benchmark from scratch every time.

## API And Solver Policy

Detailed REST route, payload, lifecycle, telemetry, and solver-policy reference
material lives in [`docs/api-and-solver-policy.md`](docs/api-and-solver-policy.md).
The runtime source of truth remains [solver.toml](./solver.toml).

## Constraints

Hard constraints:

- Assigned shift
- Required skill
- Overlapping shift
- At least 10 hours between 2 shifts
- One shift per day
- Unavailable employee

Soft constraints:

- Undesired day for employee
- Desired day for employee
- Balance employee assignments

## Docker

Build from this repository root:

```sh
make space-build
make space-run
```

The Docker build uses the same registry dependency line as local development.
