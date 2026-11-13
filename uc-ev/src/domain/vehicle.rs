//! Vehicle planning entities and their derived route shadows.
//!
//! SolverForge mutates exactly one field here: `visit_order`, the ordered
//! customer indexes the van serves. Everything else on the route is derived
//! from that order by [`walk_route`] whenever the list changes: load, driving
//! and charging time, lateness, battery shortfall, and the charging stops.

use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use solverforge_maps::{Coord, RoutingError};

use super::route_walk::{walk_route, RouteEvent, VehicleTerms};
use super::routing_network::NetworkHandle;
use super::CoordValue;

/// Prepared road data attached to one vehicle before solving.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VehicleRouting {
    pub network: NetworkHandle,
    /// This vehicle's depot node in the network matrices.
    pub depot: usize,
}

/// An electric van that leaves its depot once, serves customers, and returns.
#[planning_entity]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    #[planning_id]
    pub id: String,
    pub name: String,
    /// Crates the van can carry.
    pub capacity: i32,
    pub home_lat: CoordValue,
    pub home_lng: CoordValue,
    /// Seconds after midnight the van leaves the depot.
    pub departure_time: i64,
    /// Metres the van can drive on a full battery.
    pub battery_range_m: i64,
    /// Metres of range in the battery at departure.
    pub initial_charge_m: i64,
    /// Seconds one stop takes to refill the battery completely.
    pub recharge_seconds: i64,
    /// Metres of range the van must never dip below (a safety reserve).
    pub reserve_charge_m: i64,
    // @solverforge:begin entity-variables
    #[planning_list_variable(
        element_collection = "customers",
        distance_meter = "solverforge::cvrp::MatrixDistanceMeter",
        intra_distance_meter = "solverforge::cvrp::MatrixIntraDistanceMeter",
        route_hooks = "crate::domain::ev_route_hooks",
        savings_hooks = "crate::domain::ev_savings_hooks",
        savings_metric_class_fn = "crate::domain::ev_savings_hooks::metric_class",
        solution_trait = "crate::domain::EvRouteSolution"
    )]
    pub visit_order: Vec<usize>,
    // @solverforge:end entity-variables
    /// Road data from `prepare_plan`; absent until the plan is prepared.
    #[serde(skip)]
    pub routing: Option<VehicleRouting>,
    #[cascading_update_shadow_variable]
    #[serde(skip_deserializing)]
    pub route_demand: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip_deserializing)]
    pub route_distance_m: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip_deserializing)]
    pub route_driving_seconds: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip_deserializing)]
    pub route_charging_seconds: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip_deserializing)]
    pub route_late_seconds: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip_deserializing)]
    pub route_unreachable_legs: i64,
    #[cascading_update_shadow_variable]
    #[serde(skip_deserializing)]
    pub route_shortfall_m: i64,
    /// Timed depart/charge/visit/return events for the browser.
    #[serde(skip_deserializing)]
    pub schedule: Vec<RouteEvent>,
}

/// Vehicle input values, grouped so the constructor stays readable.
#[derive(Clone, Debug)]
pub struct VehicleInit {
    pub id: String,
    pub name: String,
    pub capacity: i32,
    pub home: (f64, f64),
    pub departure_time: i64,
    pub battery_range_m: i64,
    pub initial_charge_m: i64,
    pub recharge_seconds: i64,
    pub reserve_charge_m: i64,
}

impl Vehicle {
    /// Creates a van with an empty route.
    pub fn new(init: VehicleInit) -> Self {
        Self {
            id: init.id,
            name: init.name,
            capacity: init.capacity,
            home_lat: init.home.0.into(),
            home_lng: init.home.1.into(),
            departure_time: init.departure_time,
            battery_range_m: init.battery_range_m,
            initial_charge_m: init.initial_charge_m,
            recharge_seconds: init.recharge_seconds,
            reserve_charge_m: init.reserve_charge_m,
            // @solverforge:begin entity-variable-init
            visit_order: Vec::new(),
            // @solverforge:end entity-variable-init
            routing: None,
            route_demand: 0,
            route_distance_m: 0,
            route_driving_seconds: 0,
            route_charging_seconds: 0,
            route_late_seconds: 0,
            route_unreachable_legs: 0,
            route_shortfall_m: 0,
            schedule: Vec::new(),
        }
    }

    /// The checked depot coordinate used for road routing.
    pub fn depot_coord(&self) -> Result<Coord, RoutingError> {
        Ok(Coord::try_new(self.home_lat.get(), self.home_lng.get())?)
    }

    /// Parameters the route walk reads.
    pub fn terms(&self) -> VehicleTerms {
        VehicleTerms {
            capacity: i64::from(self.capacity),
            departure_time: self.departure_time,
            battery_range_m: self.battery_range_m,
            initial_charge_m: self.initial_charge_m,
            recharge_seconds: self.recharge_seconds,
            reserve_charge_m: self.reserve_charge_m,
        }
    }

    /// Recomputes every derived route field from `visit_order`.
    pub fn refresh_route_shadows(&mut self) {
        let Some(routing) = self.routing.clone() else {
            self.clear_route_shadows();
            return;
        };
        let mut schedule = Vec::with_capacity(self.visit_order.len() + 4);
        let summary = walk_route(
            &routing.network,
            routing.depot,
            self.terms(),
            &self.visit_order,
            Some(&mut schedule),
        );
        self.route_demand = summary.demand;
        self.route_distance_m = summary.distance_m;
        self.route_driving_seconds = summary.driving_s;
        self.route_charging_seconds = summary.charging_s;
        self.route_late_seconds = summary.late_s;
        self.route_unreachable_legs = summary.unreachable_legs;
        self.route_shortfall_m = summary.shortfall_m;
        self.schedule = schedule;
    }

    /// Crates loaded beyond capacity; read by `vehicle_capacity`.
    pub fn capacity_overage(&self) -> i64 {
        (self.route_demand - i64::from(self.capacity)).max(0)
    }

    /// Driving plus charging seconds; read by `total_travel_time`.
    pub fn route_seconds(&self) -> i64 {
        self.route_driving_seconds + self.route_charging_seconds
    }

    fn clear_route_shadows(&mut self) {
        self.route_demand = 0;
        self.route_distance_m = 0;
        self.route_driving_seconds = 0;
        self.route_charging_seconds = 0;
        self.route_late_seconds = 0;
        self.route_unreachable_legs = 0;
        self.route_shortfall_m = 0;
        self.schedule.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_construction() {
        let entity = Vehicle::new(VehicleInit {
            id: "v1".into(),
            name: "Van 1".into(),
            capacity: 40,
            home: (45.67, 9.66),
            departure_time: 25_200,
            battery_range_m: 150_000,
            initial_charge_m: 60_000,
            recharge_seconds: 2_400,
            reserve_charge_m: 10_000,
        });
        assert_eq!(entity.id, "v1");
        assert!(entity.visit_order.is_empty());
        assert!(entity.depot_coord().is_ok());
        assert_eq!(entity.capacity_overage(), 0);
    }
}
