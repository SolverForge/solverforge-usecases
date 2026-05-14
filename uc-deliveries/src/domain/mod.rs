//! Planning-model manifest and domain-layer exports.
//!
//! `planning_model!` is the single SolverForge model boundary. It lists the
//! file-backed domain modules, exports the public names used by the rest of the
//! app, and keeps list-variable hooks close to the domain they describe.

solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
    mod coord_value;
    mod delivery;
    mod plan;
    mod preview;
    mod vehicle;

    pub use coord_value::CoordValue;
    pub use delivery::Delivery;
    pub use delivery::DeliveryKind;
    pub use plan::{Plan, PlanConstraintStreams};
    pub use preview::{
        DeliveryPreview, PlanPreview, PlanViewState, RoutingMode, TimelineView, VehiclePreview,
        VehiclePreviewStop,
    };
    pub use vehicle::Vehicle;
    // @solverforge:end domain-exports

    mod route_metrics;

    pub use route_metrics::{
        build_routes_snapshot, delivery_clarke_wright_depot, delivery_element_load,
        delivery_k_opt_depot, delivery_k_opt_feasible, delivery_route_capacity,
        delivery_route_distance, evaluate_plan, get_delivery_route, prepare_plan,
        preview_for_plan, rank_delivery_insertions, replace_delivery_route,
        DeliveryInsertionCandidate, DeliveryRoutingSolution, PlanScoreComponents,
        PreparedVehicleRouting, RouteLegGeometry, RouteLegSummary, RoutesSnapshot,
        VehicleRouteMetrics, UNASSIGNED_DELIVERY_HARD_PENALTY,
    };
}

#[cfg(test)]
mod plan_tests;
