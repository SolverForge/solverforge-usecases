use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: a work package cannot occupy a dock during a declared outage window.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "dock_outage",
        true,
        HardSoftScore::one_hard(),
        support::score_dock_outages,
        support::count_dock_outages,
    )
}
