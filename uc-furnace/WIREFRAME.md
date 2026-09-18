# solverforge-furnace WIREFRAME

This file is the architectural map for the heat-treatment furnace scheduling
example.

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

This repo is a complete `solverforge-furnace` SolverForge app for heat-treatment
job-shop scheduling.

It shows how to combine:

- one `Plan` solution with two planning entities and six scalar planning
  variables per furnace assignment plus roster
- one-model scheduling: furnace timing, task operators, monitoring capacity,
  and the operator shift roster are solved together
- assignment-completeness, hard feasibility, and soft schedule-quality rules
- deterministic `STANDARD` demo-data generation from domain facts
- named grouped-scalar providers and conflict-repair providers registered from
  Rust and referenced by `solver.toml`
- retained jobs with snapshots, analysis, cancel, pause, resume, and SSE
- a browser furnace timeline built on stock `solverforge-ui` assets

## SolverForge Concepts In Plain Language

- `Furnace`, `WorkOrder`, `Operator`, `Shift`, and `ShiftCoverageDemand`
  Input facts. The solver reads them but does not move them.
- `FurnaceAssignment`
  Planning entity. Each work order receives one furnace slot and up to four
  task-operator choices.
- `OperatorShiftAssignment`
  Planning entity. Each operator and roster day receives one shift choice.
- `assignment`
  Scalar planning variable on `FurnaceAssignment`. Its value encodes both the
  target furnace and the start time slot as
  `furnace_idx * NUM_TIME_SLOTS + time_slot_idx`.
- `load_build_operator_id`, `program_operator_id`, `quench_operator_id`,
  `unload_operator_id`
  Scalar planning variables on `FurnaceAssignment` that choose the operator for
  each manual task.
- `shift_type`
  Scalar planning variable on `OperatorShiftAssignment`. Values are Morning,
  Afternoon, Night, or Off.
- `Plan`
  Planning solution. It holds facts, both entity collections, and the current
  `HardSoftScore`.
- hard score
  Furnace compatibility, overlap, changeover, shift ownership, staffing,
  monitoring capacity, and roster legality.
- soft score
  Priority-weighted lateness, thermal changeover, early starts, night starts,
  overtime, and shift load balance.
- retained job
  A solve that lives in memory so the UI can stream events, fetch snapshots,
  pause/resume, cancel, analyze, and delete terminal jobs.
- scalar group
  A named Rust provider that tells SolverForge how to generate atomic
  candidates for one or more scalar variables.
- conflict repair
  A named Rust provider that turns a hard-constraint violation into concrete
  repair candidates for the search.

## Runtime Flow

1. The browser loads `static/index.html`.
2. `static/app.js` loads `static/sf-config.json`,
   `static/generated/ui-model.json`, and `solverforge-ui` assets from `/sf/*`.
3. `static/app/runtime.js` fetches `/demo-data/STANDARD` and renders the
   unsolved plan.
4. `src/data/data_seed/` builds the canonical `Plan` from deterministic seed
   data.
5. When the user clicks Solve, the browser posts the current plan to
   `POST /jobs`.
6. `src/api/routes/jobs.rs` deserializes `PlanDto` back into `Plan`.
7. `SolverService` starts a retained solve through `SolverManager<Plan>`.
8. Solver events are converted by `src/solver/service/events.rs` into
   UI-facing JSON.
9. The browser consumes `/jobs/{id}/events`, and fetches snapshots and analysis
   for exact snapshot revisions.
10. `static/app/render.js` and `static/app/schedule.js` repaint the furnace
    timeline, crew table, order table, and constraint status.

## File Map

