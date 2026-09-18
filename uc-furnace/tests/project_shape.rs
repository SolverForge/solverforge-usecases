use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn project_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn stock_static_shell_files_exist_and_are_wired_like_a_cli_app() {
    let index_html =
        fs::read_to_string(project_path("static/index.html")).expect("static/index.html exists");
    let app_js = fs::read_to_string(project_path("static/app.js")).expect("static/app.js exists");
    let runtime_js =
        fs::read_to_string(project_path("static/app/runtime.js")).expect("runtime.js exists");
    let lifecycle_js =
        fs::read_to_string(project_path("static/app/lifecycle.js")).expect("lifecycle.js exists");
    let sf_config =
        fs::read_to_string(project_path("static/sf-config.json")).expect("sf-config exists");
    let ui_model = fs::read_to_string(project_path("static/generated/ui-model.json"))
        .expect("ui-model.json exists");

    assert!(index_html.contains("/sf/sf.css"));
    assert!(index_html.contains("/sf/sf.js"));
    assert!(index_html.contains("type=\"module\""));
    assert!(index_html.contains("/app.js"));

    assert!(app_js.contains("./app/runtime.js"));
    assert!(runtime_js.contains("/sf-config.json"));
    assert!(runtime_js.contains("/generated/ui-model.json"));
    assert!(runtime_js.contains("SF.createHeader"));
    assert!(runtime_js.contains("SF.createStatusBar"));
    assert!(runtime_js.contains("SF.createSolver"));
    assert!(runtime_js.contains("SF.createApiGuide"));
    assert!(runtime_js.contains("SF.createFooter"));
    assert!(runtime_js.contains("/demo-data/STANDARD"));
    assert!(!runtime_js.contains("installDemoDataSelector"));
    assert!(runtime_js.contains("renderAll(ctx, data)"));
    let cleanup_pos = runtime_js
        .find("cleanupTerminalJob(ctx)")
        .expect("solve path should clean up terminal retained jobs first");
    let clone_pos = runtime_js
        .find("clonePlan(ctx.currentPlan)")
        .expect("solve path should use the current canonical plan");
    assert!(cleanup_pos < clone_pos);
    assert!(!lifecycle_js.contains("resolvePlanForSolve"));
    assert!(!lifecycle_js.contains("fetch('/demo-data/STANDARD')"));
    assert!(project_path("static/app/schedule.js").exists());
    assert!(project_path("static/app/render.js").exists());

    assert!(sf_config.contains("\"title\""));
    assert!(sf_config.contains("\"subtitle\""));

    let ui_model: Value = serde_json::from_str(&ui_model).expect("ui-model.json should be json");
    assert!(ui_model.get("views").and_then(Value::as_array).is_some());
    assert!(ui_model
        .get("constraints")
        .and_then(Value::as_array)
        .is_some());
    assert!(ui_model.get("entities").and_then(Value::as_array).is_some());
    assert!(ui_model.get("facts").and_then(Value::as_array).is_some());
}

#[test]
fn source_tree_keeps_the_canonical_cli_entry_files() {
    let plan_rs = fs::read_to_string(project_path("src/domain/plan.rs")).expect("plan.rs exists");
    let main_rs = fs::read_to_string(project_path("src/main.rs")).expect("main.rs exists");
    let cargo_toml =
        fs::read_to_string(project_path("Cargo.toml")).expect("Cargo.toml should exist");

    assert!(project_path("src/api/routes.rs").exists());
    assert!(project_path("src/api/sse.rs").exists());
    assert!(project_path("src/solver/service.rs").exists());
    assert!(project_path("src/data/data_seed.rs").exists());
    assert!(project_path("solverforge.app.toml").exists());
    assert!(project_path("solver.toml").exists());
    assert!(!project_path("src/bin/server.rs").exists());

    assert!(plan_rs.contains("#[planning_solution("));
    assert!(plan_rs.contains("pub struct Plan"));
    assert!(main_rs.contains(".merge(solverforge_ui::routes())"));
    assert!(main_rs.contains(".fallback_service(ServeDir::new(\"static\"))"));

    assert!(cargo_toml.contains("rust-version = \"1.95\""));
    assert!(cargo_toml.contains("solverforge = { version = \"0.19.4\""));
    assert!(cargo_toml.contains("solverforge-ui = \"0.6.5\""));
    assert!(!cargo_toml.contains("[patch"));

    let source_root = fs::read_to_string(project_path("src/domain/mod.rs")).expect("domain mod");
    assert!(!source_root.contains("FurnacePlan"));
    assert!(!source_root.contains("WorkforcePlan"));
    assert!(!source_root.contains("stage"));
    assert!(!project_path("src/domain/stage").exists());
}

