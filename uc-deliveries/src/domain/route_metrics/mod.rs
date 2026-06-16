mod helpers;
mod insertions;
mod metrics;
mod preparation;
mod routes;
mod scoring;
mod types;

pub use insertions::rank_delivery_insertions;
pub use preparation::prepare_plan;
pub use routes::build_routes_snapshot;
pub use scoring::{evaluate_plan, preview_for_plan};
pub use types::{
    DeliveryInsertionCandidate, PlanScoreComponents, PreparedVehicleRouting, RouteLegGeometry,
    RouteLegSummary, RoutesSnapshot, VehicleRouteMetrics, UNASSIGNED_DELIVERY_HARD_PENALTY,
};

#[cfg(test)]
mod tests;
