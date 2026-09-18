use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: penalize daily demand above regular capacity but within overtime capacity.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "minimize_overtime",
        false,
        HardSoftScore::one_soft(),
        support::score_minimize_overtime,
        support::count_overtime,
    )
}
