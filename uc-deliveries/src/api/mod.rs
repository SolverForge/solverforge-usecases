//! HTTP transport surface for the deliveries tutorial.
//!
//! The API layer stays intentionally thin: routes decode requests, DTOs define
//! the browser-visible JSON contract, and `SolverService` owns retained jobs.

mod dto;
mod errors;
mod routes;
mod sse;

pub use dto::PlanDto;
pub use routes::{router, AppState};
