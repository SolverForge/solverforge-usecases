//! Clarke-Wright savings hooks for the `visit_order` list variable.
//!
//! Clarke-Wright construction starts with one route per customer and merges
//! the pair of routes that saves the most travel, as long as the merged route
//! stays feasible. Two things differ from the stock CVRP hooks:
//!
//! - `feasible` is the battery-aware check, so construction only merges
//!   routes a van can actually drive once charging stops are placed;
//! - `metric_class` groups every van that leaves from the same depot, so the
//!   routes are built once for the whole fleet and then handed to whichever
//!   van can drive each of them (capacity and battery differ per van).

pub use solverforge::cvrp::savings_hooks::{depot, distance};

use super::EvRouteSolution;

/// Construction feasibility: the same strict check as the route hooks.
pub fn feasible<S: EvRouteSolution>(plan: &S, entity_idx: usize, route: &[usize]) -> bool {
    plan.ev_route_feasible(entity_idx, route)
}

/// Vans sharing a depot share one savings computation.
pub fn metric_class<S: EvRouteSolution>(plan: &S, entity_idx: usize) -> usize {
    plan.depot_class(entity_idx)
}
