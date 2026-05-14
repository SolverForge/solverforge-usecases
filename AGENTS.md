# Repository Guidelines

## Purpose

This repo is the SolverForge use-case publication bundle. Keep each `uc-*`
directory as a deployable SolverForge app that can be split and pushed to the
matching Hugging Face Space named `solverforge-*`.

The `uc-*` prefix is intentional repository plumbing. Product-facing names,
Space names, docs, app metadata, and UI labels should use `SolverForge` and the
`solverforge-*` app name.

## Included Use Cases

- `uc-deliveries` mirrors the open-source `solverforge-deliveries` app.
- `uc-fsr` mirrors the open-source `solverforge-fsr` app.
- `uc-hospital` mirrors the open-source `solverforge-hospital` app.
- `uc-lessons` mirrors the open-source `solverforge-lessons` app.

These four directories are the open-source root allowlist. Do not add another
`uc-*` directory without updating the README, sync workflow, and metadata
verification script in the same change.

When refreshing an imported open-source app, copy source from the corresponding
repo under `../use-cases/`, excluding `.git`, build output, test output,
Playwright reports, and local caches.

## Documentation Standard

Every included use case must have the same lean documentation shape:

- `README.md` for human quick start, screenshot, model concepts, validation,
  REST API, and solver policy.
- `AGENTS.md` for Codex-facing maintenance and validation rules.
- `WIREFRAME.md` for the as-built architecture and runtime/data flow.
- `docs/screenshot.png` for the current browser surface.

Keep these files present-tense and source-backed. When code changes routes,
demo IDs, solver policy, dependency versions, app labels, or visible UI
structure, update the matching README, AGENTS, WIREFRAME, app metadata,
`static/sf-config.json`, and screenshot in the same patch.

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

For root workflow or README-only edits, validate the YAML syntax and inspect the
changed paths with `git diff --stat`.

Run `bash scripts/verify-metadata.sh` after documentation-structure changes.
Run `bash scripts/verify-imports.sh` when imported open-source app directories
change.
