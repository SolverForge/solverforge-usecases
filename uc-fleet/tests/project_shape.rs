use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn project_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn read(path: &str) -> String {
    fs::read_to_string(project_path(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[test]
fn package_metadata_and_repository_only_packaging_stay_aligned() {
    let cargo = read("Cargo.toml");
    let app = read("solverforge.app.toml");
    let dockerfile = read("Dockerfile");
    let makefile = read("Makefile");

    assert!(cargo.contains("name = \"solverforge-fleet\""));
    assert!(cargo.contains("version = \"0.1.0\""));
    assert!(cargo.contains("rust-version = \"1.95\""));
    assert!(cargo.contains("solverforge = { version = \"0.19.4\""));
    assert!(cargo.contains("solverforge-core = \"0.19.4\""));
    assert!(cargo.contains("solverforge-ui = \"0.6.5\""));
    assert!(!cargo.contains("[patch"));

    assert!(app.contains("name = \"solverforge-fleet\""));
    assert!(app.contains("cli_version = \"2.2.2\""));
    assert!(app.contains("target = \"solverforge 0.19.4\""));
    assert!(dockerfile.contains("FROM rust:1.95-alpine AS builder"));
    assert!(dockerfile.contains("release/solverforge-fleet"));
    assert!(makefile.contains("APP_NAME := solverforge-fleet"));
}

#[test]
fn source_tree_keeps_the_scalar_fleet_model_and_retained_api() {
    let plan = read("src/domain/plan.rs");
    let work = read("src/domain/work_package.rs");
    let inspection = read("src/domain/inspection_assignment.rs");
    let training = read("src/domain/training_assignment.rs");
    let routes = read("src/api/routes.rs");
    let session_routes = read("src/api/plan_sessions/routing.rs");
    let sessions = read("src/api/plan_sessions/sessions.rs");

    assert!(plan.contains("pub struct Plan"));
    assert!(plan.contains("solver_toml = \"../../solver.toml\""));
    assert!(work.contains("pub dock_idx: Option<usize>"));
    assert!(work.contains("pub start_day_idx: Option<usize>"));
    assert!(inspection.contains("pub day_idx: Option<usize>"));
    assert!(training.contains("pub day_idx: Option<usize>"));

    for route in [
        "/jobs/{id}/snapshot",
        "/jobs/{id}/analysis",
        "/jobs/{id}/events",
    ] {
        assert!(routes.contains(route), "missing retained route {route}");
    }
    for route in [
        "/plan-sessions/{id}/repair",
        "/plan-sessions/{id}/revisions/{from_revision}/compare/{to_revision}",
        "/scenarios/perturb",
        "/solves/compare",
    ] {
        assert!(
            session_routes.contains(route),
            "missing Fleet route {route}"
        );
    }
    assert!(sessions.contains("derived_resolve_with_lineage"));
}

#[test]
fn public_dataset_and_daily_readiness_contract_are_explicit() {
    let catalog = read("src/data/data_seed/catalog.rs");
    let builders = read("src/data/data_seed/builders.rs");
    let data_seed = read("src/data/data_seed.rs");
    let readiness = read("src/constraints/support/readiness.rs");
    let constraints = read("src/constraints/mod.rs");

    assert_eq!(catalog.matches("VesselSpec { id:").count(), 24);
    assert_eq!(catalog.matches("wp!(").count(), 24);
    assert!(builders.contains("(1..=56)"));
    assert!(data_seed.contains("DemoData::Large => \"BASELINE_FULL\""));
    assert!(data_seed.contains("const DEFAULT_DEMO_DATA: DemoData = DemoData::Large"));
    assert!(readiness.contains("for day in week_days(plan)"));
    assert!(readiness.contains("min_ready_overall_per_week - ready_overall"));
    assert!(readiness.contains("min_ready_patrol_cutter - ready_patrol"));
    assert!(readiness.contains("min_ready_frigate_like - ready_frigate"));
    assert!(constraints.contains("dock_outage::constraint()"));
    assert_eq!(constraints.matches("::constraint(),").count(), 15);
}

#[test]
fn browser_shell_uses_solverforge_ui_and_fleet_modules() {
    let index = read("static/index.html");
    let app = read("static/app.js");
    let config: Value = serde_json::from_str(&read("static/sf-config.json")).expect("sf config");
    let ui_model: Value =
        serde_json::from_str(&read("static/generated/ui-model.json")).expect("ui model");

    assert!(index.contains("/sf/sf.css"));
    assert!(index.contains("/sf/sf.js"));
    for module in ["utils", "schedule", "actions", "render"] {
        assert!(
            index.contains(&format!("/fleet/{module}.js")),
            "missing Fleet script {module}"
        );
    }
    assert!(app.contains("SF.createHeader"));
    assert!(app.contains("SF.createStatusBar"));
    assert!(app.contains("SF.createFooter"));
    assert!(project_path("tests/frontend/fleet-transforms.test.js").exists());
    assert_eq!(config["title"], "SolverForge Fleet");
    assert!(ui_model.get("views").and_then(Value::as_array).is_some());
}
