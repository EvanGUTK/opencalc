//! Native egui workflows: real widget events and application routing, without OS automation.

use super::*;
use crate::curriculum_checks::{self, Actual};
use egui::accesskit::{Action, ActionData, ActionRequest, NodeId};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct StudentSession {
    app: EqualApp,
    ctx: egui::Context,
    size: egui::Vec2,
    clock: f64,
    output: egui::FullOutput,
    clipboard: Option<String>,
}

impl StudentSession {
    fn new(level: usize) -> Self {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut session = Self {
            app: EqualApp::with_settings(
                Settings {
                    level,
                    precision: 10,
                    ..Settings::default()
                },
                None,
            ),
            ctx,
            size: egui::vec2(1360.0, 1100.0),
            clock: 0.0,
            output: egui::FullOutput::default(),
            clipboard: None,
        };
        session.idle();
        session.idle();
        session
    }

    fn frame(&mut self, events: Vec<egui::Event>, modifiers: egui::Modifiers) {
        self.clock += 1.0 / 60.0;
        self.app.theme(&self.ctx);
        self.output = self.ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.size)),
                events,
                modifiers,
                time: Some(self.clock),
                ..Default::default()
            },
            |ctx| {
                self.app.tutor.poll(ctx);
                self.app.poll_notebook(ctx);
                self.app.handle_keyboard_input(ctx);
                self.app.draw(ctx);
                self.app.windows(ctx);
                self.app.accessibility_help_windows(ctx);
                self.app.notebook_windows(ctx);
                self.app.sync_overlay_focus(ctx);
            },
        );
        for command in &self.output.platform_output.commands {
            if let egui::OutputCommand::CopyText(text) = command {
                self.clipboard = Some(text.clone());
            }
        }
    }

    fn idle(&mut self) {
        self.frame(Vec::new(), egui::Modifiers::NONE);
    }

    fn node(&self, label: &str) -> (NodeId, &egui::accesskit::Node) {
        self.output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("native accessibility tree")
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .map(|(id, node)| (*id, node))
            .unwrap_or_else(|| panic!("Native control missing: {label}"))
    }

    fn has_control(&self, label: &str) -> bool {
        self.output
            .platform_output
            .accesskit_update
            .as_ref()
            .is_some_and(|tree| {
                tree.nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some(label))
            })
    }

    fn action(&mut self, label: &str, action: Action, data: Option<ActionData>) {
        let target = self.node(label).0;
        self.frame(
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target,
                data,
            })],
            egui::Modifiers::NONE,
        );
        self.idle();
    }

    fn activate(&mut self, label: &str) {
        self.action(label, Action::Click, None);
    }

    fn pointer_click(&mut self, label: &str) {
        let bounds = self.node(label).1.bounds().expect("pointer target bounds");
        let pos = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        assert!(
            egui::Rect::from_min_size(egui::Pos2::ZERO, self.size).contains(pos),
            "{label} is outside the viewport"
        );
        for pressed in [true, false] {
            self.frame(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                egui::Modifiers::NONE,
            );
        }
        self.idle();
    }

    fn key(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        for pressed in [true, false] {
            self.frame(
                vec![egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: false,
                    modifiers,
                }],
                modifiers,
            );
        }
        self.idle();
    }

    fn replace(&mut self, label: &str, text: &str) {
        self.action(label, Action::Focus, None);
        self.key(egui::Key::A, egui::Modifiers::COMMAND);
        if text.is_empty() {
            self.key(egui::Key::Backspace, egui::Modifiers::NONE);
        } else {
            self.frame(vec![egui::Event::Text(text.into())], egui::Modifiers::NONE);
        }
        self.idle();
    }

    fn calculate(&mut self, text: &str) {
        self.replace("Calculation expression", text);
        self.key(egui::Key::Enter, egui::Modifiers::NONE);
    }

    fn workspace(&mut self, label: &str) {
        if !self.app.show_workspace {
            self.activate("Workspace");
        }
        self.activate(label);
    }

    fn copied_text(&self) -> Option<&str> {
        self.clipboard.as_deref()
    }

    fn wait_until(&mut self, description: &str, ready: impl Fn(&EqualApp) -> bool) {
        let start = Instant::now();
        while !ready(&self.app) {
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "Timed out: {description}; status: {}",
                self.app.message
            );
            self.idle();
            std::thread::yield_now();
        }
        self.idle();
    }
}