#[test]
fn public_demo_path_has_no_seed_or_repair_backdoors() {
    let base_data = fs::read_to_string(project_path("src/data/data_seed/base_data.rs"))
        .expect("base_data exists");
    let problem =
        fs::read_to_string(project_path("src/data/data_seed/problem.rs")).expect("problem exists");
    let furnace_seed = fs::read_to_string(project_path("src/data/data_seed/furnace_seed.rs"))
        .expect("furnace_seed exists");
    let staffing_hard =
        fs::read_to_string(project_path("src/constraints/staffing_hard.rs")).expect("staffing");
    let plan_rs = fs::read_to_string(project_path("src/domain/plan.rs")).expect("plan");
    let lifecycle =
        fs::read_to_string(project_path("static/app/lifecycle.js")).expect("lifecycle exists");
    let schedule =
        fs::read_to_string(project_path("static/app/schedule.js")).expect("schedule exists");
    let solver_toml = fs::read_to_string(project_path("solver.toml")).expect("solver.toml exists");

    assert!(!base_data.contains("seed_initial_assignments"));
    assert!(!problem.contains("seed_workforce_assignments"));
    assert!(!furnace_seed.contains("repair_"));
    assert!(!furnace_seed.contains("seed_initial_assignments"));
    assert!(!project_path("src/data/data_seed/workforce_seed.rs").exists());
    assert!(staffing_hard.contains("ManualTaskBookingEntries"));
    assert!(!plan_rs.contains("self.shifts.iter().find"));
    assert!(lifecycle.contains("cleanupTerminalJob"));
    assert!(schedule.contains("buildUnassignedSection(unassignedOrders)"));
    assert!(fs::read_to_string(project_path("static/app/runtime.js"))
        .expect("runtime exists")
        .contains("clonePlan(ctx.currentPlan)"));
    assert!(fs::read_to_string(project_path("static/app/api.js"))
        .expect("api guide exists")
        .contains("/demo-data/STANDARD"));
    assert!(!lifecycle.contains("fetch('/demo-data/STANDARD')"));
    assert!(!schedule.contains("fake"));
    assert!(solver_toml.contains("selection_order = \"round_robin\""));
    assert!(solver_toml.contains("hard_regression_policy = \"never_accept_hard_regression\""));
    assert!(solver_toml.contains("construction_obligation = \"assign_when_candidate_exists\""));
    assert!(solver_toml.contains("seconds_spent_limit = 45"));
    assert!(solver_toml.contains("unimproved_seconds_spent_limit = 12"));
    assert!(solver_toml.contains("group_name = \"furnace_assignment\""));
    assert!(solver_toml.contains("group_name = \"roster_shift_assignment\""));
    assert!(solver_toml.contains("group_name = \"task_operator_assignment\""));
    assert!(!solver_toml.contains("change_move_selector"));
    assert!(!solver_toml.contains("swap_move_selector"));
    assert!(!solver_toml.contains("ruin_recreate_move_selector"));

    let config: toml::Value = toml::from_str(&solver_toml).expect("solver.toml parses");
    let phases = config
        .get("phases")
        .and_then(toml::Value::as_array)
        .expect("solver.toml phases should be an array");
    assert_eq!(
        phases[3]
            .get("local_search_type")
            .and_then(toml::Value::as_str),
        Some("variable_neighborhood_descent")
    );
    let expected = [
        ("furnace_assignment", "weakest_fit_decreasing"),
        ("roster_shift_assignment", "cheapest_insertion"),
        ("task_operator_assignment", "weakest_fit"),
    ];
    for (idx, (group_name, heuristic_type)) in expected.into_iter().enumerate() {
        let phase = phases[idx]
            .as_table()
            .unwrap_or_else(|| panic!("phase {idx} should be a table"));
        assert_eq!(
            phase.get("type").and_then(toml::Value::as_str),
            Some("construction_heuristic")
        );
        assert_eq!(
            phase
                .get("construction_heuristic_type")
                .and_then(toml::Value::as_str),
            Some(heuristic_type)
        );
        assert_eq!(
            phase.get("group_name").and_then(toml::Value::as_str),
            Some(group_name)
        );
        assert!(!phase.contains_key("entity_class"));
        assert!(!phase.contains_key("variable_name"));
    }
}
