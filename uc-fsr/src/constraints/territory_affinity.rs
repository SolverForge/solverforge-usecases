use crate::domain::{FieldServicePlan, FieldServicePlanConstraintStreams, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: prefer visits inside the technician's familiar territory.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.route_territory_matches > 0)
        .reward(|route: &TechnicianRoute| HardSoftScore::of(0, route.route_territory_matches * 25))
        .named("Territory Affinity")
}
