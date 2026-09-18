use axum::{extract::Path, http::StatusCode, Json};
use serde::Serialize;

use crate::data::{generate, DemoData};

use super::super::dto::{solution_to_dto, PlanDto};

#[derive(Serialize)]
pub(super) struct HealthResponse {
    status: &'static str,
}

pub(super) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "UP" })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InfoResponse {
    name: &'static str,
    version: &'static str,
    solver_engine: &'static str,
}

pub(super) async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        solver_engine: "SolverForge",
    })
}

pub(super) async fn list_demo_data() -> Json<Vec<&'static str>> {
    Json(DemoData::ALL.iter().map(|demo| demo.id()).collect())
}

pub(super) async fn get_demo_data(Path(name): Path<String>) -> Result<Json<PlanDto>, StatusCode> {
    let demo = name
        .parse::<DemoData>()
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(solution_to_dto(&generate(demo))))
}
