use crate::domain::{FieldServicePlan, FieldServicePlanConstraintStreams, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: reward serving high-priority visits with slack before their deadline.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.route_priority_slack > 0)
        .reward(|route: &TechnicianRoute| HardSoftScore::of(0, route.route_priority_slack))
        .named("Priority Slack")
}
