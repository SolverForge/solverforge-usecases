use crate::domain::{Employee, Plan, PlanConstraintStreams, Shift};
use solverforge::prelude::*;
use solverforge::stream::{source, ChangeSource};
use solverforge::IncrementalConstraint;

/// Rewards assigning an employee to dates they explicitly prefer.
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
                .desired_days
                .iter()
                .any(|date| shift.touched_dates().contains(date))
        })
        .reward_with(|shift: &Shift, employee: &Employee| {
            HardSoftDecimalScore::of_soft(
                employee
                    .desired_days
                    .iter()
                    .filter(|date| shift.touched_dates().contains(date))
                    .count() as i64,
            )
        })
        .named("Desired day for employee")
}
