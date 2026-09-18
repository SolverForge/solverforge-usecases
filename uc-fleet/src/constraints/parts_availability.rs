use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: cumulative parts consumption cannot exceed arrived supply.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "parts_availability",
        true,
        HardSoftScore::one_hard(),
        support::score_parts_availability,
        support::count_parts_availability,
    )
}
