//! Versioned local work documents. Runtime jobs and access preferences stay outside them.
use crate::{EqualApp, Tab, canvas, device, notebook_files, practice, tutor, workspaces};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

const FORMAT: &str = "opencalc-notebook";
const VERSION: u32 = 2;
const MAX_TITLE_CHARS: usize = 200;
const MAX_NOTES_BYTES: usize = 64 * 1024;
const MAX_EXPRESSION_CHARS: usize = 1024;
const MAX_RESULT_BYTES: usize = 512 * 1024;
const MAX_HISTORY: usize = 200;
const MAX_MATRIX_INPUT_BYTES: usize = 32 * 1024;
const MAX_MATRIX_RESULT_BYTES: usize = 64 * 1024;
const MATRIX_OPERATIONS: [&str; 5] = [
    "A × B",
    "A + B",
    "Transpose A",
    "Determinant A",
    "Inverse A",
];

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Notebook {
    format: String,
    version: u32,
    pub(crate) title: String,
    pub(crate) notes: String,
    math: MathSettings,
    workspace: Tab,
    expanded_workspace: bool,
    calculation: Calculation,
    matrices: Matrices,
    device: device::DeviceNotebook,
    graph: workspaces::GraphNotebook,
    tutor: tutor::TutorNotebook,
    pub(crate) practice: Option<practice::PracticeNotebook>,
    canvas: canvas::CanvasNotebook,
}

#[derive(Deserialize)]
struct VersionHeader {
    version: u32,
}

/// Keep the old schema explicit: defaults on v2 would also silently accept a
/// truncated current document, while a Value-based migration duplicates ink.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyNotebookV1 {
    format: String,
    version: u32,
    title: String,
    notes: String,
    math: MathSettings,
    workspace: Tab,
    expanded_workspace: bool,
    calculation: Calculation,
    matrices: Matrices,
    device: device::DeviceNotebook,
    graph: workspaces::GraphNotebook,
    tutor: tutor::TutorNotebook,
    practice: Option<practice::PracticeNotebook>,
}

impl LegacyNotebookV1 {
    fn migrate(self) -> Result<Notebook, String> {
        if self.version != 1 || self.workspace == Tab::Notebook {
            return Err("The legacy notebook contains an unsupported version or workspace.".into());
        }
        Ok(Notebook {
            format: self.format,
            version: VERSION,
            title: self.title,
            notes: self.notes,
            math: self.math,
            workspace: self.workspace,
            expanded_workspace: self.expanded_workspace,
            calculation: self.calculation,
            matrices: self.matrices,
            device: self.device,
            graph: self.graph,
            tutor: self.tutor,
            practice: self.practice,
            canvas: canvas::CanvasNotebook::default(),
        })
    }
}

#[derive(Serialize)]
struct WorkContent<'a> {
    format: &'a str,
    version: u32,
    title: &'a str,
    notes: &'a str,
    math: &'a MathSettings,
    calculation: &'a Calculation,
    matrices: &'a Matrices,
    device: &'a device::DeviceNotebook,
    graph: &'a workspaces::GraphNotebook,
    tutor: &'a tutor::TutorNotebook,
    practice: &'a Option<practice::PracticeNotebook>,
}

#[derive(Serialize)]
struct CanvasContent<'a> {
    pages: &'a [canvas::CanvasPage],
}

#[derive(Serialize)]
struct ComparedContent<'a> {
    #[serde(flatten)]
    work: serde_json::Map<String, serde_json::Value>,
    canvas: CanvasContent<'a>,
}

struct BoundedBytes {
    bytes: Vec<u8>,
    maximum: usize,
    exceeded: bool,
}

impl BoundedBytes {
    fn new(maximum: usize) -> Self {
        Self {
            bytes: Vec::new(),
            maximum,
            exceeded: false,
        }
    }
}

