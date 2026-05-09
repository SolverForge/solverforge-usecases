use crate::domain::{Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

const SCORE_SCALE: i64 = 100_000;
const STRUCTURAL_MINUTE_HARD_UNITS: i64 = 20;

/// Penalizes overlapping time windows for the same employee.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .filter(|shift: &Shift| shift.employee_idx.is_some())
        .join(joiner::equal(|shift: &Shift| shift.employee_idx))
        .filter(|a: &Shift, b: &Shift| a.index < b.index && a.start < b.end && b.start < a.end)
        .penalize_hard_with(|a: &Shift, b: &Shift| {
            let overlap_start = a.start.max(b.start);
            let overlap_end = a.end.min(b.end);
            let overlap_minutes = if overlap_start < overlap_end {
                (overlap_end - overlap_start).num_minutes()
            } else {
                0
            };
            HardSoftDecimalScore::of_hard_scaled(
                overlap_minutes * STRUCTURAL_MINUTE_HARD_UNITS * SCORE_SCALE,
            )
        })
        .named("Overlapping shift")
}
