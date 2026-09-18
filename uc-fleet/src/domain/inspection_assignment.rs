use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

use super::Plan;

/// Inspection assignment solved over the inspection day variable.
#[planning_entity]
#[derive(Serialize, Deserialize)]
pub struct InspectionAssignment {
    #[planning_id]
    pub id: String,
    pub requirement_id: String,
    pub vessel_id: String,
    pub work_package_id: String,
    pub baseline_day: i32,
    #[serde(default)]
    pub day_candidates: Vec<usize>,
    // @solverforge:begin entity-variables
    #[planning_variable(
        value_range_provider = "days",
        allows_unassigned = true,
        candidate_values = "day_candidates",
        nearby_value_candidates = "day_candidates",
        nearby_value_distance_meter = "day_candidate_distance"
    )]
    pub day_idx: Option<usize>,
    // @solverforge:end entity-variables
}

impl InspectionAssignment {
    pub fn new(id: impl Into<String>, requirement_id: String, vessel_id: String, work_package_id: String, baseline_day: i32) -> Self {
        Self {
            id: id.into(),
            requirement_id,
            vessel_id,
            work_package_id,
            baseline_day,
            day_candidates: Vec::new(),
            // @solverforge:begin entity-variable-init
            day_idx: None,
            // @solverforge:end entity-variable-init
        }
    }
}

pub(super) fn day_candidates(plan: &Plan, entity_index: usize, _variable_index: usize) -> &[usize] {
    plan.inspection_assignments
        .get(entity_index)
        .map(|assignment| assignment.day_candidates.as_slice())
        .unwrap_or(&[])
}

pub(super) fn day_candidate_distance(
    plan: &Plan,
    assignment: &InspectionAssignment,
    day_idx: usize,
) -> f64 {
    let candidate_day = plan
        .days
        .get(day_idx)
        .map(|day| day.index)
        .unwrap_or(assignment.baseline_day);
    let reference_day = assignment
        .day_idx
        .and_then(|idx| plan.days.get(idx))
        .map(|day| day.index)
        .unwrap_or(assignment.baseline_day);

    (candidate_day - reference_day).abs() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspection_assignment_construction() {
        let entity = InspectionAssignment::new("test-id", "test".to_string(), "test".to_string(), "test".to_string(), Default::default());
        assert_eq!(entity.id, "test-id");
        let _ = &entity.requirement_id;
        let _ = &entity.vessel_id;
        let _ = &entity.work_package_id;
        let _ = &entity.baseline_day;
    }
}
