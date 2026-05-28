//! Planning solution for the field-service routing problem.
//!
//! `FieldServicePlan` is both the input to SolverForge and the domain value
//! converted to JSON snapshots after solving. Facts stay read-only; technician
//! routes carry the mutable visit list.

use serde::{Deserialize, Deserializer, Serialize};
use solverforge::prelude::*;

// @solverforge:begin solution-imports
use super::Location;
use super::route_metrics::route_stats;
use super::ServiceVisit;
use super::TechnicianRoute;
use super::TravelLeg;
// @solverforge:end solution-imports

/// Full planning solution passed to the SolverForge runtime and HTTP API.
///
/// The first three collections are read-only facts. `technician_routes` is the
/// planning entity collection because each route owns the mutable visit list.
#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[shadow_variable_updates(
    list_owner = "technician_routes",
    post_update_listener = "refresh_technician_route_shadows"
)]
#[derive(Serialize)]
pub struct FieldServicePlan {
    // @solverforge:begin solution-collections
    /// All depots and customer sites, addressed by vector index from visits and
    /// route endpoints.
    #[problem_fact_collection]
    pub locations: Vec<Location>,
    /// Customer jobs that must be inserted into technician routes.
    #[problem_fact_collection]
    pub service_visits: Vec<ServiceVisit>,
    /// Directed travel matrix used by constraints and route geometry.
    #[problem_fact_collection]
    pub travel_legs: Vec<TravelLeg>,
    /// Route entities whose `visits` lists are changed by the solver.
    #[planning_entity_collection]
    pub technician_routes: Vec<TechnicianRoute>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardSoftScore>,
}

impl<'de> Deserialize<'de> for FieldServicePlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawFieldServicePlan {
            locations: Vec<Location>,
            service_visits: Vec<ServiceVisit>,
            travel_legs: Vec<TravelLeg>,
            technician_routes: Vec<TechnicianRoute>,
            #[serde(default)]
            score: Option<HardSoftScore>,
        }

        let raw = RawFieldServicePlan::deserialize(deserializer)?;
        let mut plan = Self {
            locations: raw.locations,
            service_visits: raw.service_visits,
            travel_legs: raw.travel_legs,
            technician_routes: raw.technician_routes,
            score: raw.score,
        };
        plan.normalize();
        Ok(plan)
    }
}

impl FieldServicePlan {
    /// Builds a plan from immutable facts and initially empty route entities.
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        locations: Vec<Location>,
        service_visits: Vec<ServiceVisit>,
        travel_legs: Vec<TravelLeg>,
        technician_routes: Vec<TechnicianRoute>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        let mut plan = Self {
            // @solverforge:begin solution-constructor-init
            locations,
            service_visits,
            travel_legs,
            technician_routes,
            // @solverforge:end solution-constructor-init
            score: None,
        };
        plan.normalize();
        plan
    }

    /// Restores transient indexes and derived route shadow fields after construction or decoding.
    pub fn normalize(&mut self) {
        for (idx, visit) in self.service_visits.iter_mut().enumerate() {
            visit.index = idx;
        }

        for route_idx in 0..self.technician_routes.len() {
            self.refresh_technician_route_shadows(route_idx);
        }
    }

    /// List-variable post-update hook used by SolverForge shadow variables.
    pub fn refresh_technician_route_shadows(&mut self, route_idx: usize) {
        let stats = {
            let Some(route) = self.technician_routes.get(route_idx) else {
                return;
            };
            route_stats(self, route)
        };

        if let Some(route) = self.technician_routes.get_mut(route_idx) {
            route.apply_route_stats(stats);
        }
    }
}
