//! Solver-runtime facade exports for the lesson-timetabling app.
//!
//! Keeping the retained runtime behind `SolverService` prevents HTTP handlers
//! from depending directly on `SolverManager<Plan>`.

mod event_payload;
mod service;

pub use service::SolverService;
pub use solverforge::SolverStatus;
