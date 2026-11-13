use crate::domain::{Plan, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: a van's battery must cover every stretch between charges.
///
/// The route walk places charging stops optimally for the current visit
/// order; whatever range is still missing after that is penalised per metre.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::vehicles())
        .filter(unary_condition)
        .penalize(hard_weight(unary_weight))
        .named("battery_shortfall")
}

/// Selects each vehicle this constraint penalizes.
fn unary_condition(vehicle: &Vehicle) -> bool {
    vehicle.route_shortfall_m > 0
}

/// Penalty for one selected vehicle.
fn unary_weight(vehicle: &Vehicle) -> HardSoftScore {
    HardSoftScore::of(vehicle.route_shortfall_m, 0)
}
