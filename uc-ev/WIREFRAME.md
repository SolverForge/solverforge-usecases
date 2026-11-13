# SolverForge EV Wireframe

## Runtime Flow

```text
GET /demo-data/STANDARD
        |
        v
Plan { customers[], chargers[], vehicles[].visit_order = [] }
        |
        +--> pure browser transforms --> KPIs + route cards + map + timeline
        |
POST /jobs
        |
        v
prepare_plan: solverforge-maps road network (cached in .osm_cache/)
        --> one seconds + metres matrix over customers, depot sites, chargers
        --> RoutingNetwork (frozen) + per-van CVRP ProblemData
        |
        v
SolverManager<Plan> --> SSE events --> retained snapshots --> re-render
        |                                     |
        |                                     +--> GET /jobs/{id}/routes?snapshot_revision=n
        |                                            --> road polylines per leg
        +--> exact snapshot analysis --> score-analysis modal
```

The browser deep-clones the plan before solving. `SF.createSolver()` owns job
creation, SSE parsing, pause/resume/cancel, snapshot revisions, and terminal
retention. Route geometry is requested per snapshot revision and dropped when a
newer snapshot arrives first.

## Planning Layer

```text
Vehicle.visit_order  (the only planning variable)
        |
        v  after every list change (post_update_listener)
walk_route(network, depot, terms, visit_order)
        |
        +--> legs: depot -> customers... -> depot
        |      each leg: direct metres/seconds + up to 3 charger candidates
        |
        +--> plan_charging (dynamic program over "last charged on leg k")
        |      minimise (range shortfall, added seconds)
        |
        +--> time walk: drive / detour + recharge / wait / unload / lateness
        |
        v
route shadows: demand, distance, driving, charging, lateness,
               unreachable legs, shortfall  +  timed schedule[]
        |
        v
constraints: battery_shortfall, vehicle_capacity, customer_time_windows,
             unreachable_legs (hard)  +  total_travel_time (soft)
```

Construction calls the same walk through `EvRouteSolution::ev_route_feasible`:
Clarke-Wright merges and 2-opt changes are accepted only when the merged route
has no shortfall, lateness, overload, or missing road.

## Browser Surface

```text
Header: SolverForge EV | Overview / Schedule / Data / REST API | Solve ... Analyze
Status: lifecycle | hard/soft score | constraint dots | moves/second

Overview
  left:  fleet KPIs (distance, driving, charging stops/time, shortfall, late)
         route cards (load, km, charging, lowest charge, return; click = focus)
  right: Leaflet map: hub, pharmacies coloured by van, chargers (coloured
         when used), road polylines per van, charger legs dashed

Schedule
  rail timeline, one lane per van: deliveries (green, red when late),
  charging stops (amber)

Data: vans, pharmacies, chargers      REST API: endpoint list with curl
```

## File Map

- `src/domain/charging.rs`: charging-stop dynamic program.
- `src/domain/route_walk.rs`: per-route load, time, battery, schedule.
- `src/domain/routing_network.rs`, `prepare.rs`: frozen road data.
- `src/domain/ev_route_hooks.rs`, `ev_savings_hooks.rs`: construction hooks.
- `src/domain/route_geometry.rs`: snapshot polylines.
- `src/constraints/`: five rules and their tests.
- `src/data/data_seed.rs`, `data_seed/bergamo_places.rs`: demo plans.
- `static/app/model.mjs`: pure transforms (Node-tested).
- `static/app/render.mjs`, `layout.mjs`, `main.mjs`: `SF.*` surface and lifecycle.
