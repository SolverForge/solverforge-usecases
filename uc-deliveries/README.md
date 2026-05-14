---
title: SolverForge Deliveries
emoji: 🚚
colorFrom: green
colorTo: blue
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: SolverForge delivery-route optimization example
---

# SolverForge Deliveries

![SolverForge Deliveries screenshot](docs/screenshot.png)

`solverforge-deliveries` is a SolverForge vehicle-routing example with retained
jobs, route geometry, insertion recommendations, and a browser plan viewer.

It answers one concrete question:

"Given depots, vehicles, delivery stops, capacities, and time windows, which
vehicle should visit each delivery and in what order?"

## Documentation Map

- `README.md`
  Quick start, concepts, API surface, and the shortest learning path.
- `WIREFRAME.md`
  Architecture and request/data flow across backend, runtime, maps, and UI.
- `AGENTS.md`
  Repo-specific contribution, validation, and documentation rules.
- `Makefile`
  The supported local command surface for development, validation, and
  Docker-based Hugging Face Space preparation.
- `Dockerfile`
  The Docker Space image build, using Rust 1.95 and the declared crates.io
  dependency line.

## Current Dependency Shape

The app package version is `1.0.1`; the release binary is
`solverforge_deliveries`.

This repo requires Rust `1.95` and declares crates.io dependencies. Direct
dependency declarations currently request these versions:

- `solverforge` `0.13.0`
- `solverforge-ui` `0.6.5`
- `solverforge-maps` `2.1.4`
- `axum` `0.8.9`
- `tokio` `1.52.3`
- `tokio-stream` `0.1.18`
- `tower-http` `0.6.10`
- `tower` `0.5.3`
- `serde` `1.0.228`
- `serde_json` `1.0.149`
- `rand` `0.10.1`
- `uuid` `1.23.1`
- `parking_lot` `0.12.5`
- `http-body-util` `0.1.3`

The app metadata in `solverforge.app.toml` mirrors that published dependency
shape and records `solverforge-cli` `2.0.4` as the current scaffold metadata
line.

## What SolverForge Is Doing Here

- `Delivery` is a problem fact: a stop the solver must place in a route.
- `Vehicle` is a planning entity: each vehicle owns one mutable route.
- `Vehicle.delivery_order` is the list planning variable.
- `Plan` is the planning solution, matching the current `solverforge-cli`
  default solution name and `src/domain/plan.rs` file shape.
- Constraints score assignment coverage, capacity, time-window violations, and
  total travel time.
- `solver.toml` selects list construction heuristics and local-search moves.
  It matches the hospital example's stop policy: 30 seconds total, or 5 seconds
  without improvement.

The app ships three city datasets:

- `PHILADELPHIA` (default)
- `HARTFORD`
- `FIRENZE`

Each dataset has ten vehicles: 82 Philadelphia deliveries, 50 Hartford
deliveries, and 80 Firenze deliveries. Philadelphia is the default road-network
baseline, and all current seed data has enough aggregate vehicle capacity and
reachable street-front coordinates for the configured delivery stops.

## Quick Start

```sh
make run-release
```

Then open `http://localhost:7860`.

To inspect the command surface:

```sh
make help
```

## Validation

Standard validation:

```sh
make test
```

Full local validation:

```sh
make ci-local
```

Space image validation:

```sh
make space-build
```

Live road-network smoke:

```sh
make test-live-road
```

`make test` includes Rust tests, browserless frontend tests, and Playwright
browser tests. Playwright is a root dev dependency in this publication bundle;
the app serves `solverforge-ui` browser assets from the declared Cargo crate.

`make ci-local` adds the Docker image build used by the Hugging Face Space.
`make pre-release` runs `make ci-local` and then the live road-network smoke
test. The live road test may touch the local `.osm_cache/` through
`solverforge-maps`; that cache is ignored by git and by Docker builds.

## Hugging Face Space Deployment

This repo is Docker-Space ready. The Space reads the README front matter,
builds `Dockerfile`, and expects the app to bind `PORT=7860`.

Local Space-equivalent commands:

```sh
make space-build
make space-run
```

## Read The Code In This Order

1. `src/domain/mod.rs`
   The `planning_model!` manifest and public domain exports.
2. `src/domain/plan.rs`
   The `Plan` solution and list-variable shadow-refresh wiring.
