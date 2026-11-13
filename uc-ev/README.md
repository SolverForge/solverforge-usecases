---
title: SolverForge EV
emoji: ⚡
colorFrom: green
colorTo: yellow
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: Route electric delivery vans with optimal charging stops.
---

# SolverForge EV

SolverForge EV plans one delivery day for a fleet of electric vans. It decides
which van serves each pharmacy and in what order, while every route respects
van capacity, delivery time windows, real road travel, and battery range. Where
a van stops to charge is not guessed by the search: for every visit order the
solver tries, the app places the charging stops that cost the least time.

This app is open source and repository-only. The front matter documents its
Docker surface, but no Hugging Face Space exists and it must not be added to
the sync allowlist without a matching target.

## Quick Start

```sh
make run-release
```

Open <http://localhost:7860>. The Bergamo plan loads with every route empty;
select **Solve** to build them. The first solve downloads the OpenStreetMap road
network around Bergamo (about a minute) and caches it in `.osm_cache/`; later
solves start in seconds.

![SolverForge EV solved overview](docs/screenshot.png)

Package metadata lives in `Cargo.toml`. The app requires Rust 1.95 and uses
`solverforge` 0.26.1, `solverforge-ui` 0.9.1, and `solverforge-maps` 2.3.3 under
Apache-2.0. It was scaffolded with `solverforge-cli` 4.0.4 and grown through its
`generate` commands.

## Planning Model

- Fact `Customer`: a pharmacy with crates to deliver, a delivery window
  (latest *finish* time), and an unloading time.
- Fact `Charger`: a public charging station any van may use.
- Entity `Vehicle`: an electric van with capacity, depot, departure time,
  battery range, charge at departure, a safety reserve it never dips below,
  and the time one full recharge takes. Its single planning variable is the
  list `visit_order` of customer indexes.
- Score: `HardSoftScore`.

Charging is modelled the way vans actually charge on a delivery day:

- charge is measured in metres of range; one metre driven uses one metre;
- a charging stop is a detour `stop -> charger -> next stop` and always refills
  the battery completely in the van's fixed `rechargeSeconds`;
- only charge above the reserve is usable.

Charging stops are derived, not searched. After every change to a route, the
route walk (`src/domain/route_walk.rs`) runs a dynamic program over the route's
legs (`src/domain/charging.rs`) that finds the stops minimising first the range
shortfall, then the added time. Each leg offers three candidate chargers: the
quickest detour, the one nearest the leg start, and the one nearest the leg
end. Over those candidates the result is exact; a unit test checks it against
exhaustive search on 400 generated routes.

Road data is prepared once per solve. `prepare_plan` loads the OpenStreetMap
network with `solverforge-maps`, computes one travel-time and distance matrix
over every pharmacy, depot, and charger, and attaches it to the plan. Scoring
only ever reads that frozen matrix.

## Constraints

- Hard `battery_shortfall`: metres a route would run below the reserve, after
  the best charging plan for its visit order.
- Hard `vehicle_capacity`: crates loaded beyond the van's capacity.
- Hard `customer_time_windows`: seconds a delivery finishes after its window
  closes, including delays caused by charging.
- Hard `unreachable_legs`: route legs the road network cannot connect.
- Soft `total_travel_time`: driving seconds (charger detours included) plus
  charging seconds, so shorter routes and fewer charges score better.

## Demo Data

The plan uses real places around Bergamo, Italy, from OpenStreetMap
(© OpenStreetMap contributors, ODbL): pharmacies across the city, the Brembana
and Seriana valleys, and Lake Iseo, and 14 public car chargers. One hub in
Stezzano dispatches every van at 07:30.

It is served as demo id `STANDARD`: 48 pharmacies and 6 vans.

Two pharmacies in three take a morning delivery (08:00–12:30), the rest an
afternoon one (14:30–18:30). The two van models alternate: 40 crates with
160 km range and a 45-minute recharge, or 28 crates with 110 km range and a
35-minute recharge. Vans leave with 40–90 km of charge and keep a 10 km
reserve, so most routes need a stop.

## Browser Surface

- Overview: fleet KPIs (distance, driving, charging stops and time, range
  shortfall, lateness), one card per van (load, distance, charging, lowest
  charge, return time; click to highlight), and a map of the hub, pharmacies
  coloured by van, chargers (coloured when used), and road routes. Legs to and
  from a charger are dashed.
- Schedule: a rail timeline with one lane per van; deliveries are green (red
  when late) and charging stops amber.
- Data: vans, pharmacies, and chargers.
- REST API: every endpoint with a ready `curl` command.

## REST API

- `GET /health`, `/info`, `/demo-data`, `/demo-data/STANDARD`
- `POST /jobs` and `/jobs/qualified`: prepare road data, then start a retained
  solve
- `GET /jobs/{id}`, `/status`, `/snapshot`, `/analysis`, `/telemetry`, `/events`
- `GET /jobs/{id}/routes?snapshot_revision={n}`: road geometry for one snapshot
- `POST /jobs/{id}/pause`, `/resume`, `/cancel`
- `DELETE /jobs/{id}`

Each van in a snapshot carries its route shadows (`routeDistanceM`,
`routeDrivingSeconds`, `routeChargingSeconds`, `routeLateSeconds`,
`routeShortfallM`, …) and a timed `schedule` of depart, charge, visit, and
return events with the battery level on arrival.

## Solver Policy

`solver.toml` builds routes with SolverForge's Clarke-Wright construction and a
2-opt pass, then runs late acceptance over nearby list change and swap, reverse,
3-opt, ruin-and-recreate, and sublist moves. Construction uses this app's own
hooks: routes merge only when the battery-aware check passes, and vans sharing a
depot share one savings computation, so the first plan is already feasible.
Search stops after 30 seconds.

## Validation

Browser tests use Node.js and the Playwright version pinned in `package.json`
and `package-lock.json`. Install them once per checkout:

```sh
make install-e2e
```

Browser tests launch the Chromium named by `PLAYWRIGHT_CHROMIUM_EXECUTABLE`,
then `/usr/bin/chromium`, and otherwise Playwright's bundled Chromium. Inside
the solverforge-usecases repository the root `make install-e2e` works too.

```sh
make test           # Rust, frontend transforms, browser smoke (offline)
make test-slow      # STANDARD solve on the live road network, zero hard
make test-live-e2e  # browser Solve/Pause/Resume to completion
make ci-local       # fmt, Clippy, release build, tests, Docker image
```

`make test` needs no network: the browser smoke never solves and map tiles are
stubbed. The Rust suite checks every constraint with exact scores, incremental
scoring against full evaluation over 300 random moves, and the charging planner
against exhaustive search.
