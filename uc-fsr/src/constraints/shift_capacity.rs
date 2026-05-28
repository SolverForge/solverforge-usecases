use crate::domain::{FieldServicePlan, FieldServicePlanConstraintStreams, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: the complete route must fit inside the technician shift and route cap.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.route_overtime_minutes > 0)
        .penalize(hard_weight(|route: &TechnicianRoute| {
            HardSoftScore::of(route.route_overtime_minutes, 0)
        }))
        .named("Shift Capacity")
}