3. `src/domain/delivery.rs` and `src/domain/vehicle.rs`
   The fact and planning-entity models.
4. `src/domain/route_metrics/`
   Route preparation, CVRP hooks, scoring preview, route geometry, and insertion
   ranking.
5. `src/constraints/mod.rs`
   The constraint-set assembly point.
6. `src/constraints/*.rs`
   One scoring rule per file.
7. `src/data/data_seed/entrypoints.rs`
   Public demo-data IDs and generator dispatch.
8. `src/data/data_seed/{philadelphia,hartford,firenze}/`
   City depots and delivery coordinates.
9. `src/solver/service.rs`
   Retained-job orchestration over `SolverManager<Plan>`.
10. `src/api/routes.rs`, `src/api/dto.rs`, and `src/api/sse.rs`
    REST, DTO, and event-stream contracts.
11. `static/app/main.mjs`
    Browser controller.
12. `static/app/models/` and `static/app/ui/`
    Frontend plan normalization, preview modeling, layout, maps, tables, and
    modals.

## Project Shape

- `src/domain/`
  Planning model, domain types, route metrics, and test-only model checks.
- `src/constraints/`
  Incremental SolverForge scoring rules.
- `src/data/`
  Deterministic city demo-data generator.
- `src/solver/`
  Retained-job facade and runtime event payload formatting.
- `src/api/`
  Axum routes, DTOs, and SSE endpoint.
- `static/app/`
  Browser modules built on stock `solverforge-ui` assets.
- `Dockerfile`
  Multi-stage Rust 1.95 Alpine build for the Hugging Face Docker Space.
- `tests/api_contract/`
  Integration tests for catalog, jobs, lifecycle, SSE, and road-network smoke.
- `tests/frontend_models.test.mjs`
  Browserless frontend model tests.
- `tests/e2e/`
  Playwright tests for the served browser app, including asset loading, dataset
  switching, route highlighting, and retained solve start/stop.

## REST API

- `GET /health`
- `GET /info`
- `GET /demo-data`
- `GET /demo-data/{id}`
- `POST /jobs`
- `GET /jobs/{id}`
- `GET /jobs/{id}/status`
- `GET /jobs/{id}/snapshot`
- `GET /jobs/{id}/analysis`
- `GET /jobs/{id}/routes`
- `POST /jobs/{id}/pause`
- `POST /jobs/{id}/resume`
- `POST /jobs/{id}/cancel`
- `DELETE /jobs/{id}`
- `GET /jobs/{id}/events`
- `POST /recommendations/delivery-insertions`

Lifecycle semantics:

- `pause` requests a runtime-managed pause and checkpoint.
- `resume` continues from the retained checkpoint.
- `cancel` stops a live or paused job.
- `delete` removes a terminal retained job before the next fresh solve.
- `snapshot_revision={n}` is optional for snapshots, analysis, and routes.
- SSE reconnects bootstrap from the retained runtime status/event state.

## Routing Modes

`road_network` is the default and uses `solverforge-maps` to fetch or load a
road graph for the current plan bounds. `straight_line` is available as a fast
draft/testing mode. Route geometry is returned by `/jobs/{id}/routes` and drawn
in the browser map only when it matches the active snapshot and routing mode.
Road-network geometry follows the `solverforge-maps` graph route, projects each
leg endpoint onto the nearest road segment, and then stitches the exact
depot/delivery coordinate back in so visible legs reach the markers. The route
list highlights by vehicle id, not by line color, so one selected vehicle route
is emphasized while the rest of the map geometry is dimmed without changing the
user's current map viewport.

## Solver Policy

`solver.toml` is embedded by `Plan` and is the runtime source of truth:

- `list_clarke_wright` builds delivery routes.
- `list_k_opt` improves construction output before local search.
- local search combines nearby list change/swap, reverse, k-opt, ruin, and
  limited sublist moves.
- `late_acceptance` and `accepted_count` control acceptance and retained
  candidates.
- solving stops after 30 seconds total or after 5 unimproved seconds, matching
  the hospital tutorial.

The city seeds have coherent capacity and reachable road-network coordinates so
local search can reach feasible `0hard` road-network solutions. Current seed
sizes are 82 Philadelphia deliveries, 50 Hartford deliveries, and 80 Firenze
deliveries, each with ten vehicles.
