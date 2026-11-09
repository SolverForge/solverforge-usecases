//! Shared HTTP error mapping for runtime and routing failures.
//!
//! Keeping this out of `routes.rs` lets the handler file read as a tutorial
//! walkthrough of the public API instead of as a collection of mapping helpers.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

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
        solverforge::SolverManagerError::SnapshotAnalysisDisabled { .. } => StatusCode::CONFLICT,
    }
}

/// An HTTP status with an optional human-readable cause.
///
/// Plain status codes convert into `ApiError` through `From`, so handlers keep
/// using `?` on the status-only helpers above; only routing failures carry a
/// JSON body `{"error": "..."}`. Road-network preparation downloads OSM data
/// from the public Overpass API (overpass-api.de) on first use, and a browser
/// or `curl` user needs the cause of a failed download, not just `502`.
#[derive(Debug)]
pub(super) struct ApiError {
    status: StatusCode,
    message: Option<String>,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
        Self {
            status,
            message: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self.message {
            Some(message) => (self.status, Json(ErrorBody { error: &message })).into_response(),
            None => self.status.into_response(),
        }
    }
}

/// Maps map/routing preparation errors onto client-facing route statuses and
/// names the cause in the response body.
pub(super) fn routing_error(error: solverforge_maps::RoutingError) -> ApiError {
    // Keep the server log line too: operators read stderr, users read the body.
    eprintln!("road-network routing failed: {error}");
    let status = match error {
        solverforge_maps::RoutingError::InvalidCoordinate { .. } => StatusCode::BAD_REQUEST,
        solverforge_maps::RoutingError::Cancelled => StatusCode::REQUEST_TIMEOUT,
        solverforge_maps::RoutingError::Network(_)
        | solverforge_maps::RoutingError::Parse(_)
        | solverforge_maps::RoutingError::Io(_)
        | solverforge_maps::RoutingError::SnapFailed { .. }
        | solverforge_maps::RoutingError::NoPath { .. } => StatusCode::BAD_GATEWAY,
    };
    ApiError {
        status,
        message: Some(format!("routing data unavailable: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn overpass_failure_returns_502_with_the_cause_in_the_body() {
        let error = solverforge_maps::RoutingError::Network(
            "Overpass fetch failed after trying 1 endpoint(s): connection refused".to_string(),
        );

        let response = routing_error(error).into_response();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should collect");
        let body: serde_json::Value =
            serde_json::from_slice(&bytes).expect("error body should be JSON");
        let message = body["error"].as_str().expect("error should be a string");
        assert!(
            message.starts_with("routing data unavailable: ")
                && message.contains("Overpass fetch failed"),
            "unexpected error body: {body}"
        );
    }

    #[tokio::test]
    async fn plain_status_errors_keep_an_empty_body() {
        let response = ApiError::from(StatusCode::NOT_FOUND).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should collect");
        assert!(bytes.is_empty());
    }
}
