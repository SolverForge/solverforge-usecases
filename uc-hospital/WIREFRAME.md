# solverforge-hospital WIREFRAME

This file is the architectural map for beginners.

If `README.md` tells you how to run the app, this document tells you how the
pieces fit together and in which order to read them.

## Documentation Roles

The docs in this repo are meant to work together rather than compete:

- `README.md`
  Quick start, concepts, and user-facing orientation.
- `WIREFRAME.md`
  Architecture, execution flow, and file-map walkthrough.
- `docs/api-and-solver-policy.md`
  REST routes, lifecycle semantics, payload shape, and solver policy notes.
- `AGENTS.md`
  Rules for keeping code, comments, tests, and docs aligned in future changes.
- `Makefile`
  The shared developer command surface, including the local Space validation
  pipeline.
- `docs/screenshot.png`
  Current browser screenshot embedded by the README.

## What This Repo Is Teaching

`solverforge-hospital` is a complete SolverForge example, not just a scoring
snippet.

It shows how to build a planning app where:

- the domain model is small and explicit
- the score rules are readable one file at a time
- the dataset is deterministic and intentionally shaped
- the solver runs as a retained background job
- the browser UI watches the solve through REST and SSE

The planning question is:

"For each hospital shift, which employee should be assigned?"

## SolverForge Concepts In Plain Language

- `Employee`
  Input data. The solver reads it, but does not move it.
- `Shift`
  The thing the solver is allowed to assign.
- `employee_idx`
  The one real decision variable in this app. It points from a shift to one
  employee inside `Plan.employees`.
- hard score
  Rules that must not be broken, such as missing skills or overlapping shifts.
- soft score
  Preferences and quality goals, such as honoring desired days or balancing
  workload.
- retained job
  A solve that keeps living in memory after it starts, so the UI can poll it,
  pause it, resume it, stop it through runtime cancel, or inspect snapshots.
  Delete is terminal cleanup before the next fresh Solve, not the Stop action.

## Read Order

If you are new to this repo, read files in this order:

1. `src/domain/employee.rs`
   Learn the input facts first.
2. `src/domain/care_hub.rs`
   Learn the hospital service-line grouping used by nearby search.
3. `src/domain/mod.rs`
   See the `planning_model!` manifest that lists and exports the model modules.
4. `src/domain/plan.rs`
   Learn the planning entity, planning variable, nearby meters, and derived
   fields.
5. `src/constraints/mod.rs`
   See the full score model at a glance.
6. `src/constraints/*.rs`
   Read one scheduling rule per file.
7. `src/data/data_seed/entrypoints.rs`
   See the public demo-data surface.
8. `src/data/data_seed/large.rs`
   See how the published dataset is assembled.
9. `src/solver/service.rs`
   See how a domain solve becomes a retained runtime job.
10. `src/api/routes.rs` and `src/api/sse.rs`
   See the HTTP contract.
11. `static/app/main.mjs`
    See the browser boot sequence.
12. `static/app/shell/` and `static/app/schedule/`
    See how stock `solverforge-ui` components are adapted to this hospital demo.

## Runtime Flow

The shortest way to understand the app is to follow one request all the way
through:

1. The browser loads `static/index.html`.
2. `static/app/main.mjs` loads config and the generated UI model, validates the
   configured `LARGE` id through `/demo-data`, and then fetches
   `/demo-data/LARGE`.
3. The frontend turns the returned `PlanDto` into schedule rails and side-panel
   summaries.
4. When the user clicks Solve, the frontend sends the current plan to
   `POST /jobs`.
5. `src/api/routes.rs` converts that HTTP request into a `PlanDto`.
6. `PlanDto::to_domain()` rebuilds the in-memory `Plan`, including derived
   helper fields the solver expects.
7. `SolverService` starts a retained solve through `SolverManager<Plan>`.
8. The solver emits lifecycle events carrying telemetry and best-solution
   snapshots.
9. `src/solver/service.rs` coordinates the event stream, while
   `src/solver/service/payload.rs` converts runtime events into the JSON shapes
   expected by the UI.
10. The browser consumes those events over `/jobs/{id}/events` and updates the
    visible status and timeline; analysis is fetched from
    `/jobs/{id}/analysis` when requested.

The browser shell also contains a visible REST API guide. That makes
`static/app/shell/api-guide.mjs` part of the documentation surface, not just a
UI helper.

## File Map

