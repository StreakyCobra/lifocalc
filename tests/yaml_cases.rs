use std::{fs, path::PathBuf};

use lifocalc::{
    app::App,
    engine::{self, EngineError},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct YamlCase {
    before_stack: Vec<String>,
    input: String,
    expected_after_stack: Vec<String>,
    expected_after_input: String,
    expected_status: Option<String>,
}

#[test]
fn yaml_cases() {
    let mut case_paths = collect_case_paths();
    case_paths.sort();

    assert!(
        !case_paths.is_empty(),
        "expected at least one file in tests/cases"
    );

    for case_path in case_paths {
        let case_name = case_path
            .strip_prefix(manifest_dir())
            .unwrap_or(&case_path)
            .display()
            .to_string();

        let case = load_case(&case_path);
        run_case(&case_name, &case);
    }
}

fn run_case(case_name: &str, case: &YamlCase) {
    let mut app = App::new();
    app.set_stack(parse_stack(&case.before_stack).unwrap_or_else(|error| {
        panic!("{case_name}: invalid before_stack: {error}");
    }));
    app.set_input(case.input.clone());

    app.submit_input();

    assert_eq!(
        app.stack_as_strings(),
        case.expected_after_stack,
        "{case_name}: stack mismatch"
    );
    assert_eq!(
        app.input(),
        case.expected_after_input,
        "{case_name}: input mismatch"
    );

    match &case.expected_status {
        Some(expected_status) => assert_eq!(
            app.status(),
            Some(expected_status.as_str()),
            "{case_name}: status mismatch"
        ),
        None => assert_eq!(app.status(), None, "{case_name}: expected no status"),
    }
}

fn load_case(path: &PathBuf) -> YamlCase {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    serde_yaml::from_str(&raw)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn collect_case_paths() -> Vec<PathBuf> {
    let cases_dir = manifest_dir().join("tests/cases");
    let entries = fs::read_dir(&cases_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", cases_dir.display()));

    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml"))
        .collect()
}

fn parse_stack(values: &[String]) -> Result<Vec<f64>, EngineError> {
    values.iter().map(|value| engine::parse_number(value)).collect()
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
