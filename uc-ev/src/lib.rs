//! SolverForge EV: electric-van delivery routing with charging stops.
//!
//! - `domain/`: the planning model (pharmacies, chargers, vans, the plan),
//!   road-data preparation, and the charging-stop planner.
//! - `constraints/`: the score rules.
//! - `solver/`: retained solver jobs.
//! - `api/`: the HTTP API (Axum) and SSE stream.
//! - `data/`: deterministic Bergamo demo plans.

pub mod api;
pub mod constraints;
pub mod data;
pub mod domain;
pub mod solver;
