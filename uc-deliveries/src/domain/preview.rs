//! Browser preview structs embedded in `Plan.view_state`.
//!
//! These values are derived from the domain plan before transport. They let the
//! frontend render route summaries and timelines without duplicating every
//! scoring rule.

use serde::{Deserialize, Serialize};

/// Records the road-network routing contract in transport payloads.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    #[default]
    RoadNetwork,
}

/// Selects which timeline rail the browser shows.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineView {
    #[default]
    ByVehicle,
    ByDelivery,
}

/// UI-only state that travels with the plan between browser and backend.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanViewState {
    #[serde(default)]
    pub timeline_view: TimelineView,
    pub selected_vehicle_id: Option<usize>,
    pub selected_delivery_id: Option<usize>,
    #[serde(default)]
    pub preview: Option<PlanPreview>,
}

/// Aggregate route and score preview for the full plan.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanPreview {
    pub hard_score: i64,
    pub soft_score: i64,
    pub unassigned_delivery_ids: Vec<usize>,
    pub vehicles: Vec<VehiclePreview>,
    pub deliveries: Vec<DeliveryPreview>,
}

/// Per-vehicle route summary used by cards, lists, and timelines.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehiclePreview {
    pub vehicle_id: usize,
    pub vehicle_name: String,
    pub total_demand: i32,
    pub capacity_overage: i32,
    pub stop_count: usize,
    pub total_travel_seconds: i64,
    pub total_wait_seconds: i64,
    pub total_service_seconds: i64,
    pub total_late_seconds: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub stops: Vec<VehiclePreviewStop>,
}

/// One delivery stop on a vehicle timeline.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehiclePreviewStop {
    pub delivery_id: usize,
    pub label: String,
    pub kind: String,
    pub sequence: usize,
    pub demand: i32,
    pub min_start_time: i64,
    pub max_end_time: i64,
    pub arrival_time: i64,
    pub service_start_time: i64,
    pub departure_time: i64,
    pub travel_seconds_from_previous: i64,
    pub wait_seconds: i64,
    pub late_seconds: i64,
}

/// Per-delivery assignment summary used by data tables and timelines.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryPreview {
    pub delivery_id: usize,
    pub label: String,
    pub kind: String,
    pub demand: i32,
    pub min_start_time: i64,
    pub max_end_time: i64,
    pub service_duration: i64,
    pub assigned_vehicle_id: Option<usize>,
    pub assigned_vehicle_name: Option<String>,
    pub sequence: Option<usize>,
    pub arrival_time: Option<i64>,
    pub service_start_time: Option<i64>,
    pub departure_time: Option<i64>,
    pub late_seconds: Option<i64>,
}
