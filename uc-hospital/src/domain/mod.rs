//! Planning-model manifest and domain-layer exports.
//!
//! `planning_model!` is the current SolverForge manifest for the domain. It
//! lists the file-backed model modules, exports the public names used by the
//! rest of the app, and lets the runtime attach model-owned scalar hooks.

solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
    mod care_hub;
    mod employee;
    mod plan;

    pub use care_hub::CareHub;
    pub use employee::Employee;
    pub use plan::Plan;
    pub use plan::PlanConstraintStreams;
    pub use plan::Shift;
    // @solverforge:end domain-exports
}