fn assert_fixture(case: &curriculum_checks::Case, actual: Actual) {
    let (matches, reason) = curriculum_checks::compare(&case.expected, &actual);
    assert!(
        matches,
        "{} ({} / {}): {reason}; actual {actual:?}",
        case.id, case.stage, case.subject
    );
    eprintln!("NATIVE-E2E PASS {}", case.id);
}

#[test]
fn student_01_k3_physical_keys_division_memory_and_error_correction() {
    let mut student = StudentSession::new(0);
    student.size = egui::vec2(462.0, 940.0);
    student.idle();
    for key in ["7", "Add", "5", "Calculate answer"] {
        student.pointer_click(key);
    }
    assert_eq!(student.app.result, "12");
    assert_eq!(
        student.app.history.len(),
        1,
        "A single Enter must create one history row"
    );
    student.pointer_click("Add to memory");
    student.pointer_click("Power on or clear");
    for key in [
        "4",
        "Subtract from memory",
        "Power on or clear",
        "Recall or clear memory",
        "Calculate answer",
    ] {
        student.pointer_click(key);
    }
    assert_eq!(student.app.answer, 8.0);
    student.pointer_click("Power on or clear");
    for key in ["8", "Divide", "0", "Calculate answer"] {
        student.pointer_click(key);
    }
    assert!(
        student.app.error,
        "Division by zero must produce a recoverable error"
    );
    student.pointer_click("Power on or clear");
    for key in ["8", "Divide", "2", "Calculate answer"] {
        student.pointer_click(key);
    }
    assert_eq!(student.app.answer, 4.0);
    assert!(!student.app.error);
    eprintln!("NATIVE-E2E PASS S01 physical keypad, memory, narrow window, error correction");
}

#[test]
fn student_02_middle_school_fraction_keys_and_order_of_operations() {
    let mut student = StudentSession::new(1);
    for key in [
        "1",
        "Enter fraction",
        "4",
        "Add",
        "1",
        "Enter fraction",
        "2",
        "Calculate answer",
    ] {
        student.pointer_click(key);
    }
    assert_eq!(student.app.answer, 0.75);
    student.pointer_click("Convert fraction or decimal");
    assert!(
        student.has_control("Answer: 3/4"),
        "Fraction conversion must appear in the actual display and accessibility tree"
    );
    student.calculate("2+3*4");
    assert_eq!(student.app.answer, 14.0);
    student.calculate("(2+3)*4");
    assert_eq!(student.app.answer, 20.0);
    eprintln!("NATIVE-E2E PASS S02 fractions, conversion, grouping and operation order");
}

#[test]
fn student_03_independent_curriculum_calculations_through_native_keyboard() {
    let fixture = curriculum_checks::fixture().expect("independent teacher fixtures");
    let cases: Vec<_> = fixture
        .cases
        .iter()
        .filter(|case| case.kind == "calculator")
        .collect();
    assert!(
        cases.len() >= 20,
        "Curriculum keyboard coverage must span substantially more than smoke cases"
    );
    for case in cases {
        let mut student = StudentSession::new(case.level);
        student.app.settings.degrees = case.degrees;
        student.app.device.complex_mode = case.expected.kind == "complex";
        student.idle();
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
            student.calculate(input);
        }
        assert_fixture(case, curriculum_checks::calculator_actual(&student.app));
        if case.expected.kind == "real"
            && let Some(expected) = case.expected.value.as_f64()
            && expected != 0.0
        {
            let display = student.app.result.parse::<f64>().unwrap_or_else(|_| {
                panic!(
                    "{}: numeric answer is not readable as a number: {}",
                    case.id, student.app.result
                )
            });
            assert_ne!(
                display, 0.0,
                "{}: a nonzero scientific result must not disappear as 0",
                case.id
            );
        }
    }
}

