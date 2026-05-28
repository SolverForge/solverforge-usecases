use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

use super::route_metrics::RouteStats;

/// One technician's route, including the visit order SolverForge is allowed to change.
///
/// A `TechnicianRoute` is the planning entity in this app. Its descriptive
/// fields are fixed input data for the technician, while `visits` is the list
/// planning variable that local search reorders and moves between routes.
#[planning_entity]
#[derive(Serialize, Deserialize)]
pub struct TechnicianRoute {
    #[planning_id]
    pub id: String,
    pub technician_id: String,
    pub technician_name: String,
    pub color: String,
    pub start_location_idx: usize,
    pub end_location_idx: usize,
    pub shift_start_minute: i32,
    pub shift_end_minute: i32,
    pub max_route_minutes: i32,
    pub skill_mask: i64,
    pub inventory_mask: i64,
    pub territory: String,
    // SolverForge mutates this vector. Each value is an index into
    // `FieldServicePlan.service_visits`, not a copied `ServiceVisit`.
    // @solverforge:begin entity-variables
    #[planning_list_variable(element_collection = "service_visits")]
    pub visits: Vec<usize>,
    // @solverforge:end entity-variables
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_invalid_visits: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_valid_visits: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_scored_travel_legs: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_unreachable_legs: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_missing_skill_visits: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_missing_part_visits: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_late_visits: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_late_minutes: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_overtime_minutes: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_travel_seconds: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_distance_meters: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_service_minutes: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_waiting_minutes: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_minutes: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_finish_minute: i32,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_territory_matches: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_priority_slack: i64,
}

/// Constructor payload for `TechnicianRoute`.
///
/// Grouping the technician attributes keeps call sites readable and makes the
/// immutable technician data visually separate from the mutable route list.
#[derive(Debug, Clone)]
pub struct TechnicianRouteInit {
    pub id: String,
    pub technician_id: String,
    pub technician_name: String,
    pub color: String,
    pub start_location_idx: usize,
    pub end_location_idx: usize,
    pub shift_start_minute: i32,
    pub shift_end_minute: i32,
    pub max_route_minutes: i32,
    pub skill_mask: i64,
    pub inventory_mask: i64,
    pub territory: String,
}

impl TechnicianRoute {
    /// Builds an empty route for one technician.
    ///
    /// The list variable starts empty so construction heuristics can choose the
    /// first assignment instead of inheriting a hand-written visit order.
    pub fn new(init: TechnicianRouteInit) -> Self {
        Self {
            id: init.id,
            technician_id: init.technician_id,
            technician_name: init.technician_name,
            color: init.color,
            start_location_idx: init.start_location_idx,
            end_location_idx: init.end_location_idx,
            shift_start_minute: init.shift_start_minute,
            shift_end_minute: init.shift_end_minute,
            max_route_minutes: init.max_route_minutes,
            skill_mask: init.skill_mask,
            inventory_mask: init.inventory_mask,
            territory: init.territory,
            // @solverforge:begin entity-variable-init
            visits: Vec::new(),
            // @solverforge:end entity-variable-init
            route_invalid_visits: 0,
            route_valid_visits: 0,
            route_scored_travel_legs: 0,
            route_unreachable_legs: 0,
            route_missing_skill_visits: 0,
            route_missing_part_visits: 0,
            route_late_visits: 0,
            route_late_minutes: 0,
            route_overtime_minutes: 0,
            route_travel_seconds: 0,
            route_distance_meters: 0,
            route_service_minutes: 0,
            route_waiting_minutes: 0,
            route_minutes: 0,
            route_finish_minute: init.shift_start_minute,
            route_territory_matches: 0,
            route_priority_slack: 0,
        }
    }

    /// Copies freshly computed route metrics into SolverForge shadow fields.
    pub fn apply_route_stats(&mut self, stats: RouteStats) {
        self.route_invalid_visits = stats.invalid_visits;
        self.route_valid_visits = stats.valid_visits;
        self.route_scored_travel_legs = stats.scored_travel_legs;
        self.route_unreachable_legs = stats.unreachable_legs;
        self.route_missing_skill_visits = stats.missing_skill_visits;
        self.route_missing_part_visits = stats.missing_part_visits;
        self.route_late_visits = stats.late_visits;
        self.route_late_minutes = stats.late_minutes;
        self.route_overtime_minutes = stats.overtime_minutes;
        self.route_travel_seconds = stats.travel_seconds;
        self.route_distance_meters = stats.distance_meters;
        self.route_service_minutes = stats.service_minutes;
        self.route_waiting_minutes = stats.waiting_minutes;
        self.route_minutes = stats.route_minutes;
        self.route_finish_minute = stats.finish_minute;
        self.route_territory_matches = stats.territory_matches;
        self.route_priority_slack = stats.priority_slack;
    }

    pub fn travel_penalty(&self) -> i64 {
        div_ceil(self.route_travel_seconds, 60) + div_ceil(self.route_distance_meters, 1_000)
    }

    pub fn workload_penalty(&self) -> i64 {
        let normalized = (self.route_minutes / 15).max(0);
        normalized * normalized
    }
}

fn div_ceil(value: i64, divisor: i64) -> i64 {
    if value <= 0 {
        0
    } else {
        (value + divisor - 1) / divisor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technician_route_construction() {
        let entity = TechnicianRoute::new(TechnicianRouteInit {
            id: "test-id".to_string(),
            technician_id: "test".to_string(),
            technician_name: "test".to_string(),
            color: "test".to_string(),
            start_location_idx: Default::default(),
            end_location_idx: Default::default(),
            shift_start_minute: Default::default(),
            shift_end_minute: Default::default(),
            max_route_minutes: Default::default(),
            skill_mask: Default::default(),
            inventory_mask: Default::default(),
            territory: "test".to_string(),
        });
        assert_eq!(entity.id, "test-id");
        let _ = &entity.technician_id;
        let _ = &entity.technician_name;
        let _ = &entity.color;
        let _ = &entity.start_location_idx;
        let _ = &entity.end_location_idx;
        let _ = &entity.shift_start_minute;
        let _ = &entity.shift_end_minute;
        let _ = &entity.max_route_minutes;
        let _ = &entity.skill_mask;
        let _ = &entity.inventory_mask;
        let _ = &entity.territory;
    }
}
