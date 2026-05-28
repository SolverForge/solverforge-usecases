use crate::domain::{FieldServicePlan, FieldServicePlanConstraintStreams, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: discourage concentrating all service and travel minutes on one route.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.workload_penalty() > 0)
        .penalize(|route: &TechnicianRoute| HardSoftScore::of(0, route.workload_penalty()))
        .named("Balance Workload")
}
