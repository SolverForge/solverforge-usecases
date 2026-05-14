mod cvrp_hooks;
mod helpers;
mod insertions;
mod metrics;
mod preparation;
mod routes;
mod scoring;
mod types;

pub use cvrp_hooks::{
    delivery_clarke_wright_depot, delivery_element_load, delivery_k_opt_depot,
    delivery_k_opt_feasible, delivery_route_capacity, delivery_route_distance, get_delivery_route,
    replace_delivery_route,
};
pub use insertions::rank_delivery_insertions;
pub use preparation::prepare_plan;
pub use routes::build_routes_snapshot;
pub use scoring::{evaluate_plan, preview_for_plan};
pub use types::{
    DeliveryInsertionCandidate, DeliveryRoutingSolution, PlanScoreComponents,
    PreparedVehicleRouting, RouteLegGeometry, RouteLegSummary, RoutesSnapshot, VehicleRouteMetrics,
    UNASSIGNED_DELIVERY_HARD_PENALTY,
};

#[cfg(test)]
mod tests;
