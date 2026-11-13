# Repository Guidelines

## Project Structure And Naming

`solverforge-ev` is a Rust 1.95 SolverForge web app for electric-van delivery
routing with charging stops. Keep the repo-local directory name `uc-ev`;
product copy and metadata use `solverforge-ev` or SolverForge EV. The release
binary is `solverforge_ev`.

The app was scaffolded with `solverforge-cli` 4.0.4 (web shell) and its model
is grown through `solverforge generate|destroy`. Change facts, entities, the
`visit_order` variable metadata, and the constraint list through the CLI, never
by hand-editing managed blocks, `solverforge.app.toml`, or
`static/generated/ui-model.json`.

- `src/domain/mod.rs` owns the `planning_model!` manifest only.
- `src/domain/{customer,charger,vehicle,plan}.rs` own the facts, the `Vehicle`
  entity with its `visit_order` list variable and route shadows, and `Plan`.
- `src/domain/coord_value.rs` wraps coordinates so facts and entities can
  derive `Eq`/`Hash`.
- `src/domain/charging.rs` owns the charging-stop dynamic program.
- `src/domain/route_walk.rs` walks one route: load, time, battery, charges,
  and the timed schedule.
- `src/domain/routing_network.rs` owns the frozen road matrices and per-leg
  charger candidates; `prepare.rs` builds them with `solverforge-maps`.
- `src/domain/ev_route_hooks.rs` and `ev_savings_hooks.rs` own the list
  variable's route-local and Clarke-Wright hooks plus the `EvRouteSolution`
  trait the variable metadata requires.
- `src/domain/route_geometry.rs` builds map polylines for `/jobs/{id}/routes`.
- `src/constraints/` owns one named constraint per file; `tests.rs` holds the
  exact score tests and the incremental walk.
- `src/data/data_seed.rs` and `data_seed/bergamo_places.rs` own the
  deterministic `STANDARD` plan (the only demo; the page has no demo
  selector).
- `src/api/` owns REST, DTO, SSE, telemetry, and the routes endpoint;
  `src/solver/` owns retained jobs.
- `static/app/` owns the browser: `model.mjs` (pure transforms), `render.mjs`,
  `layout.mjs`, `api-guide.mjs`, and `main.mjs` (lifecycle wiring).

Keep the solution name `Plan` and the single demo id `STANDARD`. Keep
handwritten files under 300 lines.

## Model Contracts

`visit_order` holds indexes into `Plan.customers`; list semantics place every
customer exactly once. Chargers are never list elements: charging stops are
derived by `walk_route` after every route change. Do not add a charger list
variable, scalar charging decisions, a greedy initializer, or post-solve repair.

The battery model is metres of range, one metre per metre driven, a full
recharge in `recharge_seconds` at every stop, and a `reserve_charge_m` that is
never usable. `plan_charging` must stay exact over its candidate set; the
exhaustive-search test in `src/domain/charging/tests.rs` is the guard.

Matrix node order is `[customers, depot sites, chargers]`; vans sharing a depot
coordinate share one depot node and one Clarke-Wright class. The CVRP
`ProblemData` uses road seconds as its distance.

Constraint ids equal module names and are wire-visible in `/analysis`; the
browser maps them to labels in `static/app/model.mjs`.

## Build And Validation Commands

- `make run-release` runs the app on `:7860` (`PORT=7861` for another port).
- `make test` runs Rust tests, frontend transform tests, and the offline
  Playwright smoke.
- `make install-e2e` installs the Playwright version pinned in `package.json`
  (and Playwright's Chromium when no system Chromium exists).
- `make test-slow` solves `STANDARD` on the live road network and requires zero
  hard, exactly-once routing, and no van below zero charge.
- `make test-live-e2e` drives Solve/Pause/Resume to completion in a browser.
- `make ci-local` runs fmt, Clippy, release build, `make test`, and Docker.
- `make pre-release` runs `ci-local` plus both live checks.
- `solverforge check` validates managed blocks and model metadata.

Any constraint change needs an exact score test in `src/constraints/tests.rs`
and a green incremental walk. Any change to charging, route walking, or road
preparation needs `make test-slow`. Any browser transform change needs a Node
test in `tests/frontend/`; lifecycle changes need `make test-live-e2e`.

## Documentation

Assume a reader new to Rust and to planning optimization. Keep `README.md`,
`WIREFRAME.md`, this file, `solver.toml`, `solverforge.app.toml`,
`static/sf-config.json`, and `docs/screenshot.png` aligned with the code in the
same patch. Demo places are OpenStreetMap data; keep the ODbL attribution in
the README, the data module, and the browser footer.

This use case is repository-only. Do not add it to Hugging Face
synchronization without a real target Space and the root publication changes.
