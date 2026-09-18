use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: penalize inspections that finish after their allowed window.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "minimize_inspection_lateness",
        false,
        HardSoftScore::one_soft(),
        support::score_minimize_inspection_lateness,
        support::count_inspection_lateness,
    )
}
