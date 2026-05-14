use crate::constraints::route_metrics::{
    territory_affinity_match_count, territory_affinity_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: prefer visits inside the technician's familiar territory.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Territory Affinity",
        false,
        HardSoftScore::of(0, 1),
        territory_affinity_score,
        territory_affinity_match_count,
    )
}
