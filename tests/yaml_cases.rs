use std::{fs, path::Path};

use lifocalc::{
    app::App,
    engine::{self, EngineError},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct YamlCase {
    description: String,
    before_stack: Vec<String>,
    input: String,
    expected_after_stack: Vec<String>,
    expected_after_input: String,
    expected_status: Option<String>,
}

fn run_yaml_case(path: &Path) -> datatest_stable::Result<()> {
    let case_file = path.display().to_string();
    let case = load_case(path)?;
    let case_name = format!("{case_file} ({})", case.description);

    let mut app = App::new();
    let before_stack = parse_stack(&case.before_stack)
        .map_err(|error| format!("{case_name}: invalid before_stack: {error}"))?;
    app.set_stack(before_stack);
    app.set_input(case.input.clone());

    app.submit_input();

    if app.stack_as_strings() != case.expected_after_stack {
        return Err(format!(
            "{case_name}: stack mismatch, got {:?}, expected {:?}",
            app.stack_as_strings(),
            case.expected_after_stack
        )
        .into());
    }

    if app.input() != case.expected_after_input {
        return Err(format!(
            "{case_name}: input mismatch, got {:?}, expected {:?}",
            app.input(),
            case.expected_after_input
        )
        .into());
    }

    match &case.expected_status {
        Some(expected_status) if app.status() != Some(expected_status.as_str()) => Err(format!(
            "{case_name}: status mismatch, got {:?}, expected {:?}",
            app.status(),
            expected_status
        )
        .into()),
        Some(_) => Ok(()),
        None if app.status().is_some() => {
            Err(format!("{case_name}: expected no status, got {:?}", app.status()).into())
        }
        None => Ok(()),
    }
}

fn load_case(path: &Path) -> datatest_stable::Result<YamlCase> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    serde_yaml::from_str(&raw)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()).into())
}

fn parse_stack(values: &[String]) -> Result<Vec<f64>, EngineError> {
    values
        .iter()
        .map(|value| engine::parse_number(value))
        .collect()
}

datatest_stable::harness! {
    { test = run_yaml_case, root = "tests/cases", pattern = r"^\d+\.ya?ml$" },
}
