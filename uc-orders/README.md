---
title: SolverForge Orders
emoji: 🛒
colorFrom: amber
colorTo: green
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: Build feasible trolley routes for warehouse order picking.
---

# SolverForge Orders

SolverForge Orders assigns every immutable order item to exactly one trolley,
orders each trolley's closed warehouse walk, keeps order-dedicated bucket demand
within capacity, and minimizes split orders plus distance in meters.

This app is open source and repository-only. The front matter documents its
Docker surface, but no Hugging Face Space exists and it must not be added to the
sync allowlist without a matching target.

## Quick Start

```sh
make run-release
```

Open <http://localhost:7860>. The initial `STANDARD` plan deliberately shows all
145 pick steps as unassigned; select **Solve** to construct complete routes. The
header retains pause, resume, stop, telemetry, and exact snapshot analysis.

![SolverForge Orders solved overview](docs/screenshot.png)

Package and binary: `solverforge-orders` 0.1.0. The app requires Rust 1.95 and
uses `solverforge` 0.19.5 plus `solverforge-ui` 0.9.0 under Apache-2.0.

## Planning Model

- Fact: `PickStep`, one immutable flattened order item with globally stable step
  and order-item IDs, product ID/name/volume, order ID, and warehouse location.
- Entity: `Trolley`, with four buckets, 48,000 cm³ per bucket, depot `(A,1)`
  `LEFT` row 0, and one ordered `step_order` list variable.
- Assignment invariant: SolverForge list-variable semantics place every
  `pick_steps` index in exactly one trolley list. No duplicate scalar assignment
  model or post-solve repair exists.
- Score: integral `HardSoftScore`.

The authoritative Python source uses `HardSoftDecimalScore`, but every weight
and match value in this model is integral. `HardSoftScore` is therefore exactly
equivalent for these constraints and data, without decimal rounding behavior.

`STANDARD` reproduces source seed 37: 5 trolleys, 8 orders, 37 catalog products,
and the exact 145-product order sequences and product locations. The source
starts with round-robin routes only for visualization. This app starts lists
empty so SolverForge's construction phase owns the initial exact assignment;
all facts and optimization semantics remain unchanged.

## Constraints

- Hard `required_buckets`: for each trolley, sum
  `ceil(order volume on trolley / 48,000)` independently per order, then penalize
  one hard point per bucket beyond four.
- Soft `order_incidence`: penalize 1,000 per distinct trolley/order incidence.
  This intentionally preserves the source's unavoidable 8,000-point baseline
  when each order uses exactly one trolley.
- Soft `route_distance`: penalize one point per meter over depot-to-first,
  inter-step, and last-to-depot legs.

Warehouse distance reproduces every piecewise branch in `warehouse.py`: same
shelf/side, opposite shelf sides, contiguous aisle faces, other shelves on one
warehouse row, and shelves on different rows. Distances are displayed as meters;
the source UI's historical 100x display error is not reproduced.

## Browser Surface

The shared `SF.*` component library provides the header, status, rail timeline,
tables, API guide, modal, footer, solver controls, and lifecycle adapter. The
app adds overview KPIs, trolley route cards, ordered route lanes, an explicit
unassigned lane, and order/product/trolley data tables. Pure transforms live in
`static/app/orders-model.mjs` and are covered by Node tests.

## REST API

- `GET /health`, `/info`, `/demo-data`, `/demo-data/STANDARD`
- `POST /jobs` and `/jobs/qualified`
- `GET /jobs/{id}`, `/status`, `/snapshot`, `/analysis`, `/telemetry`, `/events`
- `POST /jobs/{id}/pause`, `/resume`, `/cancel`
- `DELETE /jobs/{id}`

`/sf/*` serves the pinned SolverForge UI assets. `/jobs/{id}/events` carries the
retained SSE lifecycle and scored snapshots used by `SF.createSolver()`.

## Solver Policy

`solver.toml` runs deterministic cheapest-insertion list construction followed by a
late-acceptance union of list change, swap, sublist, and reversal moves. Search
terminates after 30 seconds or 10 unimproved seconds. Route shadows recompute
bucket demand, incidences, and closed distance after each list-owner update.

## Validation

```sh
make test
make test-slow
make ci-local
```

`make test` covers Rust rules and distance branches, frontend transforms, and a
real Playwright lifecycle smoke. `make test-slow` requires the deterministic
demo to terminate at zero hard with every step assigned once. `make ci-local`
also runs formatting, Clippy, release build, and Docker build.
