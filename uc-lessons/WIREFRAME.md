# solverforge-lessons WIREFRAME

This file is the architectural map for the lesson timetabling example.

`README.md` explains how to run and use the app. This document explains how the
pieces fit together and where each responsibility lives.

## Documentation Roles

- `README.md`
  Quick start, dependency shape, API list, and user-facing orientation.
- `WIREFRAME.md`
  Architecture, execution flow, and file-map walkthrough.
- `AGENTS.md`
  Repo-specific contribution, validation, and documentation rules.
- `Makefile`
  Local development, validation, and Space/Docker command surface. Its
  `test-e2e` target contains the inline Playwright smoke for this app.
- `Dockerfile`
  Hugging Face Docker Space image definition.
- `docs/screenshot.png`
  Current browser screenshot embedded by the README.

## What This Repo Is Teaching

This repo is a complete `solverforge-lessons` scalar-variable SolverForge app
for weekly lesson timetabling.

It shows how to combine:

- a `Plan` solution with two scalar planning variables per lesson
- assignment-completeness, hard feasibility, and soft timetable-quality rules
- deterministic `LARGE` demo-data generation from domain facts
- retained jobs with snapshots, analysis, cancel, pause, resume, and SSE
- a browser timetable workspace built on stock `solverforge-ui` assets

## SolverForge Concepts In Plain Language

- `Timeslot`, `Teacher`, `Group`, and `Room`
  Input facts. The solver reads them but does not move them.
- `Lesson`
  Planning entity. Each lesson receives a timeslot and a room.
- `timeslot_idx` and `room_idx`
  The two scalar planning variables on `Lesson`.
- `Plan`
  Planning solution. It holds facts, lesson entities, derived indexes, and the
  current `HardMediumSoftScore`.
- hard score
  Teacher/group availability, room capacity, and teacher/group/room conflicts.
- medium score
  Assignment completeness for timeslot and room decisions.
- soft score
  Room-kind fit, late slots, and repeated subject days.
- retained job
  A solve that lives in memory so the UI can stream events, fetch snapshots,
  pause/resume, cancel, analyze, and delete terminal jobs.

## Runtime Flow

1. The browser loads `static/index.html`.
2. `static/app.js` loads `static/sf-config.json`,
   `static/generated/ui-model.json`, and `solverforge-ui` assets from `/sf/*`.
3. The app fetches `/demo-data` to discover the default `LARGE` id, then loads
   the plan from `/demo-data/LARGE`.
4. `src/data/data_seed/entrypoints.rs` dispatches to
   `src/data/data_seed/large.rs`.
5. `Plan::new()` normalizes fact and lesson indexes and filters stale scalar
   assignment indexes.
6. The browser renders group, room, teacher, data, and REST API views.
7. When the user clicks Solve, the browser posts the current plan to
   `POST /jobs`.
8. `src/api/routes.rs` deserializes `PlanDto` back into `Plan`.
9. `SolverService` starts a retained solve through `SolverManager<Plan>`.
10. Solver events are converted by `src/solver/event_payload.rs` into
    UI-facing JSON.
11. The browser consumes `/jobs/{id}/events` and fetches snapshots and analysis
    for exact snapshot revisions.

## File Map

```text
.
├── Cargo.toml
│   Rust 1.95 crate metadata for the app package and registry dependency
│   requests.
├── solver.toml
│   Embedded search policy for construction and local search.
├── solverforge.app.toml
│   App metadata, demo IDs, model facts/entities, registry dependency sources,
│   and the `solverforge 0.19.0` runtime target.
├── Makefile
│   Local build, validation, inline browser smoke, and Space/Docker commands.
├── Dockerfile
│   Multi-stage Rust 1.95 Docker image for Hugging Face Spaces.
├── README.md
│   Run guide, dependency shape, API list, and learning path.
├── AGENTS.md
│   Repo-specific rules for future edits.
├── WIREFRAME.md
│   This architectural walkthrough.
├── docs/screenshot.png
│   Current browser screenshot used by the README.
├── src/
│   ├── domain/
│   │   `planning_model!` manifest, facts, `Lesson`, `Plan`, and indexes.
│   ├── constraints/
│   │   One timetable score rule per file plus the assembler in `mod.rs`.
│   ├── data/
│   │   Deterministic `LARGE` demo-data generator and entrypoints.
│   ├── solver/
│   │   Retained-job service and runtime event payload formatting.
│   └── api/
│       DTOs, REST routes, and SSE streaming.
└── static/
    ├── index.html
    ├── sf-config.json
    ├── generated/ui-model.json
    ├── app.js
    │   Browser controller, solver lifecycle, and REST API guide.
    └── views.js
        Timetable view rendering for group, room, and teacher perspectives.
```

## Demo Data

`src/data/data_seed/entrypoints.rs` exposes one demo ID:

- `LARGE`

The generator is deterministic. It builds 40 weekly timeslots, 20 teachers, 12
student groups, 300 unassigned lessons, and 10 typed rooms. The initial score is
`0hard/-600medium/0soft` because each lesson starts without a timeslot and room.

## API And Retained Runtime

The REST API handles discovery, job control, and snapshot reads:

- `/health` and `/info` expose liveness and app metadata.
- `/demo-data` and `/demo-data/{id}` expose the deterministic demo catalog.
- `/jobs` creates a retained solver job.
- `/jobs/{id}` and `/jobs/{id}/status` expose summary state.
- `/jobs/{id}/snapshot` returns an exact or latest snapshot.
- `/jobs/{id}/analysis` runs constraint analysis for a snapshot.
- `/jobs/{id}/pause`, `/jobs/{id}/resume`, and `/jobs/{id}/cancel` control a
  live job.
- `DELETE /jobs/{id}` removes a terminal retained job.
- `/jobs/{id}/events` streams typed lifecycle events.

## Frontend Layout

`static/app.js` owns the browser shell, tab state, demo loading, retained-job
controls, SSE handling, status bar, and visible REST API guide.

`static/views.js` owns lesson-specific presentation: group timetables, room
usage, teacher load, and data tables. The app intentionally uses an inline
Playwright smoke in `Makefile` instead of a `tests/e2e/` directory.

## Validation Surfaces

Use the Makefile as the repo-local workflow:

- `make fmt-check`
- `make clippy`
- `make build-release`
- `make test`
- `make test-e2e`
- `make test-slow`
- `make space-build`
- `make ci-local`
- `make pre-release`

`make test` runs Rust tests, frontend syntax checks, and the inline Playwright
browser smoke. `make ci-local` includes the Docker image build used by the
Hugging Face Space.
