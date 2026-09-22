use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:neutral-solution
// @solverforge:begin solution-imports
use super::PickStep;
use super::Trolley;
use super::calculate_distance;
// @solverforge:end solution-imports

/// The warehouse facts and route-owning trolley entities optimized together.
#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[shadow_variable_updates(
    list_owner = "trolleys",
    post_update_listener = "refresh_trolley_route_shadows"
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    // @solverforge:begin solution-collections
    #[problem_fact_collection]
    pub pick_steps: Vec<PickStep>,
    #[planning_entity_collection]
    pub trolleys: Vec<Trolley>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardSoftScore>,
}

impl Plan {
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        pick_steps: Vec<PickStep>,
        trolleys: Vec<Trolley>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        Self {
            // @solverforge:begin solution-constructor-init
            pick_steps,
            trolleys,
            // @solverforge:end solution-constructor-init
            score: None,
        }
    }

    /// Recomputes route-level values after SolverForge mutates one trolley list.
    pub fn refresh_trolley_route_shadows(&mut self, trolley_idx: usize) {
        let Some(trolley) = self.trolleys.get_mut(trolley_idx) else {
            return;
        };
        let mut order_volumes = std::collections::BTreeMap::<&str, i64>::new();
        let mut previous = &trolley.location;
        let mut distance = 0;

        for &step_idx in &trolley.step_order {
            let step = &self.pick_steps[step_idx];
            *order_volumes.entry(&step.order_id).or_default() += step.volume_cm3;
            distance += calculate_distance(previous, &step.location);
            previous = &step.location;
        }
        if !trolley.step_order.is_empty() {
            distance += calculate_distance(previous, &trolley.location);
        }

        trolley.required_bucket_count = order_volumes
            .values()
            .map(|volume| (volume + trolley.bucket_capacity - 1) / trolley.bucket_capacity)
            .sum();
        trolley.distinct_order_count = order_volumes.len() as i64;
        trolley.route_distance_meters = distance;
    }
}
