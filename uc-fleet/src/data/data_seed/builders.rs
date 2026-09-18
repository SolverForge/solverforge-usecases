mod candidates;

pub use candidates::populate_candidate_indices;

use crate::domain::{Day, InspectionAssignment, Plan, TrainingAssignment, Vessel, WorkPackage};

#[derive(Clone, Copy)]
pub struct VesselSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub class: &'static str,
    pub compatible: &'static [&'static str],
}

#[derive(Clone, Copy)]
pub struct WorkPackageSpec {
    pub id: &'static str,
    pub vessel_id: &'static str,
    pub package_type: &'static str,
    pub duration: i32,
    pub window: (i32, i32),
    pub dock_class: &'static str,
    pub demand: (i32, i32, i32, i32),
    pub primary_part: (&'static str, i32),
    pub secondary_part: (&'static str, i32),
    pub priority: i32,
    pub defer_allowed: bool,
    pub baseline: (i32, &'static str),
}

pub fn vessel(spec: &VesselSpec) -> Vessel {
    Vessel::new(
        spec.id,
        spec.name,
        spec.class,
        spec.compatible
            .iter()
            .map(|class| class.to_string())
            .collect(),
        1,
    )
}

pub fn days() -> Vec<Day> {
    (1..=56)
        .map(|day| {
            Day::new(
                format!("DAY-{day:02}"),
                format!("Day {day}"),
                day,
                week(day),
            )
        })
        .collect()
}

pub fn work_package(spec: &WorkPackageSpec) -> WorkPackage {
    WorkPackage::new(
        spec.id,
        spec.vessel_id.to_string(),
        spec.package_type.to_string(),
        spec.duration,
        spec.window.0,
        spec.window.1,
        spec.dock_class.to_string(),
        spec.demand.0,
        spec.demand.1,
        spec.demand.2,
        spec.demand.3,
        spec.primary_part.0.to_string(),
        spec.primary_part.1,
        spec.secondary_part.0.to_string(),
        spec.secondary_part.1,
        spec.priority,
        spec.defer_allowed,
        1,
        spec.baseline.0,
        spec.baseline.1.to_string(),
    )
}

pub fn assign_baseline_decisions(plan: &mut Plan) {
    let dock_ids = plan
        .docks
        .iter()
        .map(|dock| dock.id.as_str())
        .collect::<Vec<_>>();
    for package in &mut plan.work_packages {
        package.dock_idx = dock_idx(&dock_ids, &package.baseline_dock_id);
        package.start_day_idx = day_idx(package.baseline_start_day);
    }
    for assignment in &mut plan.inspection_assignments {
        assignment.day_idx = day_idx(assignment.baseline_day);
    }
    for assignment in &mut plan.training_assignments {
        assignment.day_idx = day_idx(assignment.baseline_day);
    }
}

pub fn shift_assignment_day(day_idx: &mut Option<usize>, shift: i32) {
    let Some(current) = day_idx.and_then(|idx| i32::try_from(idx).ok()) else {
        return;
    };
    *day_idx = day_idx_from_zero_based(current + shift);
}

pub fn baseline_inspection_assignment(
    requirement_id: String,
    vessel_id: String,
    work_package_id: String,
    baseline_day: i32,
) -> InspectionAssignment {
    InspectionAssignment::new(
        format!("ASSIGN-{requirement_id}"),
        requirement_id,
        vessel_id,
        work_package_id,
        baseline_day,
    )
}

pub fn baseline_training_assignment(
    requirement_id: String,
    vessel_id: String,
    inspection_id: String,
    baseline_day: i32,
) -> TrainingAssignment {
    TrainingAssignment::new(
        format!("ASSIGN-{requirement_id}"),
        requirement_id,
        vessel_id,
        inspection_id,
        baseline_day,
    )
}

fn week(day: i32) -> i32 {
    ((day - 1) / 7) + 1
}

fn day_idx(day: i32) -> Option<usize> {
    day_idx_from_zero_based(day - 1)
}

fn day_idx_from_zero_based(idx: i32) -> Option<usize> {
    if (0..56).contains(&idx) {
        Some(idx as usize)
    } else {
        None
    }
}

fn dock_idx(dock_ids: &[&str], id: &str) -> Option<usize> {
    dock_ids.iter().position(|dock_id| *dock_id == id)
}
