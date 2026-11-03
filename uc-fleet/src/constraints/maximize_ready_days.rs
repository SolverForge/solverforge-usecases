use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: reward ready vessel-days across the horizon.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(
        false,
        support::score_maximize_ready_days,
        support::count_ready_vessel_days,
    )
    .named("maximize_ready_days")
}