```text
.
├── Cargo.toml
│   Rust 1.95 crate metadata for the app package and registry dependency
│   requests.
├── solver.toml
│   Embedded search policy: construction groups, VND hard repair, and SA
│   polish.
├── solverforge.app.toml
│   App metadata, demo IDs, model facts/entities/variables, the 30-constraint
│   manifest, and the `solverforge 0.19.4` runtime target.
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
│   │   `planning_model!` manifest, facts, both planning entities, `Plan`, and
│   │   time/changeover/label helpers.
│   ├── constraints/
│   │   Seven score-rule modules (furnace, staffing, monitoring, roster hard and
│   │   soft) plus the assembler in `mod.rs`.
│   ├── data/
│   │   Deterministic `STANDARD` demo-data generator, process specs, and seed
│   │   catalog.
│   ├── solver/
│   │   Retained-job service, SSE event bridge, grouped scalar providers, and
│   │   conflict-repair providers.
│   └── api/
│       DTOs, REST routes, analysis routes, and SSE streaming.
└── static/
    ├── index.html
    ├── sf-config.json
    ├── generated/ui-model.json
    ├── app.js
    └── app/
        runtime.js      Browser boot, solver lifecycle, tabs, and API guide.
        render.js       Overview KPIs, crew table, orders table, data tables.
        schedule.js     Furnace-lane timeline model and unassigned section.
        analysis.js     Score-analysis modal and per-constraint drilldown.
        lifecycle.js    Job id, revision, and lifecycle-status mirroring.
        api.js          Visible REST API guide endpoints.
        utils.js        Plan cloning and small formatting helpers.
        constants.js    Shared frontend constants.
```

## Demo Data

`src/data/data_seed/` exposes one demo ID:

- `STANDARD`

The generator is deterministic with seed `0x00DECAFBAD17A11A`. It builds 11
furnaces (Chamber, Pit, Car Hearth, and Aluminum types), 155 work orders across
nine heat-treatment processes, 39 operators across five roles, 22 shifts, and 44
`(shift, role)` coverage demands over a seven-day, 15-minute-step horizon. It
starts with every order and operator unassigned.

## API And Retained Runtime

The REST API handles discovery, job control, snapshot reads, and analysis:

- `/health` and `/info` expose liveness and app metadata.
- `/demo-data` and `/demo-data/{id}` expose the deterministic demo catalog.
- `/jobs` creates a retained solver job and returns `{id}`.
- `/jobs/{id}` and `/jobs/{id}/status` expose summary state with the analyzed
  snapshot score.
- `/jobs/{id}/snapshot` returns an exact or latest snapshot.
- `/jobs/{id}/analysis` runs constraint analysis for a snapshot.
- `/jobs/{id}/analysis/{constraint_name}` drills into one constraint.
- `/jobs/{id}/pause`, `/jobs/{id}/resume`, and `/jobs/{id}/cancel` control a
  live job.
- `DELETE /jobs/{id}` removes a terminal retained job.
- `/jobs/{id}/events` streams typed lifecycle events.

## Frontend Layout

`static/app/runtime.js` owns the browser shell, tab state, demo loading,
retained-job controls, SSE handling, status bar, and visible REST API guide.

`static/app/schedule.js` owns the furnace-lane timeline model. One lane is
created per furnace, and scheduled orders become timeline items colored by
priority.

`static/app/render.js` owns everything else the user sees: the overview KPI
table, the crew table, the orders table, and the generic data tables. The app
intentionally uses an inline Playwright smoke in `Makefile` instead of a
`tests/e2e/` directory.

## Solver Providers

`solver.toml` refers to provider names that are registered from Rust:

- Grouped scalar groups in `src/solver/scalar_groups.rs`:
  `furnace_assignment`, `roster_shift_assignment`, `task_operator_assignment`,
  and the composite `furnace_schedule_task_assignment`.
- Conflict-repair providers in `src/solver/conflict_repair.rs`:
  `minimumRest`, `visibleShiftLimit`, `taskOperatorOnOwningShift`,
  `shiftRoleCoverage`, `monitoringCapacity`, `furnaceOverlap`, and
  `changeoverGap`.

The `Plan` macro wires both by name through the
`scalar_groups = "crate::solver::scalar_groups::groups"` and
`conflict_repairs = "crate::solver::conflict_repair::providers"` attributes.

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
browser smoke. `make test-slow` runs the ignored
`standard_demo_solves_to_feasible_terminal_state` acceptance solve. `make
ci-local` includes the Docker image build used by the Hugging Face Space.
