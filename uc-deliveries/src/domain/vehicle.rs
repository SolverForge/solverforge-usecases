//! Vehicle planning entities and route shadow values.
//!
//! SolverForge mutates `delivery_order`. The other route fields are derived
//! shadows so constraints can read route totals without recomputing every leg.

use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use solverforge_maps::{Coord, RoutingError};

use super::{CoordValue, PreparedVehicleRouting};

/// A vehicle with its own depot, capacity, and departure time.
#[planning_entity]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    #[planning_id]
    pub id: usize,
    pub name: String,
    pub capacity: i32,
    pub home_lat: CoordValue,
    pub home_lng: CoordValue,
    pub departure_time: i64,
    // @solverforge:begin entity-variables
    /// List planning variable: ordered delivery ids assigned to this vehicle.
    ///
    /// The route hook attributes tell SolverForge how to construct and improve
    /// these lists with owner-specific depot, distance, and feasibility checks.
    #[planning_list_variable(
        element_collection = "deliveries",
        solution_trait = "crate::domain::DeliveryRoutingSolution",
        distance_meter = "solverforge::cvrp::MatrixDistanceMeter",
        intra_distance_meter = "solverforge::cvrp::MatrixIntraDistanceMeter",
        route_get_fn = "solverforge::cvrp::get_route",
        route_set_fn = "solverforge::cvrp::replace_route",
        route_depot_fn = "solverforge::cvrp::depot_for_entity",
        route_distance_fn = "solverforge::cvrp::route_distance",
        route_feasible_fn = "crate::domain::delivery_route_feasible"
    )]
    pub delivery_order: Vec<usize>,
    // @solverforge:end entity-variables
    /// Transient per-vehicle routing data built before solving or previewing.
    #[serde(skip, default)]
    pub prepared_routing: Option<PreparedVehicleRouting>,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_total_demand: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_capacity_overage: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_total_travel_seconds: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_time_window_violation_seconds: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip, default)]
    pub route_unreachable_legs: usize,
}

impl Vehicle {
    /// Creates an empty route anchored at one depot.
    pub fn new(
        id: usize,
        name: impl Into<String>,
        capacity: i32,
        home_lat: f64,
        home_lng: f64,
        departure_time: i64,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            capacity,
            home_lat: home_lat.into(),
            home_lng: home_lng.into(),
            departure_time,
            // @solverforge:begin entity-variable-init
            delivery_order: Vec::new(),
            // @solverforge:end entity-variable-init
            prepared_routing: None,
            route_total_demand: 0,
            route_capacity_overage: 0,
            route_total_travel_seconds: 0,
            route_time_window_violation_seconds: 0,
            route_unreachable_legs: 0,
        }
    }

    /// Converts the serialized depot coordinate into the map library type.
    pub fn depot_coord(&self) -> Result<Coord, RoutingError> {
        Ok(Coord::try_new(self.home_lat.get(), self.home_lng.get())?)
    }

    /// Recomputes the derived route totals read by constraints and previews.
    pub fn refresh_route_shadows(&mut self) {
        let Some(prepared) = self.prepared_routing.as_ref() else {
            self.clear_route_shadows();
            return;
        };

        let mut total_demand = 0_i64;
        let mut total_travel_seconds = 0_i64;
        let mut time_window_violation_seconds = 0_i64;
        let mut unreachable_legs = 0_usize;
        let mut current = prepared.vehicle_departure_time;
        let mut previous: Option<usize> = None;

        for &delivery_id in &self.delivery_order {
            total_demand += i64::from(prepared.demands[delivery_id]);
            let travel = match previous {
                Some(previous_id) => prepared.travel_times[previous_id][delivery_id],
                None => prepared.depot_to_delivery_seconds[delivery_id],
            };
            let normalized_travel = normalize_travel_time(travel);
            total_travel_seconds += normalized_travel;
            current += normalized_travel;

            let (min_start, max_end) = prepared.time_windows[delivery_id];
            if current < min_start {
                current = min_start;
            }
            current += prepared.service_durations[delivery_id];
            if current > max_end {
                time_window_violation_seconds += current - max_end;
            }
            if travel == i64::MAX {
                // Keep unreachable legs visible in both hard scoring and the
                // time-window summary instead of silently treating them as zero.
                unreachable_legs += 1;
                time_window_violation_seconds += 86_400;
            }
            previous = Some(delivery_id);
        }

        if let Some(last_delivery_id) = previous {
            let return_travel = prepared.delivery_to_depot_seconds[last_delivery_id];
            total_travel_seconds += normalize_travel_time(return_travel);
            if return_travel == i64::MAX {
                unreachable_legs += 1;
                time_window_violation_seconds += 86_400;
            }
        }

        self.route_total_demand = total_demand;
        self.route_capacity_overage = (total_demand - prepared.capacity).max(0);
        self.route_total_travel_seconds = total_travel_seconds;
        self.route_time_window_violation_seconds = time_window_violation_seconds;
        self.route_unreachable_legs = unreachable_legs;
    }

    /// Constraint helper for the capacity rule.
    pub fn total_assigned_demand(&self) -> i64 {
        self.route_total_demand
    }

    /// Constraint helper for the capacity rule.
    pub fn capacity_overage(&self) -> i64 {
        self.route_capacity_overage
    }

    /// Constraint helper for the soft travel-time rule.
    pub fn total_travel_seconds(&self) -> i64 {
        self.route_total_travel_seconds
    }

    /// Constraint helper for the time-window rule.
    pub fn time_window_violation_seconds(&self) -> i64 {
        self.route_time_window_violation_seconds
    }

    fn clear_route_shadows(&mut self) {
        self.route_total_demand = 0;
        self.route_capacity_overage = 0;
        self.route_total_travel_seconds = 0;
        self.route_time_window_violation_seconds = 0;
        self.route_unreachable_legs = 0;
    }
}

/// Converts an unreachable sentinel into the same one-day penalty used in UI previews.
fn normalize_travel_time(value: i64) -> i64 {
    if value == i64::MAX {
        86_400
    } else {
        value.max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_construction() {
        let entity = Vehicle::new(7, "Van 7", 28, 39.95, -75.16, 6 * 3600);
        assert_eq!(entity.id, 7);
        assert_eq!(entity.name, "Van 7");
        assert_eq!(entity.capacity, 28);
    }
}
