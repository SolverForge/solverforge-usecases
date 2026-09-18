use serde::Serialize;

use crate::domain::Plan;

use super::core::{assigned_dock, assigned_start, day_index};
use super::{
    count_churn, count_deferrals, count_dock_capacity, count_dock_compatibility,
    count_dock_outages, count_overtime, count_parts_availability, count_technician_capacity,
    count_weekly_readiness_floor,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkPackageAssignment {
    pub work_package_id: String,
    pub vessel_id: String,
    pub dock_id: Option<String>,
    pub start_day: Option<i32>,
    pub end_day: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionScheduleAssignment {
    pub inspection_id: String,
    pub vessel_id: String,
    pub day: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingScheduleAssignment {
    pub training_id: String,
    pub vessel_id: String,
    pub day: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUtilization {
    pub dock_violations: usize,
    pub technician_over_capacity_units: usize,
    pub overtime_units: usize,
    pub parts_shortage_units: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplanationSummary {
    pub binding_constraints: Vec<&'static str>,
    pub notable_tradeoffs: Vec<String>,
}

pub fn work_package_assignments(plan: &Plan) -> Vec<WorkPackageAssignment> {
    plan.work_packages
        .iter()
        .map(|package| {
            let start_day = assigned_start(plan, package);
            WorkPackageAssignment {
                work_package_id: package.id.clone(),
                vessel_id: package.vessel_id.clone(),
                dock_id: assigned_dock(plan, package).map(|dock| dock.id.clone()),
                start_day,
                end_day: start_day.map(|start| start + package.duration_days - 1),
            }
        })
        .collect()
}

pub fn inspection_schedule_assignments(plan: &Plan) -> Vec<InspectionScheduleAssignment> {
    plan.inspection_assignments
        .iter()
        .map(|assignment| InspectionScheduleAssignment {
            inspection_id: assignment.requirement_id.clone(),
            vessel_id: assignment.vessel_id.clone(),
            day: day_index(plan, assignment.day_idx),
        })
        .collect()
}

pub fn training_schedule_assignments(plan: &Plan) -> Vec<TrainingScheduleAssignment> {
    plan.training_assignments
        .iter()
        .map(|assignment| TrainingScheduleAssignment {
            training_id: assignment.requirement_id.clone(),
            vessel_id: assignment.vessel_id.clone(),
            day: day_index(plan, assignment.day_idx),
        })
        .collect()
}

pub fn resource_utilization(plan: &Plan) -> ResourceUtilization {
    ResourceUtilization {
        dock_violations: count_dock_capacity(plan),
        technician_over_capacity_units: count_technician_capacity(plan),
        overtime_units: count_overtime(plan),
        parts_shortage_units: count_parts_availability(plan),
    }
}

pub fn explanation_summary(plan: &Plan) -> ExplanationSummary {
    let mut binding_constraints = Vec::new();
    if count_dock_capacity(plan) > 0 {
        binding_constraints.push("dock_capacity");
    }
    if count_dock_outages(plan) > 0 {
        binding_constraints.push("dock_outage");
    }
    if count_dock_compatibility(plan) > 0 {
        binding_constraints.push("dock_compatibility");
    }
    if count_technician_capacity(plan) > 0 {
        binding_constraints.push("technician_capacity");
    }
    if count_parts_availability(plan) > 0 {
        binding_constraints.push("parts_availability");
    }
    if count_weekly_readiness_floor(plan) > 0 {
        binding_constraints.push("weekly_readiness_floor");
    }
    if binding_constraints.is_empty() && count_overtime(plan) > 0 {
        binding_constraints.push("minimize_overtime");
    }

    let mut notable_tradeoffs = Vec::new();
    if count_deferrals(plan) > 0 {
        notable_tradeoffs.push(
            "A deferable low-priority package is held back to protect feasibility.".to_string(),
        );
    }
    if count_overtime(plan) > 0 {
        notable_tradeoffs.push("Some overtime is used to preserve readiness.".to_string());
    }
    if count_churn(plan) > 0 {
        notable_tradeoffs.push("Repair moved assignments away from the baseline plan.".to_string());
    }
    if notable_tradeoffs.is_empty() {
        notable_tradeoffs
            .push("Baseline preserves readiness without hard constraint pressure.".to_string());
    }

    ExplanationSummary {
        binding_constraints,
        notable_tradeoffs,
    }
}
