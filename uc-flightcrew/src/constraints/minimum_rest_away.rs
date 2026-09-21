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
    to: usize,
    home: usize,
}

/// Away-base rest includes the source app's excess-transfer-time adjustment.
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
            to: assignment.arrival_airport_idx,
            home: employee.home_airport_idx,
        })
        .group_by(
            |duty: &Duty| duty.employee,
            collect_vec(|duty: &Duty| duty.clone()),
        )
        .penalize(hard_weight(|_: &usize, duties: &CollectedVec<Duty>| {
            let mut duties = duties.to_vec();
            duties.sort_by_key(|duty| duty.start);
            let shortfall: i64 = duties
                .windows(2)
                .map(|pair| {
                    let previous = &pair[0];
                    let next = &pair[1];
                    if previous.end > next.start || next.from == next.home {
                        return 0;
                    }
                    let transfer = if previous.to == next.from { 30 } else { 180 };
                    let adjustment = 2 * (transfer - 30).max(0);
                    let required = (previous.flight_minutes + 65).max(600) + adjustment;
                    (required - (next.start - previous.end)).max(0)
                })
                .sum();
            HardSoftScore::of_hard(shortfall * 100)
        }))
        .named("minimum_rest_away")
}
