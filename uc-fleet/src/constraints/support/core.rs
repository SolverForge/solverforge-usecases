use crate::domain::{
    Dock, InspectionAssignment, InspectionRequirement, Plan, TrainingAssignment,
    TrainingRequirement, Vessel, WorkPackage,
};
use solverforge::prelude::*;
use solverforge::{IncrementalConstraint, IncrementalConstraintSealed};
use solverforge_core::ConstraintRef;

pub struct FleetConstraint {
    name: &'static str,
    constraint_ref: ConstraintRef,
    hard: bool,
    weight: HardSoftScore,
    score: fn(&Plan) -> HardSoftScore,
    matches: fn(&Plan) -> usize,
    current_score: Option<HardSoftScore>,
}

impl IncrementalConstraintSealed for FleetConstraint {}

impl FleetConstraint {
    pub fn new(
        name: &'static str,
        hard: bool,
        weight: HardSoftScore,
        score: fn(&Plan) -> HardSoftScore,
        matches: fn(&Plan) -> usize,
    ) -> Self {
        Self {
            name,
            constraint_ref: ConstraintRef::new("", name),
            hard,
            weight,
            score,
            matches,
            current_score: None,
        }
    }

    fn track_delta(&mut self, solution: &Plan) -> HardSoftScore {
        let next = self.evaluate(solution);
        let previous = self.current_score.unwrap_or_else(HardSoftScore::zero);
        self.current_score = Some(next);
        next + -previous
    }
}

impl IncrementalConstraint<Plan, HardSoftScore> for FleetConstraint {
    fn evaluate(&self, solution: &Plan) -> HardSoftScore {
        (self.score)(solution)
    }

    fn match_count(&self, solution: &Plan) -> usize {
        (self.matches)(solution)
    }

    fn initialize(&mut self, solution: &Plan) -> HardSoftScore {
        let score = self.evaluate(solution);
        self.current_score = Some(score);
        score
    }

    fn on_insert(
        &mut self,
        solution: &Plan,
        _entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        self.track_delta(solution)
    }

    fn on_retract(
        &mut self,
        solution: &Plan,
        _entity_index: usize,
        _descriptor_index: usize,
    ) -> HardSoftScore {
        self.track_delta(solution)
    }

    fn reset(&mut self) {
        self.current_score = None;
    }

    fn constraint_ref(&self) -> &ConstraintRef {
        &self.constraint_ref
    }

    fn name(&self) -> &str {
        self.name
    }

    fn is_hard(&self) -> bool {
        self.hard
    }

    fn weight(&self) -> HardSoftScore {
        self.weight
    }
}

pub(super) fn hard(units: i64) -> HardSoftScore {
    HardSoftScore::of_hard(-units)
}

pub(super) fn soft(units: i64) -> HardSoftScore {
    HardSoftScore::of_soft(units)
}

pub fn day_index(plan: &Plan, day_idx: Option<usize>) -> Option<i32> {
    day_idx
        .and_then(|idx| plan.days.get(idx))
        .map(|day| day.index)
}

pub fn assigned_start(plan: &Plan, package: &WorkPackage) -> Option<i32> {
    day_index(plan, package.start_day_idx)
}

pub fn assigned_dock<'a>(plan: &'a Plan, package: &WorkPackage) -> Option<&'a Dock> {
    package.dock_idx.and_then(|idx| plan.docks.get(idx))
}

pub(super) fn vessel<'a>(plan: &'a Plan, vessel_id: &str) -> Option<&'a Vessel> {
    plan.vessels.iter().find(|vessel| vessel.id == vessel_id)
}

pub(super) fn inspection_requirement<'a>(
    plan: &'a Plan,
    assignment: &InspectionAssignment,
) -> Option<&'a InspectionRequirement> {
    plan.inspection_requirements
        .iter()
        .find(|requirement| requirement.id == assignment.requirement_id)
}

pub(super) fn training_requirement<'a>(
    plan: &'a Plan,
    assignment: &TrainingAssignment,
) -> Option<&'a TrainingRequirement> {
    plan.training_requirements
        .iter()
        .find(|requirement| requirement.id == assignment.requirement_id)
}

pub(super) fn inspection_for_vessel<'a>(
    plan: &'a Plan,
    vessel_id: &str,
) -> Option<&'a InspectionAssignment> {
    plan.inspection_assignments
        .iter()
        .find(|assignment| assignment.vessel_id == vessel_id)
}

pub(super) fn training_for_vessel<'a>(
    plan: &'a Plan,
    vessel_id: &str,
) -> Option<&'a TrainingAssignment> {
    plan.training_assignments
        .iter()
        .find(|assignment| assignment.vessel_id == vessel_id)
}

pub(super) fn work_package_for_id<'a>(plan: &'a Plan, id: &str) -> Option<&'a WorkPackage> {
    plan.work_packages.iter().find(|package| package.id == id)
}

pub(super) fn inspection_for_id<'a>(plan: &'a Plan, id: &str) -> Option<&'a InspectionAssignment> {
    plan.inspection_assignments
        .iter()
        .find(|assignment| assignment.id == id)
}

pub(super) fn dock_class_rank(dock_class: &str) -> i32 {
    match dock_class {
        "small" => 1,
        "medium" => 2,
        "large" => 3,
        _ => 0,
    }
}

pub(super) fn demand_for_pool(package: &WorkPackage, pool_id: &str) -> i32 {
    match pool_id {
        "PROP" => package.prop_demand,
        "ELEC" => package.elec_demand,
        "HULL" => package.hull_demand,
        "QA" => package.qa_demand,
        _ => 0,
    }
}

pub(super) fn week_days(plan: &Plan) -> impl Iterator<Item = i32> + '_ {
    plan.days.iter().map(|day| day.index)
}

pub(super) fn overlaps_window(
    start: i32,
    duration_days: i32,
    window_start: i32,
    window_end: i32,
) -> bool {
    let end = start + duration_days - 1;
    start <= window_end && window_start <= end
}

pub(super) fn technician_regular_capacity(plan: &Plan, pool_id: &str, day: i32) -> i32 {
    let Some(pool) = plan.technician_pools.iter().find(|pool| pool.id == pool_id) else {
        return 0;
    };
    let delta = plan
        .technician_capacity_overrides
        .iter()
        .filter(|override_| {
            override_.pool_id == pool_id && day >= override_.start_day && day <= override_.end_day
        })
        .map(|override_| override_.delta)
        .sum::<i32>();
    (pool.capacity_per_day + delta).max(0)
}

pub(super) fn technician_total_capacity(plan: &Plan, pool_id: &str, day: i32) -> i32 {
    let overtime = plan
        .technician_pools
        .iter()
        .find(|pool| pool.id == pool_id)
        .map(|pool| pool.overtime_capacity_per_day)
        .unwrap_or(0);
    technician_regular_capacity(plan, pool_id, day) + overtime
}
