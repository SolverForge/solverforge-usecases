use crate::domain::{Plan, PlanConstraintStreams, Trolley};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: penalize one point per meter across every closed trolley route.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .trolleys()
        .filter(|trolley: &Trolley| trolley.route_distance_meters > 0)
        .penalize(|trolley: &Trolley| HardSoftScore::of(0, trolley.route_distance_meters))
        .named("route_distance")
}
