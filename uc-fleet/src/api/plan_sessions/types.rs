use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::dto::{PlanDto, TelemetryDto};
use crate::constraints::support::{
    ExplanationSummary, FleetMetrics, InspectionScheduleAssignment, ResourceUtilization,
    RevisionDiff, TrainingScheduleAssignment, WeeklyReadiness, WorkPackageAssignment,
};

#[derive(Debug, Clone)]
pub struct PlanSessionRecord {
    pub active_job_id: String,
    pub baseline_plan: PlanDto,
    pub revisions: Vec<PlanSessionRevision>,
    pub repair_links: Vec<PlanSessionRepairLink>,
    pub latest_repair_job_id: Option<String>,
    pub latest_repair_mode: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct PlanSessionRevision {
    pub revision: String,
    pub job_id: String,
    pub snapshot_revision: Option<u64>,
    pub plan: PlanDto,
}

#[derive(Debug, Clone)]
pub struct PlanSessionRepairLink {
    pub from_revision: String,
    pub to_job_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanSessionRequest {
    pub domain: Option<String>,
    pub scenario: ScenarioRequest,
    #[serde(alias = "solver_profile")]
    pub solver_profile: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioRequest {
    #[serde(alias = "scenario_id")]
    pub scenario_id: String,
    pub data: Option<PlanDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanSessionResponse {
    pub plan_session_id: String,
    pub job_id: String,
    pub lifecycle_state: &'static str,
    pub checkpoint_available: bool,
    pub latest_revision: Option<u64>,
    pub status_url: String,
    pub snapshot_url: String,
    pub solver_profile: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSessionStatusResponse {
    pub plan_session_id: String,
    pub active_job_id: String,
    pub lifecycle_state: &'static str,
    pub event_sequence: u64,
    pub latest_revision: Option<u64>,
    pub checkpoint_available: bool,
    pub current_score: Option<String>,
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairRequest {
    #[serde(alias = "baseline_revision")]
    pub baseline_revision: Option<Value>,
    pub event: RepairEvent,
    #[serde(alias = "repair_policy")]
    pub repair_policy: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairEvent {
    #[serde(alias = "event_id")]
    pub event_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(alias = "occurred_at")]
    pub occurred_at: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairResponse {
    pub plan_session_id: String,
    pub baseline_revision: Option<u64>,
    pub repair_job_id: String,
    pub repair_mode: &'static str,
    pub lifecycle_state: &'static str,
    pub compare_url: String,
    pub status_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionCompareResponse {
    pub plan_session_id: String,
    pub from_revision: String,
    pub to_revision: String,
    pub metrics: FleetMetrics,
    pub diff: RevisionDiff,
    pub requires_approval: bool,
    pub approval_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadScenarioRequest {
    #[serde(alias = "scenario_id")]
    pub scenario: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadScenarioResponse {
    pub scenario_id: String,
    pub summary: ScenarioSummary,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioSummary {
    pub vessels: usize,
    pub docks: usize,
    pub work_packages: usize,
    pub planning_weeks: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSolveRequest {
    #[serde(alias = "scenario_id")]
    pub scenario_id: String,
    #[serde(alias = "objective_profile")]
    pub objective_profile: Option<String>,
    pub seed: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSolveResponse {
    pub solve_id: String,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveStatusResponse {
    pub solve_id: String,
    pub status: &'static str,
    pub best_score: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveResultResponse {
    pub solve_id: String,
    pub status: &'static str,
    pub metrics: FleetMetrics,
    pub schedule: SchedulePayload,
    pub resource_utilization: ResourceUtilization,
    pub explanations: ExplanationSummary,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulePayload {
    pub work_package_assignments: Vec<WorkPackageAssignment>,
    pub inspection_assignments: Vec<InspectionScheduleAssignment>,
    pub training_assignments: Vec<TrainingScheduleAssignment>,
    pub weekly_readiness: Vec<WeeklyReadiness>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerturbScenarioRequest {
    #[serde(alias = "base_scenario_id")]
    pub base_scenario_id: String,
    pub perturbations: Vec<RepairEvent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerturbScenarioResponse {
    pub scenario_id: String,
    pub changes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareSolvesRequest {
    #[serde(alias = "before_solve_id")]
    pub before_solve_id: String,
    #[serde(alias = "after_solve_id")]
    pub after_solve_id: String,
}
