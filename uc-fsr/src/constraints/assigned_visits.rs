//! Assignment-coverage rules for field-service visits.
//!
//! Missing and invalid assignments fit stock SolverForge streams. Duplicate
//! assignments need an exact count and an accurate analysis match count, so that
//! rule uses a small custom incremental constraint instead of a grouped stream
//! that would count singleton groups as matches.

use crate::domain::{
    FieldServicePlan, FieldServicePlanConstraintStreams, ServiceVisit, TechnicianRoute,
};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::{ConstraintSet, IncrementalConstraint, IncrementalConstraintSealed};
use solverforge_core::ConstraintRef;

pub(super) fn constraint() -> impl ConstraintSet<FieldServicePlan, HardSoftScore> {
    (
        missing_visits(),
        duplicate_assignments(),
        invalid_assignments(),
    )
}

pub(super) fn missing_visits() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .service_visits()
        .if_not_exists((
            ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
                .technician_routes()
                .flattened(|route: &TechnicianRoute| &route.visits),
            equal_bi(
                |visit: &ServiceVisit| visit.index,
                |assigned_visit_idx: &usize| *assigned_visit_idx,
            ),
        ))
        .penalize(hard_weight(|_: &ServiceVisit| HardSoftScore::of(1, 0)))
        .named("Assigned Visits")
}

pub(super) fn duplicate_assignments() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore>
{
    DuplicateAssignmentsConstraint::new()
}

pub(super) fn invalid_assignments() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    ConstraintFactory::<FieldServicePlan, HardSoftScore>::new()
        .technician_routes()
        .filter(|route: &TechnicianRoute| route.route_invalid_visits > 0)
        .penalize(hard_weight(|route: &TechnicianRoute| {
            HardSoftScore::of(route.route_invalid_visits, 0)
        }))
        .named("Invalid Visit Assignments")
}

struct DuplicateAssignmentsConstraint {
    constraint_ref: ConstraintRef,
    last_score: HardSoftScore,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct DuplicateAssignmentTotals {
    duplicate_groups: usize,
    extra_assignments: i64,
}

impl DuplicateAssignmentsConstraint {
    fn new() -> Self {
        Self {
            constraint_ref: ConstraintRef::new("", "Duplicate Visit Assignments"),
            last_score: HardSoftScore::ZERO,
        }
    }

    fn score_for(plan: &FieldServicePlan) -> HardSoftScore {
        let totals = duplicate_assignment_totals(plan);
        HardSoftScore::of(-totals.extra_assignments, 0)
    }
}

impl IncrementalConstraintSealed for DuplicateAssignmentsConstraint {}

impl IncrementalConstraint<FieldServicePlan, HardSoftScore> for DuplicateAssignmentsConstraint {
    fn evaluate(&self, solution: &FieldServicePlan) -> HardSoftScore {
        Self::score_for(solution)
    }

    fn match_count(&self, solution: &FieldServicePlan) -> usize {
        duplicate_assignment_totals(solution).duplicate_groups
    }

    fn initialize(&mut self, solution: &FieldServicePlan) -> HardSoftScore {
        self.last_score = Self::score_for(solution);
        self.last_score
    }

    fn on_insert(
        &mut self,
        solution: &FieldServicePlan,
        _entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        let next_score = Self::score_for(solution);
        let delta = next_score - self.last_score;
        self.last_score = next_score;
        delta
    }

    fn on_retract(
        &mut self,
        _solution: &FieldServicePlan,
        _entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        HardSoftScore::ZERO
    }

    fn reset(&mut self) {
        self.last_score = HardSoftScore::ZERO;
    }

    fn constraint_ref(&self) -> &ConstraintRef {
        &self.constraint_ref
    }

