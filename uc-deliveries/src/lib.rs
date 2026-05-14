//! SolverForge deliveries tutorial application.
//!
//! The crate is split the same way a generated SolverForge app is split:
//! domain types define the planning model, constraints define scoring, data
//! builds deterministic examples, `solver` owns retained runtime jobs, and
//! `api` exposes the browser-facing HTTP surface.

pub mod api;
pub mod constraints;
pub mod data;
pub mod domain;
pub mod solver;
