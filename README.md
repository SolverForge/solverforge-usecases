# SolverForge Use Cases

This repository is the SolverForge publication bundle for runnable use-case
applications. Each `uc-*` directory is a self-contained SolverForge app that can
run locally and can be published as a Hugging Face Space under the matching
`solverforge-*` name.

## Product Surface

| Directory | Published Space | Use case |
| --- | --- | --- |
| `uc-deliveries` | `solverforge-deliveries` | Capacitated delivery routing with time windows and map-backed travel data. |
| `uc-fsr` | `solverforge-fsr` | Field-service routing for technicians, visits, parts, priorities, and travel. |
| `uc-hospital` | `solverforge-hospital` | Hospital workforce scheduling with skills, availability, preferences, and coverage. |
| `uc-lessons` | `solverforge-lessons` | Lesson scheduling with teachers, cohorts, timeslots, room types, and timetable quality. |

These open-source product examples are imported from the adjacent
`../use-cases` source repos and kept as deployable app directories here.

## Documentation Shape

Each use case keeps the same small documentation surface:

- `README.md`
  Human quick start, screenshot, model concepts, validation, API, and solver
  policy.
- `AGENTS.md`
  Codex-facing contribution, validation, and comment/doc alignment rules.
- `WIREFRAME.md`
  As-built architecture, runtime flow, and file-map walkthrough.
- `docs/screenshot.png`
  One current browser screenshot for the app.

The `uc-*` directory name is repository plumbing. Public screenshots,
README text, Space names, app labels, and metadata should present the product
as `SolverForge` and the matching `solverforge-*` use case.

## Repository Standard

Codex-facing instructions belong in `AGENTS.md`. Non-Codex assistant files,
external code-intelligence directive blocks, and assistant-specific
compatibility surfaces do not belong in this repo.

Use the existing SolverForge app structure inside each use case:

- `Cargo.toml`, `Cargo.lock`, `solver.toml`, and `solverforge.app.toml` define
  the runtime and SolverForge contract.
- `src/` contains the Rust app, domain model, constraints, API routes, and
  retained solver service.
- `static/` contains the browser UI shipped with the Space.
- `README.md`, `AGENTS.md`, `WIREFRAME.md`, and `docs/screenshot.png` are the
  standard documentation surface for every included app.

## Local Workflow

Run commands from the use-case directory you are changing.

```sh
npm install
cd uc-hospital
make help
make test
make ci-local
```

The root `npm install` provides the Playwright test runner for the bundle. Each
app still serves browser assets from its declared `solverforge-ui` Cargo crate.
Prefer the app's `Makefile` when present; otherwise use the app README and
standard Cargo commands.

Root checks:

```sh
bash scripts/verify-metadata.sh
bash scripts/verify-imports.sh
```

## Hugging Face Sync

`.github/workflows/sync-hf-spaces.yml` publishes changed `uc-*` folders to
Hugging Face Spaces. The local `uc-` prefix is transformed into the public
`solverforge-` prefix:

```text
uc-deliveries -> <HF_ORGANIZATION>/solverforge-deliveries
uc-fsr -> <HF_ORGANIZATION>/solverforge-fsr
uc-hospital -> <HF_ORGANIZATION>/solverforge-hospital
uc-lessons -> <HF_ORGANIZATION>/solverforge-lessons
```

Required repository configuration:

| Name | Type | Purpose |
| --- | --- | --- |
| `HF_TOKEN` | secret | Hugging Face token with write access to the target Spaces. |
| `HF_ORGANIZATION` | variable | Hugging Face username or organization that owns the Spaces. |

Each target Space must already exist before the workflow pushes to it.
