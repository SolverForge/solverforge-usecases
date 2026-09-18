use solverforge::prelude::*;

use crate::domain::Plan;

use super::core::{
    assigned_dock, assigned_start, day_index, demand_for_pool, inspection_requirement, soft,
    technician_regular_capacity, training_requirement, week_days,
};

pub fn count_inspection_lateness(plan: &Plan) -> usize {
    plan.inspection_assignments
        .iter()
        .map(|assignment| {
            let Some(requirement) = inspection_requirement(plan, assignment) else {
                return 0;
            };
            let Some(day) = day_index(plan, assignment.day_idx) else {
                return 0;
            };
            (day + requirement.duration_days - 1 - requirement.latest_day).max(0) as usize
        })
        .sum()
}

pub fn score_minimize_inspection_lateness(plan: &Plan) -> HardSoftScore {
    soft(-(count_inspection_lateness(plan) as i64) * 200)
}

pub fn count_overtime(plan: &Plan) -> usize {
    let mut overtime = 0;
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
            overtime += (demand - technician_regular_capacity(plan, &pool.id, day)).max(0) as usize;
        }
    }
    overtime
}

pub fn score_minimize_overtime(plan: &Plan) -> HardSoftScore {
    soft(-(count_overtime(plan) as i64) * 20)
}

pub fn count_churn(plan: &Plan) -> usize {
    let work_churn = plan
        .work_packages
        .iter()
        .map(|package| {
            let start_changed = package.baseline_start_day > 0
                && assigned_start(plan, package) != Some(package.baseline_start_day);
            let dock_changed = !package.baseline_dock_id.is_empty()
                && assigned_dock(plan, package)
                    .is_none_or(|dock| dock.id != package.baseline_dock_id);
            usize::from(start_changed) + usize::from(dock_changed)
        })
        .sum::<usize>();
    let inspection_churn = plan
        .inspection_assignments
        .iter()
        .filter(|assignment| {
            assignment.baseline_day > 0
                && day_index(plan, assignment.day_idx) != Some(assignment.baseline_day)
        })
        .count();
    let training_churn = plan
        .training_assignments
        .iter()
        .filter(|assignment| {
            assignment.baseline_day > 0
                && day_index(plan, assignment.day_idx) != Some(assignment.baseline_day)
        })
        .count();
    work_churn + inspection_churn + training_churn
}

pub fn score_minimize_churn(plan: &Plan) -> HardSoftScore {
    soft(-(count_churn(plan) as i64) * 10)
}

pub fn count_deferrals(plan: &Plan) -> usize {
    plan.work_packages
        .iter()
        .filter(|package| {
            package.defer_allowed && (package.start_day_idx.is_none() || package.dock_idx.is_none())
        })
        .count()
}

pub fn score_minimize_deferral(plan: &Plan) -> HardSoftScore {
    soft(-(count_deferrals(plan) as i64) * 80)
}
