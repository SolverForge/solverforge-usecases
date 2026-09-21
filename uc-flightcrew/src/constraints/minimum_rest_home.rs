use crate::domain::{CrewAssignment, Employee, Plan};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

#[derive(Clone)]
struct Duty {
    employee: usize,
    start: i64,
    end: i64,
    flight_minutes: i64,
    from: usize,
    home: usize,
}

/// Home-base rest is at least 12 hours or the preceding duty period, whichever is longer.
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
            start: assignment.departure_minute - 45,
            end: assignment.arrival_minute + 20,
            flight_minutes: assignment.arrival_minute - assignment.departure_minute,
            from: assignment.departure_airport_idx,
            home: employee.home_airport_idx,
        })
        .group_by(
            |duty: &Duty| duty.employee,
            collect_vec(|duty: &Duty| duty.clone()),
        )
        .penalize(hard_weight(|_: &usize, duties: &CollectedVec<Duty>| {
            HardSoftScore::of_hard(rest_shortfall(duties, true) * 100)
        }))
        .named("minimum_rest_home")
}

fn rest_shortfall(duties: &CollectedVec<Duty>, at_home: bool) -> i64 {
    let mut duties = duties.to_vec();
    duties.sort_by_key(|duty| duty.start);
    duties
        .windows(2)
        .map(|pair| {
            let previous = &pair[0];
            let next = &pair[1];
            if previous.end > next.start || (next.from == next.home) != at_home {
                return 0;
            }
            let required = (previous.flight_minutes + 65).max(720);
            (required - (next.start - previous.end)).max(0)
        })
        .sum()
}
