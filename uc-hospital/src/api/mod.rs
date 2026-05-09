//! HTTP transport surface for the example application.
//!
//! The API layer is split into:
//! - DTOs that translate runtime/domain types into JSON
//! - routes that bind those DTOs to HTTP endpoints
//! - SSE glue for live lifecycle updates

mod dto;
mod routes;
mod sse;

pub use dto::PlanDto;
pub use routes::{router, AppState};
