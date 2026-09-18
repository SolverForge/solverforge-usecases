use super::support::AppSpecContract;

struct DependencyContract<'a> {
    version: &'a str,
    path: Option<&'a str>,
}

fn dependency_contract<'a>(manifest: &'a toml::Value, name: &str) -> DependencyContract<'a> {
    let dependency = manifest
        .get("dependencies")
        .and_then(|dependencies| dependencies.get(name))
        .unwrap_or_else(|| panic!("{name} dependency should be declared"));

    if let Some(version) = dependency.as_str() {
        return DependencyContract {
            version,
            path: None,
        };
    }

    let table = dependency
        .as_table()
        .unwrap_or_else(|| panic!("{name} dependency should be a string or table"));
    DependencyContract {
        version: table
            .get("version")
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("{name} dependency should declare version")),
        path: table.get("path").and_then(toml::Value::as_str),
    }
}

fn dependency_source(name: &str, dependency: &DependencyContract<'_>) -> String {
    match dependency.path {
        Some(path) => format!("local path: {path} ({})", dependency.version),
        None => format!("crates.io: {name} {}", dependency.version),
    }
}

#[test]
fn solverforge_app_contract_parses() {
    let spec: AppSpecContract = toml::from_str(include_str!("../../../../solverforge.app.toml"))
        .expect("solverforge.app.toml should parse");
    let manifest: toml::Value =
        toml::from_str(include_str!("../../../../Cargo.toml")).expect("Cargo.toml should parse");
    let solverforge_dependency = dependency_contract(&manifest, "solverforge");
    let solverforge_ui_dependency = dependency_contract(&manifest, "solverforge-ui");

    assert_eq!(spec.app.name, "SolverForge Furnace");
    assert_eq!(spec.app.starter, "neutral-shell");
    assert_eq!(spec.app.cli_version, "2.2.2");
    assert_eq!(spec.app.shell.as_deref(), Some("web"));
    assert_eq!(
        spec.runtime.target,
        format!("solverforge {}", solverforge_dependency.version)
    );
    assert_eq!(
        spec.runtime.runtime_source,
        dependency_source("solverforge", &solverforge_dependency)
    );
    assert_eq!(
        spec.runtime.ui_source,
        dependency_source("solverforge-ui", &solverforge_ui_dependency)
    );
    assert_eq!(spec.demo.default_size, "STANDARD");
    assert_eq!(spec.demo.available_sizes, vec!["STANDARD"]);
    assert_eq!(spec.solution.name, "Plan");
    assert_eq!(spec.solution.score, "HardSoftScore");
}
