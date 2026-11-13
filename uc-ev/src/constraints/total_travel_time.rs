use crate::domain::{Plan, Vehicle};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: minimise the time vans spend on the road, charging included.
///
/// Driving seconds (with charger detours) plus charging seconds, so the
/// solver prefers visit orders that need fewer or shorter charging stops.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    ConstraintFactory::<Plan, HardSoftScore>::new()
        .for_each(Plan::vehicles())
        .filter(unary_condition)
        .penalize(unary_weight)
        .named("total_travel_time")
}

/// Selects each vehicle this constraint penalizes.
fn unary_condition(vehicle: &Vehicle) -> bool {
    vehicle.route_seconds() > 0
}

/// Penalty for one selected vehicle.
fn unary_weight(vehicle: &Vehicle) -> HardSoftScore {
    HardSoftScore::of(0, vehicle.route_seconds())
}
