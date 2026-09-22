use crate::domain::{Plan, PlanConstraintStreams, Trolley};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: preserve the source baseline of 1,000 per trolley/order incidence.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .trolleys()
        .filter(|trolley: &Trolley| trolley.distinct_order_count > 0)
        .penalize(|trolley: &Trolley| HardSoftScore::of(0, trolley.order_incidence_penalty()))
        .named("order_incidence")
}
