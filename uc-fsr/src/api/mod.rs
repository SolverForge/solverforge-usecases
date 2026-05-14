//! HTTP transport surface for the field-service routing app.
//!
//! Routes decode browser requests, DTOs define the JSON contract, route
//! geometry adapts road-network output, and `SolverService` owns retained jobs.

mod dto;
mod route_dto;
mod route_geometry;
mod routes;
mod sse;

pub use dto::PlanDto;
pub use routes::{router, AppState};
