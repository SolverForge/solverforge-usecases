//! Planning-model manifest and domain-layer exports.
//!
//! `planning_model!` is the single SolverForge model boundary. It lists the
//! file-backed domain modules, exports the public names used by the rest of the
//! app, and keeps route preparation close to the domain it describes.

solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
    mod coord_value;
    mod delivery;
    mod plan;
    mod vehicle;

    pub use coord_value::CoordValue;
    pub use delivery::Delivery;
    pub use delivery::DeliveryKind;
    pub use plan::Plan;
    pub use plan::PlanConstraintStreams;
    pub use vehicle::Vehicle;
    // @solverforge:end domain-exports

    mod preview;
    mod route_metrics;

    pub use preview::{
        DeliveryPreview, PlanPreview, PlanViewState, RoutingMode, TimelineView, VehiclePreview,
        VehiclePreviewStop,
    };
    pub use route_metrics::{
        build_routes_snapshot, evaluate_plan, prepare_plan, preview_for_plan,
        rank_delivery_insertions, DeliveryInsertionCandidate, PlanScoreComponents,
        PreparedVehicleRouting, RouteLegGeometry, RouteLegSummary, RoutesSnapshot,
        VehicleRouteMetrics, UNASSIGNED_DELIVERY_HARD_PENALTY,
    };
}

#[cfg(test)]
mod clarke_wright_tests;
#[cfg(test)]
mod plan_tests;
