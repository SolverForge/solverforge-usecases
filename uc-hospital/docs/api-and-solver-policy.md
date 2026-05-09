# API And Solver Policy

This page holds the longer reference material that must stay aligned with
`src/api/routes.rs`, `src/api/dto.rs`, `src/solver/service.rs`, `solver.toml`,
and the visible API guide in `static/app/shell/api-guide.mjs`.

## REST API

- `GET /health`
- `GET /info`
- `GET /demo-data`
- `GET /demo-data/{id}`
- `POST /jobs`
- `GET /jobs/{id}`
- `GET /jobs/{id}/status`
- `GET /jobs/{id}/snapshot`
- `GET /jobs/{id}/analysis`
- `POST /jobs/{id}/pause`
- `POST /jobs/{id}/resume`
- `POST /jobs/{id}/cancel`
- `DELETE /jobs/{id}`
- `GET /jobs/{id}/events`

## Lifecycle Semantics

- `pause` requests an exact runtime-managed pause and checkpoint
- `resume` continues from the retained checkpoint
- the user-facing Stop control calls `cancel` to stop a live or paused job
- `delete` removes a terminal retained job before the next fresh solve
- `GET /jobs/{id}` and `GET /jobs/{id}/status` return the same summary payload
- `snapshot_revision={n}` is optional on both snapshot and analysis requests
- reconnects bootstrap from current runtime status plus retained snapshot
  revision, not from cached SSE text

## Payload Shape

The transport payload mirrors the domain model directly. The important field is
`employeeIdx`, which is the scalar planning assignment chosen for each shift.

```json
{
  "employees": [
    {
      "id": "employee-0",
      "name": "Alex Smith",
      "homeHub": "critical_care",
      "skills": ["Critical care doctor"],
      "unavailableDates": [],
      "undesiredDates": [],
      "desiredDates": []
    }
  ],
  "shifts": [
    {
      "id": "shift-1",
      "start": "2024-01-01T08:00:00",
      "end": "2024-01-01T16:00:00",
      "location": "Critical care",
      "careHub": "critical_care",
      "requiredSkill": "Critical care doctor",
      "employeeIdx": 0
    }
  ],
  "score": "0hard/0soft"
}
```

Telemetry is intentionally a UI-facing projection, not the raw runtime type.
Fields like `elapsedMs`, `movesPerSecond`, and `acceptanceRate` are derived for
display.

## Solver Policy

The runtime source of truth is [../solver.toml](../solver.toml).

The currently shipped policy is deliberately narrow:

- `construction_heuristic = cheapest_insertion`
- one local-search phase
- nearby scalar change/swap selectors
- `late_acceptance`
- `accepted_count`

That is not an accident. A candidate sweep against broader scalar phase-2/3
variants showed that the current nearby baseline remained the best balanced
policy for this dataset and 30-second budget.

The canonical description of current behavior is `solver.toml`, the code,
`README.md`, this file, and `WIREFRAME.md`.
