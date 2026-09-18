use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: penalize movement away from baseline assignments during repair solves.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "minimize_churn",
        false,
        HardSoftScore::one_soft(),
        support::score_minimize_churn,
        support::count_churn,
    )
}
