---
title: SolverForge Fleet
emoji: ⚓
colorFrom: blue
colorTo: slate
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: SolverForge synthetic fleet readiness planning example
---

# SolverForge Fleet

`solverforge-fleet` is a synthetic, non-operational fleet-readiness planning
app. It coordinates vessel maintenance, inspections, training, docks,
technicians, parts, and readiness policy over a 56-day horizon.

It answers one concrete question:

"When and where should each vessel's maintenance run, and when should its
inspection and training occur, while preserving daily fleet readiness?"

## Quick Start

```sh
make run-release
```

Then open `http://localhost:7860`. Run `make help` to inspect the complete
local command surface.

## Documentation Map

- `README.md`: quick start, model, constraints, APIs, solver policy, and validation.
- `WIREFRAME.md`: as-built architecture and runtime/data flow.
- `AGENTS.md`: maintenance, documentation, and validation rules.
- `Makefile`: supported development, test, browser, Docker, and slow-solve hooks.
- `Dockerfile`: multi-stage repository-local container image.

## Current Dependency Shape

- Package and release binary: `solverforge-fleet` `0.1.0`
- Rust: `1.95`
- SolverForge request/runtime target: `solverforge` and `solverforge-core`
  `0.19.4`; the current lockfile resolves the compatible `0.19.5` releases
- Browser UI assets: `solverforge-ui` `0.6.5`
- Scaffold metadata: `solverforge-cli` `2.2.2` in `solverforge.app.toml`
- HTTP runtime: Axum `0.8.9`, Tokio `1.52.3`, and tower-http `0.6.10`

These values come from `Cargo.toml` and `solverforge.app.toml`. The app serves
the REST API, retained-job SSE stream, static fleet workspace, and registry
backed `solverforge-ui` assets from one process.

## Model Concepts

Problem facts include vessels, docks, 56 days, technician pools and dated
capacity overrides, parts and deliveries, dock outages, readiness policy, and
inspection/training requirements.

Planning entities and scalar variables:

- Each `WorkPackage` chooses `dock_idx` and `start_day_idx`.
- Each `InspectionAssignment` chooses `day_idx`.
- Each `TrainingAssignment` chooses `day_idx`.
- `Plan` holds all facts and entities plus the current `HardSoftScore`.

All variables allow unassigned values. Candidate lists limit each decision to
domain-valid docks and days, while nearby-value distance meters keep local
search centered around the current or baseline assignment. There are no list
planning variables or route-sequencing decisions.

The deterministic public dataset has 24 synthetic vessels in three classes,
24 work packages, 5 docks, 5 technician pools, 12 parts, 8 deliveries, and 56
days grouped into eight display weeks. The default `BASELINE_FULL` demo starts
all planning variables unassigned. `BASELINE` is an alias of the same baseline
problem, while `TECHNICIAN_SHORTAGE` applies an electrical-capacity reduction.

## Constraints

Hard constraints:

- All non-deferrable work and all inspection/training decisions are assigned.
- Work, inspection, and training stay inside their allowed windows.
- Dock class compatibility and per-dock capacity are respected.
- Persisted dock outages cannot overlap scheduled maintenance.
- Inspections follow maintenance and training follows inspections.
- Daily technician demand stays within regular plus overtime capacity and any
  dated capacity override.
- Parts consumption cannot exceed opening stock plus arrived deliveries.
- Overall, patrol-cutter, and frigate-like readiness floors hold every day.

Soft constraints:

- Maximize ready vessel-days.
- Minimize readiness shortfall, inspection lateness, overtime, changes from
  baseline assignments, and allowed low-priority deferrals.

The identifier `weekly_readiness_floor` is historical. Its hard score does not
sample once per week: `count_weekly_readiness_floor` iterates every modeled day
and enforces all three readiness thresholds daily. The workspace renders all 56
days; the separate weekly result summary samples the first day of each display
week and does not weaken that enforcement.

## REST And SSE API

Retained SolverForge jobs:

- `GET /health`
- `GET /info`
- `GET /demo-data`
- `GET /demo-data/{id}`
- `POST /jobs`
- `GET /jobs/{id}`
- `DELETE /jobs/{id}`
- `GET /jobs/{id}/status`
- `GET /jobs/{id}/snapshot`
- `GET /jobs/{id}/analysis`
- `POST /jobs/{id}/pause`
- `POST /jobs/{id}/resume`
- `POST /jobs/{id}/cancel`
- `GET /jobs/{id}/events`

Snapshots and analysis accept optional `snapshot_revision={n}`. The SSE route
sends a bootstrap status or snapshot event before live retained-job events.

Fleet plan sessions and compatibility routes:

- `POST /plan-sessions`
- `GET /plan-sessions/{id}/status`
- `GET /plan-sessions/{id}/result`
- `GET /plan-sessions/{id}/snapshots/{revision}`
- `GET /plan-sessions/{id}/snapshots/{revision}/analysis`
- `POST /plan-sessions/{id}/repair`
- `GET /plan-sessions/{id}/revisions/{from}/compare/{to}`
- `POST /scenarios/load`
- `POST /scenarios/perturb`
- `POST /solves`
- `GET /solves/{id}/status`
- `GET /solves/{id}/result`
- `POST /solves/compare`

The literal revision `latest` selects the latest snapshot, and `next` selects
the repair job linked from a baseline revision. Repair events cover technician
capacity changes, parts delays, readiness-floor changes, and dock outages.
Repair reports `repair_mode: "derived_resolve_with_lineage"`: it mutates a
selected snapshot, starts a new retained solve, and records comparison lineage.
It is not checkpoint continuation.

## Solver Policy

`solver.toml` is embedded by `Plan` and is the runtime source of truth.

- `environment_mode = "reproducible"` and `random_seed = 23` make search repeatable.
- Cheapest-insertion construction assigns scalar decisions when candidates exist.
- Late-acceptance local search uses a 400-step history.
- The accepted-count forager considers up to four accepted moves per step.
- Solving stops after 60 seconds.

## Validation

Standard validation:

```sh
make test
```

Full local validation:

```sh
make ci-local
```

Slow acceptance solve:

```sh
make test-slow
```

`make test` runs Rust tests, Node syntax and transform tests, and a Playwright
browser smoke against the release binary. `make ci-local` adds
formatting, clippy, a release build, and the Docker image build. `make
pre-release` adds the ignored full `BASELINE_FULL` solve, which must terminate
completed with a zero hard score.

## Deployment

This app is repository-only. Its Docker image binds `PORT=7860` and can be
built and exercised locally, but `uc-fleet` is not in the repository's Hugging
Face synchronization allowlist and has no hosted Space.

```sh
make docker-build
make docker-run
```

## Read The Code In This Order

1. `src/domain/mod.rs` and `src/domain/plan.rs`: planning manifest and solution.
2. `src/domain/work_package.rs`, `inspection_assignment.rs`, and
   `training_assignment.rs`: scalar planning entities and candidate domains.
3. `src/data/data_seed/`: deterministic 24-vessel, 56-day scenario.
4. `src/constraints/mod.rs` and `src/constraints/support/`: score rules and domain calculations.
5. `src/solver/service.rs`: retained-job orchestration and SSE event bridge.
6. `src/api/routes.rs` and `src/api/plan_sessions/`: REST, sessions, repair, and comparison.
7. `static/app.js` and `static/fleet/`: planner workspace and timelines.
