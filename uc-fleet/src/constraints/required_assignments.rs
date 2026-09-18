use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: non-deferable work, inspections, and training must be assigned.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::new(
        "required_assignments",
        true,
        HardSoftScore::one_hard(),
        support::score_required_assignments,
        support::count_required_assignments,
    )
}
