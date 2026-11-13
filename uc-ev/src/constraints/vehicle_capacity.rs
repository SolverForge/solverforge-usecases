use crate::domain::{Plan, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: the crates loaded for a route cannot exceed the van's capacity.
///
/// Penalised per crate over capacity.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::vehicles())
        .filter(unary_condition)
        .penalize(hard_weight(unary_weight))
        .named("vehicle_capacity")
}

/// Selects each vehicle this constraint penalizes.
fn unary_condition(vehicle: &Vehicle) -> bool {
    vehicle.capacity_overage() > 0
}

/// Penalty for one selected vehicle.
fn unary_weight(vehicle: &Vehicle) -> HardSoftScore {
    HardSoftScore::of(vehicle.capacity_overage(), 0)
}
