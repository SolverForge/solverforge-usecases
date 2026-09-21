---
title: SolverForge Flight Crew
emoji: ✈️
colorFrom: blue
colorTo: purple
sdk: docker
app_port: 7860
pinned: false
license: apache-2.0
short_description: Assign qualified flight crews while enforcing duty and recovery rules.
---

# SolverForge Flight Crew

SolverForge Flight Crew assigns every required pilot and flight-attendant seat
across a global flight network. It preserves the executable scheduling concepts
from the source Java/Timefold application while using SolverForge's retained-job
runtime, exact snapshot analysis, and shared UI components.

![SolverForge Flight Crew browser workspace](docs/screenshot.png)

## Quick Start

```sh
make run-release
```

Open <http://localhost:7860>, inspect the unassigned schedule, and select
**Solve**. The header exposes pause, resume, stop, and snapshot analysis while
the employee and flight timelines update from retained solver snapshots.

Package: `solverforge-flightcrew` (version is defined in `Cargo.toml`). This app
is repository-only until a matching Hugging Face Space is created.

## Planning Model

- Facts: airports, employees, and flight legs.
- Entity: one `CrewAssignment` for every required seat on every flight.
- Variable: `employee_idx`, an optional index into `Plan.employees` while the
  solver constructs the schedule.
- Score: `HardSoftScore`; hard feasibility dominates home-base quality.
- Candidate range: pilot seats consider qualified pilots and cabin seats
  consider qualified flight attendants. Constraints still state and analyze
  the qualification rule independently.

The deterministic `STANDARD` demo contains 6 airports, 24 employees, 10 flight
legs, and 42 required crew seats. Airport and flight times are synthetic and
must not be used for operational dispatch or regulatory compliance.

## Constraints

Hard rules require every seat to be assigned, exact skill qualification, no
overlapping duties, connected consecutive flights, employee availability,
home-base and away-base minimum rest, 36-hour extended recovery, and 48-hour
recovery after standalone or connected long-haul operations.

Soft rules prefer each employee's first flight to depart from their home base
and their last flight to return there. The long-haul threshold follows the
source application's executable rule: at least 10 hours.

## REST API

- `GET /health`
- `GET /demo-data`
- `GET /demo-data/STANDARD`
- `POST /jobs`
- `GET /jobs/{id}` and `GET /jobs/{id}/status`
- `GET /jobs/{id}/snapshot`
- `GET /jobs/{id}/analysis?snapshot_revision={n}`
- `GET /jobs/{id}/events`
- `POST /jobs/{id}/pause`, `/resume`, and `/cancel`
- `DELETE /jobs/{id}`

## Solver Policy

`solver.toml` runs first-fit construction, then late-acceptance local search
with an accepted-count forager. Search terminates after 30 seconds. Qualified
candidate hooks reduce impossible skill choices without encoding score values
or repairing the returned solution.

## Validation

```sh
make test
make test-slow
make ci-local
```

`make test` covers Rust rules, JavaScript transforms, and the real Playwright
browser flow. `make test-slow` executes the full 30-second solve and requires a
zero-hard terminal score. `make ci-local` also checks formatting, Clippy, the
release build, and the Docker image.

## Documentation

- `AGENTS.md` defines maintenance and verification rules.
- `WIREFRAME.md` describes the as-built runtime and browser flow.
- `solverforge.app.toml` records the generated model contract.
- `solver.toml` owns search policy.

Source problem: [blackopsrepl/flight-crew-scheduling-java](https://huggingface.co/spaces/blackopsrepl/flight-crew-scheduling-java).
