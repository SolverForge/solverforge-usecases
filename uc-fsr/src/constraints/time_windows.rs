use crate::constraints::route_metrics::{
    time_windows_match_count, time_windows_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: each visit must start no later than its latest service minute.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Time Windows",
        true,
        HardSoftScore::of(1, 0),
        time_windows_score,
        time_windows_match_count,
    )
}
