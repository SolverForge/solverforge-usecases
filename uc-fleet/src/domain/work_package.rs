use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

use super::Plan;

/// Work package assignment solved over dock and start-day variables.
#[planning_entity]
#[derive(Serialize, Deserialize)]
pub struct WorkPackage {
    #[planning_id]
    pub id: String,
    pub vessel_id: String,
    pub package_type: String,
    pub duration_days: i32,
    pub earliest_start_day: i32,
    pub latest_finish_day: i32,
    pub dock_class_required: String,
    pub prop_demand: i32,
    pub elec_demand: i32,
    pub hull_demand: i32,
    pub qa_demand: i32,
    pub required_part_id: String,
    pub required_part_quantity: i32,
    pub secondary_part_id: String,
    pub secondary_part_quantity: i32,
    pub priority: i32,
    pub defer_allowed: bool,
    pub return_to_service_buffer_days: i32,
    pub baseline_start_day: i32,
    pub baseline_dock_id: String,
    #[serde(default)]
    pub dock_candidates: Vec<usize>,
    #[serde(default)]
    pub start_day_candidates: Vec<usize>,
    // @solverforge:begin entity-variables
    #[planning_variable(
        value_range_provider = "docks",
        allows_unassigned = true,
        candidate_values = "dock_candidates",
        nearby_value_candidates = "dock_candidates",
        nearby_value_distance_meter = "dock_candidate_distance"
    )]
    pub dock_idx: Option<usize>,
    #[planning_variable(
        value_range_provider = "days",
        allows_unassigned = true,
        candidate_values = "start_day_candidates",
        nearby_value_candidates = "start_day_candidates",
        nearby_value_distance_meter = "start_day_candidate_distance"
    )]
    pub start_day_idx: Option<usize>,
    // @solverforge:end entity-variables
}

impl WorkPackage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(id: impl Into<String>, vessel_id: String, package_type: String, duration_days: i32, earliest_start_day: i32, latest_finish_day: i32, dock_class_required: String, prop_demand: i32, elec_demand: i32, hull_demand: i32, qa_demand: i32, required_part_id: String, required_part_quantity: i32, secondary_part_id: String, secondary_part_quantity: i32, priority: i32, defer_allowed: bool, return_to_service_buffer_days: i32, baseline_start_day: i32, baseline_dock_id: String) -> Self {
        Self {
            id: id.into(),
            vessel_id,
            package_type,
            duration_days,
            earliest_start_day,
            latest_finish_day,
            dock_class_required,
            prop_demand,
            elec_demand,
            hull_demand,
            qa_demand,
            required_part_id,
            required_part_quantity,
            secondary_part_id,
            secondary_part_quantity,
            priority,
            defer_allowed,
            return_to_service_buffer_days,
            baseline_start_day,
            baseline_dock_id,
            dock_candidates: Vec::new(),
            start_day_candidates: Vec::new(),
            // @solverforge:begin entity-variable-init
            dock_idx: None,
            start_day_idx: None,
            // @solverforge:end entity-variable-init
        }
    }
}

pub(super) fn dock_candidates(plan: &Plan, entity_index: usize, _variable_index: usize) -> &[usize] {
    plan.work_packages
        .get(entity_index)
        .map(|package| package.dock_candidates.as_slice())
        .unwrap_or(&[])
}

pub(super) fn start_day_candidates(
    plan: &Plan,
    entity_index: usize,
    _variable_index: usize,
) -> &[usize] {
    plan.work_packages
        .get(entity_index)
        .map(|package| package.start_day_candidates.as_slice())
        .unwrap_or(&[])
}

pub(super) fn dock_candidate_distance(
    plan: &Plan,
    package: &WorkPackage,
    dock_idx: usize,
) -> f64 {
    let reference = package
        .dock_idx
        .or_else(|| baseline_dock_idx(plan, package))
        .unwrap_or(dock_idx);
    reference.abs_diff(dock_idx) as f64
}

pub(super) fn start_day_candidate_distance(
    plan: &Plan,
    package: &WorkPackage,
    day_idx: usize,
) -> f64 {
    let candidate_day = plan
        .days
        .get(day_idx)
        .map(|day| day.index)
        .unwrap_or(package.baseline_start_day);
    let reference_day = package
        .start_day_idx
        .and_then(|idx| plan.days.get(idx))
        .map(|day| day.index)
        .unwrap_or(package.baseline_start_day);

    (candidate_day - reference_day).abs() as f64
}

fn baseline_dock_idx(plan: &Plan, package: &WorkPackage) -> Option<usize> {
    plan.docks
        .iter()
        .position(|dock| dock.id == package.baseline_dock_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_package_construction() {
        let entity = WorkPackage::new("test-id", "test".to_string(), "test".to_string(), Default::default(), Default::default(), Default::default(), "test".to_string(), Default::default(), Default::default(), Default::default(), Default::default(), "test".to_string(), Default::default(), "test".to_string(), Default::default(), Default::default(), false, Default::default(), Default::default(), "test".to_string());
        assert_eq!(entity.id, "test-id");
        let _ = &entity.vessel_id;
        let _ = &entity.package_type;
        let _ = &entity.duration_days;
        let _ = &entity.earliest_start_day;
        let _ = &entity.latest_finish_day;
        let _ = &entity.dock_class_required;
        let _ = &entity.prop_demand;
        let _ = &entity.elec_demand;
        let _ = &entity.hull_demand;
        let _ = &entity.qa_demand;
        let _ = &entity.required_part_id;
        let _ = &entity.required_part_quantity;
        let _ = &entity.secondary_part_id;
        let _ = &entity.secondary_part_quantity;
        let _ = &entity.priority;
        let _ = &entity.defer_allowed;
        let _ = &entity.return_to_service_buffer_days;
        let _ = &entity.baseline_start_day;
        let _ = &entity.baseline_dock_id;
    }
}
