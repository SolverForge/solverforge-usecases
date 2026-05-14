# SolverForge Lessons Agent Notes

This directory is the lesson scheduling use case published as
`solverforge-lessons`. Keep the repo-local directory name `uc-lessons`; product
copy, metadata, and UI labels should use `solverforge-lessons` or
SolverForge Lessons.

## App Shape

- `src/domain/` defines the timetable facts, lessons, and `Plan`.
- `src/constraints/` defines the scoring rules for assignments, conflicts,
  capacity, availability, room kind, late lessons, and repeated subjects.
- `src/solver/` owns retained-job solve orchestration.
- `src/api/` owns HTTP routes, DTOs, and SSE events.
- `static/` owns the browser shell and generated view model.

## Validation

Run commands from this directory:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run --bin solverforge-lessons
```

Use `PORT=7861 cargo run --bin solverforge-lessons` if port `7860` is already
occupied.

When routes, solver policy, metadata, visible labels, or UI structure change,
update `README.md`, `WIREFRAME.md`, `solverforge.app.toml`,
`static/sf-config.json`, and `docs/screenshot.png` in the same patch.
