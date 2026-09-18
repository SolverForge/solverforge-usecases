use solverforge::prelude::*;

use crate::domain::{Plan, WorkPackage};

use super::core::{
    assigned_dock, assigned_start, day_index, demand_for_pool, dock_class_rank, hard,
    inspection_for_id, inspection_requirement, overlaps_window, technician_total_capacity,
    training_requirement, vessel, week_days, work_package_for_id,
};

pub fn count_required_assignments(plan: &Plan) -> usize {
    let missing_work = plan
        .work_packages
        .iter()
        .map(|package| {
            if package.defer_allowed {
                0
            } else {
                usize::from(package.start_day_idx.is_none())
                    + usize::from(package.dock_idx.is_none())
            }
        })
        .sum::<usize>();
    let missing_inspections = plan
        .inspection_assignments
        .iter()
        .filter(|assignment| assignment.day_idx.is_none())
        .count();
    let missing_training = plan
        .training_assignments
        .iter()
        .filter(|assignment| assignment.day_idx.is_none())
        .count();
    missing_work + missing_inspections + missing_training
}

pub fn score_required_assignments(plan: &Plan) -> HardSoftScore {
    hard(count_required_assignments(plan) as i64)
}

pub fn count_work_window(plan: &Plan) -> usize {
    let work = plan
        .work_packages
        .iter()
        .filter(|package| {
            assigned_start(plan, package).is_some_and(|start| {
                start < package.earliest_start_day
                    || start + package.duration_days - 1 > package.latest_finish_day
            })
        })
        .count();
    let inspections = plan
        .inspection_assignments
        .iter()
        .filter(|assignment| {
            let Some(requirement) = inspection_requirement(plan, assignment) else {
                return true;
            };
            day_index(plan, assignment.day_idx).is_some_and(|day| {
                day < requirement.earliest_day
                    || day + requirement.duration_days - 1 > requirement.latest_day
            })
        })
        .count();
    let training = plan
        .training_assignments
        .iter()
        .filter(|assignment| {
            let Some(requirement) = training_requirement(plan, assignment) else {
                return true;
            };
            day_index(plan, assignment.day_idx).is_some_and(|day| {
                day < requirement.earliest_day
                    || day + requirement.duration_days - 1 > requirement.latest_day
            })
        })
        .count();
    work + inspections + training
}

pub fn score_work_window(plan: &Plan) -> HardSoftScore {
    hard(count_work_window(plan) as i64)
}

pub fn count_dock_compatibility(plan: &Plan) -> usize {
    plan.work_packages
        .iter()
        .filter(|package| {
            let Some(dock) = assigned_dock(plan, package) else {
                return false;
            };
            let Some(vessel) = vessel(plan, &package.vessel_id) else {
                return true;
            };
            !vessel
                .compatible_dock_classes
                .iter()
                .any(|dock_class| dock_class == &dock.dock_class)
                || dock_class_rank(&dock.dock_class) < dock_class_rank(&package.dock_class_required)
        })
        .count()
}

pub fn score_dock_compatibility(plan: &Plan) -> HardSoftScore {
    hard(count_dock_compatibility(plan) as i64)
}

pub fn count_dock_capacity(plan: &Plan) -> usize {
    let mut violations = 0;
    for dock in &plan.docks {
        for day in week_days(plan) {
            let active = plan
                .work_packages
                .iter()
                .filter(|package| {
                    assigned_dock(plan, package).is_some_and(|assigned| assigned.id == dock.id)
                        && assigned_start(plan, package).is_some_and(|start| {
                            day >= start && day < start + package.duration_days
                        })
                })
                .count() as i32;
            violations += (active - dock.capacity).max(0) as usize;
        }
    }
    violations
}

pub fn score_dock_capacity(plan: &Plan) -> HardSoftScore {
    hard(count_dock_capacity(plan) as i64)
}

pub fn count_dock_outages(plan: &Plan) -> usize {
    plan.dock_outages
        .iter()
        .map(|outage| {
            plan.work_packages
                .iter()
                .filter(|package| {
                    assigned_dock(plan, package).is_some_and(|dock| dock.id == outage.dock_id)
                        && assigned_start(plan, package).is_some_and(|start| {
                            overlaps_window(
                                start,
                                package.duration_days,
                                outage.start_day,
                                outage.end_day,
                            )
                        })
                })
                .count()
        })
        .sum()
}