#[test]
fn student_04_actual_mode_menu_changes_degree_and_radian_answers() {
    let mut student = StudentSession::new(2);
    student.activate("Calculator mode");
    student.activate("Radians");
    student.activate("Power on or clear");
    student.calculate("sin(pi/2)");
    assert!((student.app.answer - 1.0).abs() < 1e-12);
    assert!(!student.app.settings.degrees);
    student.activate("Calculator mode");
    student.activate("Degrees");
    student.activate("Power on or clear");
    student.calculate("sin(30)");
    assert!((student.app.answer - 0.5).abs() < 1e-12);
    assert!(student.app.settings.degrees);
    student.activate("Calculator mode");
    student.activate("Complex number mode (a+bi)");
    student.activate("Power on or clear");
    student.calculate("sqrt(-1)");
    assert!(!student.app.error);
    assert!(student.app.device.complex_answer().re.abs() < 1e-12);
    assert!((student.app.device.complex_answer().im - 1.0).abs() < 1e-12);
    student.activate("Calculator mode");
    student.activate("Real number mode");
    student.activate("Power on or clear");
    student.calculate("sqrt(-1)");
    assert!(
        student.app.error,
        "Returning to real mode must reject a negative square root"
    );
    eprintln!("NATIVE-E2E PASS S04 physical mode menu and angle conventions");
}

#[test]
fn student_05_on_calculator_graph_trace_and_value_table() {
    let mut student = StudentSession::new(2);
    student.activate("Edit graph functions");
    student.replace("Y1 function on calculator", "x^2");
    student.replace("Y2 function on calculator", "");
    student.activate("Graph on calculator");
    assert_eq!(student.app.device.screen, device::DeviceScreen::Graph);
    student.activate("Trace graph on calculator");
    for _ in 0..4 {
        student.activate("cursor right");
    }
    assert_eq!(student.app.graph_access.trace_x, 2.0);
    assert!(student.app.trace_text().contains("y = 4"));
    student.activate("Toggle second functions");
    student.activate("Graph on calculator");
    assert_eq!(student.app.device.screen, device::DeviceScreen::Table);
    student.activate("Table settings");
    student.replace("Table start", "0");
    student.replace("Table step", "1");
    student.activate("Apply table settings");
    let csv = student
        .app
        .graph_table_csv()
        .expect("table of a valid function");
    let rows: Vec<_> = csv.lines().collect();
    assert!(
        rows.iter()
            .any(|row| row.split(',').collect::<Vec<_>>() == ["2", "4"]),
        "Expected x=2, Y1=4 in table: {csv}"
    );
    eprintln!("NATIVE-E2E PASS S05 embedded graph, trace and table without leaving calculator");
}

#[test]
fn student_06_teacher_graph_and_matrix_cases_use_native_workspaces() {
    let fixture = curriculum_checks::fixture().expect("independent teacher fixtures");
    for case in fixture
        .cases
        .iter()
        .filter(|case| matches!(case.kind.as_str(), "graph" | "matrix"))
    {
        let mut student = StudentSession::new(case.level);
        student.app.settings.degrees = case.degrees;
        if case.kind == "graph" {
            student.workspace("Graph");
            student.replace("Y1 graph function", &case.input);
            let actual = match student.app.graph_value(0, case.x) {
                Ok(value) => Actual::Real {
                    value,
                    display: math::format(value, 10),
                },
                Err(message) => Actual::Error { message },
            };
            assert_fixture(case, actual);
        } else {
            student.workspace("Matrices");
            student.replace("Matrix A", &case.matrix_a.text());
            student.replace("Matrix B", &case.matrix_b.text());
            student.activate("Matrix operation");
            student.activate(&case.operation);
            student.activate("Calculate");
            let matrix: Result<Vec<Vec<f64>>, _> = student
                .app
                .matrix_result
                .lines()
                .map(|line| line.split_whitespace().map(str::parse::<f64>).collect())
                .collect();
            let actual = match matrix {
                Ok(value) if !value.is_empty() => Actual::Matrix { value },
                _ => Actual::Error {
                    message: student.app.matrix_result.clone(),
                },
            };
            assert_fixture(case, actual);
        }
    }
}

