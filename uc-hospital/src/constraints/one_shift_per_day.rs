use crate::domain::{Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

const SCORE_SCALE: i64 = 100_000;

/// Hard-penalizes assigning two shifts that touch the same calendar day to one employee.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .filter(|shift: &Shift| shift.employee_idx.is_some())
        .join(joiner::equal(|shift: &Shift| shift.employee_idx))
        .filter(|a: &Shift, b: &Shift| {
            a.index < b.index
                && a.touched_dates()
                    .iter()
                    .any(|date| b.touched_dates().contains(date))
        })
        .penalize(HardSoftDecimalScore::of_hard_scaled(20 * SCORE_SCALE))
        .named("One shift per day")
}
