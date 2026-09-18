mod core;
mod hard_constraints;
mod metrics;
mod objectives;
mod readiness;
mod repairs;
mod schedule;

#[allow(unused_imports)]
pub use core::{assigned_dock, assigned_start, day_index, FleetConstraint};
pub use hard_constraints::{
    count_dock_capacity, count_dock_compatibility, count_dock_outages,
    count_inspection_training_precedence, count_parts_availability, count_required_assignments,
    count_technician_capacity, count_work_window, score_dock_capacity, score_dock_compatibility,
    score_dock_outages, score_inspection_training_precedence, score_parts_availability,
    score_required_assignments, score_technician_capacity, score_work_window,
};
pub use metrics::{fleet_metrics, revision_diff, FleetMetrics, RevisionDiff};
pub use objectives::{
    count_churn, count_deferrals, count_inspection_lateness, count_overtime, score_minimize_churn,
    score_minimize_deferral, score_minimize_inspection_lateness, score_minimize_overtime,
};
pub use readiness::{
    count_ready_vessel_days, count_weekly_readiness_floor, score_maximize_ready_days,
    score_minimize_readiness_shortfall, score_weekly_readiness_floor, weekly_readiness,
    WeeklyReadiness,
};
pub use repairs::{
    apply_dock_outage, apply_parts_delay, apply_readiness_floor, apply_technician_shortage,
    apply_technician_shortage_window, seed_technician_shortage_repair,
};
pub use schedule::{
    explanation_summary, inspection_schedule_assignments, resource_utilization,
    training_schedule_assignments, work_package_assignments, ExplanationSummary,
    InspectionScheduleAssignment, ResourceUtilization, TrainingScheduleAssignment,
    WorkPackageAssignment,
};

#[cfg(test)]
mod tests;
