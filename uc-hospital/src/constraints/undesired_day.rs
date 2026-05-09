use crate::domain::{Employee, Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::stream::{source, ChangeSource};
use solverforge::IncrementalConstraint;

/// Softly penalizes assignments that land on an employee's undesired dates.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftDecimalScore> {
    ConstraintFactory::<Plan, HardSoftDecimalScore>::new()
        .shifts()
        .filter(|shift: &Shift| shift.employee_idx.is_some())
        .join((
            source(Plan::employees_slice, ChangeSource::Static),
            joiner::equal_bi(
                |shift: &Shift| shift.employee_idx,
                |employee: &Employee| Some(employee.index),
            ),
        ))
        .filter(|shift: &Shift, employee: &Employee| {
            employee
                .undesired_days
                .iter()
                .any(|date| shift.touched_dates().contains(date))
        })
        .penalize_with(|shift: &Shift, employee: &Employee| {
            HardSoftDecimalScore::of_soft(
                employee
                    .undesired_days
                    .iter()
                    .filter(|date| shift.touched_dates().contains(date))
                    .count() as i64,
            )
        })
        .named("Undesired day for employee")
}
