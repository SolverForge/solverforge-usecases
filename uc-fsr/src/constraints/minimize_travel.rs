use crate::constraints::route_metrics::{
    minimize_travel_match_count, minimize_travel_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: minimize road travel time and distance across technician routes.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Minimize Travel",
        false,
        HardSoftScore::of(0, 1),
        minimize_travel_score,
        minimize_travel_match_count,
    )
}
