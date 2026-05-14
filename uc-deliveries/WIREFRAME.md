# solverforge-deliveries WIREFRAME

This file is the architectural map for the deliveries example.

`README.md` explains how to run and use the app. This document explains how the
pieces fit together and where each responsibility lives.

## Documentation Roles

- `README.md`
  Quick start, dependency shape, route list, and user-facing orientation.
- `WIREFRAME.md`
  Architecture, execution flow, and file-map walkthrough.
- `AGENTS.md`
  Repo-specific contribution, validation, and documentation rules.
- `Makefile`
  Local development, validation, and Space/Docker command surface.
- `Dockerfile`
  Hugging Face Docker Space image definition.
- `docs/screenshot.png`
  Current browser screenshot embedded by the README.

## What This Repo Is Teaching

This repo is a complete `solverforge-deliveries` `1.0.1` list-variable
SolverForge app for delivery routing.

It shows how to combine:

- a `Plan` solution with a list planning variable
- route-specific score rules
- SolverForge CVRP construction and local-search hooks
- `solverforge-maps` road-network preparation and route geometry
- retained jobs with snapshots, analysis, cancel, pause, resume, and SSE
- a browser plan viewer built on stock `solverforge-ui` assets

## SolverForge Concepts In Plain Language

- `Delivery`
  Input stop data. The solver assigns delivery IDs into vehicle routes.
- `Vehicle`
  Planning entity. Each vehicle owns one ordered `delivery_order` list.
- `Plan`
  Planning solution. It holds deliveries, vehicles, score, routing mode, view
  state, and prepared routing data.
- hard score
  Missing assignments, capacity overage, late delivery seconds, and unreachable
  route legs.
- soft score
  Total travel seconds.
- retained job
  A solve that lives in memory so the UI can stream events, fetch snapshots,
  pause/resume, cancel, analyze, and delete terminal jobs.

## Runtime Flow

1. The browser loads `static/index.html`.
2. `static/app/main.mjs` loads `static/sf-config.json`.
3. The app fetches `/demo-data/PHILADELPHIA`.
4. `PlanDto::from_plan()` serializes a refreshed transport plan with preview
   state.
5. The browser normalizes the plan, renders summary cards, tables, timelines,
   and a map.
6. When the user clicks Solve, the browser sends the current plan to
   `POST /jobs`.
7. `src/api/routes.rs` deserializes the `PlanDto`, rebuilds a `Plan`, and calls
   `prepare_plan()`.
8. `prepare_plan()` builds straight-line or road-network matrices and attaches
   `PreparedVehicleRouting` to each vehicle.
9. `SolverService` starts a retained solve through `SolverManager<Plan>`.
10. Solver events are converted by `src/solver/service/runtime_payload.rs` into
    UI-facing JSON.
11. The browser consumes `/jobs/{id}/events` and fetches snapshots, analysis,
    and route geometry for exact snapshot revisions.
12. Road-network route geometry follows the `solverforge-maps` graph route,
    projects each leg endpoint onto the nearest road segment, and stitches the
    exact depot or delivery coordinate back in before encoding so each displayed
    leg reaches the visible markers.

## File Map