```text
.
├── Cargo.toml
│   Rust crate metadata and registry dependency requests.
├── solver.toml
│   Embedded solver policy. This is the runtime source of truth for search.
├── solverforge.app.toml
│   App metadata, model surface, and the `solverforge 0.19.0` runtime target.
├── Dockerfile
│   Container build for running the app outside the dev checkout.
├── Makefile
│   Local build, validation, and Docker/Space workflow wrapper.
├── README.md
│   Beginner run guide and learning path.
├── docs/api-and-solver-policy.md
│   REST, payload, lifecycle, telemetry, and solver-policy reference.
├── WIREFRAME.md
│   This architectural walkthrough.
├── AGENTS.md
│   Repo-specific contributor and documentation rules.
├── docs/screenshot.png
│   Current browser screenshot used by the README.
├── src/
│   ├── lib.rs
│   │   Crate root and public module surface.
│   ├── main.rs
│   │   Axum server bootstrap, CORS, static serving, and route composition.
│   ├── domain/
│   │   `planning_model!` manifest plus problem model modules.
│   ├── constraints/
│   │   One scheduling rule per file plus the assembler in `mod.rs`.
│   ├── data/
│   │   Deterministic demo-data generator and published entrypoints.
│   ├── solver/
│   │   Retained-job facade over the SolverForge runtime.
│   └── api/
│       DTOs, REST routes, and SSE streaming.
├── static/
│   ├── index.html
│   │   Browser entrypoint.
│   ├── sf-config.json
│   │   Runtime UI config for the stock frontend shell.
│   ├── generated/ui-model.json
│   │   Generated view metadata used by `solverforge-ui`.
│   └── app/
│       ├── main.mjs
│       │   Browser boot and wiring.
│       ├── shell/
│       │   App shell, state, solver controls, and panels.
│       ├── schedule/
│       │   Hospital-specific grouping, presentation, and rail rendering.
│       └── views/registry.mjs
│           Named view registration.
└── tests/
    ├── frontend/
    │   Browserless frontend tests.
    ├── e2e/
    │   Playwright browser tests for the served app.
    └── support/
        Fake DOM support used by the frontend tests.
```

## Why The Model Looks This Way

This app is intentionally narrow.

- There is one planning entity type: `Shift`.
- There is one scalar planning variable: `employee_idx`.
- Nearby search is attached directly to that scalar variable.
- The solver does not juggle multiple variable types, sequence assignments, or
  list planning.

That makes the example easier to learn because the optimization problem stays
visible:

- facts live in `Plan.employees`
- decisions live in `Plan.shifts[*].employee_idx`
- score rules read those two things and judge the assignment

## Why The Demo Data Is Structured

The demo-data generator is not filler.

It is designed to give beginners a problem that is:

- deterministic
- feasible
- interesting enough that local search still has work to do
- stable enough that tests and comparisons stay meaningful

One important design trick is the hidden witness roster in
`src/data/data_seed/witness.rs`.

That internal roster gives the generator a known feasible staffing pattern.
The published dataset is then shaped around that witness, while the solver only
sees the final public problem. This lets the repo ship a realistic-feeling
demo without random feasibility failures.

## Why The Runtime Uses REST And SSE

The solve is long-lived compared with a normal request/response handler, so the
backend splits the contract into two parts:

- REST for control and snapshots
- SSE for live progress

The REST surface is:

- `/health` and `/info` expose liveness and app metadata.
- `/demo-data` and `/demo-data/{id}` expose the deterministic demo catalog.
- `/jobs` creates a retained solver job.
- `/jobs/{id}` and `/jobs/{id}/status` expose summary state.
- `/jobs/{id}/snapshot` returns an exact or latest snapshot.
- `/jobs/{id}/analysis` runs constraint analysis for a snapshot.
- `/jobs/{id}/pause`, `/jobs/{id}/resume`, and `/jobs/{id}/cancel` control a
  live job.
- `DELETE /jobs/{id}` removes a terminal retained job.
- `/jobs/{id}/events` streams typed lifecycle events.

That separation keeps the frontend simple:

- create a job
- poll or fetch details when needed
- subscribe once to the live event stream

It also mirrors how a real retained SolverForge app behaves in production.

One small but important detail: `GET /jobs/{id}` and `GET /jobs/{id}/status`
return the same summary payload. The second route exists as a stock-compatible
alias for clients that expect the explicit `/status` URL shape.

## Solver Policy

`solver.toml` is embedded by `Plan` and is therefore part of the actual model,
not a side document.

The shipped search policy is deliberately conservative:

- `cheapest_insertion` builds a feasible first assignment
- local search stays in the nearby scalar neighborhood
- `late_acceptance` and `accepted_count` keep the search moving without blowing
  up step cost

That narrow configuration is the shipped 30-second policy for this demo.
`make test-slow` is the acceptance gate for any solver-policy change.

## Frontend Design

The frontend is intentionally thin.

This repo is not trying to teach a custom framework. It is showing how to take
stock `solverforge-ui` pieces and adapt them to one concrete planning problem.

- `shell/` owns app lifecycle, backend wiring, and side panels
- `schedule/` owns the hospital-specific transformation from domain data to
  visual rails
- tests in `tests/frontend/` lock browserless presentation behavior down
- tests in `tests/e2e/` verify the served browser app with Playwright

## Validation Surfaces

When you change this repo, think in six separate layers:

1. Domain and constraint logic:
   `make test-rust`
2. Slow end-to-end solve quality:
   `make test-slow`
3. Frontend module correctness:
   `make test-frontend-syntax`
4. Frontend module behavior:
   `make test-frontend`
5. Served browser behavior:
   `make test-e2e`
6. Local deploy readiness for the Docker-based Space target:
   `make ci-local`

If you keep those six layers green, the repo remains teachable and usable.
