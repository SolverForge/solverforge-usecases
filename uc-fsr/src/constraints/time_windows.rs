use crate::domain::{FieldServicePlan, FieldServicePlanConstraintStreams, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: each visit must start no later than its latest service minute.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.route_late_minutes > 0)
        .penalize(hard_weight(|route: &TechnicianRoute| {
            HardSoftScore::of(route.route_late_minutes, 0)
        }))
        .named("Time Windows")
}
