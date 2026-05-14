use crate::domain::FieldServicePlan;
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;
use solverforge_core::ConstraintRef;

/// HARD: every service visit must appear exactly once in a technician route.
pub fn constraint() -> impl IncrementalConstraint<FieldServicePlan, HardSoftScore> {
    AssignedVisitsConstraint::new()
}

struct AssignedVisitsConstraint {
    constraint_ref: ConstraintRef,
}

impl AssignedVisitsConstraint {
    fn new() -> Self {
        Self {
            constraint_ref: ConstraintRef::new("field_service_routing", "Assigned Visits"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct AssignmentIssues {
    unassigned: i64,
    duplicate_assignments: i64,
    invalid_assignments: i64,
}

impl AssignmentIssues {
    fn total(self) -> i64 {
        self.unassigned + self.duplicate_assignments + self.invalid_assignments
    }
}

impl IncrementalConstraint<FieldServicePlan, HardSoftScore> for AssignedVisitsConstraint {
    fn evaluate(&self, solution: &FieldServicePlan) -> HardSoftScore {
        HardSoftScore::of(-assignment_issues(solution).total(), 0)
    }

    fn match_count(&self, solution: &FieldServicePlan) -> usize {
        assignment_issues(solution).total() as usize
    }

    fn initialize(&mut self, solution: &FieldServicePlan) -> HardSoftScore {
        self.evaluate(solution)
    }

    fn on_insert(
        &mut self,
        solution: &FieldServicePlan,
        _entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        self.evaluate(solution)
    }

    fn on_retract(
        &mut self,
        solution: &FieldServicePlan,
        _entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        -self.evaluate(solution)
    }

    fn reset(&mut self) {}

    fn name(&self) -> &str {
        &self.constraint_ref.name
    }

    fn is_hard(&self) -> bool {
        true
    }

    fn weight(&self) -> HardSoftScore {
        HardSoftScore::of(1, 0)
    }

    fn constraint_ref(&self) -> &ConstraintRef {
        &self.constraint_ref
    }
}

fn assignment_issues(plan: &FieldServicePlan) -> AssignmentIssues {
    let mut counts = vec![0usize; plan.service_visits.len()];
    let mut issues = AssignmentIssues::default();

    for route in &plan.technician_routes {
        for &visit_idx in &route.visits {
            if let Some(count) = counts.get_mut(visit_idx) {
                *count += 1;
            } else {
                issues.invalid_assignments += 1;
            }
        }
    }

    for count in counts {
        match count {
            0 => issues.unassigned += 1,
            1 => {}
            extra => issues.duplicate_assignments += (extra - 1) as i64,
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        FieldServicePlan, ServiceVisit, ServiceVisitInit, TechnicianRoute, TechnicianRouteInit,
    };
    use solverforge::IncrementalConstraint;

    #[test]
    fn empty_routes_are_penalized_for_unassigned_visits() {
        let score = constraint().evaluate(&sample_plan(vec![vec![]]));

        assert_eq!(score, HardSoftScore::of(-2, 0));
    }

    #[test]
    fn every_visit_once_is_feasible() {
        let score = constraint().evaluate(&sample_plan(vec![vec![0, 1]]));

        assert_eq!(score, HardSoftScore::ZERO);
    }

    #[test]
    fn duplicate_or_invalid_visit_indexes_are_hard_issues() {
        let score = constraint().evaluate(&sample_plan(vec![vec![0, 0, 99]]));

        assert_eq!(score, HardSoftScore::of(-3, 0));
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
