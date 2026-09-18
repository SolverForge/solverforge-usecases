# solverforge-fleet WIREFRAME

This is the as-built architecture map for the synthetic fleet-readiness
planning app. `README.md` explains use; this document traces ownership and data
flow.

## What This App Demonstrates

- One `Plan` with problem facts, three planning-entity collections, and four scalar variables.
- A deterministic 24-vessel scenario over 56 days and eight display weeks.
- Maintenance dock/start assignment plus inspection and training day assignment.
- Hard feasibility across windows, docks, precedence, technicians, parts, outages, and daily readiness.
- Soft optimization of ready vessel-days, shortfall, lateness, overtime, churn, and deferral.
- Retained jobs with status, snapshots, analysis, lifecycle controls, and SSE.
- Optional plan sessions that apply disruption events and launch derived repair solves with lineage.
- A vessel and dock timeline workspace built with `solverforge-ui` globals.

## SolverForge Concepts

- Facts: vessels, docks, days, technician pools/overrides, parts, deliveries,
  dock outages, readiness policies, and inspection/training requirements.
- Planning entities: work packages, inspection assignments, and training assignments.
- Scalar variables: `dock_idx`, `start_day_idx`, and the two assignment `day_idx` fields.
- `Plan`: the complete input/current assignment graph and optional `HardSoftScore`.
- Retained job: an in-memory solve exposed through REST snapshots, analysis, controls, and SSE.
- Plan session: Fleet's lineage wrapper around one active retained job and its repairs.

## Runtime Flow

1. `src/main.rs` builds shared `AppState`, merges Fleet API routes with
   `solverforge_ui::routes()`, and serves `static/` as the fallback.
2. The browser loads `static/index.html`, the global `/sf/sf.js` library,
   Fleet scripts, and `static/app.js`.
3. `app.js` reads `/sf-config.json` and `/generated/ui-model.json`, builds the
   header/status/tabs, and asks Fleet actions to load the baseline scenario.
4. `src/data/data_seed/` builds facts, empty planning entities, and candidate indices.
5. A solve posts a plan to `/jobs` or creates a `/plan-sessions` wrapper.
6. `SolverService` starts the embedded `solver.toml` policy through `SolverManager<Plan>`.
7. Status, snapshots, analysis, and typed SSE payloads expose retained progress.
8. Fleet schedule helpers convert snapshots into assignments, readiness
   summaries, resource pressure, metrics, and explanations.
9. A repair request reads a selected snapshot, applies a disruption, launches
   another retained solve, and stores lineage for revision comparison.

## Source Map

```text
.
├── Cargo.toml                 Rust 1.95 package and dependency metadata
├── solver.toml                Reproducible construction + late-acceptance policy
├── solverforge.app.toml       CLI 2.2.2 app/model metadata
├── Makefile                   Build, test, browser, Docker, and acceptance hooks
├── Dockerfile                 Multi-stage repository-local image
├── src/
│   ├── domain/                Plan, facts, entities, scalar candidate contracts
│   ├── constraints/           15 assembled hard/soft constraints
│   │   └── support/           Scoring, readiness, repairs, metrics, schedules
│   ├── data/data_seed/        24-vessel deterministic scenario and variants
│   ├── solver/                Retained manager and SSE lifecycle payloads
│   └── api/                   REST, SSE, sessions, repair, comparison
└── static/
    ├── index.html             Global SolverForge UI and Fleet script loading
    ├── app.js                 Workspace bootstrap, scenarios, and solver lifecycle
    ├── sf-config.json         Visible labels and constraint list
    ├── generated/ui-model.json
    └── fleet/
        ├── render.js          Tabs, summaries, API guide, comparison surfaces
        ├── schedule.js        Vessel and dock rail timelines
        └── utils.js           Shared frontend accessors and formatting
```

## Scenario Shape

The catalog contains 10 patrol cutters, 8 frigate-like vessels, and 6 support
vessels. Each vessel has one work package and one inspection assignment; the 18
patrol/frigate-like vessels also have training assignments. Five docks span
small, medium, and large classes. Days 1 through 56 carry week numbers 1 through
8 for summaries.

`BASELINE_FULL` and `BASELINE` produce the same fully unassigned baseline plan.
`TECHNICIAN_SHORTAGE` applies a global electrical technician reduction. Repair
events can additionally persist dated technician overrides and dock outages or
change delivery/readiness facts.

## Constraint Ownership

The hard rules are `required_assignments`, `work_window`,
`dock_compatibility`, `dock_capacity`, `dock_outage`,
`inspection_training_precedence`, `technician_capacity`,
`parts_availability`, and `weekly_readiness_floor`.

The soft rules are `maximize_ready_days`, `minimize_readiness_shortfall`,
`minimize_inspection_lateness`, `minimize_overtime`, `minimize_churn`, and
`minimize_deferral`.

`weekly_readiness_floor` is a historical identifier. Its counter traverses
`week_days(plan)`, which returns every day index, and evaluates overall plus
class floors each day. `weekly_readiness(plan)` is only a UI/result summary and
samples the first day of each week.

## API Layers

The base layer owns `/health`, `/info`, `/demo-data`, and retained `/jobs`
routes. `/jobs/{id}/events` sends an immediate bootstrap representation and
then broadcast lifecycle events.

The Fleet layer owns `/plan-sessions`, `/scenarios`, and `/solves`. A plan
session tracks its baseline DTO, active job, stored source revisions, and repair
links. `repair_mode = "derived_resolve_with_lineage"` accurately describes the
implementation: no solver checkpoint is resumed.

## Validation Surfaces

- `make test-rust`: all ordinary Rust tests, including project-shape contracts.
- `make test-frontend`: Node syntax checks plus `tests/frontend/` transform tests.
- `make test-e2e`: release server plus Playwright workspace smoke.
- `make test`: all three standard surfaces.
- `make test-slow`: ignored full unassigned-demo acceptance solve.
- `make ci-local`: fmt, clippy, release build, standard tests, and Docker build.
- `make pre-release`: `ci-local` followed by the slow solve.

The Docker image and README front matter are repository-local packaging
surfaces. They do not imply a hosted Space; `uc-fleet` is outside the root sync
allowlist.
