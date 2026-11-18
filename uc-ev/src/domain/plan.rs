//! The planning solution for electric-van delivery routing.
//!
//! `Plan` is both the solver input and the JSON shape the API returns. Its
//! collections are CLI-managed; the transient `problem_data` cache is rebuilt
//! by `prepare_plan` before each solve and never serialized.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use solverforge::cvrp::ProblemData;
use solverforge::prelude::*;

// @solverforge:begin solution-imports
use super::Charger;
use super::Customer;
use super::Vehicle;
// @solverforge:end solution-imports

/// Customers to serve, chargers available on the way, and the vans whose
/// routes the solver builds.
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
    // @solverforge:begin solution-collections
    #[problem_fact_collection]
    pub customers: Vec<Customer>,
    #[problem_fact_collection]
    pub chargers: Vec<Charger>,
    #[planning_entity_collection]
    pub vehicles: Vec<Vehicle>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardSoftScore>,
    /// Per-vehicle matrices for SolverForge's stock route construction and
    /// distance meters; index `i` belongs to `vehicles[i]`.
    #[serde(skip)]
    pub problem_data: Vec<Arc<ProblemData>>,
}

impl Plan {
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        customers: Vec<Customer>,
        chargers: Vec<Charger>,
        vehicles: Vec<Vehicle>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        Self {
            // @solverforge:begin solution-constructor-init
            customers,
            chargers,
            vehicles,
            // @solverforge:end solution-constructor-init
            score: None,
            problem_data: Vec::new(),
        }
    }

    /// List-variable listener: SolverForge calls this after it changes a
    /// vehicle's `visit_order`, so the route shadows are current before the
    /// constraints read them.
    pub fn refresh_vehicle_route_shadows(&mut self, vehicle_idx: usize) {
        if let Some(vehicle) = self.vehicles.get_mut(vehicle_idx) {
            vehicle.refresh_route_shadows();
        }
    }
}

impl solverforge::cvrp::VrpSolution for Plan {
    /// Every van reads its legs from a dense road-seconds table.
    type Metric = solverforge::cvrp::DenseMatrix;

    /// Matrices for the stock Clarke-Wright construction and distance meters.
    fn vehicle_data(&self, entity_idx: usize) -> Option<&ProblemData> {
        self.problem_data.get(entity_idx).map(Arc::as_ref)
    }

    fn vehicle_visits(&self, entity_idx: usize) -> &[usize] {
        &self.vehicles[entity_idx].visit_order
    }

    fn vehicle_visits_mut(&mut self, entity_idx: usize) -> &mut Vec<usize> {
        &mut self.vehicles[entity_idx].visit_order
    }

    fn vehicle_count(&self) -> usize {
        self.vehicles.len()
    }
}
