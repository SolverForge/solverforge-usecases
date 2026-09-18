use std::collections::HashMap;

use serde::Serialize;
use solverforge::prelude::*;

use crate::domain::{Plan, Vessel};

use super::core::{
    assigned_start, day_index, hard, inspection_for_vessel, inspection_requirement, soft,
    training_for_vessel, training_requirement, week_days,
};

pub(super) fn vessel_ready_on_day(plan: &Plan, vessel: &Vessel, day: i32) -> bool {
    if day < vessel.ready_from_day {
        return false;
    }

    for package in plan
        .work_packages
        .iter()
        .filter(|package| package.vessel_id == vessel.id)
    {
        let Some(work_start) = assigned_start(plan, package) else {
            if package.defer_allowed {
                continue;
            }
            return false;
        };
        if day < work_start {
            continue;
        }

        let mut ready_day =
            work_start + package.duration_days + package.return_to_service_buffer_days;
        let Some(inspection) = inspection_for_vessel(plan, &vessel.id) else {
            return false;
        };
        let Some(inspection_day) = day_index(plan, inspection.day_idx) else {
            return false;
        };
        let Some(inspection_requirement) = inspection_requirement(plan, inspection) else {
            return false;
        };
        if day >= inspection_day && day < inspection_day + inspection_requirement.duration_days {
            return false;
        }
        ready_day = ready_day.max(
            inspection_day
                + inspection_requirement.duration_days
                + package.return_to_service_buffer_days,
        );

        if let Some(training) = training_for_vessel(plan, &vessel.id) {
            let Some(training_day) = day_index(plan, training.day_idx) else {
                return false;
            };
            let Some(training_requirement) = training_requirement(plan, training) else {
                return false;
            };
            if day >= training_day && day < training_day + training_requirement.duration_days {
                return false;
            }
            ready_day = ready_day.max(
                training_day
                    + training_requirement.duration_days
                    + package.return_to_service_buffer_days,
            );
        }

        if day < ready_day {
            return false;
        }
    }

    true
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReadiness {
    pub week: i32,
    pub ready_overall: usize,
    pub ready_by_class: HashMap<String, usize>,
}

pub fn weekly_readiness(plan: &Plan) -> Vec<WeeklyReadiness> {
    let mut summaries = Vec::new();
    let max_week = plan.days.iter().map(|day| day.week).max().unwrap_or(0);
    for week in 1..=max_week {
        let Some(day) = plan
            .days
            .iter()
            .filter(|day| day.week == week)
            .map(|day| day.index)
            .min()
        else {
            continue;
        };
        let mut ready_by_class = HashMap::new();
        let mut ready_overall = 0;
        for vessel in &plan.vessels {
            if vessel_ready_on_day(plan, vessel, day) {
                ready_overall += 1;
                *ready_by_class
                    .entry(vessel.vessel_class.clone())
                    .or_insert(0) += 1;
            }
        }
        summaries.push(WeeklyReadiness {
            week,
            ready_overall,
            ready_by_class,
        });
    }
    summaries
}

pub fn count_weekly_readiness_floor(plan: &Plan) -> usize {
    let Some(policy) = plan.readiness_policies.first() else {
        return 0;
    };
    let mut violations = 0;
    for day in week_days(plan) {
        let mut ready_overall = 0;
        let mut ready_patrol = 0;
        let mut ready_frigate = 0;
        for vessel in &plan.vessels {
            if vessel_ready_on_day(plan, vessel, day) {
                ready_overall += 1;
                if vessel.vessel_class == "patrol_cutter" {
                    ready_patrol += 1;
                }
                if vessel.vessel_class == "frigate_like" {
                    ready_frigate += 1;
                }
            }
        }
        violations += (policy.min_ready_overall_per_week - ready_overall).max(0) as usize;
        violations += (policy.min_ready_patrol_cutter - ready_patrol).max(0) as usize;
        violations += (policy.min_ready_frigate_like - ready_frigate).max(0) as usize;
    }
    violations
}

pub fn score_weekly_readiness_floor(plan: &Plan) -> HardSoftScore {
    hard(count_weekly_readiness_floor(plan) as i64)
}

pub fn score_minimize_readiness_shortfall(plan: &Plan) -> HardSoftScore {
    soft(-(count_weekly_readiness_floor(plan) as i64) * 1_000)
}

pub fn count_ready_vessel_days(plan: &Plan) -> usize {
    let mut ready_days = 0;
    for day in week_days(plan) {
        ready_days += plan
            .vessels
            .iter()
            .filter(|vessel| vessel_ready_on_day(plan, vessel, day))
            .count();
    }
    ready_days
}

pub fn score_maximize_ready_days(plan: &Plan) -> HardSoftScore {
    soft(count_ready_vessel_days(plan) as i64)
}
