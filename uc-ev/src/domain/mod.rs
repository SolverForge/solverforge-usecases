//! Planning-model manifest and domain-layer exports.
//!
//! `planning_model!` is the single SolverForge model boundary: it lists the
//! file-backed domain modules and the public names the rest of the app uses.

solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
    mod charger;
    mod coord_value;
    mod customer;
    mod plan;
    mod vehicle;

    pub use charger::Charger;
    pub use coord_value::CoordValue;
    pub use customer::Customer;
    pub use plan::Plan;
    pub use vehicle::Vehicle;
    // @solverforge:end domain-exports

    mod charging;
    pub mod ev_route_hooks;
    pub mod ev_savings_hooks;
    mod prepare;
    mod route_geometry;
    mod route_walk;
    mod routing_network;

    pub use ev_route_hooks::EvRouteSolution;
    pub use charging::{ChargingPlan, LegChargerOption, LegInput, PlannedCharge};
    pub use prepare::{attach_network, depot_sites, plan_coords, prepare_plan};
    pub use route_geometry::{build_routes_snapshot, LegGeometry, RoutesSnapshot, VehicleGeometry};
    pub use route_walk::{walk_route, RouteEvent, RouteEventKind, RouteSummary, VehicleTerms};
    pub use routing_network::{CustomerTerms, NetworkHandle, RoutingNetwork};
    pub use vehicle::{VehicleInit, VehicleRouting};
}
