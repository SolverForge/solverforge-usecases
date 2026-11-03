use crate::constraints::support;
use crate::domain::Plan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: work, inspection, and training assignments must respect their windows.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardSoftScore> {
    support::FleetConstraint::scored(true, support::score_work_window, support::count_work_window)
        .named("work_window")
}
