//! SolverForge field-service routing application.
//!
//! The crate follows the same teaching shape as the other use cases: `domain`
//! defines the planning model, `constraints` defines scoring, `data` builds the
//! deterministic Bergamo instance, `solver` owns retained runtime jobs, and
//! `api` exposes the browser-facing HTTP surface.

pub mod api;
pub mod constraints;
pub mod data;
pub mod domain;
pub mod solver;
