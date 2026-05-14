use crate::domain::{Plan, PlanConstraintStreams, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: a vehicle's assigned demand cannot exceed its capacity.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .vehicles()
        // Capacity overage is also a route shadow value. SolverForge updates it
        // after list moves, and this constraint only scores positive excess.
        .filter(|vehicle: &Vehicle| vehicle.capacity_overage() > 0)
        .penalize(hard_weight(|vehicle: &Vehicle| {
            HardSoftScore::of(vehicle.capacity_overage(), 0)
        }))
        .named("Vehicle Capacity")
}
