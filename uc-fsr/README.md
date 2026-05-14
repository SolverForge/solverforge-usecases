---
title: SolverForge Field Service Routing
emoji: 🧰
colorFrom: indigo
colorTo: blue
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: SolverForge field-service routing example
---

# SolverForge FSR

![SolverForge FSR screenshot](docs/screenshot.png)

`solverforge-fsr` is a SolverForge field-service routing example with retained
jobs, route geometry, technician schedules, and a browser map workspace.

It answers one concrete question:

"Given technicians, service visits, skills, parts, shifts, territories, and
road-network travel, which technician should serve each visit and in what
order?"

## Documentation Map

- `README.md`
  Quick start, concepts, API surface, and the shortest learning path.
- `WIREFRAME.md`
  Architecture and request/data flow across backend, routing, runtime, and UI.
- `AGENTS.md`
  Repo-specific contribution, validation, and documentation rules.
- `Makefile`
  The supported local command surface for development, validation, and
  Docker-based Hugging Face Space preparation.
- `Dockerfile`
  The Docker Space image build, using Rust 1.95 and the declared crates.io
  dependency line.

## Current Dependency Shape

The app package version is `1.0.1`; the release binary is `solverforge_fsr`.

This repo requires Rust `1.95` and declares crates.io dependencies. Direct
dependency declarations currently request these versions:

- `solverforge` `0.13.0`
- `solverforge-core` `0.13.0`
- `solverforge-ui` `0.6.5`
- `solverforge-maps` `2.1.4`
- `axum` `0.8.9`
- `tokio` `1.52.3`
- `tokio-stream` `0.1.18`
- `tower-http` `0.6.10`
- `tower` `0.5.3`
- `serde` `1.0.228`
- `serde_json` `1.0.149`
- `uuid` `1.23.1`
- `parking_lot` `0.12.5`

The app metadata in `solverforge.app.toml` records `solverforge-cli` `2.0.4`
as the current scaffold metadata line.

## What SolverForge Is Doing Here

- `Location` is a problem fact: a depot or customer coordinate.
- `ServiceVisit` is a problem fact: a customer job the solver must place in a
  technician route.
- `TravelLeg` is a problem fact: precomputed duration, distance, and reachability
  between two locations.
- `TechnicianRoute` is the planning entity: each technician owns one mutable
  route.
- `TechnicianRoute.visits` is the list planning variable.
- `FieldServicePlan` is the planning solution.
- Constraints score assignment coverage, route reachability, skills, parts,
  time windows, shift capacity, travel time, workload balance, territory
  affinity, and priority slack.
- `solver.toml` selects list construction and local-search moves over the visit
  list variable.

The app ships one deterministic `STANDARD` Bergamo dataset with two depots, six
technicians, 24 customer locations, and 48 service visits.

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

Full local Space validation:

```sh
make ci-local
```

`make test` runs Rust tests, JavaScript syntax checks, and Playwright browser
tests. Playwright is a root dev dependency in this publication bundle; the app
serves `solverforge-ui` browser assets from the declared Cargo crate.
`make ci-local` adds formatting, clippy, release build, and the Docker image
build used by the Hugging Face Space.

## Hugging Face Space Deployment

This repo is Docker-Space ready. Hugging Face reads the README front matter,
builds `Dockerfile`, and expects the app to bind `PORT=7860`.

Local Space-equivalent commands:

```sh
make space-build
make space-run
```

## Read The Code In This Order

1. `src/domain/mod.rs`
   The `planning_model!` manifest and public domain exports.
2. `src/domain/field_service_plan.rs`
   The solution type, fact collections, route entities, and score.
3. `src/domain/location.rs`, `src/domain/service_visit.rs`, and
   `src/domain/travel_leg.rs`
   The problem facts the solver reads.
4. `src/domain/technician_route.rs`
   The planning entity and list variable SolverForge mutates.
5. `src/data/data_seed.rs`
   Demo ID, Bergamo data assembly, road-network matrix preparation, and OSM
   cache policy.
6. `src/constraints/mod.rs`
   The constraint-set assembly point.
7. `src/constraints/route_metrics.rs`
   Shared route scoring math used by the individual constraints.
8. `src/api/routes.rs`, `src/api/dto.rs`, `src/api/route_geometry.rs`, and
   `src/api/sse.rs`
   REST, DTO, route geometry, and event-stream contracts.
9. `src/solver/service.rs`
   Retained-job orchestration over `SolverManager<FieldServicePlan>`.
10. `static/app.js` and `static/app-*.js`
    Browser lifecycle, dataset loading, route rendering, maps, tables, and API
    guide.

## Project Shape

- `src/domain/`
  Planning model, domain types, and route entities.
- `src/constraints/`
  Incremental SolverForge scoring rules.
- `src/data/`
  Deterministic Bergamo data and road-network preparation.
- `src/solver/`
  Retained-job facade and runtime event payload formatting.
- `src/api/`
  Axum routes, DTOs, route geometry, and SSE endpoint.
- `static/`
  Browser UI built on stock `solverforge-ui` assets.
- `Dockerfile`
  Multi-stage Rust 1.95 Alpine build for the Hugging Face Docker Space.

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

`snapshot_revision={n}` is optional for snapshots, analysis, and route
geometry. Route geometry reports unreachable, snap-failed, and no-path legs as
segment statuses so one bad road leg does not hide the rest of the route.

## Solver Policy

`solver.toml` is embedded by `FieldServicePlan` and is the runtime source of
truth:

- `list_round_robin` creates the first visit distribution.
- local search combines list change, swap, sublist change, sublist swap, and
  reverse moves over `TechnicianRoute.visits`.
- `hill_climbing` with `first_best_score_improving` keeps the tutorial easy to
  reason about.
- solving stops after 60 seconds.

## Constraints

Hard constraints:

- Assigned visits
- Reachable legs
- Required skills
- Required parts
- Time windows
- Shift capacity

Soft constraints:

- Minimize travel
- Balance workload
- Territory affinity
- Priority slack
