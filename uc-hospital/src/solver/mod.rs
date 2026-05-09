//! Solver-runtime exports.
//!
//! The actual orchestration lives in `service.rs`. This module just exposes the
//! few types the HTTP layer needs, keeping the rest of the runtime machinery
//! private to the crate.

mod service;

pub use service::SolverService;
pub use solverforge::SolverStatus;
