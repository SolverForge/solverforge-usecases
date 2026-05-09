use crate::domain::{Plan, PlanConstraintStreams, ShiftUnassignedFilter};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

const SCORE_SCALE: i64 = 100_000;

/// Hard-penalizes each shift whose scalar planning variable is still unassigned.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .unassigned()
        .penalize(HardSoftDecimalScore::of_hard_scaled(SCORE_SCALE))
        .named("Assigned shift")
}
