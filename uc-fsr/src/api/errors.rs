//! HTTP error responses for the field-service routing API.
//!
//! Most failures are fully described by their status code: `404` for an unknown
//! job, `409` for a lifecycle transition the job cannot make. Road-network
//! failures are different. `POST /jobs` and `GET /jobs/{id}/routes` download
//! the Bergamo road network from the public Overpass API (overpass-api.de) the
//! first time they run and cache it in `.osm_cache/`; when that download fails,
//! a bare `503` tells a browser or `curl` user nothing. Those responses carry a
//! JSON body `{"error": "..."}` that names the cause.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::data::DemoDataError;

/// An HTTP status with an optional human-readable cause.
///
/// Plain status codes convert into `ApiError` through `From`, so handlers can
/// keep using `?` on the existing status-only helpers; only errors built with a
/// message gain a JSON body.
#[derive(Debug)]
pub(super) struct ApiError {
    status: StatusCode,
    message: Option<String>,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl ApiError {
    fn with_message(status: StatusCode, message: String) -> Self {
        Self {
            status,
            message: Some(message),
        }
    }
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

/// Maps a failed road-network preparation (`POST /jobs`) to `503` plus cause.
pub(super) fn routing_data_unavailable(error: DemoDataError) -> ApiError {
    // Keep the server log line too: operators read stderr, users read the body.
    eprintln!("{error}");
    let DemoDataError::Routing(cause) = error;
    ApiError::with_message(
        StatusCode::SERVICE_UNAVAILABLE,
        format!("routing data unavailable: {cause}"),
    )
}

/// Maps a failed route-geometry build (`GET /jobs/{id}/routes`) to a status
/// plus cause.
pub(super) fn route_geometry_failed(error: solverforge_maps::RoutingError) -> ApiError {
    eprintln!("Bergamo route geometry failed: {error}");
    let status = match error {
        solverforge_maps::RoutingError::InvalidCoordinate { .. } => StatusCode::BAD_REQUEST,
        solverforge_maps::RoutingError::Cancelled => StatusCode::REQUEST_TIMEOUT,
        solverforge_maps::RoutingError::Network(_)
        | solverforge_maps::RoutingError::Parse(_)
        | solverforge_maps::RoutingError::Io(_)
        | solverforge_maps::RoutingError::SnapFailed { .. }
        | solverforge_maps::RoutingError::NoPath { .. } => StatusCode::BAD_GATEWAY,
    };
    ApiError::with_message(status, format!("route geometry unavailable: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error body should collect");
        serde_json::from_slice(&bytes).expect("error body should be JSON")
    }

    #[tokio::test]
    async fn overpass_failure_returns_503_with_the_cause_in_the_body() {
        let error = DemoDataError::Routing(solverforge_maps::RoutingError::Network(
            "Overpass fetch failed after trying 1 endpoint(s): connection refused".to_string(),
        ));

        let response = routing_data_unavailable(error).into_response();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = body_json(response).await;
        let message = body["error"].as_str().expect("error should be a string");
        assert!(
            message.starts_with("routing data unavailable: ")
                && message.contains("Overpass fetch failed"),
            "unexpected error body: {body}"
        );
    }

    #[tokio::test]
    async fn route_geometry_failure_keeps_its_status_and_names_the_cause() {
        let error = solverforge_maps::RoutingError::Network("Overpass fetch failed".to_string());

        let response = route_geometry_failed(error).into_response();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let body = body_json(response).await;
        assert!(body["error"]
            .as_str()
            .is_some_and(|message| message.contains("Overpass fetch failed")));
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
