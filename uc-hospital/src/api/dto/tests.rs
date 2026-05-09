//! DTO tests that keep transport JSON and generated UI metadata aligned.

use super::*;
use serde::Deserialize;
use solverforge::ConstraintSet;
use std::fs;
use std::time::Duration;

#[derive(Deserialize)]
struct UiModel {
    entities: Vec<UiNamedEntry>,
    facts: Vec<UiNamedEntry>,
    constraints: Vec<UiConstraint>,
    views: Vec<UiView>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct UiConstraint {
    name: String,
    #[serde(rename = "type")]
    constraint_type: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct UiNamedEntry {
    name: String,
    plural: String,
    label: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct UiView {
    id: String,
    kind: String,
    label: String,
    entity: String,
    entity_plural: String,
    source_plural: String,
    variable_field: String,
    allows_unassigned: bool,
}

#[test]
fn plan_dto_returns_decode_errors_for_semantically_invalid_payloads() {
    let dto = PlanDto {
        fields: Map::new(),
        score: None,
    };

    assert!(dto.to_domain().is_err());
}

#[test]
fn runtime_telemetry_derives_stock_transport_fields() {
    let telemetry = SolverTelemetry {
        elapsed: Duration::from_millis(2_500),
        step_count: 9,
        moves_generated: 300,
        moves_evaluated: 200,
        moves_accepted: 50,
        score_calculations: 80,
        generation_time: Duration::from_millis(400),
        evaluation_time: Duration::from_millis(900),
        ..SolverTelemetry::default()
    };

    let dto = TelemetryDto::from_runtime(&telemetry);

    assert_eq!(dto.elapsed_ms, 2_500);
    assert_eq!(dto.step_count, 9);
    assert_eq!(dto.moves_generated, 300);
    assert_eq!(dto.moves_evaluated, 200);
    assert_eq!(dto.moves_accepted, 50);
    assert_eq!(dto.score_calculations, 80);
    assert_eq!(dto.generation_ms, 400);
    assert_eq!(dto.evaluation_ms, 900);
    assert_eq!(dto.moves_per_second, 80);
    assert!((dto.acceptance_rate - 0.25).abs() < f64::EPSILON);
}

#[test]
fn analyzed_constraint_names_match_ui_model() {
    let ui_model_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/static/generated/ui-model.json"
    );
    let ui_model: UiModel =
        serde_json::from_str(&fs::read_to_string(ui_model_path).unwrap()).unwrap();

    let plan = Plan::new(Vec::new(), Vec::new());
    let constraints = crate::constraints::create_constraints();
    let analysis = constraints.evaluate_detailed(&plan);
    let analyzed_constraints: Vec<String> = analysis
        .iter()
        .map(|analysis| analysis.constraint_ref.name.to_string())
        .collect();

    let ui_constraints: Vec<String> = ui_model
        .constraints
        .iter()
        .map(|constraint| constraint.name.clone())
        .collect();
    assert_eq!(analyzed_constraints, ui_constraints);
    assert_eq!(
        ui_model.entities,
        vec![UiNamedEntry {
            name: "shift".to_string(),
            plural: "shifts".to_string(),
            label: "Shifts".to_string(),
        }]
    );
    assert_eq!(
        ui_model.facts,
        vec![UiNamedEntry {
            name: "employee".to_string(),
            plural: "employees".to_string(),
            label: "Employees".to_string(),
        }]
    );
    assert_eq!(
        ui_model.views,
        vec![
            UiView {
                id: "by-location".to_string(),
                kind: "schedule-by-location".to_string(),
                label: "By location".to_string(),
                entity: "shift".to_string(),
                entity_plural: "shifts".to_string(),
                source_plural: "employees".to_string(),
                variable_field: "employeeIdx".to_string(),
                allows_unassigned: true,
            },
            UiView {
                id: "by-employee".to_string(),
                kind: "schedule-by-employee".to_string(),
                label: "By employee".to_string(),
                entity: "shift".to_string(),
                entity_plural: "shifts".to_string(),
                source_plural: "employees".to_string(),
                variable_field: "employeeIdx".to_string(),
                allows_unassigned: true,
            }
        ]
    );
}
