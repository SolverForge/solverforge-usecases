//! HTTP transport surface for the lesson-timetabling app.
//!
//! Routes decode browser requests, DTOs define the JSON contract, and
//! `SolverService` owns retained jobs.

mod dto;
mod routes;
mod sse;

pub use dto::PlanDto;
pub use routes::{router, AppState};
