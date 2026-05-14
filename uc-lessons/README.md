---
title: SolverForge Lessons
emoji: 📚
colorFrom: blue
colorTo: green
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: SolverForge lesson scheduling example
---

# solverforge-lessons

A SolverForge lesson timetable optimizer.

## Versioning

- CLI version used to scaffold this project: `2.0.4`
- SolverForge runtime target for this use case: `solverforge 0.13.0`
- SolverForge UI target for this scaffold: `solverforge-ui 0.6.5`
- Runtime dependency currently wired into `Cargo.toml`: `crates.io: solverforge 0.13.0`
- Frontend UI dependency currently wired into `Cargo.toml`: `crates.io: solverforge-ui 0.6.5`

This project is maintained as the SolverForge Lessons use case and currently targets `SolverForge 0.13.0` through the configured crate dependency.

## Model

The app schedules 300 lessons for 12 student cohorts across 40 weekly timeslots
and 10 typed rooms. Lessons are planning entities; timeslots, teachers, groups,
and rooms are problem facts.

The score type is `HardMediumSoftScore`:

- Hard constraints prevent teacher, group, and room overlaps; enforce teacher
  and group availability; and require enough room capacity.
- Medium constraints require every lesson to receive a timeslot and a room.
- Soft constraints prefer matching room types, earlier-day lessons, and avoid
  repeating the same subject twice on the same day for a cohort.

The shipped demo data surface exposes a single deterministic dataset:
`LARGE`. It starts with every lesson unassigned, so the initial score is
`0hard/-600medium/0soft`. Solving should reach hard/medium feasibility while
retaining soft penalties that continue to show timetable-quality optimization.

## Quick Start

```bash
# Start the solver server
solverforge server

# Or run directly
cargo run --release --bin solverforge-lessons
```

## Development

```bash
# Add a new constraint
solverforge generate constraint my_rule --unary --hard

# Add a domain entity
solverforge generate entity worker --planning-variable shift_idx

# Add a problem fact
solverforge generate fact location

# Remove a resource
solverforge destroy constraint my_rule
```

## Project Structure

| Directory | Purpose |
|-----------|--------|
| `src/domain/` | Planning entities, facts, and solution struct |
| `src/constraints/` | Constraint definitions (scored by the solver) |
| `src/solver/` | Solver service and configuration |
| `src/api/` | HTTP routes and DTOs |
| `src/data/` | Data loading and generation |
| `solverforge.app.toml` | Scaffolded app/domain contract |
| `solver.toml` | Solver configuration (termination, phases) |
