//! Solver-runtime facade exports for the FSR app.
//!
//! Keeping the retained runtime behind `SolverService` prevents HTTP handlers
//! from depending directly on `SolverManager<FieldServicePlan>`.

mod event_payload;
mod service;

pub use service::SolverService;
pub use solverforge::SolverStatus;
