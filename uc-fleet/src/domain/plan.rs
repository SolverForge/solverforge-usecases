use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:neutral-solution
// @solverforge:begin solution-imports
use super::Vessel;
use super::Dock;
use super::Day;
use super::TechnicianPool;
use super::TechnicianCapacityOverride;
use super::Part;
use super::Delivery;
use super::DockOutage;
use super::ReadinessPolicy;
use super::InspectionRequirement;
use super::TrainingRequirement;
use super::WorkPackage;
use super::InspectionAssignment;
use super::TrainingAssignment;
// @solverforge:end solution-imports

/// The root planning solution.
///
/// Fresh projects start as a neutral shell. Add fact collections, planning
/// entity collections, and variable fields through the CLI as your domain
/// takes shape.
#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[derive(Serialize, Deserialize)]
pub struct Plan {
    // @solverforge:begin solution-collections
    #[problem_fact_collection]
    pub vessels: Vec<Vessel>,
    #[problem_fact_collection]
    pub docks: Vec<Dock>,
    #[problem_fact_collection]
    pub days: Vec<Day>,
    #[problem_fact_collection]
    pub technician_pools: Vec<TechnicianPool>,
    #[serde(default)]
    #[problem_fact_collection]
    pub technician_capacity_overrides: Vec<TechnicianCapacityOverride>,
    #[problem_fact_collection]
    pub parts: Vec<Part>,
    #[problem_fact_collection]
    pub deliveries: Vec<Delivery>,
    #[serde(default)]
    #[problem_fact_collection]
    pub dock_outages: Vec<DockOutage>,
    #[problem_fact_collection]
    pub readiness_policies: Vec<ReadinessPolicy>,
    #[problem_fact_collection]
    pub inspection_requirements: Vec<InspectionRequirement>,
    #[problem_fact_collection]
    pub training_requirements: Vec<TrainingRequirement>,
    #[planning_entity_collection]
    pub work_packages: Vec<WorkPackage>,
    #[planning_entity_collection]
    pub inspection_assignments: Vec<InspectionAssignment>,
    #[planning_entity_collection]
    pub training_assignments: Vec<TrainingAssignment>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardSoftScore>,
}

impl Plan {
    #[allow(clippy::too_many_arguments)]
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        vessels: Vec<Vessel>,
        docks: Vec<Dock>,
        days: Vec<Day>,
        technician_pools: Vec<TechnicianPool>,
        parts: Vec<Part>,
        deliveries: Vec<Delivery>,
        readiness_policies: Vec<ReadinessPolicy>,
        inspection_requirements: Vec<InspectionRequirement>,
        training_requirements: Vec<TrainingRequirement>,
        work_packages: Vec<WorkPackage>,
        inspection_assignments: Vec<InspectionAssignment>,
        training_assignments: Vec<TrainingAssignment>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        Self {
            // @solverforge:begin solution-constructor-init
            vessels,
            docks,
            days,
            technician_pools,
            technician_capacity_overrides: Vec::new(),
            parts,
            deliveries,
            dock_outages: Vec::new(),
            readiness_policies,
            inspection_requirements,
            training_requirements,
            work_packages,
            inspection_assignments,
            training_assignments,
            // @solverforge:end solution-constructor-init
            score: None,
        }
    }
}
