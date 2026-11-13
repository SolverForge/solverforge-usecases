//! Route-local hooks for the `visit_order` list variable.
//!
//! SolverForge's stock route construction (Clarke-Wright merging and k-opt
//! polishing) asks these hooks to read, write, and check a single route.
//! Reading, writing, the depot, and leg distance are the stock CVRP hooks.
//! `feasible` is this app's own: a route is only feasible when it also keeps
//! the battery above zero once charging stops are placed, so construction
//! never builds a route that would strand a van.

pub use solverforge::cvrp::route_hooks::{depot, distance, get, set};

use solverforge::cvrp::VrpSolution;

use super::route_walk::walk_route;
use super::Plan;

/// The solution contract the `visit_order` metadata requires: the stock CVRP
/// accessors plus this app's battery-aware route check.
pub trait EvRouteSolution: VrpSolution {
    /// Whether `route` is fully feasible for vehicle `entity_idx`.
    fn ev_route_feasible(&self, entity_idx: usize, route: &[usize]) -> bool;

    /// Construction class of a vehicle: vans leaving the same depot share it.
    fn depot_class(&self, entity_idx: usize) -> usize;
}

/// Route-local feasibility hook called by SolverForge's route construction.
pub fn feasible<S: EvRouteSolution>(plan: &S, entity_idx: usize, route: &[usize]) -> bool {
    plan.ev_route_feasible(entity_idx, route)
}

impl EvRouteSolution for Plan {
    fn depot_class(&self, entity_idx: usize) -> usize {
        self.vehicles
            .get(entity_idx)
            .and_then(|vehicle| vehicle.routing.as_ref())
            .map_or(entity_idx, |routing| routing.depot)
    }

    /// Load, time windows, road reachability, and battery range all hold.
    fn ev_route_feasible(&self, entity_idx: usize, route: &[usize]) -> bool {
        if route.is_empty() {
            return true;
        }
        let Some(vehicle) = self.vehicles.get(entity_idx) else {
            return false;
        };
        let Some(routing) = vehicle.routing.as_ref() else {
            return false;
        };
        if route
            .iter()
            .any(|&customer| customer >= routing.network.customers.len())
        {
            return false;
        }
        let summary = walk_route(&routing.network, routing.depot, vehicle.terms(), route, None);
        summary.demand <= i64::from(vehicle.capacity)
            && summary.late_s == 0
            && summary.unreachable_legs == 0
            && summary.shortfall_m == 0
    }
}
