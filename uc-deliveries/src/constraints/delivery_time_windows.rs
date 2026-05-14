use crate::domain::{Plan, PlanConstraintStreams, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: each vehicle route must respect delivery time windows.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .vehicles()
        // Time-window work is precomputed as a vehicle shadow value, so this
        // rule can stay incremental and read one scalar per changed route.
        .filter(|vehicle: &Vehicle| vehicle.time_window_violation_seconds() > 0)
        .penalize(hard_weight(|vehicle: &Vehicle| {
            HardSoftScore::of(vehicle.time_window_violation_seconds(), 0)
        }))
        .named("Delivery Time Windows")
}
