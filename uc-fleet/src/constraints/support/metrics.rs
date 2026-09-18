use std::collections::HashMap;

use serde::Serialize;

use crate::domain::Plan;

use super::core::{assigned_dock, assigned_start};
use super::{
    count_churn, count_deferrals, count_overtime, count_parts_availability,
    count_ready_vessel_days, count_technician_capacity, count_weekly_readiness_floor,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FleetMetrics {
    pub ready_vessel_days: usize,
    pub readiness_shortfall_units: usize,
    pub overtime_units: usize,
    pub deferred_packages: usize,
    pub churn_units: usize,
}

pub fn fleet_metrics(plan: &Plan) -> FleetMetrics {
    FleetMetrics {
        ready_vessel_days: count_ready_vessel_days(plan),
        readiness_shortfall_units: count_weekly_readiness_floor(plan),
        overtime_units: count_overtime(plan),
        deferred_packages: count_deferrals(plan),
        churn_units: count_churn(plan),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionDiff {
    pub unchanged_assignments: usize,
    pub moved_assignments: usize,
    pub deferred_assignments: usize,
    pub readiness_delta: i64,
    pub affected_constraints: Vec<&'static str>,
    pub human_summary: String,
}

pub fn revision_diff(before: &Plan, after: &Plan) -> RevisionDiff {
    let mut baseline: HashMap<&str, (Option<i32>, Option<&str>)> = HashMap::new();
    for package in &before.work_packages {
        baseline.insert(
            package.id.as_str(),
            (
                assigned_start(before, package),
                assigned_dock(before, package).map(|dock| dock.id.as_str()),
            ),
        );
    }

    let mut unchanged = 0;
    let mut moved = 0;
    let mut deferred = 0;
    for package in &after.work_packages {
        let current = (
            assigned_start(after, package),
            assigned_dock(after, package).map(|dock| dock.id.as_str()),
        );
        if current.0.is_none() || current.1.is_none() {
            deferred += 1;
            continue;
        }
        if baseline
            .get(package.id.as_str())
            .is_some_and(|entry| *entry == current)
        {
            unchanged += 1;
        } else {
            moved += 1;
        }
    }

    let readiness_delta =
        count_ready_vessel_days(after) as i64 - count_ready_vessel_days(before) as i64;
    let mut affected_constraints = Vec::new();
    if count_technician_capacity(after) > count_technician_capacity(before) {
        affected_constraints.push("technician_capacity");
    }
    if count_parts_availability(after) > count_parts_availability(before) {
        affected_constraints.push("parts_availability");
    }
    if count_weekly_readiness_floor(after) > count_weekly_readiness_floor(before) {
        affected_constraints.push("weekly_readiness_floor");
    }
    if affected_constraints.is_empty() && moved > 0 {
        affected_constraints.push("schedule_churn");
    }

    RevisionDiff {
        unchanged_assignments: unchanged,
        moved_assignments: moved,
        deferred_assignments: deferred,
        readiness_delta,
        affected_constraints,
        human_summary: format!(
            "{moved} assignments moved and {deferred} deferred under derived repair lineage"
        ),
    }
}
