use crate::constraints::route_metrics::{
    required_skills_match_count, required_skills_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: a technician route may only contain visits whose skill mask is covered.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Required Skills",
        true,
        HardSoftScore::of(1, 0),
        required_skills_score,
        required_skills_match_count,
    )
}
