//! Shared HTTP error mapping for runtime and routing failures.
//!
//! Keeping this out of `routes.rs` lets the handler file read as a tutorial
//! walkthrough of the public API instead of as a collection of mapping helpers.

use axum::http::StatusCode;

/// Parses the path segment used by stock retained-job routes.
pub(super) fn parse_job_id(id: &str) -> Result<usize, StatusCode> {
    id.parse::<usize>().map_err(|_| StatusCode::NOT_FOUND)
}

/// Maps retained-runtime errors onto HTTP statuses the stock UI understands.
pub(super) fn status_from_solver_error(error: solverforge::SolverManagerError) -> StatusCode {
    match error {
        solverforge::SolverManagerError::NoFreeJobSlots => StatusCode::SERVICE_UNAVAILABLE,
        solverforge::SolverManagerError::JobNotFound { .. } => StatusCode::NOT_FOUND,
        solverforge::SolverManagerError::InvalidStateTransition { .. } => StatusCode::CONFLICT,
        solverforge::SolverManagerError::NoSnapshotAvailable { .. } => StatusCode::CONFLICT,
        solverforge::SolverManagerError::SnapshotNotFound { .. } => StatusCode::NOT_FOUND,
    }
}

/// Maps map/routing preparation errors onto client-facing route statuses.
pub(super) fn status_from_routing_error(error: solverforge_maps::RoutingError) -> StatusCode {
    match error {
        solverforge_maps::RoutingError::InvalidCoordinate { .. } => StatusCode::BAD_REQUEST,
        solverforge_maps::RoutingError::Cancelled => StatusCode::REQUEST_TIMEOUT,
        solverforge_maps::RoutingError::Network(_)
        | solverforge_maps::RoutingError::Parse(_)
        | solverforge_maps::RoutingError::Io(_)
        | solverforge_maps::RoutingError::SnapFailed { .. }
        | solverforge_maps::RoutingError::NoPath { .. } => StatusCode::BAD_GATEWAY,
    }
}
