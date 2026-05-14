use serde::Serialize;
use crate::domain::Plan;

pub const UNASSIGNED_DELIVERY_HARD_PENALTY: i64 = 1_000_000;
pub(super) const UNREACHABLE_HARD_PENALTY: i64 = 86_400;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PreparedVehicleRouting {
    pub problem_data_index: usize,
    pub capacity: i64,
    pub demands: Vec<i32>,
    pub distance_matrix: Vec<Vec<i64>>,
    pub time_windows: Vec<(i64, i64)>,
    pub service_durations: Vec<i64>,
    pub travel_times: Vec<Vec<i64>>,
    pub vehicle_departure_time: i64,
    pub depot_to_delivery_seconds: Vec<i64>,
    pub delivery_to_depot_seconds: Vec<i64>,
    pub depot_to_delivery_meters: Vec<i64>,
    pub delivery_to_depot_meters: Vec<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct VehicleRouteMetrics {
    pub vehicle_id: usize,
    pub total_demand: i32,
    pub capacity_overage: i32,
    pub total_travel_seconds: i64,
    pub total_wait_seconds: i64,
    pub total_service_seconds: i64,
    pub total_late_seconds: i64,
    pub unreachable_legs: usize,
    pub total_distance_meters: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub stops: Vec<RouteStopMetrics>,
}

#[derive(Clone, Debug, Default)]
pub struct RouteStopMetrics {
    pub delivery_id: usize,
    pub sequence: usize,
    pub arrival_time: i64,
    pub service_start_time: i64,
    pub departure_time: i64,
    pub travel_seconds_from_previous: i64,
    pub wait_seconds: i64,
    pub late_seconds: i64,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutesSnapshot {
    pub routing_mode: String,
    pub bounds: Option<RouteBounds>,
    pub vehicles: Vec<RouteLegSummary>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteBounds {
    pub south_west: [f64; 2],
    pub north_east: [f64; 2],
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteLegSummary {
    pub vehicle_id: usize,
    pub vehicle_name: String,
    pub total_travel_seconds: i64,
    pub total_distance_meters: i64,
    pub total_demand: i32,
    pub total_late_seconds: i64,
    pub stop_count: usize,
    pub segments: Vec<RouteLegGeometry>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteLegGeometry {
    pub vehicle_id: usize,
    pub from_kind: &'static str,
    pub from_id: Option<usize>,
    pub to_kind: &'static str,
    pub to_id: Option<usize>,
    pub duration_seconds: i64,
    pub distance_meters: i64,
    pub encoded_polyline: String,
}

#[derive(Clone, Debug, Default)]
pub struct PlanScoreComponents {
    pub unassigned_count: usize,
    pub capacity_overage: i64,
    pub late_seconds: i64,
    pub unreachable_legs: usize,
    pub travel_seconds: i64,
}

#[derive(Clone, Debug)]
pub struct DeliveryInsertionCandidate {
    pub vehicle_id: usize,
    pub vehicle_name: String,
    pub insert_index: usize,
    pub hard_score: i64,
    pub soft_score: i64,
    pub delta_hard: i64,
    pub delta_soft: i64,
    pub preview_plan: Plan,
}

pub trait DeliveryRoutingSolution: solverforge::cvrp::VrpSolution {
    fn delivery_plan(&self) -> &Plan;
}

impl DeliveryRoutingSolution for Plan {
    fn delivery_plan(&self) -> &Plan {
        self
    }
}
