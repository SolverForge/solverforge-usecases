use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: assigned docks must satisfy vessel compatibility and required class.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(
        true,
        support::score_dock_compatibility,
        support::count_dock_compatibility,
    )
    .named("dock_compatibility")
}