```text
.
├── Cargo.toml
│   Rust 1.95 crate metadata for app version 1.0.1 and registry dependency
│   requests.
├── solver.toml
│   Embedded search policy for construction heuristics and local search.
├── solverforge.app.toml
│   App metadata, demo IDs, model facts/entities, registry dependency sources,
│   and the `solverforge 0.13.1` runtime target.
├── Makefile
│   Hospital-style local build, validation, and Space/Docker commands.
├── Dockerfile
│   Multi-stage Rust 1.95 Docker image for Hugging Face Spaces.
├── .dockerignore
│   Keeps build artifacts, git metadata, route cache, and Playwright output out
│   of the Docker context.
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
│   │   `planning_model!` manifest, `Plan`, entities, facts, preview structs,
│   │   and route metrics.
│   ├── constraints/
│   │   Assignment, capacity, time-window, and travel-time score rules.
│   ├── data/
│   │   Deterministic city seed data and generator entrypoints.
│   ├── solver/
│   │   Retained-job service and runtime event payload formatting.
│   └── api/
│       Axum routes, DTOs, and SSE endpoint.
├── static/
│   ├── index.html
│   ├── sf-config.json
│   ├── generated/ui-model.json
│   └── app/
│       Browser controller, plan models, and UI renderers.
└── tests/
    ├── api_contract.rs
    │   Single integration-test crate composed from `tests/api_contract/*.rs`.
    ├── api_contract/
    │   Catalog, job, lifecycle, SSE, and live-road modules.
    ├── support/
    │   Shared integration-test helpers used by that single crate.
    ├── e2e/
    │   Playwright browser tests for the served app.
    └── frontend_models.test.mjs
        Frontend model tests.
```

## Domain And Route Metrics

`src/domain/plan.rs` stays small and macro-facing. It owns the `Plan` struct,
normalization, list shadow refresh, transport refresh, and the
`VrpSolution` implementation.

Route-specific behavior lives under `src/domain/route_metrics/`:

- `preparation.rs`
  Builds matrices and per-vehicle prepared routing data.
- `cvrp_hooks.rs`
  Supplies Clarke-Wright, k-opt, load, capacity, and route replacement hooks.
- `metrics.rs`
  Computes per-vehicle route metrics.
- `scoring.rs`
  Builds preview DTOs and aggregate hard/soft score components.
- `routes.rs`
  Builds straight-line or road-network route geometry snapshots, including
  edge-projected visual endpoints for road legs.
- `insertions.rs`
  Ranks candidate insertion positions for one delivery.
- `types.rs`
  Shared route metrics, snapshots, candidates, and routing trait types.

This split keeps the public domain API stable while avoiding oversized files.

## Demo Data

`src/data/data_seed/entrypoints.rs` exposes three demo IDs:

- `PHILADELPHIA`
- `HARTFORD`
- `FIRENZE`

Each city has a small module with separate depot and grouped visit files. The
generator is deterministic. Every demo has ten vehicle depots, enough scaled
deliveries for those vehicles, reachable road-network coordinates, and enough
aggregate capacity before route ordering.

## API And Retained Runtime

The REST API handles job control and snapshot reads:

- `/jobs` creates a retained solver job.
- `/jobs/{id}` and `/jobs/{id}/status` expose summary state.
- `/jobs/{id}/snapshot` returns an exact or latest snapshot.
- `/jobs/{id}/analysis` runs constraint analysis for a snapshot.
- `/jobs/{id}/routes` returns route geometry for a snapshot.
- `/jobs/{id}/events` streams typed lifecycle events.

The insertion endpoint, `/recommendations/delivery-insertions`, is app-specific.
It prepares the submitted plan, removes the requested delivery from any
existing route, evaluates candidate insert positions, and returns preview plans.

## Frontend Layout

`static/app/main.mjs` is the controller. It owns current plan state, retained
job state, route identity tracking, and event handlers.

Supporting modules are split by responsibility:

- `static/app/models/core.mjs`
  Clone and normalize incoming plans.
- `static/app/models/preview.mjs`
  Draft straight-line preview scoring.
- `static/app/models/timeline.mjs`
  Vehicle and delivery rail models.
- `static/app/models/formatters.mjs`
  Labels, icons, tones, clocks, and durations.
- `static/app/ui/layout.mjs`
  Page shell and stock SolverForge UI component composition.
- `static/app/ui/overview.mjs`
  Summary, route list, vehicle-id keyed route highlighting, map, and timeline
  rendering. The tutorial uses ten distinct map colors for its ten fixed
  vehicles.
- `static/app/ui/data-tables.mjs`
  Read-only vehicle and delivery tables with delivery insertion recommendations.
- `static/app/ui/modals.mjs`
  Analysis and insertion recommendation bodies.
- `static/app/ui/api-guide.mjs`
  Visible API guide content.
- `static/app/ui/lifecycle.mjs`
  Dataset markers and route-identity helpers.

## Validation Surfaces

Use the Makefile as the repo-local workflow:

- `make fmt-check`
- `make clippy`
- `make build-release`
- `make test`
- `make test-e2e`
- `make space-build`
- `make test-live-road`
- `make ci-local`
- `make pre-release`

`make ci-local` includes the Docker image build used by the Hugging Face Space.
The Playwright command uses the publication bundle's root Node dev dependency;
runtime UI assets are served from the declared `solverforge-ui` Cargo crate.

The file-size rule is part of the architecture: keep files below 300 lines and
split by responsibility before they become broad catch-all modules.
