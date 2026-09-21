use crate::domain::{CrewAssignment, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

#[derive(Clone)]
struct Duty {
    start: i64,
    end: i64,
}

/// Require a 36-hour recovery break before 168 hours of continuous duty history elapse.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::crew_assignments())
        .filter(|assignment: &CrewAssignment| assignment.employee_idx.is_some())
        .group_by(
            |assignment: &CrewAssignment| assignment.employee_idx.unwrap_or(usize::MAX),
            collect_vec(|assignment: &CrewAssignment| Duty {
                start: assignment.departure_minute - 45,
                end: assignment.arrival_minute + 20,
            }),
        )
        .penalize(hard_weight(|_: &usize, duties: &CollectedVec<Duty>| {
            let mut duties = duties.to_vec();
            duties.sort_by_key(|duty| duty.start);
            if duties.len() < 2 {
                return HardSoftScore::ZERO;
            }
            let mut reference = duties[0].start;
            let mut violations = 0;
            for pair in duties.windows(2) {
                let rest = pair[1].start - pair[0].end;
                if rest >= 2_160 {
                    reference = pair[0].end;
                } else if pair[1].start - reference > 10_080 {
                    violations += 1;
                }
            }
            HardSoftScore::of_hard(violations * 1_000)
        }))
        .named("extended_recovery_rest")
}
