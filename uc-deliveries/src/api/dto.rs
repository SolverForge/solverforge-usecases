use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use solverforge::{HardSoftScore, SolverSnapshot, SolverSnapshotAnalysis, SolverStatus};

use crate::domain::{DeliveryInsertionCandidate, Plan, RoutesSnapshot};

mod runtime;

pub use runtime::{lifecycle_state_label, terminal_reason_label, TelemetryDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDto {
    #[serde(flatten)]
    pub fields: Map<String, Value>,
    #[serde(default)]
    pub score: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintAnalysisDto {
    pub name: String,
    pub weight: String,
    pub score: String,
    pub match_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResponse {
    pub score: String,
    pub constraints: Vec<ConstraintAnalysisDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummaryDto {
    pub id: String,
    pub job_id: String,
    pub lifecycle_state: &'static str,
    pub terminal_reason: Option<&'static str>,
    pub checkpoint_available: bool,
    pub event_sequence: u64,
    pub snapshot_revision: Option<u64>,
    pub current_score: Option<String>,
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSnapshotDto {
    pub id: String,
    pub job_id: String,
    pub snapshot_revision: u64,
    pub lifecycle_state: &'static str,
    pub terminal_reason: Option<&'static str>,
    pub current_score: Option<String>,
    pub best_score: Option<String>,
    pub telemetry: TelemetryDto,
    pub solution: PlanDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobAnalysisDto {
    pub id: String,
    pub job_id: String,
    pub snapshot_revision: u64,
    pub lifecycle_state: &'static str,
    pub terminal_reason: Option<&'static str>,
    pub analysis: AnalyzeResponse,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRoutesDto {
    pub id: String,
    pub job_id: String,
    pub snapshot_revision: u64,
    #[serde(flatten)]
    pub routes: RoutesSnapshot,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryInsertionRequestDto {
    pub plan: PlanDto,
    pub delivery_id: usize,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryInsertionCandidateDto {
    pub vehicle_id: usize,
    pub vehicle_name: String,
    pub insert_index: usize,
    pub hard_score: i64,
    pub soft_score: i64,
    pub score: String,
    pub delta_hard: i64,
    pub delta_soft: i64,
    pub preview_plan: PlanDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryInsertionResponseDto {
    pub delivery_id: usize,
    pub candidates: Vec<DeliveryInsertionCandidateDto>,
}

impl PlanDto {
    pub fn from_plan(plan: &Plan) -> Self {
        let plan = plan.refreshed_for_transport();
        let score = plan.score.as_ref().map(ToString::to_string);
        let mut fields = match serde_json::to_value(plan).expect("failed to serialize plan") {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        fields.remove("score");

        Self { fields, score }
    }

    pub fn to_domain(&self) -> Result<Plan, serde_json::Error> {
        let mut fields = self.fields.clone();
        fields.insert("score".to_string(), Value::Null);
        let mut plan: Plan = serde_json::from_value(Value::Object(fields))?;
        plan.normalize();
        Ok(plan)
    }
}

impl JobSummaryDto {
    pub fn from_status(job_id: usize, status: &SolverStatus<HardSoftScore>) -> Self {
        Self {
            id: job_id.to_string(),
            job_id: job_id.to_string(),
            lifecycle_state: lifecycle_state_label(status.lifecycle_state),
            terminal_reason: status.terminal_reason.map(terminal_reason_label),
            checkpoint_available: status.checkpoint_available,
            event_sequence: status.event_sequence,
            snapshot_revision: status.latest_snapshot_revision,
            current_score: status.current_score.map(|score| score.to_string()),
            best_score: status.best_score.map(|score| score.to_string()),
            telemetry: TelemetryDto::from_runtime(&status.telemetry),
        }
    }
}

impl JobSnapshotDto {
    pub fn from_snapshot(snapshot: &SolverSnapshot<Plan>) -> Self {
        Self {
            id: snapshot.job_id.to_string(),
            job_id: snapshot.job_id.to_string(),
            snapshot_revision: snapshot.snapshot_revision,
            lifecycle_state: lifecycle_state_label(snapshot.lifecycle_state),
            terminal_reason: snapshot.terminal_reason.map(terminal_reason_label),
            current_score: snapshot.current_score.map(|score| score.to_string()),
            best_score: snapshot.best_score.map(|score| score.to_string()),
            telemetry: TelemetryDto::from_runtime(&snapshot.telemetry),
            solution: PlanDto::from_plan(&snapshot.solution),
        }
    }
}

impl JobAnalysisDto {
    pub fn from_snapshot_analysis(
        snapshot: &SolverSnapshotAnalysis<HardSoftScore>,
        analysis: AnalyzeResponse,
    ) -> Self {
        Self {
            id: snapshot.job_id.to_string(),
            job_id: snapshot.job_id.to_string(),
            snapshot_revision: snapshot.snapshot_revision,
            lifecycle_state: lifecycle_state_label(snapshot.lifecycle_state),
            terminal_reason: snapshot.terminal_reason.map(terminal_reason_label),
            analysis,
        }
    }
}

impl JobRoutesDto {
    pub fn new(job_id: usize, snapshot_revision: u64, routes: RoutesSnapshot) -> Self {
        Self {
            id: job_id.to_string(),
            job_id: job_id.to_string(),
            snapshot_revision,
            routes,
        }
    }
}

impl DeliveryInsertionCandidateDto {
    pub fn from_candidate(candidate: DeliveryInsertionCandidate) -> Self {
        Self {
            vehicle_id: candidate.vehicle_id,
            vehicle_name: candidate.vehicle_name,
            insert_index: candidate.insert_index,
            hard_score: candidate.hard_score,
            soft_score: candidate.soft_score,
            score: HardSoftScore::of(candidate.hard_score, candidate.soft_score).to_string(),
            delta_hard: candidate.delta_hard,
            delta_soft: candidate.delta_soft,
            preview_plan: PlanDto::from_plan(&candidate.preview_plan),
        }
    }
}

pub fn analysis_response(analysis: &solverforge::ScoreAnalysis<HardSoftScore>) -> AnalyzeResponse {
    AnalyzeResponse {
        score: analysis.score.to_string(),
        constraints: analysis
            .constraints
            .iter()
            .map(|constraint| ConstraintAnalysisDto {
                name: constraint.name.clone(),
                weight: constraint.weight.to_string(),
                score: constraint.score.to_string(),
                match_count: constraint.match_count,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests;
