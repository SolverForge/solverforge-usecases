# solverforge-fsr WIREFRAME

This file is the architectural map for the field-service routing example.

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
  Local development, validation, and Space/Docker command surface.
- `Dockerfile`
  Hugging Face Docker Space image definition.

## What This Repo Is Teaching

This repo is a complete `solverforge-fsr` list-variable SolverForge app for
field-service routing in Bergamo.

It shows how to combine:

- a `FieldServicePlan` solution with a list planning variable
- route-level hard and soft score rules
- precomputed travel-leg facts and `solverforge-maps` road-network geometry
- retained jobs with snapshots, analysis, cancel, pause, resume, and SSE
- a browser map workspace built on stock `solverforge-ui` assets

## SolverForge Concepts In Plain Language

- `Location`
  Input place data. Depots and customer sites are indexed so routes can refer to
  them cheaply.
- `ServiceVisit`
  Input job data. The solver places visit indexes into technician routes.
- `TravelLeg`
  Input travel data. Each leg records duration, distance, and whether the road
  graph can connect the two locations.
- `TechnicianRoute`
  Planning entity. Each technician owns one ordered `visits` list.
- `FieldServicePlan`
  Planning solution. It holds facts, route entities, and the current score.
- hard score
  Missing visits, duplicate visits, invalid visit indexes, unreachable legs,
  missing skills or parts, late visits, and route overtime.
- soft score
  Travel cost, workload balance, territory fit, and priority slack.
- retained job
  A solve that lives in memory so the UI can stream events, fetch snapshots,
  pause/resume, cancel, analyze, and delete terminal jobs.

## Runtime Flow

1. The browser loads `static/index.html`.
2. `static/app.js` loads `static/sf-config.json` and
   `static/generated/ui-model.json`.
3. The app fetches `/demo-data/STANDARD`.
4. The backend returns a `FieldServicePlan` with seed travel legs.
5. The browser renders route cards, tables, timeline, map shell, and the visible
   REST API guide.
6. When the user clicks Solve, the browser posts the current plan to
   `POST /jobs`.
7. `src/api/routes.rs` deserializes the `PlanDto`; domain deserialization and
   `FieldServicePlan::normalize()` restore transient service-visit indexes and
   route shadow fields before routing preparation.
8. `prepare_routing()` loads or fetches the Bergamo road network, computes the
   full travel matrix, and replaces seed legs with road-network legs.
9. `SolverService` starts a retained solve through
   `SolverManager<FieldServicePlan>`.
10. Solver events are converted by `src/solver/event_payload.rs` into
    UI-facing JSON.
11. The browser consumes `/jobs/{id}/events` and fetches snapshots, analysis,
    and route geometry for exact snapshot revisions.
12. `src/api/route_geometry.rs` builds map segments, preserving non-routed
    statuses so one unreachable leg does not hide the rest of a route.

## File Map

```text
.
├── Cargo.toml
│   Rust 1.95 crate metadata for the app package and registry dependency
│   requests.
├── solver.toml
│   Embedded search policy for list construction and local search.
├── solverforge.app.toml
│   App metadata, demo IDs, model facts/entities, registry dependency sources,
│   and the `solverforge 0.15.0` runtime target.
├── Makefile
│   Local build, validation, and Space/Docker commands.
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
│   │   `planning_model!` manifest, facts, route entity, shadows, and solution.
│   ├── constraints/
│   │   Stock SolverForge constraint streams, one score rule per file.
│   ├── data/
│   │   Deterministic Bergamo seeds, demo entrypoints, and OSM matrix loading.
│   ├── solver/
│   │   Retained-job service and runtime event payload formatting.
│   └── api/
│       Axum routes, DTOs, route geometry, and SSE endpoint.
└── static/
    ├── index.html
    ├── sf-config.json
    ├── generated/ui-model.json
    └── app*.js
        Browser controller, map rendering, route state, layout, and tables.
```

## Domain And Route Metrics

`src/domain/field_service_plan.rs` owns the public solution shape. It keeps the
SolverForge model explicit: facts are read-only inputs, while
`TechnicianRoute.visits` is the one mutable list variable. Its custom
deserializer and `normalize()` method rebuild skipped transient visit indexes
and route shadow values after JSON round trips, direct deserialization, and
seed travel-matrix replacement.

Route-specific scoring math lives in `src/domain/route_metrics.rs`. That module
walks a route from depot to visits to depot, advances a service clock, and
records reusable counters as `TechnicianRoute` shadow values. The constraint
files then use stock SolverForge `ConstraintFactory` streams over those shadow
fields. The assignment module uses stock streams for missing visits and invalid
route visit indexes; duplicate assignments use a small custom
`IncrementalConstraint` so `/jobs/{id}/analysis` reports matches only for visit
indexes assigned more than once while scoring each extra valid assignment.

`src/api/route_geometry.rs` is separate because map drawing has a different
job from scoring. Scoring consumes matrix facts already on the plan; geometry
loads the road graph to draw visible polylines for a retained snapshot.

## Demo Data

`src/data/data_seed.rs` exposes one demo ID:

- `STANDARD`

The generator is deterministic. It builds two depots, 24 customer locations, 48
visits, six technicians, and seed self-leg travel facts. Full road-network
travel facts are prepared when a job is created.

## API And Retained Runtime

The REST API handles discovery, job control, snapshots, and route geometry:

- `/health` and `/info` expose liveness and app metadata.
- `/demo-data` and `/demo-data/{id}` expose the deterministic demo catalog.
- `/jobs` creates a retained solver job.
- `/jobs/{id}` and `/jobs/{id}/status` expose summary state.
- `/jobs/{id}/snapshot` returns an exact or latest snapshot.
- `/jobs/{id}/analysis` runs constraint analysis for a snapshot.
- `/jobs/{id}/routes` returns route geometry for a snapshot.
- `/jobs/{id}/pause`, `/jobs/{id}/resume`, and `/jobs/{id}/cancel` control a
  live job.
- `DELETE /jobs/{id}` removes a terminal retained job.
- `/jobs/{id}/events` streams typed lifecycle events.

## Frontend Layout

`static/app.js` is the controller. It owns current plan state, retained job
state, route focus, event handlers, and analysis modal wiring.

Supporting modules split the UI by responsibility:

- `static/app-dataset.js`
  Demo catalog and plan loading.
- `static/app-layout.js`
  Page shell and stock SolverForge UI component composition.
- `static/app-route-state.js`
  Snapshot identity and route geometry cache coordination.
- `static/app-render*.js`
  Summary cards, route cards, maps, timeline, tables, and API guide.
- `static/app-utils.js`
  Plan cloning, labels, formatting, and color helpers.

## Validation Surfaces

Use the Makefile as the repo-local workflow:

- `make fmt-check`
- `make clippy`
- `make build-release`
- `make test`
- `make test-e2e`
- `make space-build`
- `make ci-local`
- `make pre-release`

`make ci-local` includes the Docker image build used by the Hugging Face Space.
The Playwright command uses the publication bundle's root Node dev dependency;
runtime UI assets are served from the declared `solverforge-ui` Cargo crate.
