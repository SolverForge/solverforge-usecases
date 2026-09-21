use crate::domain::{CrewAssignment, Employee, Plan};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

/// Crew cannot work a flight touching one of their unavailable UTC days.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::crew_assignments())
        .join((
            Plan::employees(),
            equal_bi(
                |assignment: &CrewAssignment| assignment.employee_idx,
                |employee: &Employee| Some(employee.index),
            ),
        ))
        .filter(|assignment: &CrewAssignment, employee: &Employee| {
            let first_day = assignment.departure_minute.div_euclid(1_440);
            let last_day = assignment
                .arrival_minute
                .saturating_sub(1)
                .div_euclid(1_440);
            (first_day..=last_day).any(|day| employee.unavailable_days.contains(&day))
        })
        .penalize(hard_weight(|_: &CrewAssignment, _: &Employee| {
            HardSoftScore::of_hard(10)
        }))
        .named("employee_unavailable")
}
