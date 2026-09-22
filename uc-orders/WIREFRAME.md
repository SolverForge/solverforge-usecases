# SolverForge Orders Wireframe

## Runtime Flow

```text
GET /demo-data/STANDARD
        |
        v
Plan { pick_steps[145], trolleys[5].step_order[] }
        |
        +--> pure browser transforms --> overview / routes / data
        |
POST /jobs
        |
        v
SolverManager<Plan> --> SSE events --> retained snapshots --> re-render
        |
        +--> exact snapshot analysis --> score-analysis modal
```

The browser deep-clones the rendered plan before solving. `SF.createSolver()`
owns job creation, SSE parsing, pause/resume/cancel, snapshot revisions, and
terminal retention. The Axum app preserves `/sf`, `/jobs`, and `/demo-data`.

## Planning Layer

`PickStep` is an immutable fact representing one globally identified order item.
`Trolley` is the route-owning entity, and `step_order` is an ordered list of
dense pick-step indexes. List semantics construct one exact assignment for every
step. After a route mutation, the plan refreshes three trolley shadows: required
buckets, distinct orders, and closed-route distance in meters.

The depot and all product locations use the source's five-column, three-row
warehouse. A route begins at the trolley depot, visits each step in list order,
and closes at the same depot.

## Browser Surface

```text
Header: SolverForge Orders | Overview | Routes / Picking Plan |
        Warehouse / Data | REST API | Solve / Pause / Resume / Stop / Analyze
Status: lifecycle | hard/soft score | constraints | moves/second

Overview
  warehouse statement + depot marker
  KPIs: orders | pick items | active trolleys | closed-route meters
  five trolley route cards: stops | meters | required/available buckets
  explicit unassigned-work notice

Routes / Picking Plan
  assignment summary table
  rail timeline: one ordered lane per trolley
  explicit Unassigned work lane during construction

Warehouse / Data
  orders: item counts, volumes, trolley incidences
  products: 37 catalog rows, volume, location, pick count
  trolleys: bucket configuration, depot, route size

REST API
  demo, retained job, snapshots, analysis, telemetry, controls, and SSE guide
```

Timeline positions are integer minutes used only as sequence geometry. Domain
distance remains integer meters and is never converted to timeline duration.

## File Map

- `src/domain/warehouse.rs`: exact source distance semantics.
- `src/domain/plan.rs`: facts, list owners, score, and shadow refresh.
- `src/constraints/`: required buckets, incidence baseline, route meters.
- `src/data/data_seed.rs`: deterministic product catalog and order sequences.
- `src/api/`, `src/solver/`: retained API, SSE, analysis, and telemetry.
- `static/app/orders-model.mjs`: pure data, metrics, and timeline transforms.
- `static/app/main.mjs`: `SF.*` shell, rendering, and lifecycle controls.
- `tests/`: constraint, distance, frontend, browser, and solve acceptance tests.
