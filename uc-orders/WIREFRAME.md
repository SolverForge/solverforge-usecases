# SolverForge Orders Wireframe

## Runtime Flow

```text
GET /demo-data/STANDARD        POST /demo-data/generate
        |                              |
        +--------------+---------------+
                       v
Plan { pick_steps[145], trolleys[5].step_order[] }
        |
        +--> pure browser transforms --> isometric canvas + legend + stats + picking-plan cards
        |
POST /jobs
        |
        v
SolverManager<Plan> --> SSE events --> retained snapshots --> animated re-render
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

This is a faithful port of the source application's single-page interface on
top of the stock SolverForge lifecycle.

```text
Header: SolverForge Orders | Solve / Pause / Resume / Stop / Analyze
Status: lifecycle | hard/soft score | constraints | moves/second
Toolbar: Data toggle | current dataset summary

Dataset panel (toggled by Data)
  Orders / Trolleys / Buckets sliders + Generate New

Content
  statistics sidebar: Orders | Items | Active | meters
  isometric warehouse canvas (30-degree projection, 5x3 shelving grid)
    aisle-routed trolley paths, numbered pickup markers, animated while solving
    trolley legend overlay with colour swatch and item count

Picking plan
  one card per trolley: colour badge, item count, bucket-volume capacity bar,
  and the ordered item list
```

The canvas is a direct port of the source `logisim-view.js`; only its data
contract changed, because SolverForge serializes flat pick steps instead of
nested `orderItem.product` records. Card capacity bars show aggregate bucket
volume (total product volume against `bucketCount x bucketCapacity`), the source
visual; the hard constraint itself remains per-order ceiling demand.

## File Map

- `src/domain/warehouse.rs`: exact source distance semantics.
- `src/domain/plan.rs`: facts, list owners, score, and shadow refresh.
- `src/constraints/`: required buckets, incidence baseline, route meters.
- `src/data/data_seed.rs`: deterministic product catalog, the exact `STANDARD`
  sequences, and the requested-size generator behind Generate New.
- `src/api/`, `src/solver/`: retained API, SSE, analysis, telemetry, and the
  `POST /demo-data/generate` endpoint.
- `static/app/orders-model.mjs`: pure plan-to-view transforms and warehouse
  distance.
- `static/app/warehouse.mjs`: isometric canvas renderer ported from the source.
- `static/app/main.mjs`: `SF.*` shell, page layout, and lifecycle controls.
- `tests/`: constraint, distance, frontend, browser, and solve acceptance tests.
