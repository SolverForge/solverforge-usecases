# Repository Guidelines

## Purpose

This repo is the SolverForge use-case publication bundle. Keep each `uc-*`
directory as a deployable SolverForge app that can be split and pushed to the
matching Hugging Face Space named `solverforge-*`.

The `uc-*` prefix is intentional repository plumbing. Product-facing names,
Space names, docs, app metadata, and UI labels should use `SolverForge` and the
`solverforge-*` app name.

## Included Use Cases

- `uc-deliveries` is the source for the open-source `solverforge-deliveries` app.
- `uc-fsr` is the source for the open-source `solverforge-fsr` app.
- `uc-hospital` is the source for the open-source `solverforge-hospital` app.
- `uc-lessons` is the source for the open-source `solverforge-lessons` app.

These four directories are the open-source root allowlist. Do not add another
`uc-*` directory without updating the README, sync workflow, and metadata
verification script in the same change.

## Documentation Standard

Every included use case must have the same lean documentation shape:

- `README.md` for human quick start, screenshot, model concepts, validation,
  REST API, and solver policy.
- `AGENTS.md` for Codex-facing maintenance and validation rules.
- `WIREFRAME.md` for the as-built architecture and runtime/data flow.
- `CHANGELOG.md` for app-scoped release history.
- `docs/screenshot.png` for the current browser surface.

Keep these files present-tense and source-backed. When code changes routes,
demo IDs, solver policy, dependency versions, app labels, or visible UI
structure, update the matching README, AGENTS, WIREFRAME, app metadata,
`static/sf-config.json`, and screenshot in the same patch.

Use app-prefixed release tags in this bundle. Bare `vX.Y.Z` tags are ambiguous;
the release tag for a use case is `solverforge-<app>@<version>`, matching the
version in that app's `Cargo.toml`, that app's `Cargo.lock`, and a heading in
that app's `CHANGELOG.md`.

Comments should assume a reader who is new to Rust and new to planning
optimization. Explain domain meaning, solver roles, invariants, and runtime
consequences. Do not keep scaffold placeholders, stale planning prose, or
comments that merely restate syntax.

## Agent Standard

Codex instructions belong in `AGENTS.md`. Do not add non-Codex assistant
instruction files, external code-intelligence directive blocks, or
assistant-specific fallback instruction trees.

## Validation

Run validation from the app directory being changed. Prefer the app `Makefile`
when present:

- `make test` for ordinary source or frontend changes.
- `make ci-local` before deployment, Docker, dependency, or Space-surface
  changes.
- `make release-usecase-dry-run APP=uc-hospital` before cutting an app release.
- `make release-usecase APP=uc-hospital RELEASE_AS=patch` to generate the app
  changelog/version/lockfile/tag release from the bundle root.
- `make release-usecase APP=uc-hospital PREPARED=1` only when the matching app
  version, lockfile, and changelog heading are already committed; it verifies
  those surfaces and creates the current annotated tag without another bump.
- `make publish-usecase-dry-run TAG=solverforge-hospital@x.y.z` before pushing
  one release, then `make publish-usecase TAG=...` to push `main` and that tag.
- `make publish-usecases-dry-run` before publishing the current releases for
  all four apps; `make publish-usecases` pushes the tags separately so GitHub
  emits one Hugging Face sync event per app.

For root workflow or README-only edits, validate the YAML syntax and inspect the
changed paths with `git diff --stat`. The root CI workflow is
`.github/workflows/ci.yml`; it installs browser-test dependencies and then runs
`make ci-local`. The Hugging Face publication workflow remains
`.github/workflows/sync-hf-spaces.yml`.

Run `bash scripts/verify-metadata.sh` after documentation-structure changes.
