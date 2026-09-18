/* furnace — retained one-model optimizer built with SolverForge

Structure:
  domain/      — canonical Plan, facts, entities, and live derived helpers
  constraints/ — stock SolverForge constraints over the canonical Plan
  solver/      — retained job service for submitted Plan values
  api/         — HTTP API (axum)
  data/        — canonical demo data and value ranges */

pub mod api;
pub mod constraints;
pub mod data;
pub mod domain;
pub mod solver;