#[test]
fn student_07_worked_algebra_and_practice_keep_progress_honest() {
    let mut student = StudentSession::new(2);
    student.workspace("Learn");
    student.replace("Math problem to explain", "2*x+3=11");
    student.activate("Explain my problem");
    student.activate("Show all steps");
    student.activate("Copy worked solution");
    assert!(
        student
            .copied_text()
            .is_some_and(|text| text.contains("x = 4")),
        "The local worked lesson must solve the entered equation"
    );

    let mut student = StudentSession::new(0);
    student.workspace("Practice");
    let question = student.app.practice.current_question();
    let numbers: Vec<i32> = question
        .prompt
        .split_whitespace()
        .filter_map(|word| word.parse().ok())
        .collect();
    assert_eq!(
        numbers.len(),
        2,
        "This workflow begins with an observable two-number addition prompt"
    );
    let independently_added = numbers.iter().sum::<i32>().to_string();
    student.replace("Practice answer", "not a number");
    student.activate("Check answer");
    assert_eq!(student.app.practice.current_session().progress.attempted, 0);
    student.replace("Practice answer", &independently_added);
    student.activate("Check answer");
    assert_eq!(
        student.app.practice.current_session().feedback_kind,
        practice::FeedbackKind::Correct
    );
    assert_eq!(student.app.practice.current_session().progress.correct, 1);
    student.activate("Check answer");
    assert_eq!(
        student.app.practice.current_session().progress.attempted,
        1,
        "Repeated checks cannot inflate progress"
    );
    student.activate("Next question");
    student.activate("Show a hint");
    student.activate("Reveal answer & method");
    assert!(student.app.practice.current_session().hint_shown);
    assert_eq!(
        student.app.practice.current_session().progress.correct,
        1,
        "Revealing a solution cannot add a correct answer"
    );
    eprintln!(
        "NATIVE-E2E PASS S07 local lesson, invalid input, independent addition and no score inflation"
    );
}

#[test]
fn student_08_physics_and_chemistry_forms_use_teacher_expected_results() {
    let fixture = curriculum_checks::fixture().expect("independent teacher fixtures");
    for tool_id in [
        "newton-force",
        "moles-particles",
        "photon-energy",
        "series-rlc",
    ] {
        let case = fixture
            .cases
            .iter()
            .find(|case| {
                case.kind == "science" && case.tool_id == tool_id && case.expected.kind == "real"
            })
            .unwrap_or_else(|| panic!("Missing independent science form case: {tool_id}"));
        let tool = crate::science::TOOLS
            .iter()
            .find(|tool| tool.id == tool_id)
            .expect("science tool exists");
        let mut student = StudentSession::new(case.level);
        student.workspace("Tools");
        student.activate("Physics & chemistry");
        student.activate("All");
        student.activate(tool.name);
        assert_eq!(
            tool.fields.len(),
            case.inputs.len(),
            "{}: every science field needs an independent fixture input",
            case.id
        );
        for (field, value) in tool.fields.iter().zip(&case.inputs) {
            student.replace(field.label, value);
        }
        student.activate("Calculate science");
        let report = student
            .app
            .toolkit
            .science
            .report
            .as_ref()
            .expect("Calculate science must create a result")
            .as_ref()
            .expect("valid science input must succeed");
        assert_fixture(
            case,
            Actual::Science {
                values: report.values.iter().map(|value| value.value).collect(),
                units: report
                    .values
                    .iter()
                    .map(|value| value.unit.to_string())
                    .collect(),
            },
        );
        let visible_result = report.text(student.app.settings.precision);
        let expected_unit = match tool_id {
            "newton-force" => "N",
            "moles-particles" => "specified entities",
            "photon-energy" => "J",
            "series-rlc" => "ohm",
            _ => unreachable!("the form cases have explicit dimensional expectations"),
        };
        assert_eq!(
            report.values[0].unit, expected_unit,
            "{}: result unit must match the physical quantity",
            case.id
        );
        let rendered: Vec<String> = student
            .output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("science tree")
            .nodes
            .iter()
            .flat_map(|(_, node)| {
                [node.label(), node.value()]
                    .into_iter()
                    .flatten()
                    .map(str::to_owned)
            })
            .collect();
        assert!(
            rendered.iter().any(|label| label.contains(&visible_result)),
            "{}: expected result must be present on the actual form: {rendered:?}",
            case.id
        );
        eprintln!(
            "NATIVE-E2E PASS {} science selection, labeled field entry and displayed result",
            case.id
        );
    }
}

