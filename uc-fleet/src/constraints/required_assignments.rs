use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: non-deferable work, inspections, and training must be assigned.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(
        true,
        support::score_required_assignments,
        support::count_required_assignments,
    )
    .named("required_assignments")
}
