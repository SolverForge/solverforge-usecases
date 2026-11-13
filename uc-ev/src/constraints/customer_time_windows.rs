use crate::domain::{Plan, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: each delivery must finish before the customer's window closes.
///
/// Arriving early is allowed (the van waits); finishing late is penalised
/// per second, including delays caused by charging stops.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::vehicles())
        .filter(unary_condition)
        .penalize(hard_weight(unary_weight))
        .named("customer_time_windows")
}

/// Selects each vehicle this constraint penalizes.
fn unary_condition(vehicle: &Vehicle) -> bool {
    vehicle.route_late_seconds > 0
}

/// Penalty for one selected vehicle.
fn unary_weight(vehicle: &Vehicle) -> HardSoftScore {
    HardSoftScore::of(vehicle.route_late_seconds, 0)
}