#[test]
fn student_09_notebook_typed_path_roundtrip_and_bad_file_preserve_work() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("opencalc-student-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("isolated test directory");
    let path = directory.join("homework.opencalc.json");
    let malformed = directory.join("damaged.opencalc.json");
    std::fs::write(&malformed, b"{broken notebook").expect("malformed fixture");
    let mut student = StudentSession::new(3);
    student.calculate("12+8");
    student.activate("Notebook");
    student.replace("Notebook title", "Chemistry homework");
    student.replace(
        "Notebook notes",
        "Keep units and assumptions with each calculation.",
    );
    student.activate("Type a file path");
    student.replace("Notebook file path", path.to_str().expect("test path"));
    student.activate("Save to typed path");
    student.wait_until("typed-path notebook save", |app| {
        app.message.contains("saved")
    });
    assert!(path.is_file());
    let saved = std::fs::read(&path).expect("saved notebook bytes");
    let notebook =
        notebook::Notebook::decode(&saved).expect("the file decodes with production validation");
    assert_eq!(notebook.title, "Chemistry homework");
    student.activate("Close notebook panel");
    student.calculate("99+1");
    student.activate("Notebook");
    student.replace("Notebook file path", path.to_str().expect("test path"));
    student.activate("Open typed path");
    assert!(student.has_control("Cancel"));
    student.activate("Cancel");
    assert_eq!(
        student.app.answer, 100.0,
        "Cancel must preserve unsaved homework"
    );
    student.activate("Open typed path");
    student.activate("Discard changes");
    student.wait_until("notebook restores calculations", |app| {
        app.expression == "12+8"
    });
    assert_eq!(student.app.answer, 20.0);
    assert_eq!(
        student.app.notebook.notes,
        "Keep units and assumptions with each calculation."
    );
    student.activate("Notebook");
    if !student.has_control("Notebook file path") {
        student.activate("Type a file path");
    }
    student.replace("Notebook file path", malformed.to_str().expect("test path"));
    student.activate("Open typed path");
    if student.has_control("Discard changes") {
        student.activate("Discard changes");
    }
    student.wait_until("damaged notebook rejected", |app| {
        app.message
            .contains("Could not read this OpenCalc notebook")
    });
    assert_eq!(student.app.expression, "12+8");
    assert_eq!(student.app.answer, 20.0);
    assert_eq!(
        student.app.notebook.notes,
        "Keep units and assumptions with each calculation."
    );
    std::fs::remove_file(path).expect("remove synthetic saved notebook");
    std::fs::remove_file(malformed).expect("remove synthetic malformed notebook");
    std::fs::remove_dir(directory).expect("remove isolated test directory");
    eprintln!(
        "NATIVE-E2E PASS S09 typed-path save/reopen, unsaved Cancel, damaged-file preservation"
    );
}

#[test]
fn student_10_notebook_pages_text_shapes_ink_undo_and_file_roundtrip() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("opencalc-pages-{}-{unique}", std::process::id()));
    std::fs::create_dir(&directory).expect("isolated canvas directory");
    let path = directory.join("lab-notes.opencalc.json");
    let mut student = StudentSession::new(3);
    student.workspace("Notebook pages");
    student.replace("Page title", "Particle count lab");
    student.activate("Text");
    student.replace("Object text", "n = N / N_A");
    student.replace(
        "Object description",
        "Amount of substance from particle count",
    );
    student.activate("Add text");
    student.activate("Rectangle");
    student.replace("Object X percent", "10");
    student.replace("Object Y percent", "30");
    student.replace("Object width percent", "40");
    student.replace("Object height percent", "20");
    student.replace("Object description", "Sample container");
    student.activate("Add shape");
    student.replace("Object X percent", "20");
    student.replace("Object Y percent", "40");
    student.replace("Object description", "Two-point sketch");
    student.activate("Add ink point");
    student.replace("Object X percent", "30");
    student.replace("Object Y percent", "50");
    student.activate("Add ink point");
    student.activate("Finish ink stroke");
    let page = &student.app.canvas.notebook_snapshot().pages[0];
    assert_eq!(page.elements.len(), 3);
    assert!(
        matches!(&page.elements[0], canvas::CanvasElement::Text { text, .. } if text == "n = N / N_A")
    );
    assert!(matches!(
        &page.elements[1],
        canvas::CanvasElement::Rectangle { .. }
    ));
    assert!(
        matches!(&page.elements[2], canvas::CanvasElement::Ink { points, .. } if points.len() == 2)
    );
    student.activate("Undo");
    assert_eq!(
        student.app.canvas.notebook_snapshot().pages[0]
            .elements
            .len(),
        2
    );
    student.activate("Redo");
    assert_eq!(
        student.app.canvas.notebook_snapshot().pages[0]
            .elements
            .len(),
        3
    );
    student.activate("Add page");
    student.replace("Page title", "Results");
    let original = student.app.canvas.notebook_snapshot();
    assert_eq!(original.pages.len(), 2);
    assert_eq!(original.pages[1].title, "Results");
    student.activate("Notebook file");
    student.activate("Type a file path");
    student.replace("Notebook file path", path.to_str().expect("canvas path"));
    student.activate("Save to typed path");
    student.wait_until("canvas notebook saved", |app| {
        app.message.contains("Notebook saved")
    });
    student.activate("Close notebook panel");
    student.replace("Page title", "Unsaved scratch");
    student.activate("Notebook file");
    student.activate("Open typed path");
    student.activate("Discard changes");
    student.wait_until("canvas notebook reopened", |app| {
        app.canvas.notebook_snapshot().pages[1].title == "Results"
    });
    assert_eq!(
        student.app.canvas.notebook_snapshot(),
        original,
        "Pages and their exact typed objects must survive a native save/open workflow"
    );
    std::fs::remove_file(path).expect("remove synthetic canvas notebook");
    std::fs::remove_dir(directory).expect("remove isolated canvas directory");
    eprintln!("NATIVE-E2E PASS S10 notebook pages, typed text/shapes/ink, undo/redo and save/open");
}

