use crate::domain::{Employee, Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

const SCORE_SCALE: i64 = 100_000;

/// Penalizes assignments where the employee lacks the required skill label.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .filter(|shift: &Shift| shift.employee_idx.is_some())
        .join((
            ConstraintFactory::<Plan, HardSoftDecimalScore>::new().employees(),
            joiner::equal_bi(
                |shift: &Shift| shift.employee_idx,
                |employee: &Employee| Some(employee.index),
            ),
        ))
        .filter(|shift: &Shift, employee: &Employee| {
            !employee.skills.contains(&shift.required_skill)
        })
        .penalize(HardSoftDecimalScore::of_hard_scaled(10 * SCORE_SCALE))
        .named("Required skill")
}
