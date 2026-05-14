use crate::domain::{
    Delivery, Plan, PlanConstraintStreams, Vehicle, UNASSIGNED_DELIVERY_HARD_PENALTY,
};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

/// HARD: every delivery must appear in some vehicle route.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .deliveries()
        // The right side flattens every vehicle route into assigned delivery
        // ids. A delivery that does not exist in that flattened stream is
        // unassigned and receives the dominant hard penalty.
        .if_not_exists((
            ConstraintFactory::<Plan, HardSoftScore>::new()
                .vehicles()
                .flattened(|vehicle: &Vehicle| &vehicle.delivery_order),
            equal_bi(
                |delivery: &Delivery| delivery.id,
                |assigned: &usize| *assigned,
            ),
        ))
        .penalize(hard_weight(|_: &Delivery| {
            HardSoftScore::of(UNASSIGNED_DELIVERY_HARD_PENALTY, 0)
        }))
        .named("All Deliveries Assigned")
}
