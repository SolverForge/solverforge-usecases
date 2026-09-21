use crate::domain::{CrewAssignment, Employee, Plan};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

#[derive(Clone)]
struct Duty {
    employee: usize,
    departure: i64,
    from: usize,
    home: usize,
}

/// Prefer each employee's first assigned flight to depart from their home base.
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
        .project(|assignment: &CrewAssignment, employee: &Employee| Duty {
            employee: employee.index,
            departure: assignment.departure_minute,
            from: assignment.departure_airport_idx,
            home: employee.home_airport_idx,
        })
        .group_by(
            |duty: &Duty| duty.employee,
            collect_vec(|duty: &Duty| duty.clone()),
        )
        .penalize(|_: &usize, duties: &CollectedVec<Duty>| {
            let violation = duties
                .iter()
                .min_by_key(|duty| duty.departure)
                .is_some_and(|duty| duty.from != duty.home);
            HardSoftScore::of_soft(if violation { 1_000 } else { 0 })
        })
        .named("first_assignment_home")
}
