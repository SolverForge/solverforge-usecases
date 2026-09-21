use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// Every required cockpit or cabin seat must receive a crew member.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::crew_assignments())
        .unassigned()
        .penalize(HardSoftScore::of_hard(10_000))
        .named("assigned_crew")
}
