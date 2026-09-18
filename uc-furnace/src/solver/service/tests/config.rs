use super::support::load_plan_solver_config;
use crate::data::build_demo_problem;

fn solver_toml() -> toml::Value {
    toml::from_str(include_str!("../../../../solver.toml"))
        .expect("solver.toml should parse as TOML")
}

fn table<'a>(value: &'a toml::Value, context: &str) -> &'a toml::value::Table {
    value
        .as_table()
        .unwrap_or_else(|| panic!("{context} should be a TOML table"))
}

fn array<'a>(value: &'a toml::Value, context: &str) -> &'a Vec<toml::Value> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{context} should be a TOML array"))
}

fn field<'a>(value: &'a toml::Value, key: &str, context: &str) -> &'a toml::Value {
    table(value, context)
        .get(key)
        .unwrap_or_else(|| panic!("{context}.{key} should be present"))
}

fn string_field<'a>(value: &'a toml::Value, key: &str, context: &str) -> &'a str {
    field(value, key, context)
        .as_str()
        .unwrap_or_else(|| panic!("{context}.{key} should be a string"))
}

fn integer_field(value: &toml::Value, key: &str, context: &str) -> i64 {
    field(value, key, context)
        .as_integer()
        .unwrap_or_else(|| panic!("{context}.{key} should be an integer"))
}

fn bool_field(value: &toml::Value, key: &str, context: &str) -> bool {
    field(value, key, context)
        .as_bool()
        .unwrap_or_else(|| panic!("{context}.{key} should be a bool"))
}

fn optional_integer_field(value: &toml::Value, key: &str, context: &str) -> Option<i64> {
    table(value, context)
        .get(key)
        .and_then(toml::Value::as_integer)
}

fn string_array_field(value: &toml::Value, key: &str, context: &str) -> Vec<String> {
    array(field(value, key, context), &format!("{context}.{key}"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("{context}.{key} entries should be strings"))
                .to_string()
        })
        .collect()
}

fn phases(config: &toml::Value) -> &Vec<toml::Value> {
    array(field(config, "phases", "solver"), "solver.phases")
}

