use crate::constraints::route_metrics::{
    priority_slack_match_count, priority_slack_score, RouteConstraint,
};
use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: reward serving high-priority visits with slack before their deadline.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    RouteConstraint::new(
        "Priority Slack",
        false,
        HardSoftScore::of(0, 1),
        priority_slack_score,
        priority_slack_match_count,
    )
}
