# Repository Guidelines

## Project Structure & Module Organization

`src/` follows the current `solverforge-cli` app shape: `api/` for HTTP
routes, SSE, and DTOs; `solver/` for retained-job orchestration; `domain/mod.rs`
for the current `solverforge::planning_model!` manifest; `domain/` for the
exported model modules; `constraints/mod.rs` for constraint assembly; and
`constraints/*.rs` for the individual score rules. `data/mod.rs` is the stable
wrapper; `data/data_seed.rs` is the thin public data surface; and
`data/data_seed/` holds the deterministic sample dataset modules, including the
public entrypoints and the `LARGE` instance builder. `domain/plan.rs` owns
`Plan`, the scalar-variable `Shift`, and the scalar nearby hook functions;
`domain/employee.rs` and `domain/care_hub.rs` hold supporting domain types.
`static/` holds the browser app (`static/app/**/*.mjs`) and generated UI config.
Frontend tests live in `tests/frontend/`. Container packaging is defined by
`Dockerfile`.

This project depends on the published `solverforge` and `solverforge-ui` crates.

## Build, Test, and Development Commands

- `make help` — show the supported local development and validation commands.
- `make run-release` — run the app locally on `:7860`.
- `make test` — run the standard Rust, frontend, and Playwright validation surface.
- `make test-e2e` — run the real browser Playwright smoke.
- `make ci-local` — run the Space-oriented local CI pipeline: fmt, clippy,
  release build, standard tests, and Docker image build.
- `make test-slow` — run the ignored large-demo acceptance solve.
- `make pre-release` — run `make ci-local` plus the slow acceptance solve.
- `make space-build` — build the Docker image used by the Docker-based Space
  deployment path.
- `cargo run --release --bin solverforge-hospital` — run the app locally on
  `:7860`.
- `cargo test` — run Rust unit and integration tests.
- `cargo test large_demo_solves_to_feasible_terminal_state -- --ignored --nocapture`
  — slow end-to-end solver acceptance test.
- `find static/app -name '*.mjs' -print0 | xargs -0 -n1 node --check` —
  syntax-check frontend modules.
- `node --test tests/frontend/*.test.js` — run browserless frontend tests.
- `docker build -f Dockerfile -t solverforge-hospital .` — build the image from
  the repository root context.

## Coding Style & Naming Conventions

Use Rust 2021 style with `cargo fmt`; keep imports and formatting
rustfmt-compatible. Prefer small, explicit functions over clever indirection.
Rust module and file names are `snake_case`; types are `UpperCamelCase`; tests
should describe behavior plainly. Frontend modules are plain ES modules in
`snake-case` filenames. Keep generator logic deterministic: do not introduce
random behavior without a fixed seed and an explicit reason.

## Documentation And Commenting Policy

Assume a beginner reader who is new to SolverForge and new to optimization
modeling.

- Treat `README.md`, `WIREFRAME.md`, this file,
  `docs/api-and-solver-policy.md`, `solver.toml` comments, and the visible API
  help in `static/app/shell/api-guide.mjs` as one canonical documentation
  surface. When one changes, audit the others that describe the same behavior.
- Add module-level docs or comments for every new module that explain its role
  in the app and where it sits in the data flow.
- Add function comments when the function does real coordination work, rebuilds
  invariants, shapes demo data, converts between layers, or otherwise does
  something a beginner would not infer immediately from the signature.
- Write comments that explain intent, domain meaning, invariants, and runtime
  consequences. Do not write comments that merely restate syntax.
- Keep comments truthful. If behavior changes, update or delete the stale
  comment in the same patch.
- When docs mention versions, counts, routes, solver policy, or validation
  expectations, verify those facts against the current code and tests in the
  same turn.
- Prefer present-tense current-state docs after a refactor lands. Do not leave
  future-tense planning language in repo docs unless the file is intentionally a
  still-pending plan.
- When onboarding surfaces change, keep `README.md`, `WIREFRAME.md`, and this
  file aligned.

The standard to aim for is: a new reader should be able to understand why a
piece of code exists before they need to understand every line of how it works.

This repo does not have a canonical hosted GitHub Actions workflow yet. Treat
the Makefile targets, especially `make ci-local` and `make pre-release`, as the
authoritative validation surface for the current Hugging Face Space-oriented
deployment path.

## Testing Guidelines

Add Rust tests next to the behavior they protect, usually in `src/...`
`#[cfg(test)]` modules. Frontend behavior belongs in `tests/frontend/` and
should use the existing fake DOM support in `tests/support/`; real browser
flows belong in `tests/e2e/`. If you change solver behavior, run both
`cargo test` and the ignored large-demo solve. If you change UI modules, run
the Node syntax check, frontend tests, and Playwright tests.

## Commit & Pull Request Guidelines

Follow the workspace commit style seen upstream: conventional prefixes such as
`fix(...)`, `feat(...)`, `refactor(...)`, `test(...)`, and `chore(...)` (for
example, `fix(runtime): route pure scalar construction to descriptor path`).
PRs should state user-visible impact, changed config or API surface, and the
exact validation commands run. Include screenshots only for visible UI changes.

## Configuration & Runtime Notes

`solver.toml` is embedded from `domain/plan.rs` via
`#[planning_solution(..., solver_toml = "../../solver.toml")]`; treat it as
the runtime source of truth. Keep `solverforge.app.toml`,
`static/sf-config.json`, and Docker/runtime port settings aligned with any port
or route changes.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **solverforge-hospital** (1271 symbols, 2596 relationships, 107 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/solverforge-hospital/context` | Codebase overview, check index freshness |
| `gitnexus://repo/solverforge-hospital/clusters` | All functional areas |
| `gitnexus://repo/solverforge-hospital/processes` | All execution flows |
| `gitnexus://repo/solverforge-hospital/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