#[test]
fn student_11_restart_restores_opt_in_recovery_through_native_prompt() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "opencalc-returning-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("isolated recovery directory");
    let latest = directory.join("recovery").join("latest.opencalc.json");
    let mut student = StudentSession::new(3);
    student.app.data_dir = Some(directory.clone());
    student.app.recovery = recovery::RecoveryManager::new(Some(directory.clone()));
    student.calculate("6.02214076e23*2");
    student.app.settings.protect_unsaved_work = true;
    student.idle();
    student.wait_until("opt-in recovery checkpoint", |_| latest.is_file());
    student.wait_until("recovery worker finished", |app| !app.recovery.busy());
    drop(student);

    let mut returning = StudentSession::new(3);
    returning.app.data_dir = Some(directory.clone());
    returning.app.recovery = recovery::RecoveryManager::new(Some(directory.clone()));
    returning.app.settings.protect_unsaved_work = true;
    let start = Instant::now();
    while !returning.has_control("Restore recovery copy") {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "Recovery prompt must appear after reopening"
        );
        returning.idle();
        std::thread::yield_now();
    }
    assert_eq!(
        returning.app.expression, "",
        "Discovery must not replace current work before a choice"
    );
    returning.activate("Restore recovery copy");
    returning.wait_until("recovered calculation", |app| {
        app.expression == "6.02214076e23*2"
    });
    assert_eq!(returning.app.answer, 1.204_428_152e24);
    returning.activate("Notebook");
    assert!(returning.has_control("Notebook title"));
    let labels: Vec<_> = returning
        .output
        .platform_output
        .accesskit_update
        .as_ref()
        .expect("recovery notebook tree")
        .nodes
        .iter()
        .flat_map(|(_, node)| [node.label(), node.value()].into_iter().flatten())
        .collect();
    assert!(
        labels.iter().any(|text| text.contains("Unsaved changes")),
        "Recovered work must require an explicit save"
    );
    returning.app.settings.protect_unsaved_work = false;
    returning.idle();
    returning.wait_until("recovery cleanup complete", |app| !app.recovery.busy());
    drop(returning);
    let recovery_directory = directory.join("recovery");
    for entry in std::fs::read_dir(&recovery_directory).expect("synthetic recovery files") {
        std::fs::remove_file(entry.expect("recovery entry").path())
            .expect("remove isolated recovery file");
    }
    std::fs::remove_dir(recovery_directory).expect("remove isolated recovery folder");
    std::fs::remove_dir(directory).expect("remove isolated returning-user directory");
    eprintln!(
        "NATIVE-E2E PASS S11 opt-in checkpoint, fresh session prompt and native restore as unsaved work"
    );
}
