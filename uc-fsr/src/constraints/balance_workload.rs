use crate::constraints::route_metrics::{
    balance_workload_match_count, balance_workload_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: discourage concentrating all service and travel minutes on one route.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Balance Workload",
        false,
        HardSoftScore::of(0, 1),
        balance_workload_score,
        balance_workload_match_count,
    )
}
