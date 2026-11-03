use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: cumulative parts consumption cannot exceed arrived supply.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(
        true,
        support::score_parts_availability,
        support::count_parts_availability,
    )
    .named("parts_availability")
}