pub fn score_dock_outages(plan: &Plan) -> HardSoftScore {
    hard(count_dock_outages(plan) as i64)
}

pub fn count_inspection_training_precedence(plan: &Plan) -> usize {
    let inspection_violations = plan
        .inspection_assignments
        .iter()
        .filter(|assignment| {
            let Some(package) = work_package_for_id(plan, &assignment.work_package_id) else {
                return true;
            };
            let Some(work_start) = assigned_start(plan, package) else {
                return false;
            };
            let Some(day) = day_index(plan, assignment.day_idx) else {
                return false;
            };
            day < work_start + package.duration_days
        })
        .count();
    let training_violations = plan
        .training_assignments
        .iter()
        .filter(|assignment| {
            let Some(inspection) = inspection_for_id(plan, &assignment.inspection_id) else {
                return true;
            };
            let Some(requirement) = inspection_requirement(plan, inspection) else {
                return true;
            };
            let Some(inspection_day) = day_index(plan, inspection.day_idx) else {
                return false;
            };
            let Some(training_day) = day_index(plan, assignment.day_idx) else {
                return false;
            };
            training_day < inspection_day + requirement.duration_days
        })
        .count();
    inspection_violations + training_violations
}

pub fn score_inspection_training_precedence(plan: &Plan) -> HardSoftScore {
    hard(count_inspection_training_precedence(plan) as i64)
}

pub fn count_technician_capacity(plan: &Plan) -> usize {
    let mut violations = 0;
    for pool in &plan.technician_pools {
        for day in week_days(plan) {
            let mut demand = 0;
            for package in &plan.work_packages {
                if assigned_start(plan, package)
                    .is_some_and(|start| day >= start && day < start + package.duration_days)
                {
                    demand += demand_for_pool(package, &pool.id);
                }
            }
            if pool.id == "QA" {
                for assignment in &plan.inspection_assignments {
                    let Some(requirement) = inspection_requirement(plan, assignment) else {
                        continue;
                    };
                    if day_index(plan, assignment.day_idx).is_some_and(|start| {
                        day >= start && day < start + requirement.duration_days
                    }) {
                        demand += requirement.required_qa_per_day;
                    }
                }
            }
            if pool.id == "TRNG" {
                for assignment in &plan.training_assignments {
                    let Some(requirement) = training_requirement(plan, assignment) else {
                        continue;
                    };
                    if day_index(plan, assignment.day_idx).is_some_and(|start| {
                        day >= start && day < start + requirement.duration_days
                    }) {
                        demand += requirement.required_capacity_per_day;
                    }
                }
            }
            let limit = technician_total_capacity(plan, &pool.id, day);
            violations += (demand - limit).max(0) as usize;
        }
    }
    violations
}

pub fn score_technician_capacity(plan: &Plan) -> HardSoftScore {
    hard(count_technician_capacity(plan) as i64)
}

fn part_requirements(package: &WorkPackage) -> impl Iterator<Item = (&str, i32)> {
    [
        (
            package.required_part_id.as_str(),
            package.required_part_quantity,
        ),
        (
            package.secondary_part_id.as_str(),
            package.secondary_part_quantity,
        ),
    ]
    .into_iter()
    .filter(|(part_id, quantity)| !part_id.is_empty() && *quantity > 0)
}

pub fn count_parts_availability(plan: &Plan) -> usize {
    let mut violations = 0;
    for part in &plan.parts {
        for day in week_days(plan) {
            let supply = part.initial_on_hand
                + plan
                    .deliveries
                    .iter()
                    .filter(|delivery| delivery.part_id == part.id && delivery.arrival_day <= day)
                    .map(|delivery| delivery.quantity)
                    .sum::<i32>();
            let consumed = plan
                .work_packages
                .iter()
                .filter(|package| assigned_start(plan, package).is_some_and(|start| start <= day))
                .flat_map(part_requirements)
                .filter(|(part_id, _)| *part_id == part.id)
                .map(|(_, quantity)| quantity)
                .sum::<i32>();
            violations += (consumed - supply).max(0) as usize;
        }
    }
    violations
}

pub fn score_parts_availability(plan: &Plan) -> HardSoftScore {
    hard(count_parts_availability(plan) as i64)
}
