use crate::domain::{Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

const SCORE_SCALE: i64 = 100_000;
const STRUCTURAL_MINUTE_HARD_UNITS: i64 = 20;

/// Hard-penalizes same-employee shift pairs separated by less than 10 hours.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .filter(|shift: &Shift| shift.employee_idx.is_some())
        .join(joiner::equal(|shift: &Shift| shift.employee_idx))
        .filter(|a: &Shift, b: &Shift| {
            if a.index >= b.index {
                return false;
            }

            let (earlier, later) = if a.end <= b.start {
                (a, b)
            } else if b.end <= a.start {
                (b, a)
            } else {
                return false;
            };

            let gap_minutes = (later.start - earlier.end).num_minutes();
            (0..600).contains(&gap_minutes)
        })
        .penalize_hard_with(|a: &Shift, b: &Shift| {
            let (earlier, later) = if a.end <= b.start { (a, b) } else { (b, a) };
            let gap_minutes = (later.start - earlier.end).num_minutes();
            HardSoftDecimalScore::of_hard_scaled(
                (600 - gap_minutes) * STRUCTURAL_MINUTE_HARD_UNITS * SCORE_SCALE,
            )
        })
        .named("At least 10 hours between 2 shifts")
}
