use crate::domain::{CrewAssignment, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

#[derive(Clone)]
struct Duty {
    start: i64,
    end: i64,
    duration: i64,
    from: usize,
    to: usize,
}

/// Standalone flights of at least ten hours require 48 hours before the next duty.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::crew_assignments())
        .filter(|assignment: &CrewAssignment| assignment.employee_idx.is_some())
        .group_by(
            |assignment: &CrewAssignment| assignment.employee_idx.unwrap_or(usize::MAX),
            collect_vec(|assignment: &CrewAssignment| Duty {
                start: assignment.departure_minute - 45,
                end: assignment.arrival_minute + 20,
                duration: assignment.arrival_minute - assignment.departure_minute,
                from: assignment.departure_airport_idx,
                to: assignment.arrival_airport_idx,
            }),
        )
        .penalize(hard_weight(|_: &usize, duties: &CollectedVec<Duty>| {
            let mut duties = duties.to_vec();
            duties.sort_by_key(|duty| duty.start);
            let shortfall: i64 = duties
                .windows(2)
                .map(|pair| {
                    let connection =
                        pair[0].to == pair[1].from && pair[1].start - pair[0].end <= 120;
                    if pair[0].duration < 600 || connection {
                        0
                    } else {
                        (2_880 - (pair[1].start - pair[0].end)).max(0)
                    }
                })
                .sum();
            HardSoftScore::of_hard(shortfall * 100)
        }))
        .named("long_haul_recovery")
}
