//! Keeps the browser metadata aligned with the Rust scoring model.
//!
//! `static/generated/ui-model.json` is written by `solverforge generate` and
//! `solverforge destroy`; it lists collections and constraint ids.
//! `static/app/app-model.json` is owned by this app; it gives every constraint
//! id a readable label and a hard/soft type, and declares the schedule views.

use serde::Deserialize;
use solverforge::ConstraintSet;
use solverforge_hospital::constraints::create_constraints;
use solverforge_hospital::domain::Plan;
use std::collections::BTreeSet;
use std::fs;

#[derive(Deserialize)]
struct GeneratedUiModel {
    entities: Vec<GeneratedCollection>,
    facts: Vec<GeneratedCollection>,
    constraints: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct GeneratedCollection {
    name: String,
    plural: String,
}

#[derive(Deserialize)]
struct AppModel {
    constraints: Vec<CatalogEntry>,
    views: Vec<AppView>,
}

#[derive(Deserialize)]
struct CatalogEntry {
    id: String,
    label: String,
    #[serde(rename = "type")]
    constraint_type: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AppView {
    id: String,
    kind: String,
    entity_plural: String,
    source_plural: String,
    variable_field: String,
}

fn read_json<T: for<'de> Deserialize<'de>>(relative_path: &str) -> T {
    let path = format!("{}/{relative_path}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
}

#[test]
fn generated_ui_model_lists_every_scored_constraint_id() {
    let ui_model: GeneratedUiModel = read_json("static/generated/ui-model.json");
    let constraints = create_constraints();
    let plan = Plan::new(Vec::new(), Vec::new());
    let analysis = constraints.evaluate_detailed(&plan);
    let analyzed: BTreeSet<String> = analysis
        .iter()
        .map(|entry| entry.constraint_ref.name.to_string())
        .collect();

    assert_eq!(analyzed, ui_model.constraints.into_iter().collect());
    assert_eq!(
        ui_model.entities,
        vec![GeneratedCollection {
            name: "shift".to_string(),
            plural: "shifts".to_string(),
        }]
    );
    assert_eq!(
        ui_model.facts,
        vec![GeneratedCollection {
            name: "employee".to_string(),
            plural: "employees".to_string(),
        }]
    );
}

#[test]
fn app_model_labels_constraints_in_analysis_order_with_their_score_level() {
    let app_model: AppModel = read_json("static/app/app-model.json");
    let constraints = create_constraints();
    let plan = Plan::new(Vec::new(), Vec::new());
    let analysis = constraints.evaluate_detailed(&plan);

    assert_eq!(app_model.constraints.len(), analysis.len());
    for (entry, analyzed) in app_model.constraints.iter().zip(&analysis) {
        assert_eq!(entry.id, analyzed.constraint_ref.name);
        assert!(!entry.label.trim().is_empty(), "{} needs a label", entry.id);
        let expected_type = if analyzed.is_hard { "hard" } else { "soft" };
        assert_eq!(entry.constraint_type, expected_type, "{}", entry.id);
    }
}

#[test]
fn app_model_declares_both_schedule_views_over_shift_assignments() {
    let app_model: AppModel = read_json("static/app/app-model.json");
    let view = |id: &str, kind: &str| AppView {
        id: id.to_string(),
        kind: kind.to_string(),
        entity_plural: "shifts".to_string(),
        source_plural: "employees".to_string(),
        variable_field: "employeeIdx".to_string(),
    };

    assert_eq!(
        app_model.views,
        vec![
            view("by-location", "schedule-by-location"),
            view("by-employee", "schedule-by-employee"),
        ]
    );
}
