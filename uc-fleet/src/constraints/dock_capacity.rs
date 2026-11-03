use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: a dock cannot host more active packages than its capacity on a day.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(
        true,
        support::score_dock_capacity,
        support::count_dock_capacity,
    )
    .named("dock_capacity")
}
