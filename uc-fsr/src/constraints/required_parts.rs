use crate::domain::{FieldServicePlan, FieldServicePlanConstraintStreams, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: route inventory must cover every assigned visit's required parts.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.route_missing_part_visits > 0)
        .penalize(hard_weight(|route: &TechnicianRoute| {
            HardSoftScore::of(route.route_missing_part_visits, 0)
        }))
        .named("Required Parts")
}
