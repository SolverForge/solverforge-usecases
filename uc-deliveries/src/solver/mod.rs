//! Solver-runtime facade exports for the deliveries app.
//!
//! `service.rs` hides the retained `SolverManager<Plan>` details so the HTTP
//! layer only needs a small application-specific API.

mod service;

pub use service::SolverService;
pub use solverforge::SolverStatus;
