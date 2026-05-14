//! Planning solution for the field-service routing problem.
//!
//! `FieldServicePlan` is both the input to SolverForge and the domain value
//! converted to JSON snapshots after solving. Facts stay read-only; technician
//! routes carry the mutable visit list.

use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:begin solution-imports
use super::Location;
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
#[derive(Serialize, Deserialize)]
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
        Self {
            // @solverforge:begin solution-constructor-init
            locations,
            service_visits,
            travel_legs,
            technician_routes,
            // @solverforge:end solution-constructor-init
            score: None,
        }
    }
}
