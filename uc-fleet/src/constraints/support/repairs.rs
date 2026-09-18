use crate::domain::{DockOutage, Plan, TechnicianCapacityOverride};

use super::core::{day_index, overlaps_window};

pub fn apply_technician_shortage(plan: &mut Plan, pool_id: &str, delta: i32) {
    for pool in &mut plan.technician_pools {
        if pool.id == pool_id {
            pool.capacity_per_day = (pool.capacity_per_day + delta).max(0);
        }
    }
}

pub fn apply_technician_shortage_window(
    plan: &mut Plan,
    pool_id: &str,
    delta: i32,
    start_day: i32,
    end_day: i32,
) {
    let id = format!("TECH-{}-{}-{}", pool_id, start_day, end_day);
    if let Some(existing) = plan
        .technician_capacity_overrides
        .iter_mut()
        .find(|override_| override_.id == id)
    {
        existing.delta = delta;
        return;
    }
    plan.technician_capacity_overrides
        .push(TechnicianCapacityOverride::new(
            id,
            pool_id.to_string(),
            start_day,
            end_day,
            delta,
        ));
}

pub fn apply_parts_delay(plan: &mut Plan, delivery_id: &str, new_arrival_day: i32) {
    let delayed_part_id = plan
        .deliveries
        .iter()
        .find(|delivery| delivery.id == delivery_id)
        .map(|delivery| delivery.part_id.clone());
    for delivery in &mut plan.deliveries {
        if delivery.id == delivery_id {
            delivery.arrival_day = new_arrival_day;
        }
    }
    if let Some(part_id) = delayed_part_id {
        let shifted_day_idx = plan
            .days
            .iter()
            .position(|day| day.index == new_arrival_day);
        for package in &mut plan.work_packages {
            let requires_delayed_part =
                package.required_part_id == part_id || package.secondary_part_id == part_id;
            if package.start_day_idx.is_some()
                && requires_delayed_part
                && package.baseline_start_day < new_arrival_day
            {
                package.start_day_idx = shifted_day_idx;
            }
        }
    }
}

pub fn apply_readiness_floor(plan: &mut Plan, min_ready_overall_per_week: i32) {
    if let Some(policy) = plan.readiness_policies.first_mut() {
        policy.min_ready_overall_per_week = min_ready_overall_per_week;
    }
}

pub fn apply_dock_outage(plan: &mut Plan, dock_id: &str, start_day: i32, end_day: i32) {
    let shifted_day_idx = plan.days.iter().position(|day| day.index == end_day + 1);
    let outage_id = format!("DOCK-{}-{}-{}", dock_id, start_day, end_day);
    if !plan
        .dock_outages
        .iter()
        .any(|outage| outage.id == outage_id)
    {
        plan.dock_outages.push(DockOutage::new(
            outage_id,
            dock_id.to_string(),
            start_day,
            end_day,
        ));
    }

    let affected_package_ids = plan
        .work_packages
        .iter()
        .filter(|package| {
            let current_dock_matches = package
                .dock_idx
                .and_then(|idx| plan.docks.get(idx))
                .is_some_and(|dock| dock.id == dock_id);
            let current_overlaps = package
                .start_day_idx
                .and_then(|idx| plan.days.get(idx))
                .is_some_and(|day| {
                    overlaps_window(day.index, package.duration_days, start_day, end_day)
                });
            let baseline_overlaps = overlaps_window(
                package.baseline_start_day,
                package.duration_days,
                start_day,
                end_day,
            );
            (current_dock_matches && current_overlaps)
                || (package.baseline_dock_id == dock_id && baseline_overlaps)
        })
        .map(|package| package.id.clone())
        .collect::<Vec<_>>();

    for package_id in affected_package_ids {
        if let Some(package) = plan
            .work_packages
            .iter_mut()
            .find(|package| package.id == package_id)
        {
            package.start_day_idx = shifted_day_idx;
            if let Some(idx) = shifted_day_idx {
                push_candidate(&mut package.start_day_candidates, idx);
            }
            shift_dependent_assignments(plan, &package_id);
        }
    }
}

pub fn seed_technician_shortage_repair(
    plan: &mut Plan,
    pool_id: &str,
    start_day: i32,
    end_day: i32,
) {
    let shifted_day_idx = plan.days.iter().position(|day| day.index == end_day + 1);
    for package in &mut plan.work_packages {
        let Some(current_start) = package
            .start_day_idx
            .and_then(|idx| plan.days.get(idx))
            .map(|day| day.index)
        else {
            continue;
        };
        let current_end = current_start + package.duration_days - 1;
        let overlaps_shortage = current_start <= end_day && start_day <= current_end;
        let consumes_pool = match pool_id {
            "PROP" => package.prop_demand > 0,
            "ELEC" => package.elec_demand > 0,
            "HULL" => package.hull_demand > 0,
            "QA" => package.qa_demand > 0,
            _ => false,
        };
        if overlaps_shortage && consumes_pool {
            package.start_day_idx = shifted_day_idx;
            if let Some(idx) = shifted_day_idx {
                push_candidate(&mut package.start_day_candidates, idx);
            }
            let package_id = package.id.clone();
            shift_dependent_assignments(plan, &package_id);
            break;
        }
    }
}

fn shift_dependent_assignments(plan: &mut Plan, work_package_id: &str) {
    let Some(package) = plan
        .work_packages
        .iter()
        .find(|package| package.id == work_package_id)
    else {
        return;
    };
    let Some(work_start) = day_index(plan, package.start_day_idx) else {
        return;
    };
    let inspection_day = work_start + package.duration_days;
    let inspection_day_idx = plan.days.iter().position(|day| day.index == inspection_day);
    let inspection_id = plan
        .inspection_assignments
        .iter()
        .find(|assignment| assignment.work_package_id == work_package_id)
        .map(|assignment| assignment.id.clone());

    if let Some(inspection_id) = inspection_id {
        if let Some(assignment) = plan
            .inspection_assignments
            .iter_mut()
            .find(|assignment| assignment.id == inspection_id)
        {
            assignment.day_idx = inspection_day_idx;
            if let Some(idx) = inspection_day_idx {
                push_candidate(&mut assignment.day_candidates, idx);
            }
            if let Some(requirement) = plan
                .inspection_requirements
                .iter_mut()
                .find(|requirement| requirement.id == assignment.requirement_id)
            {
                let window = (requirement.latest_day - requirement.earliest_day).max(0);
                requirement.earliest_day = inspection_day;
                requirement.latest_day = (inspection_day + window).min(56);
            }
        }

        let training_day = inspection_day + 1;
        let training_day_idx = plan.days.iter().position(|day| day.index == training_day);
        if let Some(training) = plan
            .training_assignments
            .iter_mut()
            .find(|training| training.inspection_id == inspection_id)
        {
            training.day_idx = training_day_idx;
            if let Some(idx) = training_day_idx {
                push_candidate(&mut training.day_candidates, idx);
            }
            if let Some(requirement) = plan
                .training_requirements
                .iter_mut()
                .find(|requirement| requirement.id == training.requirement_id)
            {
                let window = (requirement.latest_day - requirement.earliest_day).max(0);
                requirement.earliest_day = training_day;
                requirement.latest_day = (training_day + window).min(56);
            }
        }
    }
}

fn push_candidate(candidates: &mut Vec<usize>, candidate: usize) {
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}
