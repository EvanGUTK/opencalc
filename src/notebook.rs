//! Versioned local work documents. Runtime jobs and access preferences stay outside them.
use crate::{EqualApp, Tab, device, notebook_files, practice, tutor, workspaces};
use serde::{Deserialize, Serialize};

const FORMAT: &str = "opencalc-notebook";
const VERSION: u32 = 1;
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
        let mut value = serde_json::to_value(self)
            .map_err(|error| format!("Could not compare notebook content: {error}"))?;
        omit_fields(&mut value, &["workspace", "expanded_workspace"]);
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
        serde_json::to_vec(&value)
            .map_err(|error| format!("Could not compare notebook content: {error}"))
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
        if let Some(practice) = &self.practice {
            practice.validate()?;
        }
        Ok(())
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > notebook_files::MAX_NOTEBOOK_BYTES {
            return Err("OpenCalc notebooks must be no larger than 2 MiB.".into());
        }
        let notebook: Self = serde_json::from_slice(bytes)
            .map_err(|error| format!("Could not read this OpenCalc notebook: {error}"))?;
        notebook.validate()?;
        Ok(notebook)
    }

    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("Could not encode this OpenCalc notebook: {error}"))?;
        bytes.push(b'\n');
        if bytes.len() > notebook_files::MAX_NOTEBOOK_BYTES {
            return Err(
                "This notebook exceeds 2 MiB. Shorten its notes or history before saving.".into(),
            );
        }
        Ok(bytes)
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
        let values = std::iter::repeat_n("1e300", 1000)
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
        oversized.calculation.history = vec![("1".into(), "\t".repeat(8 * 1024)); MAX_HISTORY];
        oversized.validate().unwrap();
        assert!(
            oversized.to_bytes().is_err(),
            "Escaped JSON must respect the file limit too"
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
    }
}