impl Write for BoundedBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::Error::other("Notebook output exceeds the file limit"));
        }
        if bytes.len() > self.bytes.capacity().saturating_sub(self.bytes.len()) {
            let required = self.bytes.len() + bytes.len();
            let capacity = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(4096)
                .max(required)
                .min(self.maximum);
            self.bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .map_err(io::Error::other)?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode_bounded(value: &impl Serialize, pretty: bool) -> Result<Vec<u8>, String> {
    let mut output = BoundedBytes::new(notebook_files::MAX_NOTEBOOK_BYTES);
    let encoded = if pretty {
        serde_json::to_writer_pretty(&mut output, value)
    } else {
        serde_json::to_writer(&mut output, value)
    };
    if let Err(error) = encoded {
        return Err(if output.exceeded {
            file_limit_error()
        } else {
            format!("Could not encode this OpenCalc notebook: {error}")
        });
    }
    if pretty {
        output.write_all(b"\n").map_err(|error| {
            if output.exceeded {
                file_limit_error()
            } else {
                format!("Could not finish this OpenCalc notebook: {error}")
            }
        })?;
    }
    Ok(output.bytes)
}

fn file_limit_error() -> String {
    format!(
        "This notebook exceeds {} MiB. Reduce its pages, notes or history before saving.",
        notebook_files::MAX_NOTEBOOK_BYTES / (1024 * 1024)
    )
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MathSettings {
    level: usize,
    degrees: bool,
    precision: usize,
    decimal_comma: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Calculation {
    expression: String,
    result: String,
    error: bool,
    history: Vec<(String, String)>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrices {
    first: String,
    second: String,
    operation: String,
    result: String,
}

fn text_bytes(text: &str, maximum: usize, label: &str) -> Result<(), String> {
    if text.len() > maximum {
        return Err(format!(
            "The notebook {label} exceeds its {maximum}-byte limit."
        ));
    }
    if text
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(format!(
            "The notebook {label} contains unsupported control characters."
        ));
    }
    Ok(())
}

fn expression(text: &str, label: &str) -> Result<(), String> {
    if text.chars().count() > MAX_EXPRESSION_CHARS {
        return Err(format!(
            "The notebook {label} exceeds {MAX_EXPRESSION_CHARS} characters."
        ));
    }
    text_bytes(text, MAX_EXPRESSION_CHARS * 4, label)
}

fn omit_fields(value: &mut serde_json::Value, fields: &[&str]) {
    if let Some(object) = value.as_object_mut() {
        for field in fields {
            object.remove(*field);
        }
    }
}

fn parameter_draft(value: &mut serde_json::Value, field: &str, active: bool, applied: &[f64]) {
    let matches_applied = value[field].as_array().is_some_and(|values| {
        values.len() == applied.len()
            && values.iter().zip(applied).all(|(value, expected)| {
                value
                    .as_str()
                    .and_then(|text| text.trim().parse::<f64>().ok())
                    == Some(*expected)
            })
    });
    if !active || matches_applied {
        omit_fields(value, &[field]);
    }
}

fn device_content(value: &mut serde_json::Value, graph: &workspaces::GraphNotebook) {
    let screen = value["screen"].as_str().unwrap_or_default().to_owned();
    let input = value["input"].as_str().unwrap_or_default().to_owned();
    // Statistics edits use a second buffer until that screen is left. Compare
    // the effective list so entering or leaving the editor is not itself work.
    if screen == "statistics"
        && let (Some(index), Some(draft)) = (
            value["selected_list"].as_u64(),
            value["statistics_draft"].as_str().map(str::to_owned),
        )
        && let Some(list) = value["data"]["lists"].as_array_mut().and_then(|lists| {
            usize::try_from(index)
                .ok()
                .and_then(|index| lists.get_mut(index))
        })
    {
        *list = draft.into();
    }
    parameter_draft(
        value,
        "window_inputs",
        matches!(input.as_str(), "window_x" | "window_y" | "window_range"),
        &[graph.center[0], graph.center[1], graph.range],
    );
    parameter_draft(
        value,
        "table_inputs",
        matches!(input.as_str(), "table_start" | "table_step"),
        &[graph.trace.table_start, graph.trace.table_step],
    );
    let stored_program = value["program_name"].as_str().and_then(|name| {
        value["data"]["programs"]
            .get(name)
            .and_then(serde_json::Value::as_str)
    });
    if stored_program.is_some() && stored_program == value["program_source"].as_str() {
        omit_fields(value, &["program_name", "program_source"]);
    }
    let generated_transfer = value["transfer_draft"].as_str().is_some_and(|text| {
        text.is_empty()
            || serde_json::from_str::<serde_json::Value>(text)
                .is_ok_and(|packet| packet == value["data"])
    });
    if generated_transfer {
        omit_fields(value, &["transfer_draft"]);
    }
    omit_fields(
        value,
        &[
            "screen",
            "input",
            "cursor",
            "selection",
            "menu_index",
            "store_target",
            "selected_list",
            "selected_sequence",
            "power",
            "insert_mode",
            "just_evaluated",
            "statistics_draft",
        ],
    );
}

fn practice_content(value: &mut serde_json::Value) {
    omit_fields(value, &["level", "selected", "difficulties"]);
    if let Some(sessions) = value["sessions"].as_array_mut() {
        sessions.retain(|session| {
            session["response"]
                .as_str()
                .is_some_and(|text| !text.is_empty())
                || session["feedback"]
                    .as_str()
                    .is_some_and(|text| !text.is_empty())
                || session["counted"].as_bool() == Some(true)
                || session["progress"]["attempted"].as_u64().unwrap_or(0) != 0
                || session["progress"]["correct"].as_u64().unwrap_or(0) != 0
        });
        for session in sessions.iter_mut() {
            omit_fields(
                session,
                &["hint_shown", "show_steps", "revealed_steps", "show_visual"],
            );
        }
        sessions.sort_by(|first, second| {
            (first["topic"].as_str(), first["difficulty"].as_str())
                .cmp(&(second["topic"].as_str(), second["difficulty"].as_str()))
        });
    }
}

impl Notebook {
    /// Comparison only: saved files still retain their complete editor context.
    /// Do not validate drafts here; an invalid edit must still mark work dirty.
    pub(crate) fn content_bytes(&self) -> Result<Vec<u8>, String> {
        // Keep the existing small workspace projection, but stream ink directly
        // from its bounded page vectors instead of expanding every point into a
        // temporary JSON Value tree. Bound even the projection before parsing it.
        let mut value: serde_json::Value = {
            let bytes = encode_bounded(
                &WorkContent {
                    format: &self.format,
                    version: self.version,
                    title: &self.title,
                    notes: &self.notes,
                    math: &self.math,
                    calculation: &self.calculation,
                    matrices: &self.matrices,
                    device: &self.device,
                    graph: &self.graph,
                    tutor: &self.tutor,
                    practice: &self.practice,
                },
                false,
            )?;
            serde_json::from_slice(&bytes)
                .map_err(|error| format!("Could not compare notebook content: {error}"))?
        };
        device_content(&mut value["device"], &self.graph);
        omit_fields(&mut value["graph"], &["table"]);
        omit_fields(
            &mut value["graph"]["trace"],
            &["trace_x", "trace_step", "selected", "keyboard_trace"],
        );
        omit_fields(&mut value["tutor"], &["revealed", "all_steps", "hint"]);
        if !value["practice"].is_null() {
            practice_content(&mut value["practice"]);
        }
        let work = value
            .as_object_mut()
            .map(std::mem::take)
            .ok_or("Could not compare the notebook work object.")?;
        encode_bounded(
            &ComparedContent {
                work,
                canvas: CanvasContent {
                    pages: &self.canvas.pages,
                },
            },
            false,
        )
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.format != FORMAT {
            return Err("This file is not an OpenCalc notebook.".into());
        }
        if self.version != VERSION {
            return Err(format!(
                "OpenCalc notebook version {} is not supported; this app supports version {VERSION}.",
                self.version
            ));
        }
        if self.title.chars().count() > MAX_TITLE_CHARS || self.title.chars().any(char::is_control)
        {
            return Err(format!(
                "Use a notebook title of at most {MAX_TITLE_CHARS} characters on one line."
            ));
        }
        text_bytes(&self.notes, MAX_NOTES_BYTES, "notes")?;
        if self.math.level > 3 || !(2..=10).contains(&self.math.precision) {
            return Err("The notebook learning level or decimal precision is unavailable.".into());
        }
        if matches!(self.workspace, Tab::Graph) && self.math.level < 2
            || matches!(self.workspace, Tab::Matrix) && self.math.level != 3
            || self.graph.surface && self.math.level != 3
        {
            return Err("The notebook workspace does not match its learning level.".into());
        }
        if self.workspace != Tab::Calculator && !self.expanded_workspace {
            return Err(
                "The notebook workspace must be expanded to show its selected task.".into(),
            );
        }
        expression(&self.calculation.expression, "calculation")?;
        text_bytes(
            &self.calculation.result,
            MAX_RESULT_BYTES,
            "calculation result",
        )?;
        if self.calculation.history.len() > MAX_HISTORY {
            return Err(format!(
                "A notebook can contain at most {MAX_HISTORY} history entries."
            ));
        }
        for (index, (input, result)) in self.calculation.history.iter().enumerate() {
            expression(input, &format!("history entry {}", index + 1))?;
            text_bytes(result, MAX_RESULT_BYTES, "history result")?;
        }
        text_bytes(&self.matrices.first, MAX_MATRIX_INPUT_BYTES, "matrix A")?;
        text_bytes(&self.matrices.second, MAX_MATRIX_INPUT_BYTES, "matrix B")?;
        text_bytes(
            &self.matrices.result,
            MAX_MATRIX_RESULT_BYTES,
            "matrix result",
        )?;
        if !MATRIX_OPERATIONS.contains(&self.matrices.operation.as_str()) {
            return Err("The notebook matrix operation is unavailable.".into());
        }
        // Draft syntax is preserved. Evaluation and program execution stay explicit.
        self.device.validate()?;
        self.device.validate_editor_context(
            &self.calculation.expression,
            &self.matrices.first,
            &self.matrices.second,
        )?;
        self.graph.validate()?;
        self.tutor.validate()?;
        self.canvas.validate()?;
        if let Some(practice) = &self.practice {
            practice.validate()?;
        }
        Ok(())
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > notebook_files::MAX_NOTEBOOK_BYTES {
            return Err(file_limit_error());
        }
        let read_error = |error| format!("Could not read this OpenCalc notebook: {error}");
        let header: VersionHeader = serde_json::from_slice(bytes).map_err(read_error)?;
        let notebook = match header.version {
            1 => serde_json::from_slice::<LegacyNotebookV1>(bytes)
                .map_err(read_error)?
                .migrate()?,
            VERSION => serde_json::from_slice::<Self>(bytes).map_err(read_error)?,
            other => {
                return Err(format!(
                    "OpenCalc notebook version {other} is not supported; this app supports versions 1 and {VERSION}."
                ));
            }
        };
        notebook.validate()?;
        Ok(notebook)
    }

    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        // Whitespace must not consume most of an ink document's file budget.
        encode_bounded(self, false)
    }
}

impl EqualApp {
    pub(crate) fn capture_notebook(
        &self,
        title: &str,
        notes: &str,
        include_practice: bool,
    ) -> Notebook {
        Notebook {
            format: FORMAT.into(),
            version: VERSION,
            title: title.into(),
            notes: notes.into(),
            math: MathSettings {
                level: self.settings.level,
                degrees: self.settings.degrees,
                precision: self.settings.precision,
                decimal_comma: self.settings.access.decimal_comma,
            },
            workspace: self.tab,
            expanded_workspace: self.show_workspace || self.tab != Tab::Calculator,
            calculation: Calculation {
                expression: self.expression.clone(),
                result: self.result.clone(),
                error: self.error,
                history: self.history.clone(),
            },
            matrices: Matrices {
                first: self.matrix_a.clone(),
                second: self.matrix_b.clone(),
                operation: self.matrix_op.clone(),
                result: self.matrix_result.clone(),
            },
            device: self.capture_device_notebook(),
            graph: self.capture_graph_notebook(),
            tutor: self.tutor.notebook_snapshot(),
            practice: include_practice.then(|| self.practice.notebook_snapshot()),
            canvas: self.canvas.notebook_snapshot(),
        }
    }

    pub(crate) fn apply_notebook(&mut self, notebook: Notebook) -> Result<(), String> {
        notebook.validate()?;
        let audio_errors: Vec<_> = [self.speech.stop().err(), self.graph_audio.stop().err()]
            .into_iter()
            .flatten()
            .collect();
        let Notebook {
            math,
            workspace,
            expanded_workspace,
            calculation,
            matrices,
            device,
            graph,
            tutor,
            practice,
            canvas,
            ..
        } = notebook;
        self.settings.level = math.level;
        self.settings.degrees = math.degrees;
        self.settings.precision = math.precision;
        // Number notation belongs to the saved math; visual/motor/audio preferences do not.
        self.settings.access.decimal_comma = math.decimal_comma;
        self.expression = calculation.expression;
        self.result = calculation.result;
        self.error = calculation.error;
        self.history = calculation.history;
        self.matrix_a = matrices.first;
        self.matrix_b = matrices.second;
        self.matrix_op = matrices.operation;
        self.matrix_result = matrices.result;
        self.apply_device_notebook(device);
        self.apply_graph_notebook(graph);
        self.tutor.apply_notebook(tutor);
        self.canvas.apply_notebook(canvas);
        if let Some(practice) = practice {
            self.practice.apply_notebook(practice);
        }
        self.tab = workspace;
        self.show_workspace = expanded_workspace;
        self.handheld_preview = false;
        self.second = false;
        self.device.alpha = false;
        self.input_access.pause();
        self.show_history = false;
        self.show_help = false;
        self.show_settings = false;
        self.a11y.show_glossary = false;
        self.a11y.show_shortcuts = false;
        self.a11y.overlay_return_focus = None;
        self.a11y.workspace_focus = None;
        self.a11y.overlay_had_settings = false;
        self.tool = false;
        self.graph_error.clear();
        self.message = if audio_errors.is_empty() {
            "Notebook opened.".into()
        } else {
            format!(
                "Notebook opened. Audio stop error: {}",
                audio_errors.join("; ")
            )
        };
        if self.settings.access.captions {
            self.a11y.latest_announcement = self.message.clone();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Settings, accessibility, graph_tools};

    fn app() -> EqualApp {
        EqualApp::with_settings(Settings::default(), None)
    }

    fn legacy_v1_fixture() -> (Vec<u8>, serde_json::Value) {
        let mut application = app();
        application.expression = "12+8".into();
        application.evaluate();
        application.graph_a = "x^2+1".into();
        application.matrix_a = "1 2; 3 4".into();
        application.tutor.problem = "2*x+3=11".into();
        application.tutor.solve_local(2, true);
        let answer = application.practice.current_question().answer.display();
        application.practice.current_session_mut().response = answer;
        application.practice.check_answer();
        let mut fixture = serde_json::to_value(application.capture_notebook(
            "Previous homework",
            "Keep every old workspace.",
            true,
        ))
        .unwrap();
        fixture["version"] = 1.into();
        fixture.as_object_mut().unwrap().remove("canvas");
        (encode_bounded(&fixture, true).unwrap(), fixture)
    }

    fn canvas_fixture() -> canvas::CanvasNotebook {
        serde_json::from_value(serde_json::json!({
            "pages": [{
                "title": "Physics", "background": "grid", "elements": [
                    {"kind":"ink", "points":[
                        {"position":[0.1,0.2],"pressure":0.5},
                        {"position":[0.2,0.3],"pressure":1.0}
                    ], "width":2.0,"color":"ink","description":"Force arrow"},
                    {"kind":"text","bounds":[0.2,0.1,0.4,0.15],"text":"F = ma",
                        "font_size":18.0,"color":"blue","description":"Newton's second law"},
                    {"kind":"rectangle","bounds":[0.1,0.5,0.3,0.2],"width":2.0,
                        "color":"teal","description":"System boundary"}
                ]
            }], "selected_page":0, "zoom":1.0, "pan":[0.0,0.0]
        }))
        .unwrap()
    }

    #[test]
    fn v2_round_trip_retains_ink_text_shapes_and_math() {
        let mut application = app();
        application.expression = "12+8".into();
        application.evaluate();
        application.canvas.apply_notebook(canvas_fixture());
        application.tab = Tab::Notebook;
        application.show_workspace = true;
        let document = application.capture_notebook("Physics", "Keep the force labels.", false);
        let bytes = document.to_bytes().unwrap();
        let decoded = Notebook::decode(&bytes).unwrap();
        assert_eq!(decoded.version, VERSION);
        assert_eq!(decoded.canvas, canvas_fixture());
        let mut restored = app();
        restored.apply_notebook(decoded).unwrap();
        assert_eq!(restored.result, "20");
        assert_eq!(restored.answer, 20.0);
        assert_eq!(restored.tab, Tab::Notebook);
        assert_eq!(restored.canvas.notebook_snapshot(), canvas_fixture());
        assert_eq!(
            restored
                .capture_notebook("Physics", "Keep the force labels.", false)
                .content_bytes()
                .unwrap(),
            document.content_bytes().unwrap()
        );
    }

    #[test]
    fn canvas_content_comparison_keeps_edits_but_ignores_view_navigation() {
        let mut original = app().capture_notebook("Pages", "", false);
        original.canvas = canvas_fixture();
        let mut second = original.canvas.pages[0].clone();
        second.title = "Second page".into();
        original.canvas.pages.push(second);
        let bytes = original.content_bytes().unwrap();
        let mut navigated = original.clone();
        navigated.canvas.selected_page = 1;
        navigated.canvas.zoom = 2.0;
        navigated.canvas.pan = [0.25, -0.25];
        assert_eq!(navigated.content_bytes().unwrap(), bytes);
        assert_ne!(navigated.to_bytes().unwrap(), original.to_bytes().unwrap());
        for change in [
            (|document: &mut Notebook| document.canvas.pages[0].title.push('!'))
                as fn(&mut Notebook),
            |document| document.canvas.pages[0].background = canvas::Paper::Ruled,
            |document| {
                if let canvas::CanvasElement::Ink { points, .. } =
                    &mut document.canvas.pages[0].elements[0]
                {
                    points[0].pressure = 0.75;
                }
            },
            |document| {
                if let canvas::CanvasElement::Text { text, .. } =
                    &mut document.canvas.pages[0].elements[1]
                {
                    text.push_str("; m in kg");
                }
            },
            |document| {
                if let canvas::CanvasElement::Rectangle { bounds, .. } =
                    &mut document.canvas.pages[0].elements[2]
                {
                    bounds[0] = 0.2;
                }
            },
        ] {
            let mut edited = original.clone();
            change(&mut edited);
            edited.validate().unwrap();
            assert_ne!(edited.content_bytes().unwrap(), bytes);
        }
    }

    #[test]
    fn aggregate_bounded_ink_serializes_and_compares_within_the_file_budget() {
        let mut document = app().capture_notebook("Long notes", "", false);
        let mut remaining = 100_000;
        while remaining != 0 {
            let count = remaining.min(4096);
            document.canvas.pages[0]
                .elements
                .push(canvas::CanvasElement::Ink {
                    points: vec![
                        canvas::InkPoint {
                            position: [0.5, 0.5],
                            pressure: 1.0
                        };
                        count
                    ],
                    width: 2.0,
                    color: canvas::InkColor::Ink,
                    description: "A bounded long note".into(),
                });
            remaining -= count;
        }
        let full = document.to_bytes().unwrap();
        let content = document.content_bytes().unwrap();
        assert!(full.len() < notebook_files::MAX_NOTEBOOK_BYTES);
        assert!(content.len() < notebook_files::MAX_NOTEBOOK_BYTES);
        assert_eq!(Notebook::decode(&full).unwrap().canvas, document.canvas);
    }

    #[test]
    fn legacy_v1_fixture_migrates_without_losing_existing_work() {
        let (bytes, mut expected) = legacy_v1_fixture();
        let migrated = Notebook::decode(&bytes).unwrap();
        assert_eq!(migrated.canvas, canvas::CanvasNotebook::default());
        expected["version"] = VERSION.into();
        expected["canvas"] = serde_json::to_value(canvas::CanvasNotebook::default()).unwrap();
        assert_eq!(serde_json::to_value(&migrated).unwrap(), expected);
        let saved = migrated.to_bytes().unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&saved).unwrap()["version"],
            VERSION
        );
        let mut restored = app();
        restored.apply_notebook(migrated).unwrap();
        assert_eq!(restored.answer, 20.0);
        assert_eq!(restored.graph_a, "x^2+1");
        assert_eq!(restored.matrix_a, "1 2; 3 4");
        assert_eq!(restored.practice.current_session().progress.correct, 1);
        assert_eq!(
            restored.canvas.notebook_snapshot(),
            canvas::CanvasNotebook::default()
        );
    }

    #[test]
    fn versioned_schemas_reject_missing_or_mislabelled_canvas_data() {
        let mut current = serde_json::to_value(app().capture_notebook("", "", false)).unwrap();
        current.as_object_mut().unwrap().remove("canvas");
        assert!(
            Notebook::decode(&serde_json::to_vec(&current).unwrap()).is_err(),
            "Missing v2 canvas must not silently become a blank page"
        );
        let (_, mut old) = legacy_v1_fixture();
        old["canvas"] = serde_json::to_value(canvas::CanvasNotebook::default()).unwrap();
        assert!(
            Notebook::decode(&serde_json::to_vec(&old).unwrap()).is_err(),
            "A v1 file must not discard unexpected canvas content"
        );
        old.as_object_mut().unwrap().remove("canvas");
        old["workspace"] = "notebook".into();
        assert!(Notebook::decode(&serde_json::to_vec(&old).unwrap()).is_err());
        let (_, mut truncated) = legacy_v1_fixture();
        truncated.as_object_mut().unwrap().remove("calculation");
        assert!(Notebook::decode(&serde_json::to_vec(&truncated).unwrap()).is_err());
    }

    #[test]
    fn capped_json_output_never_appends_or_allocates_past_its_limit() {
        let mut output = BoundedBytes::new(16);
        output.write_all(b"twelve bytes").unwrap();
        assert!(output.write_all(b"too large").is_err());
        assert_eq!(output.bytes, b"twelve bytes");
        assert!(output.exceeded);
        assert!(output.bytes.capacity() <= 16);
        let mut escaped = BoundedBytes::new(32);
        assert!(serde_json::to_writer(&mut escaped, &"\t".repeat(32)).is_err());
        assert!(escaped.exceeded);
        assert!(escaped.bytes.len() <= 32 && escaped.bytes.capacity() <= 32);
    }

    #[test]
    fn invalid_canvas_is_rejected_before_replacing_existing_math_or_pages() {
        let mut application = app();
        application.expression = "3+4".into();
        application.evaluate();
        let mut current_pages = application.canvas.notebook_snapshot();
        current_pages.pages[0].title = "Current page".into();
        application.canvas.apply_notebook(current_pages);
        let before = application
            .capture_notebook("Current", "Keep this.", true)
            .to_bytes()
            .unwrap();
        let mut invalid = application.capture_notebook("Replacement", "Do not apply.", true);
        invalid.calculation.expression = "900".into();
        invalid.canvas.selected_page = usize::MAX;
        assert!(application.apply_notebook(invalid).is_err());
        assert_eq!(
            application
                .capture_notebook("Current", "Keep this.", true)
                .to_bytes()
                .unwrap(),
            before
        );
        let mut invalid_ink = application.capture_notebook("Replacement", "", true);
        invalid_ink.canvas = canvas_fixture();
        invalid_ink.calculation.expression = "900".into();
        if let canvas::CanvasElement::Ink { points, .. } =
            &mut invalid_ink.canvas.pages[0].elements[0]
        {
            points[0].position[0] = -0.1;
        }
        assert!(application.apply_notebook(invalid_ink).is_err());
        assert_eq!(
            application
                .capture_notebook("Current", "Keep this.", true)
                .to_bytes()
                .unwrap(),
            before
        );
    }

    #[test]
    fn navigation_and_presentation_do_not_change_content_comparison() {
        let mut application = app();
        application.center = [1.0, -2.0];
        application.range = 7.0;
        application.tutor.solve_local(2, true);
        let original = application.capture_notebook("Work", "Notes", true);
        let content = original.content_bytes().unwrap();
        for screen in [
            device::DeviceScreen::Window,
            device::DeviceScreen::TableSettings,
            device::DeviceScreen::Math,
            device::DeviceScreen::Variables,
            device::DeviceScreen::Lists,
            device::DeviceScreen::Statistics,
            device::DeviceScreen::Mode,
        ] {
            application.set_device_screen(screen);
            assert_eq!(
                application
                    .capture_notebook("Work", "Notes", true)
                    .content_bytes()
                    .unwrap(),
                content,
                "Opening {screen:?} must not be treated as edited math",
            );
        }
        application.tab = Tab::Tutor;
        application.show_workspace = true;
        application.graph_access.trace_x = 2.0;
        application.graph_access.trace_step = 0.25;
        application.graph_access.selected = 1;
        application.graph_access.keyboard_trace = true;
        application.table = true;
        application.tutor.set_pacing(true);
        application
            .practice
            .select_topic(practice::TopicId::Subtraction);
        application
            .practice
            .select_difficulty(practice::Difficulty::Challenge);
        application.practice.set_level(1);
        application.practice.show_hint();
        let navigated = application.capture_notebook("Work", "Notes", true);
        assert_eq!(navigated.content_bytes().unwrap(), content);
        assert_ne!(
            navigated.to_bytes().unwrap(),
            original.to_bytes().unwrap(),
            "Saved files must still retain navigation and presentation"
        );
    }

    #[test]
    fn stats_and_unapplied_parameter_or_transfer_edits_remain_content() {
        let ctx = crate::egui::Context::default();
        let mut application = app();
        let original = application
            .capture_notebook("Work", "", false)
            .content_bytes()
            .unwrap();
        application.set_device_screen(device::DeviceScreen::Statistics);
        assert_eq!(
            application
                .capture_notebook("Work", "", false)
                .content_bytes()
                .unwrap(),
            original
        );
        for key in ["clear", "1", ",", "2", ",", "3"] {
            application.press_device_key(&ctx, key);
        }
        let edited_stats = application
            .capture_notebook("Work", "", false)
            .content_bytes()
            .unwrap();
        assert_ne!(edited_stats, original);
        let content: serde_json::Value = serde_json::from_slice(&edited_stats).unwrap();
        assert_eq!(content["device"]["data"]["lists"][0], "1,2,3");
        application.set_device_screen(device::DeviceScreen::Home);
        assert_eq!(
            application
                .capture_notebook("Work", "", false)
                .content_bytes()
                .unwrap(),
            edited_stats,
            "Leaving Statistics commits the same effective content"
        );

        for screen in [
            device::DeviceScreen::Window,
            device::DeviceScreen::TableSettings,
            device::DeviceScreen::Link,
        ] {
            let mut application = app();
            application.set_device_screen(screen);
            let before = application
                .capture_notebook("Work", "", false)
                .content_bytes()
                .unwrap();
            for key in ["clear", "3"] {
                application.press_device_key(&ctx, key);
            }
            assert_ne!(
                application
                    .capture_notebook("Work", "", false)
                    .content_bytes()
                    .unwrap(),
                before,
                "An unapplied {screen:?} draft is still work"
            );
        }
    }

    #[test]
    fn content_comparison_keeps_notes_math_storage_and_progress() {
        let original = app().capture_notebook("Work", "", true);
        let original_content = original.content_bytes().unwrap();
        for change in [
            (|value: &mut serde_json::Value| value["notes"] = "Changed notes".into())
                as fn(&mut serde_json::Value),
            |value| value["math"]["degrees"] = false.into(),
            |value| value["math"]["precision"] = 4.into(),
            |value| value["calculation"]["expression"] = "1+2".into(),
            |value| value["calculation"]["result"] = "3".into(),
            |value| value["calculation"]["history"] = serde_json::json!([["1+2", "3"]]),
            |value| value["device"]["memory"] = 5.into(),
            |value| value["device"]["data"]["variables"]["A"] = 5.into(),
            |value| value["device"]["data"]["lists"][0] = "1;2".into(),
            |value| value["device"]["data"]["sequences"][0] = "n+2".into(),
            |value| value["device"]["data"]["programs"]["SAVED"] = "Disp 3".into(),
            |value| value["graph"]["range"] = 8.into(),
            |value| value["graph"]["drawings"] = serde_json::json!([{"Point":[1.0, 2.0]}]),
            |value| value["practice"]["sessions"][0]["response"] = "thinking".into(),
        ] {
            let mut value = serde_json::to_value(&original).unwrap();
            change(&mut value);
            let changed: Notebook = serde_json::from_value(value).unwrap();
            assert_ne!(changed.content_bytes().unwrap(), original_content);
        }
        let mut application = app();
        let answer = application.practice.current_question().answer.display();
        application.practice.current_session_mut().response = answer;
        application.practice.check_answer();
        let with_progress = application
            .capture_notebook("Work", "", true)
            .content_bytes()
            .unwrap();
        assert_ne!(with_progress, original_content);
        let without_progress = application
            .capture_notebook("Work", "", false)
            .content_bytes()
            .unwrap();
        assert_eq!(
            without_progress,
            app()
                .capture_notebook("Work", "", false)
                .content_bytes()
                .unwrap()
        );
    }

    #[test]
    fn successful_restore_drops_old_focus_but_rejected_restore_keeps_it() {
        let mut application = app();
        let old_editor = crate::egui::Id::new("old-document-editor");
        application.a11y.overlay_return_focus = Some(old_editor);
        application.a11y.workspace_focus = Some(old_editor);
        application.a11y.overlay_had_settings = true;
        let valid = application.capture_notebook("New document", "", false);
        let mut invalid = valid.clone();
        invalid.version = 999;
        assert!(application.apply_notebook(invalid).is_err());
        assert_eq!(application.a11y.overlay_return_focus, Some(old_editor));
        assert_eq!(application.a11y.workspace_focus, Some(old_editor));
        assert!(application.a11y.overlay_had_settings);
        application.apply_notebook(valid).unwrap();
        assert!(application.a11y.overlay_return_focus.is_none());
        assert!(application.a11y.workspace_focus.is_none());
        assert!(!application.a11y.overlay_had_settings);
    }

    #[test]
    fn large_valid_list_results_can_be_saved_and_reopened() {
        let ctx = crate::egui::Context::default();
        let mut application = app();
        application.set_device_screen(device::DeviceScreen::Lists);
        application.press_device_key(&ctx, "clear");
        let values = std::iter::repeat_n("1.23456789e300", 1000)
            .collect::<Vec<_>>()
            .join(";");
        application.press_device_key(&ctx, &values);
        application.set_device_screen(device::DeviceScreen::Home);
        application.expression = "L1".into();
        application.evaluate();
        assert!(!application.error, "{}", application.result);
        assert!(application.result.len() > 8 * 1024);
        let result = application.result.clone();
        let bytes = application
            .capture_notebook("Large list", "", false)
            .to_bytes()
            .unwrap();
        assert!(bytes.len() < notebook_files::MAX_NOTEBOOK_BYTES);
        let mut restored = app();
        restored
            .apply_notebook(Notebook::decode(&bytes).unwrap())
            .unwrap();
        assert_eq!(restored.result, result);
        assert_eq!(restored.history[0].1, result);
        restored.expression = "L1".into();
        restored.evaluate();
        assert!(!restored.error, "{}", restored.result);
        assert_eq!(
            restored.result, result,
            "The underlying list must also survive reopening"
        );
    }

    #[test]
    fn complete_work_round_trips_without_overwriting_access_preferences() {
        let mut source = app();
        source.settings.level = 3;
        source.settings.degrees = false;
        source.settings.precision = 6;
        source.settings.access.decimal_comma = true;
        source.expression = "12+8".into();
        source.evaluate();
        source.graph_extra[7] = "sqrt(x)".into();
        source.range = 7.5;
        source.center = [1.0, -2.0];
        source.graph_access.trace_x = 3.0;
        source.graph_access.selected = 9;
        source
            .graph_tools
            .drawings
            .push(graph_tools::Drawing::Circle {
                center: [1.0, 2.0],
                radius: 3.0,
            });
        source.matrix_a = "1 2; 3 4".into();
        source.matrix_result = "7 10; 15 22".into();
        source.tutor.problem = "2*x + 3 = 11".into();
        source.tutor.solve_local(3, false);
        let question = source.practice.current_question();
        source.practice.current_session_mut().response = question.answer.display();
        source.practice.check_answer();
        source.tab = Tab::Graph;
        source.show_workspace = true;
        let expected = source.capture_notebook("Lesson 1", "Try x=3 next.", true);
        let bytes = expected.to_bytes().unwrap();
        let decoded = Notebook::decode(&bytes).unwrap();

        let mut restored = app();
        restored.settings.scale = 1.25;
        restored.settings.contrast = true;
        restored.settings.readable = true;
        restored.settings.color_safe = true;
        restored.settings.access.theme = accessibility::ColorTheme::Dark;
        restored.settings.access.left_handed = true;
        restored.settings.access.font_scale = 1.4;
        let mut original_access = serde_json::to_value(&restored.settings.access).unwrap();
        restored.apply_notebook(decoded).unwrap();
        let mut restored_access = serde_json::to_value(&restored.settings.access).unwrap();
        original_access
            .as_object_mut()
            .unwrap()
            .remove("decimal_comma");
        restored_access
            .as_object_mut()
            .unwrap()
            .remove("decimal_comma");
        assert_eq!(original_access, restored_access);
        assert_eq!(restored.settings.scale, 1.25);
        assert!(
            restored.settings.contrast
                && restored.settings.readable
                && restored.settings.color_safe
        );
        assert!(restored.settings.access.decimal_comma);
        assert_eq!(restored.answer, 20.0);
        assert_eq!(restored.practice.current_session().progress.correct, 1);
        assert_eq!(
            serde_json::to_value(restored.capture_notebook("Lesson 1", "Try x=3 next.", true))
                .unwrap(),
            serde_json::to_value(expected).unwrap(),
        );
    }

    #[test]
    fn malformed_parts_are_rejected_before_any_workspace_changes() {
        let mut target = app();
        target.expression = "3+4".into();
        target.evaluate();
        target.settings.access.left_handed = true;
        let before = target
            .capture_notebook("Current", "Keep this.", true)
            .to_bytes()
            .unwrap();
        for change in [
            (|value: &mut Notebook| value.version = 999) as fn(&mut Notebook),
            |value| value.math.level = 4,
            |value| value.math.precision = 0,
            |value| value.graph.range = f64::NAN,
            |value| value.matrices.operation = "Run a command".into(),
            |value| value.calculation.expression = "1".repeat(MAX_EXPRESSION_CHARS + 1),
            |value| value.notes = "x".repeat(MAX_NOTES_BYTES + 1),
        ] {
            let mut invalid = target.capture_notebook("Bad", "Replacement", true);
            change(&mut invalid);
            assert!(target.apply_notebook(invalid).is_err());
            assert_eq!(
                target
                    .capture_notebook("Current", "Keep this.", true)
                    .to_bytes()
                    .unwrap(),
                before
            );
            assert!(target.settings.access.left_handed);
        }
        for (pointer, invalid_value) in [
            ("/device/version", serde_json::json!(2)),
            ("/graph/trace/selected", serde_json::json!(10)),
            ("/tutor/revealed", serde_json::json!(0)),
            (
                "/practice/sessions/0/progress/correct",
                serde_json::json!(1),
            ),
        ] {
            let mut value = serde_json::to_value(target.capture_notebook("Bad", "", true)).unwrap();
            value["math"]["level"] = serde_json::json!(3);
            value["calculation"]["expression"] = serde_json::json!("900");
            *value.pointer_mut(pointer).unwrap() = invalid_value;
            let invalid = serde_json::from_value(value).unwrap();
            assert!(
                target.apply_notebook(invalid).is_err(),
                "Invalid section {pointer}"
            );
            assert_eq!(
                target
                    .capture_notebook("Current", "Keep this.", true)
                    .to_bytes()
                    .unwrap(),
                before,
                "A later invalid section must not replace earlier work: {pointer}",
            );
        }
    }

    #[test]
    fn omitted_practice_keeps_existing_progress_and_draft_answers() {
        let source = app();
        let document = source.capture_notebook("No scores", "", false);
        assert!(document.practice.is_none());
        let mut target = app();
        target.practice.current_session_mut().response = "still thinking".into();
        target.practice.show_hint();
        let before = serde_json::to_value(target.practice.notebook_snapshot()).unwrap();
        target.apply_notebook(document).unwrap();
        assert_eq!(
            serde_json::to_value(target.practice.notebook_snapshot()).unwrap(),
            before
        );
    }

    #[test]
    fn incomplete_drafts_are_saved_without_evaluation_or_rewriting() {
        let mut source = app();
        source.expression = "sqrt(".into();
        source.graph_a = "x^(".into();
        source.matrix_a = "1 2;".into();
        let document = source.capture_notebook("Draft", "", false);
        let decoded = Notebook::decode(&document.to_bytes().unwrap()).unwrap();
        let mut restored = app();
        restored.apply_notebook(decoded).unwrap();
        assert_eq!(restored.expression, "sqrt(");
        assert_eq!(restored.graph_a, "x^(");
        assert_eq!(restored.matrix_a, "1 2;");
        assert_eq!(restored.result, "0");
        assert!(restored.history.is_empty());
    }

    #[test]
    fn schema_format_and_byte_limits_are_explicit() {
        let document = app().capture_notebook("Document", "", false);
        let mut value = serde_json::to_value(&document).unwrap();
        value["format"] = "another-app".into();
        assert!(Notebook::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        value["format"] = FORMAT.into();
        value["machine_path"] = "not accepted".into();
        assert!(Notebook::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(Notebook::decode(&vec![b' '; notebook_files::MAX_NOTEBOOK_BYTES + 1]).is_err());
        assert!(Notebook::decode(b"not JSON").is_err());
        let mut oversized = document;
        let entries = notebook_files::MAX_NOTEBOOK_BYTES / MAX_RESULT_BYTES + 1;
        assert!(entries <= MAX_HISTORY);
        oversized.calculation.history = vec![("1".into(), "x".repeat(MAX_RESULT_BYTES)); entries];
        oversized.validate().unwrap();
        assert!(
            oversized.to_bytes().is_err(),
            "Serialized JSON must respect the file limit too"
        );
    }

    #[test]
    fn metadata_and_schema_exclude_paths_access_preferences_and_ai_state() {
        let document = app().capture_notebook("Local work", "Notes", true);
        let value = serde_json::to_value(document).unwrap();
        for key in [
            "path",
            "preferences",
            "accessibility",
            "settings",
            "attachments",
            "pending",
            "recovery",
            "data_dir",
        ] {
            assert!(value.get(key).is_none(), "Unexpected document field {key}");
        }
        for key in [
            "attachment",
            "pending",
            "ai_text",
            "question",
            "model",
            "ai_enabled",
        ] {
            assert!(
                value["tutor"].get(key).is_none(),
                "Tutor runtime field {key} must remain local"
            );
        }
        for key in [
            "tool",
            "selection",
            "undo",
            "pointer",
            "file_path",
            "recovery",
        ] {
            assert!(
                value["canvas"].get(key).is_none(),
                "Canvas runtime field {key} must not leak"
            );
        }
        let mut unexpected_runtime = value;
        unexpected_runtime["canvas"]["tool"] = "eraser".into();
        assert!(Notebook::decode(&serde_json::to_vec(&unexpected_runtime).unwrap()).is_err());
    }
}
