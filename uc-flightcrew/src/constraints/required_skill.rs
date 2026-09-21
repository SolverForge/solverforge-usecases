use crate::domain::{CrewAssignment, Employee, Plan};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

/// Assigned crew must hold the exact qualification required by the seat.
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
            !employee.skills.contains(&assignment.required_skill)
        })
        .penalize(hard_weight(|_: &CrewAssignment, _: &Employee| {
            HardSoftScore::of_hard(100)
        }))
        .named("required_skill")
}
