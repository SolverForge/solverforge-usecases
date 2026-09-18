use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: assigned docks must satisfy vessel compatibility and required class.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "dock_compatibility",
        true,
        HardSoftScore::one_hard(),
        support::score_dock_compatibility,
        support::count_dock_compatibility,
    )
}
