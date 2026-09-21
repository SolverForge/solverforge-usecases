# SolverForge Flight Crew Wireframe

## Runtime Flow

```text
GET /demo-data/STANDARD
        |
        v
Plan { airports, employees, flights, crew_assignments }
        |
        +--> browser transforms --> By crew / By flight / Data
        |
POST /jobs
        |
        v
SolverManager<Plan> --> SSE events --> retained snapshots --> re-render
        |
        +--> exact snapshot analysis --> score-analysis modal
```

The browser deep-clones the rendered plan before starting a solve. SolverForge
owns job creation, SSE parsing, pause/resume/cancel, terminal retention, and
snapshot revision tracking.

## Planning Layer

`Airport`, `Employee`, and `Flight` are immutable facts. `CrewAssignment` is the
only planning entity, and `employee_idx` is its only planning variable. Dense
fact indexes are rebuilt after construction and API decoding. Pilot and cabin
candidate lists are model-owned slices rebuilt from employee qualifications.

Sequence rules group assigned duties by employee and sort them by duty start
before evaluating adjacent legs. This makes connection, rest, recovery, and
first/last-home behavior independent of collection insertion order.

## Browser Surface

```text
Header: SolverForge Flight Crew | By crew | By flight | Data | REST API | Solve
Status: lifecycle | hard/soft score | constraint indicators | moves/second

By crew
  KPI table: flights | crew | required seats | unassigned
  rail timeline: one lane per employee plus an explicit unassigned lane
  overlays: unavailable whole-day windows
  blocks: exact flight intervals, colored by pilot/cabin qualification

By flight
  KPI table: legs | fully crewed | open seats | long-haul legs
  rail timeline: one lane per flight with route, crew manifest, and coverage tone

Data
  flight, route, minute bounds, required-seat count

REST API
  retained job and exact snapshot endpoint guide
```

All timeline geometry uses integer minutes. Empty employee lanes remain visible,
and unassigned work is never discarded.

## File Map

- `src/domain/`: planning facts, entity, solution, and candidate hooks.
- `src/constraints/`: 12 named hard/soft rules.
- `src/data/data_seed.rs`: deterministic six-airport demo.
- `src/solver/`: retained `SolverManager` adapter and SSE payloads.
- `src/api/`: health, demo, jobs, snapshots, analysis, and control routes.
- `static/flightcrew-model.js`: pure domain-to-timeline transforms.
- `static/app.js`: shared `SF.*` shell and solver lifecycle.
- `tests/`: score deltas, transforms, browser flow, and full solve acceptance.
