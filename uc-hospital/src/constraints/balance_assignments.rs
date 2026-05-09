use crate::domain::{Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// Softly penalizes uneven distribution of assigned shifts by `employee_idx`.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .balance(|shift: &Shift| shift.employee_idx)
        .penalize_soft()
        .named("Balance employee assignments")
}
