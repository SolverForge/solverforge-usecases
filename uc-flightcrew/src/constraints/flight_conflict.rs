use crate::domain::{CrewAssignment, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// One employee cannot cover two overlapping flight intervals.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::crew_assignments())
        .filter(|assignment: &CrewAssignment| assignment.employee_idx.is_some())
        .join(joiner::equal(|assignment: &CrewAssignment| {
            assignment.employee_idx
        }))
        .filter(|left: &CrewAssignment, right: &CrewAssignment| {
            left.index < right.index
                && left.departure_minute < right.arrival_minute
                && right.departure_minute < left.arrival_minute
        })
        .penalize(hard_weight(|_: &CrewAssignment, _: &CrewAssignment| {
            HardSoftScore::of_hard(10)
        }))
        .named("flight_conflict")
}
