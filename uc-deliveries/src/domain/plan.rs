//! Planning solution for the delivery-routing problem.
//!
//! The `Plan` owns facts, planning entities, score, routing mode, and transient
//! prepared route matrices. It is both the solver input and the shape serialized
//! through the API after `PlanDto` flattens it.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use solverforge::cvrp::ProblemData;
use solverforge::prelude::*;

// @solverforge:begin solution-imports
use super::route_metrics::preview_for_plan;
use super::{Delivery, PlanViewState, RoutingMode, Vehicle};
// @solverforge:end solution-imports

#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[shadow_variable_updates(
    list_owner = "vehicles",
    post_update_listener = "refresh_vehicle_route_shadows"
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub name: String,
    #[serde(default)]
    pub routing_mode: RoutingMode,
    #[serde(default)]
    pub view_state: PlanViewState,
    // @solverforge:begin solution-collections
    #[problem_fact_collection]
    pub deliveries: Vec<Delivery>,
    #[planning_entity_collection]
    pub vehicles: Vec<Vehicle>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardSoftScore>,
    /// Transient CVRP matrices shared with SolverForge list-variable hooks.
    ///
    /// This is skipped in JSON because it is rebuilt from coordinates before a
    /// solve or route-geometry request.
    #[serde(skip, default)]
    pub prepared_problem_data: Vec<Arc<ProblemData>>,
}

impl Plan {
    /// Builds a normalized plan from facts and route-owning vehicles.
    pub fn new(name: impl Into<String>, deliveries: Vec<Delivery>, vehicles: Vec<Vehicle>) -> Self {
        let mut plan = Self {
            name: name.into(),
            routing_mode: RoutingMode::default(),
            view_state: PlanViewState::default(),
            deliveries,
            vehicles,
            score: None,
            prepared_problem_data: Vec::new(),
        };
        plan.normalize();
        plan
    }

    /// Reassigns dense ids and clears transient routing caches after decoding.
    ///
    /// SolverForge list variables store delivery indexes. If transport data
    /// arrived with older public ids, this maps route entries back onto the
    /// current dense delivery positions before scoring.
    pub fn normalize(&mut self) {
        let delivery_id_map: HashMap<usize, usize> = self
            .deliveries
            .iter()
            .enumerate()
            .map(|(idx, delivery)| (delivery.id, idx))
            .collect();

        for (idx, delivery) in self.deliveries.iter_mut().enumerate() {
            delivery.id = idx;
        }

        for (idx, vehicle) in self.vehicles.iter_mut().enumerate() {
            vehicle.id = idx;
            vehicle.prepared_routing = None;
            vehicle.delivery_order = vehicle
                .delivery_order
                .iter()
                .filter_map(|old_id| delivery_id_map.get(old_id).copied())
                .collect();
            vehicle.refresh_route_shadows();
        }
        self.prepared_problem_data.clear();
    }

    /// Removes one delivery id from every route before insertion previewing.
    pub fn remove_delivery_assignments(&mut self, delivery_id: usize) {
        for vehicle in &mut self.vehicles {
            vehicle
                .delivery_order
                .retain(|assigned| *assigned != delivery_id);
        }
    }

    /// List-variable post-update hook used by SolverForge shadow variables.
    pub fn refresh_vehicle_route_shadows(&mut self, vehicle_idx: usize) {
        if let Some(vehicle) = self.vehicles.get_mut(vehicle_idx) {
            vehicle.refresh_route_shadows();
        }
    }

    /// Clones the plan and attaches the UI-facing route preview.
    pub fn refreshed_for_transport(&self) -> Self {
        let mut plan = self.clone();
        plan.view_state.preview = Some(preview_for_plan(&plan));
        plan
    }
}

impl solverforge::cvrp::VrpSolution for Plan {
    /// Gives SolverForge's CVRP move selectors access to prepared matrices.
    fn vehicle_data_ptr(&self, entity_idx: usize) -> *const ProblemData {
        self.vehicles[entity_idx]
            .prepared_routing
            .as_ref()
            .and_then(|prepared| self.prepared_problem_data.get(prepared.problem_data_index))
            .map(Arc::as_ptr)
            .unwrap_or(std::ptr::null())
    }

    /// Reads the mutable list variable for one vehicle as visit ids.
    fn vehicle_visits(&self, entity_idx: usize) -> &[usize] {
        &self.vehicles[entity_idx].delivery_order
    }

    /// Lets CVRP construction and local-search hooks replace one vehicle route.
    fn vehicle_visits_mut(&mut self, entity_idx: usize) -> &mut Vec<usize> {
        &mut self.vehicles[entity_idx].delivery_order
    }

    /// Reports the number of route-owning planning entities.
    fn vehicle_count(&self) -> usize {
        self.vehicles.len()
    }
}

#[cfg(test)]
impl Plan {
    pub(crate) fn test_has_list_variable() -> bool {
        Self::__solverforge_has_list_variable()
    }

    pub(crate) fn test_total_list_entities(plan: &Self) -> usize {
        Self::__solverforge_total_list_entities(plan)
    }

    pub(crate) fn test_total_list_elements(plan: &Self) -> usize {
        Self::__solverforge_total_list_elements(plan)
    }

    pub(crate) fn test_is_trivial(plan: &Self) -> bool {
        Self::__solverforge_is_trivial(plan)
    }

    pub(crate) fn test_phase_count(config: &solverforge::SolverConfig) -> usize {
        Self::__solverforge_build_phases(config).phases().len()
    }
}
