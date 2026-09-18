solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
mod vessel;
mod dock;
mod day;
mod technician_pool;
mod technician_capacity_override;
mod part;
mod delivery;
mod dock_outage;
mod readiness_policy;
mod inspection_requirement;
mod training_requirement;
mod work_package;
mod inspection_assignment;
mod training_assignment;
mod plan;

pub use vessel::Vessel;
pub use dock::Dock;
pub use day::Day;
pub use technician_pool::TechnicianPool;
pub use technician_capacity_override::TechnicianCapacityOverride;
pub use part::Part;
pub use delivery::Delivery;
pub use dock_outage::DockOutage;
pub use readiness_policy::ReadinessPolicy;
pub use inspection_requirement::InspectionRequirement;
pub use training_requirement::TrainingRequirement;
pub use work_package::WorkPackage;
pub use inspection_assignment::InspectionAssignment;
pub use training_assignment::TrainingAssignment;
pub use plan::Plan;
    // @solverforge:end domain-exports
}
