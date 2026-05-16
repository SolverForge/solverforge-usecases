# SolverForge Lessons Wireframe

`README.md` explains how to run and use the app. This file records the current
runtime shape so maintainers can keep backend, UI, and publication metadata in
sync.

## Runtime Flow

```text
Browser
  |
  | GET /
  v
Axum server in src/main.rs
  |
  | serves /sf/* from solverforge-ui
  | serves static/* from this app
  | exposes health, info, demo-data, and retained job routes
  v
Retained solver service in src/solver/
  |
  | builds demo data from src/data/
  | solves Plan from src/domain/
  | scores constraints from src/constraints/
  v
SSE and JSON DTOs in src/api/
```

## Browser Surface

The first viewport shows the standard SolverForge UI shell with tabs for group,
room, teacher, data, and REST API views. The Solve button starts the retained
job and the status strip reports the current score, lifecycle state, and
constraint count.

## Model

Lessons are planning entities. Teachers, student groups, rooms, and timeslots
are problem facts. The solver assigns each lesson to a timeslot and room while
avoiding teacher, group, and room conflicts and preferring better timetable
quality.

## Key Files

- `Cargo.toml`: crate and binary identity for `solverforge-lessons`.
- `solverforge.app.toml`: app metadata, entities, facts, variables, and
  constraints.
- `solver.toml`: solver termination and phase policy.
- `src/main.rs`: single-process Axum server used locally and in the Space.
- `static/sf-config.json`: visible title and view metadata.
- `docs/screenshot.png`: current browser screenshot.
