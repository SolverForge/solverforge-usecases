use crate::constraints::route_metrics::{
    shift_capacity_match_count, shift_capacity_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: the complete route must fit inside the technician shift and route cap.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Shift Capacity",
        true,
        HardSoftScore::of(1, 0),
        shift_capacity_score,
        shift_capacity_match_count,
    )
}
