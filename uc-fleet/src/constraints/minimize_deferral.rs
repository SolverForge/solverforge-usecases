use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: penalize use of the allowed low-priority deferral lever.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "minimize_deferral",
        false,
        HardSoftScore::one_soft(),
        support::score_minimize_deferral,
        support::count_deferrals,
    )
}
