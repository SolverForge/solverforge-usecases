use crate::constraints::route_metrics::{
    required_parts_match_count, required_parts_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: route inventory must cover every assigned visit's required parts.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Required Parts",
        true,
        HardSoftScore::of(1, 0),
        required_parts_score,
        required_parts_match_count,
    )
}
