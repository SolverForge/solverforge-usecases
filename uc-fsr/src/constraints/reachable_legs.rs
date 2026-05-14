use crate::constraints::route_metrics::{reachable_match_count, reachable_score, RouteConstraint};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: every depot-to-visit, visit-to-visit, and visit-to-depot leg must be routable.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Reachable Legs",
        true,
        HardSoftScore::of(1, 0),
        reachable_score,
        reachable_match_count,
    )
}
