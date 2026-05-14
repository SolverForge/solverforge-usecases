// The planning model macro scans these exports to connect SolverForge metadata,
// generated transport code, and handwritten domain modules. Keep this list in
// the same conceptual order as `solverforge.app.toml`: facts, entity, solution.
solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
mod location;
mod service_visit;
mod travel_leg;
mod technician_route;
mod field_service_plan;

pub use location::Location;
pub use service_visit::ServiceVisit;
pub use service_visit::ServiceVisitInit;
pub use travel_leg::TravelLeg;
pub use travel_leg::TravelLegInit;
pub use technician_route::TechnicianRoute;
pub use technician_route::TechnicianRouteInit;
pub use field_service_plan::FieldServicePlan;
// @solverforge:end domain-exports
}