    fn is_hard(&self) -> bool {
        true
    }
}

fn duplicate_assignment_totals(plan: &FieldServicePlan) -> DuplicateAssignmentTotals {
    let mut counts = vec![0usize; plan.service_visits.len()];
    for route in &plan.technician_routes {
        for &visit_idx in &route.visits {
            if let Some(count) = counts.get_mut(visit_idx) {
                *count += 1;
            }
        }
    }

    counts.iter().filter(|&&count| count > 1).fold(
        DuplicateAssignmentTotals::default(),
        |mut totals, &count| {
            totals.duplicate_groups += 1;
            totals.extra_assignments += (count - 1) as i64;
            totals
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        FieldServicePlan, ServiceVisit, ServiceVisitInit, TechnicianRoute, TechnicianRouteInit,
    };
    use solverforge::ConstraintSet;

    #[test]
    fn empty_routes_are_penalized_for_unassigned_visits() {
        let score = assignment_constraints().evaluate_all(&sample_plan(vec![vec![]]));

        assert_eq!(score, HardSoftScore::of(-2, 0));
    }

    #[test]
    fn every_visit_once_is_feasible() {
        let score = assignment_constraints().evaluate_all(&sample_plan(vec![vec![0, 1]]));

        assert_eq!(score, HardSoftScore::ZERO);
    }

    #[test]
    fn duplicate_assignments_are_penalized_even_when_no_visit_is_missing() {
        let score = assignment_constraints().evaluate_all(&sample_plan(vec![vec![0, 1, 1]]));

        assert_eq!(score, HardSoftScore::of(-1, 0));
    }

    #[test]
    fn duplicate_assignment_analysis_counts_only_duplicate_groups() {
        let feasible_plan = sample_plan(vec![vec![0, 1]]);
        let duplicate_plan = sample_plan(vec![vec![0, 1, 1]]);
        let triplicate_plan = sample_plan(vec![vec![0, 0, 0, 1]]);
        let constraint = duplicate_assignments();

        assert_eq!(constraint.match_count(&feasible_plan), 0);
        assert_eq!(constraint.match_count(&duplicate_plan), 1);
        assert_eq!(constraint.match_count(&triplicate_plan), 1);
        assert_eq!(
            constraint.evaluate(&triplicate_plan),
            HardSoftScore::of(-2, 0)
        );
    }

    #[test]
    fn duplicate_or_invalid_visit_indexes_are_hard_issues() {
        let score = assignment_constraints().evaluate_all(&sample_plan(vec![vec![0, 0, 99]]));

        assert_eq!(score, HardSoftScore::of(-3, 0));
    }

    #[test]
    fn invalid_visit_indexes_are_not_counted_as_duplicate_service_visits() {
        let score = assignment_constraints().evaluate_all(&sample_plan(vec![vec![0, 1, 99, 99]]));

        assert_eq!(score, HardSoftScore::of(-2, 0));
    }

    #[test]
    fn duplicate_assignment_incremental_delta_matches_fresh_score() {
        let mut plan = sample_plan(vec![vec![0, 1]]);
        let mut constraints = assignment_constraints();
        let initial = constraints.initialize_all(&plan);

        let retract_delta = constraints.on_retract_all(&plan, 0, 0);
        plan.technician_routes[0].visits.push(1);
        plan.refresh_technician_route_shadows(0);
        let insert_delta = constraints.on_insert_all(&plan, 0, 0);

        assert_eq!(
            initial + retract_delta + insert_delta,
            constraints.evaluate_all(&plan)
        );
    }

    fn assignment_constraints() -> impl ConstraintSet<FieldServicePlan, HardSoftScore> {
        (
            missing_visits(),
            duplicate_assignments(),
            invalid_assignments(),
        )
    }

    fn sample_plan(route_visits: Vec<Vec<usize>>) -> FieldServicePlan {
        let service_visits = (0..2)
            .map(|idx| {
                ServiceVisit::new(ServiceVisitInit {
                    id: format!("visit-{idx}"),
                    name: format!("Visit {idx}"),
                    customer: format!("Customer {idx}"),
                    location_idx: idx,
                    duration_minutes: 30,
                    earliest_minute: 480,
                    latest_minute: 1020,
                    required_skill_mask: 0,
                    required_parts_mask: 0,
                    priority: 1,
                    territory: "center".to_string(),
                })
            })
            .collect();
        let technician_routes = route_visits
            .into_iter()
            .enumerate()
            .map(|(idx, visits)| {
                let mut route = TechnicianRoute::new(TechnicianRouteInit {
                    id: format!("route-{idx}"),
                    technician_id: format!("tech-{idx}"),
                    technician_name: format!("Tech {idx}"),
                    color: "#2563eb".to_string(),
                    start_location_idx: 0,
                    end_location_idx: 0,
                    shift_start_minute: 480,
                    shift_end_minute: 1020,
                    max_route_minutes: 480,
                    skill_mask: 0,
                    inventory_mask: 0,
                    territory: "center".to_string(),
                });
                route.visits = visits;
                route
            })
            .collect();

        FieldServicePlan::new(Vec::new(), service_visits, Vec::new(), technician_routes)
    }
}
