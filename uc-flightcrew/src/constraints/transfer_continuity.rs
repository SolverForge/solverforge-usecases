use crate::domain::{CrewAssignment, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

#[derive(Clone)]
struct Duty {
    departure: i64,
    arrival: i64,
    from: usize,
    to: usize,
}

/// Consecutive duties for an employee must connect at the same airport.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::crew_assignments())
        .filter(|assignment: &CrewAssignment| assignment.employee_idx.is_some())
        .group_by(
            |assignment: &CrewAssignment| assignment.employee_idx.unwrap_or(usize::MAX),
            collect_vec(|assignment: &CrewAssignment| Duty {
                departure: assignment.departure_minute,
                arrival: assignment.arrival_minute,
                from: assignment.departure_airport_idx,
                to: assignment.arrival_airport_idx,
            }),
        )
        .penalize(hard_weight(|_: &usize, duties: &CollectedVec<Duty>| {
            let mut duties = duties.to_vec();
            duties.sort_by_key(|duty| duty.departure);
            let violations = duties
                .windows(2)
                .filter(|pair| pair[0].arrival <= pair[1].departure && pair[0].to != pair[1].from)
                .count();
            HardSoftScore::of_hard(violations as i64)
        }))
        .named("transfer_continuity")
}
