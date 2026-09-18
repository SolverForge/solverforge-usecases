use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: heavily penalize readiness-floor shortfall in relaxable profiles.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "minimize_readiness_shortfall",
        false,
        HardSoftScore::one_soft(),
        support::score_minimize_readiness_shortfall,
        support::count_weekly_readiness_floor,
    )
}
