use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: overall and class readiness floors must be preserved for every day/week bucket.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(
        true,
        support::score_weekly_readiness_floor,
        support::count_weekly_readiness_floor,
    )
    .named("weekly_readiness_floor")
}
