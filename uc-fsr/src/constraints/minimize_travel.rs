use crate::domain::{FieldServicePlan, FieldServicePlanConstraintStreams, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: minimize road travel time and distance across technician routes.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.travel_penalty() > 0)
        .penalize(|route: &TechnicianRoute| HardSoftScore::of(0, route.travel_penalty()))
        .named("Minimize Travel")
}
