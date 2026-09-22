use crate::domain::{Plan, PlanConstraintStreams, Trolley};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: penalize each order-dedicated bucket needed beyond trolley capacity.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .trolleys()
        .filter(|trolley: &Trolley| trolley.excess_buckets() > 0)
        .penalize(hard_weight(|trolley: &Trolley| {
            HardSoftScore::of(trolley.excess_buckets(), 0)
        }))
        .named("required_buckets")
}
