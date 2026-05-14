use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRoutesDto {
    pub id: String,
    pub job_id: String,
    pub snapshot_revision: u64,
    pub routes: Vec<TechnicianRouteGeometryDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicianRouteGeometryDto {
    pub route_id: String,
    pub technician_id: String,
    pub technician_name: String,
    pub color: String,
    pub segments: Vec<RouteSegmentDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RouteGeometryStatus {
    Routed,
    UnreachableLeg,
    SnapFailed,
    NoPath,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSegmentDto {
    pub route_id: String,
    pub from_location_idx: usize,
    pub to_location_idx: usize,
    pub duration_seconds: i64,
    pub distance_meters: i64,
    pub reachable: bool,
    pub geometry_status: RouteGeometryStatus,
    pub encoded_polyline: String,
}

impl JobRoutesDto {
    pub fn new(
        job_id: usize,
        snapshot_revision: u64,
        routes: Vec<TechnicianRouteGeometryDto>,
    ) -> Self {
        Self {
            id: job_id.to_string(),
            job_id: job_id.to_string(),
            snapshot_revision,
            routes,
        }
    }
}
