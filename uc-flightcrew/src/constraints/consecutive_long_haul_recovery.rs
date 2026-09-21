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

/// Two connected legs totaling ten hours require 48 hours after the second leg.
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
                .windows(3)
                .map(|triple| {
                    let connected =
                        triple[0].to == triple[1].from && triple[1].start - triple[0].end <= 120;
                    if !connected || triple[0].duration + triple[1].duration < 600 {
                        0
                    } else {
                        (2_880 - (triple[2].start - triple[1].end)).max(0)
                    }
                })
                .sum();
            HardSoftScore::of_hard(shortfall * 100)
        }))
        .named("consecutive_long_haul_recovery")
}
