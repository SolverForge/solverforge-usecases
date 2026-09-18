use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: inspection follows maintenance and training follows inspection.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "inspection_training_precedence",
        true,
        HardSoftScore::one_hard(),
        support::score_inspection_training_precedence,
        support::count_inspection_training_precedence,
    )
}
