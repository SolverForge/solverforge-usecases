//! `GET /jobs/{id}/routes`: road geometry for one retained snapshot.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use solverforge_maps::RoutingError;

use super::routes::AppState;
use crate::domain::{build_routes_snapshot, RoutesSnapshot};

#[derive(Debug, Default, Deserialize)]
pub(super) struct RoutesQuery {
    snapshot_revision: Option<u64>,
}

/// Routes bound to the snapshot revision they were drawn from, so the map
/// never shows geometry from a different solution than the cards beside it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RoutesResponse {
    id: String,
    snapshot_revision: u64,
    routes: RoutesSnapshot,
}

pub(super) async fn get_routes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<RoutesQuery>,
) -> Result<Json<RoutesResponse>, StatusCode> {
    let snapshot = state
        .solver
        .get_snapshot(&id, query.snapshot_revision)
        .map_err(|error| match error {
            solverforge::SolverManagerError::JobNotFound { .. }
            | solverforge::SolverManagerError::SnapshotNotFound { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::CONFLICT,
        })?;
    let routes = build_routes_snapshot(&snapshot.solution)
        .await
        .map_err(routing_status)?;
    Ok(Json(RoutesResponse {
        id,
        snapshot_revision: snapshot.snapshot_revision,
        routes,
    }))
}

/// Road-network failures are upstream failures (OpenStreetMap download or an
/// unroutable coordinate), not bad requests from the browser.
pub(super) fn routing_status(error: RoutingError) -> StatusCode {
    eprintln!("routing error: {error}");
    match error {
        RoutingError::InvalidCoordinate { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        _ => StatusCode::BAD_GATEWAY,
    }
}
