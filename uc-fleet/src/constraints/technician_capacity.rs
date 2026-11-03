use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: daily demand by technician/training pool cannot exceed capacity plus overtime.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(
        true,
        support::score_technician_capacity,
        support::count_technician_capacity,
    )
    .named("technician_capacity")
}
