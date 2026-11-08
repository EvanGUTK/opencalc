//! Independent curriculum fixtures exercised by the packaged application's self-test.
//! These calculation checks do not establish native OS or assistive-hardware usability.

use crate::{EqualApp, Settings, math};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct Fixture {
    pub cases: Vec<Case>,
    #[serde(default)]
    pub sources: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct Case {
    pub id: String,
    pub level: usize,
    pub stage: String,
    pub subject: String,
    pub kind: String,
    #[serde(default)]
    pub input: String,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default = "default_degrees")]
    pub degrees: bool,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub operation: String,
    #[serde(default)]
    pub matrix_a: MatrixInput,
    #[serde(default)]
    pub matrix_b: MatrixInput,
    #[serde(default)]
    pub tool_id: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    pub expected: Expected,
    #[serde(default)]
    pub source_ids: Vec<String>,
}

fn default_degrees() -> bool {
    true
}

#[derive(Default, Deserialize)]
#[serde(untagged)]
pub(crate) enum MatrixInput {
    Text(String),
    Rows(Vec<Vec<f64>>),
    #[default]
    Empty,
}

impl MatrixInput {
    pub fn text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Rows(rows) => rows
                .iter()
                .map(|row| row.iter().map(f64::to_string).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Empty => "0".into(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Expected {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub value: serde_json::Value,
    #[serde(default)]
    pub re: f64,
    #[serde(default)]
    pub im: f64,
    #[serde(default)]
    pub values: Vec<f64>,
    #[serde(default)]
    pub abs_tolerance: f64,
    #[serde(default)]
    pub rel_tolerance: f64,
    #[serde(default)]
    pub code_class: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum Actual {
    Real {
        value: f64,
        display: String,
    },
    Complex {
        re: f64,
        im: f64,
        display: String,
    },
    Matrix {
        value: Vec<Vec<f64>>,
    },
    Science {
        values: Vec<f64>,
        units: Vec<String>,
    },
    Error {
        message: String,
    },
    Unsupported {
        message: String,
    },
}

#[derive(Serialize)]
pub(crate) struct Check {
    pub case_id: String,
    pub stage: String,
    pub subject: String,
    pub kind: String,
    pub passed: bool,
    pub expected: Expected,
    pub actual: Actual,
    pub reason: String,
    pub source_ids: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct Report {
    pub schema: u32,
    pub application_version: &'static str,
    pub scope: &'static str,
    pub passed: usize,
    pub failed: usize,
    pub rows: Vec<Check>,
    pub sources: serde_json::Value,
}

pub(crate) fn fixture() -> Result<Fixture, String> {
    let fixture: Fixture = serde_json::from_str(include_str!("../tests/fixtures/curriculum.json"))
        .map_err(|error| format!("The compiled curriculum fixture is invalid: {error}"))?;
    if fixture.cases.is_empty() {
        return Err("The compiled curriculum fixture contains no cases.".into());
    }
    Ok(fixture)
}

pub(crate) fn run() -> Report {
    let mut report = Report {
        schema: 1,
        application_version: env!("CARGO_PKG_VERSION"),
        scope: "production calculation checks; native OS and assistive hardware not tested",
        passed: 0,
        failed: 0,
        rows: Vec::new(),
        sources: serde_json::Value::Null,
    };
    match fixture() {
        Ok(fixture) => {
            report.sources = fixture.sources;
            for case in fixture.cases {
                let actual = evaluate(&case);
                let (passed, reason) = compare(&case.expected, &actual);
                report.passed += usize::from(passed);
                report.failed += usize::from(!passed);
                report.rows.push(Check {
                    case_id: case.id,
                    stage: case.stage,
                    subject: case.subject,
                    kind: case.kind,
                    passed,
                    expected: case.expected,
                    actual,
                    reason,
                    source_ids: case.source_ids,
                });
            }
        }
        Err(error) => {
            report.failed = 1;
            report.rows.push(Check {
                case_id: "fixture-load".into(),
                stage: "infrastructure".into(),
                subject: "fixture validation".into(),
                kind: "infrastructure".into(),
                passed: false,
                expected: Expected {
                    kind: "valid_fixture".into(),
                    value: serde_json::Value::Null,
                    re: 0.0,
                    im: 0.0,
                    values: Vec::new(),
                    abs_tolerance: 0.0,
                    rel_tolerance: 0.0,
                    code_class: String::new(),
                },
                actual: Actual::Unsupported {
                    message: error.clone(),
                },
                reason: error,
                source_ids: Vec::new(),
            });
        }
    }
    report
}

fn evaluate(case: &Case) -> Actual {
    let mut app = EqualApp::with_settings(
        Settings {
            level: case.level,
            degrees: case.degrees,
            precision: 10,
            ..Settings::default()
        },
        None,
    );
    match case.kind.as_str() {
        "calculator" => {
            app.device.complex_mode = case.expected.kind == "complex";
            for input in case
                .steps
                .iter()
                .map(String::as_str)
                .chain(if case.steps.is_empty() {
                    Some(case.input.as_str())
                } else {
                    None
                })
            {
                app.expression = input.into();
                app.evaluate();
                if app.error {
                    return Actual::Error {
                        message: app.result,
                    };
                }
            }
            calculator_actual(&app)
        }
        "graph" => {
            app.graph_a.clone_from(&case.input);
            match app.graph_value(0, case.x) {
                Ok(value) => Actual::Real {
                    value,
                    display: math::format(value, 10),
                },
                Err(message) => Actual::Error { message },
            }
        }
        "matrix" => {
            let result = math::matrix(&case.matrix_a.text()).and_then(|a| {
                math::matrix(&case.matrix_b.text())
                    .and_then(|b| math::matrix_op(&a, &b, &case.operation))
            });
            match result {
                Ok(value) => Actual::Matrix { value },
                Err(message) => Actual::Error { message },
            }
        }
        "science" => match crate::science::evaluate(&case.tool_id, &case.inputs, false) {
            Ok(report) => Actual::Science {
                values: report.values.iter().map(|value| value.value).collect(),
                units: report
                    .values
                    .iter()
                    .map(|value| value.unit.to_string())
                    .collect(),
            },
            Err(message) => Actual::Error { message },
        },
        kind => Actual::Unsupported {
            message: format!("Unsupported fixture kind: {kind}"),
        },
    }
}

pub(crate) fn calculator_actual(app: &EqualApp) -> Actual {
    if app.error {
        return Actual::Error {
            message: app.result.clone(),
        };
    }
    let value = app.device.complex_answer();
    if value.im == 0.0 {
        Actual::Real {
            value: app.answer,
            display: app.result.clone(),
        }
    } else {
        Actual::Complex {
            re: value.re,
            im: value.im,
            display: app.result.clone(),
        }
    }
}

fn close(expected: f64, actual: f64, specification: &Expected) -> bool {
    expected.is_finite()
        && actual.is_finite()
        && (expected - actual).abs()
            <= specification.abs_tolerance + specification.rel_tolerance * expected.abs()
}

pub(crate) fn compare(expected: &Expected, actual: &Actual) -> (bool, String) {
    let matches = match (expected.kind.as_str(), actual) {
        ("error", Actual::Error { message }) => !message.trim().is_empty(),
        ("real", Actual::Real { value, display }) => expected.value.as_f64().is_some_and(|want| {
            close(want, *value, expected)
                && display
                    .parse::<f64>()
                    .is_ok_and(|shown| shown.is_finite() && (want == 0.0 || shown != 0.0))
        }),
        ("real", Actual::Science { values, .. }) => {
            if expected.values.is_empty() {
                expected
                    .value
                    .as_f64()
                    .zip(values.first())
                    .is_some_and(|(want, got)| close(want, *got, expected))
            } else {
                expected.values.len() <= values.len()
                    && expected
                        .values
                        .iter()
                        .zip(values)
                        .all(|(want, got)| close(*want, *got, expected))
            }
        }
        ("complex", Actual::Complex { re, im, .. }) => {
            close(expected.re, *re, expected) && close(expected.im, *im, expected)
        }
        ("complex", Actual::Real { value, .. }) => {
            close(expected.re, *value, expected) && close(expected.im, 0.0, expected)
        }
        ("matrix", Actual::Matrix { value }) => {
            serde_json::from_value::<Vec<Vec<f64>>>(expected.value.clone()).is_ok_and(|want| {
                want.len() == value.len()
                    && want.iter().zip(value).all(|(want, got)| {
                        want.len() == got.len()
                            && want
                                .iter()
                                .zip(got)
                                .all(|(want, got)| close(*want, *got, expected))
                    })
            })
        }
        _ => false,
    };
    let reason = if matches {
        if expected.kind == "error" {
            format!(
                "Invalid input rejected (fixture class: {}); error wording is not an oracle.",
                expected.code_class
            )
        } else {
            "Matches the independent expected value within the fixture's absolute and relative tolerances.".into()
        }
    } else {
        format!(
            "Expected {}; actual outcome or numeric tolerance differs.",
            expected.kind
        )
    };
    (matches, reason)
}
