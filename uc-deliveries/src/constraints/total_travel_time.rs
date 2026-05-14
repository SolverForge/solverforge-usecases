use crate::domain::{Plan, PlanConstraintStreams, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: prefer less total travel time across all routes.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .vehicles()
        .penalize(|vehicle: &Vehicle| HardSoftScore::of(0, vehicle.total_travel_seconds()))
        .named("Total Travel Time")
}