#[test]
fn plan_solver_toml_matches_canonical_contract() {
    let config = solver_toml();
    assert_eq!(integer_field(&config, "random_seed", "solver"), 42);
    assert_eq!(
        integer_field(
            field(&config, "termination", "solver"),
            "seconds_spent_limit",
            "solver.termination"
        ),
        45
    );
    assert_eq!(
        integer_field(
            field(&config, "termination", "solver"),
            "unimproved_seconds_spent_limit",
            "solver.termination"
        ),
        12
    );
    assert_eq!(phases(&config).len(), 5);
    load_plan_solver_config();

    let expected_construction = [
        (
            "furnace_assignment",
            "weakest_fit_decreasing",
            Some(64),
            Some(160),
        ),
        (
            "roster_shift_assignment",
            "cheapest_insertion",
            Some(4),
            None,
        ),
        ("task_operator_assignment", "weakest_fit", None, None),
    ];
    for (idx, (group_name, heuristic_type, value_candidate_limit, group_candidate_limit)) in
        expected_construction.into_iter().enumerate()
    {
        let phase = &phases(&config)[idx];
        let context = format!("phases[{idx}]");
        assert_eq!(
            string_field(phase, "type", &context),
            "construction_heuristic"
        );
        assert_eq!(
            string_field(phase, "construction_heuristic_type", &context),
            heuristic_type
        );
        assert_eq!(
            string_field(phase, "construction_obligation", &context),
            "assign_when_candidate_exists"
        );
        assert_eq!(string_field(phase, "group_name", &context), group_name);
        assert_eq!(
            optional_integer_field(phase, "value_candidate_limit", &context),
            value_candidate_limit
        );
        assert_eq!(
            optional_integer_field(phase, "group_candidate_limit", &context),
            group_candidate_limit
        );
        assert!(!table(phase, &context).contains_key("entity_class"));
        assert!(!table(phase, &context).contains_key("variable_name"));
    }

    let hard_repair = &phases(&config)[3];
    assert_eq!(
        string_field(hard_repair, "type", "phases[3]"),
        "local_search"
    );
    assert_eq!(
        string_field(hard_repair, "local_search_type", "phases[3]"),
        "variable_neighborhood_descent"
    );
    let neighborhoods = array(
        field(hard_repair, "neighborhoods", "phases[3]"),
        "phases[3].neighborhoods",
    );
    assert_eq!(neighborhoods.len(), 5);
    assert_eq!(
        string_field(&neighborhoods[0], "type", "phases[3].neighborhoods[0]"),
        "compound_conflict_repair_move_selector"
    );
    assert_eq!(
        string_array_field(
            &neighborhoods[0],
            "constraints",
            "phases[3].neighborhoods[0]"
        ),
        [
            "minimumRest",
            "visibleShiftLimit",
            "taskOperatorOnOwningShift",
            "shiftRoleCoverage",
            "monitoringCapacity",
            "furnaceOverlap",
            "changeoverGap",
        ]
    );
    assert!(bool_field(
        &neighborhoods[0],
        "require_hard_improvement",
        "phases[3].neighborhoods[0]"
    ));

    assert_eq!(
        string_field(&neighborhoods[1], "type", "phases[3].neighborhoods[1]"),
        "cartesian_product_move_selector"
    );
    let cartesian_children = array(
        field(&neighborhoods[1], "selectors", "phases[3].neighborhoods[1]"),
        "phases[3].neighborhoods[1].selectors",
    );
    assert_eq!(cartesian_children.len(), 2);
    assert_eq!(
        string_array_field(
            &cartesian_children[0],
            "constraints",
            "phases[3].neighborhoods[1].selectors[0]"
        ),
        [
            "minimumRest",
            "visibleShiftLimit",
            "furnaceOverlap",
            "changeoverGap",
        ]
    );
    assert_eq!(
        string_field(
            &cartesian_children[1],
            "group_name",
            "phases[3].neighborhoods[1].selectors[1]"
        ),
        "task_operator_assignment"
    );

    for (idx, group_name) in [
        (2, "furnace_assignment"),
        (3, "roster_shift_assignment"),
        (4, "task_operator_assignment"),
    ] {
        let context = format!("phases[3].neighborhoods[{idx}]");
        assert_eq!(
            string_field(&neighborhoods[idx], "type", &context),
            "grouped_scalar_move_selector"
        );
        assert_eq!(
            string_field(&neighborhoods[idx], "group_name", &context),
            group_name
        );
        assert!(bool_field(
            &neighborhoods[idx],
            "require_hard_improvement",
            &context
        ));
    }

    let local_search = &phases(&config)[4];
    assert_eq!(
        string_field(local_search, "type", "phases[4]"),
        "local_search"
    );
    let acceptor = field(local_search, "acceptor", "phases[4]");
    assert_eq!(
        string_field(acceptor, "type", "phases[4].acceptor"),
        "simulated_annealing"
    );
    assert_eq!(
        string_field(acceptor, "hard_regression_policy", "phases[4].acceptor"),
        "never_accept_hard_regression"
    );
    assert_eq!(
        field(acceptor, "decay_rate", "phases[4].acceptor")
            .as_float()
            .expect("phases[4].acceptor.decay_rate should be a float"),
        0.999985
    );

    let forager = field(local_search, "forager", "phases[4]");
    assert_eq!(
        string_field(forager, "type", "phases[4].forager"),
        "accepted_count"
    );
    assert_eq!(integer_field(forager, "limit", "phases[4].forager"), 256);

    let move_selector = field(local_search, "move_selector", "phases[4]");
    assert_eq!(
        string_field(move_selector, "type", "phases[4].move_selector"),
        "union_move_selector"
    );
    assert_eq!(
        string_field(move_selector, "selection_order", "phases[4].move_selector"),
        "round_robin"
    );
    let selectors = array(
        field(move_selector, "selectors", "phases[4].move_selector"),
        "phases[4].move_selector.selectors",
    );
    assert_eq!(selectors.len(), 6);

    assert_eq!(
        string_field(
            &selectors[0],
            "type",
            "phases[4].move_selector.selectors[0]"
        ),
        "compound_conflict_repair_move_selector"
    );
    assert_eq!(
        integer_field(
            &selectors[0],
            "max_moves_per_step",
            "phases[4].move_selector.selectors[0]"
        ),
        256
    );
    assert_eq!(
        string_field(
            &selectors[2],
            "group_name",
            "phases[4].move_selector.selectors[2]"
        ),
        "furnace_schedule_task_assignment"
    );
    assert_eq!(
        integer_field(
            &selectors[2],
            "max_moves_per_step",
            "phases[4].move_selector.selectors[2]"
        ),
        512
    );
    assert_eq!(
        string_field(
            &selectors[5],
            "group_name",
            "phases[4].move_selector.selectors[5]"
        ),
        "task_operator_assignment"
    );

    for (idx, selector) in selectors.iter().enumerate() {
        let context = format!("phases[4].move_selector.selectors[{idx}]");
        assert!(!table(selector, &context).contains_key("entity_class"));
        assert!(!table(selector, &context).contains_key("variable_name"));
    }
}

#[test]
fn canonical_demo_builder_returns_single_plan() {
    let plan = build_demo_problem();
    assert!(!plan.assignments.is_empty());
    assert!(!plan.operator_shift_assignments.is_empty());
    assert!(!plan.shift_coverage_demands.is_empty());
    assert!(plan
        .assignments
        .iter()
        .any(|assignment| !assignment.compatible_assignments.is_empty()));
}
