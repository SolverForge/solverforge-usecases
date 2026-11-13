use crate::domain::{Plan, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: every leg of a route must exist on the road network.
///
/// Penalised per leg the road graph cannot connect.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::vehicles())
        .filter(unary_condition)
        .penalize(hard_weight(unary_weight))
        .named("unreachable_legs")
}

/// Selects each vehicle this constraint penalizes.
fn unary_condition(vehicle: &Vehicle) -> bool {
    vehicle.route_unreachable_legs > 0
}

/// Penalty for one selected vehicle.
fn unary_weight(vehicle: &Vehicle) -> HardSoftScore {
    HardSoftScore::of(vehicle.route_unreachable_legs, 0)
}
