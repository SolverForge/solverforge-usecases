//! Small reusable incremental constraint wrapper for route-level rules.
//!
//! Most FSR rules score one technician route at a time. This adapter keeps that
//! pattern explicit: a rule provides a route scorer and match counter, while
//! SolverForge calls `on_insert` and `on_retract` when a route entity changes.

use crate::domain::{FieldServicePlan, TechnicianRoute};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;
use solverforge_core::ConstraintRef;

pub struct RouteConstraint {
    constraint_ref: ConstraintRef,
    hard: bool,
    weight: HardSoftScore,
    scorer: fn(&FieldServicePlan, &TechnicianRoute) -> HardSoftScore,
    match_counter: fn(&FieldServicePlan, &TechnicianRoute) -> usize,
}

impl RouteConstraint {
    /// Creates a named route-level scoring rule.
    pub fn new(
        name: &'static str,
        hard: bool,
        weight: HardSoftScore,
        scorer: fn(&FieldServicePlan, &TechnicianRoute) -> HardSoftScore,
        match_counter: fn(&FieldServicePlan, &TechnicianRoute) -> usize,
    ) -> Self {
        Self {
            constraint_ref: ConstraintRef::new("field_service_routing", name),
            hard,
            weight,
            scorer,
            match_counter,
        }
    }

    /// Computes only the changed route's score for incremental callbacks.
    fn route_score(&self, solution: &FieldServicePlan, entity_index: usize) -> HardSoftScore {
        solution
            .technician_routes
            .get(entity_index)
            .map(|route| (self.scorer)(solution, route))
            .unwrap_or(HardSoftScore::ZERO)
    }
}

impl IncrementalConstraint<FieldServicePlan, HardSoftScore> for RouteConstraint {
    fn evaluate(&self, solution: &FieldServicePlan) -> HardSoftScore {
        solution
            .technician_routes
            .iter()
            .map(|route| (self.scorer)(solution, route))
            .fold(HardSoftScore::ZERO, |total, score| total + score)
    }

    fn match_count(&self, solution: &FieldServicePlan) -> usize {
        solution
            .technician_routes
            .iter()
            .map(|route| (self.match_counter)(solution, route))
            .sum()
    }

    fn initialize(&mut self, solution: &FieldServicePlan) -> HardSoftScore {
        self.evaluate(solution)
    }

    fn on_insert(
        &mut self,
        solution: &FieldServicePlan,
        entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        self.route_score(solution, entity_index)
    }

    fn on_retract(
        &mut self,
        solution: &FieldServicePlan,
        entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        -self.route_score(solution, entity_index)
    }

    fn reset(&mut self) {}

    fn name(&self) -> &str {
        &self.constraint_ref.name
    }

    fn is_hard(&self) -> bool {
        self.hard
    }

    fn weight(&self) -> HardSoftScore {
        self.weight
    }

    fn constraint_ref(&self) -> &ConstraintRef {
        &self.constraint_ref
    }
}
