use crate::domain::{Day, Plan, WorkPackage};

pub fn populate_candidate_indices(plan: &mut Plan) {
    let work_dock_candidates = plan
        .work_packages
        .iter()
        .map(|package| {
            let vessel = plan
                .vessels
                .iter()
                .find(|vessel| vessel.id == package.vessel_id)
                .expect("work package vessel");
            let candidates = plan
                .docks
                .iter()
                .enumerate()
                .filter(|(_, dock)| {
                    vessel
                        .compatible_dock_classes
                        .iter()
                        .any(|class| class == &dock.dock_class)
                        && dock_class_rank(&dock.dock_class)
                            >= dock_class_rank(&package.dock_class_required)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            ordered_candidates(
                candidates,
                dock_idx_for_plan(plan, &package.baseline_dock_id),
            )
        })
        .collect::<Vec<_>>();
    let work_day_candidates = plan
        .work_packages
        .iter()
        .map(|package| {
            let latest_start = latest_feasible_work_start(plan, package);
            candidate_days(
                &plan.days,
                package.earliest_start_day,
                latest_start + package.duration_days - 1,
                package.duration_days,
                package.baseline_start_day,
            )
        })
        .collect::<Vec<_>>();
    let inspection_day_candidates = plan
        .inspection_assignments
        .iter()
        .map(|assignment| {
            let requirement = plan
                .inspection_requirements
                .iter()
                .find(|requirement| requirement.id == assignment.requirement_id)
                .expect("inspection requirement");
            candidate_days_from_preferred(
                &plan.days,
                requirement.earliest_day,
                requirement.latest_day,
                requirement.duration_days,
                assignment.baseline_day,
            )
        })
        .collect::<Vec<_>>();
    let training_day_candidates = plan
        .training_assignments
        .iter()
        .map(|assignment| {
            let requirement = plan
                .training_requirements
                .iter()
                .find(|requirement| requirement.id == assignment.requirement_id)
                .expect("training requirement");
            candidate_days_from_preferred(
                &plan.days,
                requirement.earliest_day,
                requirement.latest_day,
                requirement.duration_days,
                assignment.baseline_day,
            )
        })
        .collect::<Vec<_>>();

    for (package, (dock_candidates, day_candidates)) in plan
        .work_packages
        .iter_mut()
        .zip(work_dock_candidates.into_iter().zip(work_day_candidates))
    {
        package.dock_candidates = dock_candidates;
        package.start_day_candidates = day_candidates;
    }
    for (assignment, candidates) in plan
        .inspection_assignments
        .iter_mut()
        .zip(inspection_day_candidates)
    {
        assignment.day_candidates = candidates;
    }
    for (assignment, candidates) in plan
        .training_assignments
        .iter_mut()
        .zip(training_day_candidates)
    {
        assignment.day_candidates = candidates;
    }
}

fn dock_idx_for_plan(plan: &Plan, id: &str) -> Option<usize> {
    plan.docks.iter().position(|dock| dock.id == id)
}

fn candidate_days(
    days: &[Day],
    earliest_day: i32,
    latest_day: i32,
    duration_days: i32,
    preferred_day: i32,
) -> Vec<usize> {
    let latest_start = latest_day - duration_days + 1;
    let candidates = days
        .iter()
        .enumerate()
        .filter(|(_, day)| day.index >= earliest_day && day.index <= latest_start)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    ordered_candidates(candidates, day_idx(preferred_day))
}

fn candidate_days_from_preferred(
    days: &[Day],
    earliest_day: i32,
    latest_day: i32,
    duration_days: i32,
    preferred_day: i32,
) -> Vec<usize> {
    candidate_days(
        days,
        earliest_day.max(preferred_day),
        latest_day,
        duration_days,
        preferred_day,
    )
}

fn day_idx(day: i32) -> Option<usize> {
    let idx = day - 1;
    if (0..56).contains(&idx) {
        Some(idx as usize)
    } else {
        None
    }
}

fn ordered_candidates(mut candidates: Vec<usize>, preferred: Option<usize>) -> Vec<usize> {
    candidates.sort_by_key(|candidate| {
        let distance = preferred
            .map(|preferred| candidate.abs_diff(preferred))
            .unwrap_or(*candidate);
        (distance, *candidate)
    });
    candidates
}

fn dock_class_rank(dock_class: &str) -> i32 {
    match dock_class {
        "small" => 1,
        "medium" => 2,
        "large" => 3,
        _ => 0,
    }
}

fn latest_feasible_work_start(plan: &Plan, package: &WorkPackage) -> i32 {
    let package_latest = package.latest_finish_day - package.duration_days + 1;
    let inspection_latest = plan
        .inspection_assignments
        .iter()
        .find(|assignment| assignment.work_package_id == package.id)
        .and_then(|assignment| {
            plan.inspection_requirements
                .iter()
                .find(|requirement| requirement.id == assignment.requirement_id)
        })
        .map(|requirement| {
            requirement.latest_day - requirement.duration_days + 1 - package.duration_days
        })
        .unwrap_or(package_latest);
    let training_latest = plan
        .inspection_assignments
        .iter()
        .find(|assignment| assignment.work_package_id == package.id)
        .and_then(|inspection| {
            plan.training_assignments
                .iter()
                .find(|training| training.inspection_id == inspection.id)
                .and_then(|training| {
                    plan.training_requirements
                        .iter()
                        .find(|requirement| requirement.id == training.requirement_id)
                })
                .and_then(|training_requirement| {
                    plan.inspection_requirements
                        .iter()
                        .find(|requirement| requirement.id == inspection.requirement_id)
                        .map(|inspection_requirement| {
                            training_requirement.latest_day - training_requirement.duration_days + 1
                                - inspection_requirement.duration_days
                                - package.duration_days
                        })
                })
        })
        .unwrap_or(package_latest);

    package_latest
        .min(inspection_latest)
        .min(training_latest)
        .min(package.baseline_start_day + 1)
        .max(package.earliest_start_day)
}
