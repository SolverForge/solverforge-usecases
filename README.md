# SolverForge Use Cases

This repository is the SolverForge publication bundle for runnable use-case
applications. Each `uc-*` directory is a self-contained SolverForge app that can
run locally and can be published as a Hugging Face Space under the matching
`solverforge-*` name.

## Product Surface

| Directory       | Published Space                                                                              | Use case                                                                                |
| --------------- | -------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `uc-deliveries` | [`solverforge-deliveries`](https://huggingface.co/spaces/SolverForge/solverforge-deliveries) | Capacitated delivery routing with time windows and map-backed travel data.              |
| `uc-fsr`        | [`solverforge-fsr`](https://huggingface.co/spaces/SolverForge/solverforge-fsr)               | Field-service routing for technicians, visits, parts, priorities, and travel.           |
| `uc-hospital`   | [`solverforge-hospital`](https://huggingface.co/spaces/SolverForge/solverforge-hospital)     | Hospital workforce scheduling with skills, availability, preferences, and coverage.     |
| `uc-lessons`    | [`solverforge-lessons`](https://huggingface.co/spaces/SolverForge/solverforge-lessons)       | Lesson scheduling with teachers, cohorts, timeslots, room types, and timetable quality. |

These open-source product examples are imported from source repos and kept as
deployable app directories here. The default source root is the adjacent
`../use-cases` directory; set `USECASE_SOURCE_ROOT=/path/to/use-cases` when
the source repos live somewhere else.

## Documentation Shape

Each use case keeps the same small documentation surface:

- `README.md`
  Human quick start, screenshot, model concepts, validation, API, and solver
  policy.
- `AGENTS.md`
  Codex-facing contribution, validation, and comment/doc alignment rules.
- `WIREFRAME.md`
  As-built architecture, runtime flow, and file-map walkthrough.
- `CHANGELOG.md`
  App-scoped release history generated from conventional commits that touched
  that use case.
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
make install-e2e
cd uc-hospital
make help
make test
make ci-local
```

The root `make install-e2e` target installs the Node and browser dependencies
used by Playwright checks. Each app still serves browser assets from its
declared `solverforge-ui` Cargo crate.
Prefer the app's `Makefile` when present; otherwise use the app README and
standard Cargo commands.

Root checks:

```sh
bash scripts/verify-metadata.sh
bash scripts/verify-imports.sh
```

`verify-imports` compares the bundled `uc-*` directories against
`USECASE_SOURCE_ROOT` when that source root exists. Publication-only checkouts
can still run CI and pre-release without the source repos; the verifier reports
the missing source root and skips only the import drift comparison.

The CI workflow in `.github/workflows/ci.yml` installs the root browser-test
dependencies with `make install-e2e` and then runs `make ci-local` on both
GitHub and Forgejo-style runners.

## App Releases

Each use case has its own package version in `uc-*/Cargo.toml`, its own
`CHANGELOG.md`, and its own app-prefixed tag stream. Do not use bare
`v<version>` tags in this bundle because they are ambiguous.

```text
solverforge-deliveries@<version>
solverforge-fsr@<version>
solverforge-hospital@<version>
solverforge-lessons@<version>
```

Preview or cut an app release from the bundle root:

```sh
make release-usecase-dry-run APP=uc-hospital
make release-usecase APP=uc-hospital RELEASE_AS=patch
```

If the app version, lockfile, and changelog entry were already prepared and
committed, validate and tag that exact version without bumping it again:

```sh
make release-usecase-dry-run APP=uc-hospital PREPARED=1
make release-usecase APP=uc-hospital PREPARED=1
```

Prepared mode verifies the three release surfaces and creates only the
annotated app-prefixed tag. Do not combine `PREPARED=1` with `RELEASE_AS`.

The release wrapper uses `commit-and-tag-version` with an app path filter,
the app changelog, the app `Cargo.toml`, the app `Cargo.lock`, and an
app-prefixed tag. The split app Makefiles stay in place because each `uc-*`
directory becomes the root of a standalone Hugging Face Space after subtree
splitting.

## Hugging Face Sync

`.github/workflows/sync-hf-spaces.yml` publishes release-tagged `uc-*` folders
to Hugging Face Spaces. Manual workflow dispatch remains available for
recovery, but app tags are the canonical release path. The local `uc-` prefix is
transformed into the public `solverforge-` prefix:

```text
uc-deliveries -> <HF_ORGANIZATION>/solverforge-deliveries
uc-fsr -> <HF_ORGANIZATION>/solverforge-fsr
uc-hospital -> <HF_ORGANIZATION>/solverforge-hospital
uc-lessons -> <HF_ORGANIZATION>/solverforge-lessons
```

Required repository configuration:

| Name              | Type     | Purpose                                                     |
| ----------------- | -------- | ----------------------------------------------------------- |
| `HF_TOKEN`        | secret   | Hugging Face token with write access to the target Spaces.  |
| `HF_ORGANIZATION` | variable | Hugging Face username or organization that owns the Spaces. |

Each target Space must already exist before the workflow pushes to it.
