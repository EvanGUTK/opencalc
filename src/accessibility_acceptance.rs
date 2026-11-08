//! Native software acceptance evidence. These tests do not emulate a disability
//! or establish compatibility with a physical assistive-technology setup.
use crate::accessibility::{AccessibilitySettings, CalculatorLayout, ColorTheme, ScanMode};
use crate::{EqualApp, Settings, Tab, egui};
use egui::accesskit::{Action, ActionRequest, Live, Node, Role};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Session {
    app: EqualApp,
    ctx: egui::Context,
    output: egui::FullOutput,
    size: egui::Vec2,
    now: f64,
}

impl Session {
    fn new(settings: Settings, size: egui::Vec2) -> Self {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(settings, None);
        app.show_workspace = true;
        let mut session = Self {
            app,
            ctx,
            output: egui::FullOutput::default(),
            size,
            now: 0.0,
        };
        for _ in 0..3 {
            session.frame(vec![]);
        }
        session
    }

    fn frame(&mut self, events: Vec<egui::Event>) {
        self.now += 0.02;
        self.app.theme(&self.ctx);
        let modifiers = events
            .iter()
            .find_map(|event| match event {
                egui::Event::Key { modifiers, .. } => Some(*modifiers),
                _ => None,
            })
            .unwrap_or_default();
        self.output = self.ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    self.size / self.ctx.zoom_factor(),
                )),
                time: Some(self.now),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| {
                self.app.poll_notebook(ctx);
                self.app.handle_keyboard_input(ctx);
                self.app.draw(ctx);
                self.app.windows(ctx);
                self.app.accessibility_help_windows(ctx);
                self.app.notebook_windows(ctx);
                self.app.sync_overlay_focus(ctx);
            },
        );
    }

    fn node(&self, label: &str) -> &Node {
        &self
            .output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("native tree")
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("Missing native control: {label}"))
            .1
    }

    fn action(&mut self, label: &str, action: Action) {
        let target = self
            .output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("Missing action target: {label}"))
            .0;
        self.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
            target,
            action,
            data: None,
        })]);
    }

    fn key(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        for pressed in [true, false] {
            self.frame(vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            }]);
        }
    }

    fn command(&mut self, key: egui::Key) {
        self.key(
            key,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        );
    }

    fn replace_expression(&mut self, text: &str) {
        self.replace_text("Calculation expression", text);
        self.key(egui::Key::Enter, egui::Modifiers::NONE);
    }

    fn replace_text(&mut self, label: &str, text: &str) {
        self.action(label, Action::Focus);
        self.frame(vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Text(text.into()),
        ]);
    }

    fn tab_to(&mut self, label: &str) {
        for _ in 0..200 {
            let tree = self
                .output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap();
            if tree
                .nodes
                .iter()
                .any(|(id, node)| *id == tree.focus && node.label() == Some(label))
            {
                return;
            }
            self.key(egui::Key::Tab, egui::Modifiers::NONE);
        }
        panic!("Keyboard traversal did not reach {label}");
    }

    fn status(&self, prefix: &str) -> &Node {
        self.output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| {
                node.role() == Role::Status
                    && node.label().is_some_and(|label| label.starts_with(prefix))
            })
            .unwrap_or_else(|| panic!("Missing visible live status: {prefix}"))
    }

    fn window_contains_status(&self, window: &str, status: &str) -> bool {
        let tree = self
            .output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap();
        let Some((root, _)) = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Window && node.label() == Some(window))
        else {
            return false;
        };
        let nodes: std::collections::HashMap<_, _> =
            tree.nodes.iter().map(|(id, node)| (*id, node)).collect();
        let mut pending = vec![*root];
        let mut visited = std::collections::HashSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                continue;
            }
            if let Some(node) = nodes.get(&id) {
                if node.role() == Role::Status && node.label() == Some(status) {
                    return true;
                }
                pending.extend_from_slice(node.children());
            }
        }
        false
    }

    fn settle_file(&mut self, expected: &str) {
        for _ in 0..1000 {
            self.frame(vec![]);
            if self.app.message.starts_with(expected) {
                return;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        panic!("File action did not reach {expected}: {}", self.app.message);
    }
}

fn accessible_settings() -> Settings {
    Settings {
        level: 3,
        access: AccessibilitySettings {
            layout: CalculatorLayout::Accessible,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn combined_magnification_contrast_text_and_target_settings_keep_named_keys_and_math() {
    for (scale, font_scale, theme) in [
        (1.0, 1.0, ColorTheme::Light),
        (2.0, 1.4, ColorTheme::Warm),
        (4.0, 1.8, ColorTheme::Dark),
    ] {
        let settings = Settings {
            scale,
            contrast: true,
            readable: true,
            access: AccessibilitySettings {
                layout: CalculatorLayout::Accessible,
                theme,
                font_scale,
                target_size: 80.0,
                key_spacing: 24.0,
                left_handed: true,
                word_labels: true,
                large_controls: true,
                line_spacing: 2.0,
                ..Default::default()
            },
            ..accessible_settings()
        };
        let mut session = Session::new(settings, egui::vec2(480.0, 900.0));
        for label in ["7", "Add", "5", "Calculate answer"] {
            session.action(label, Action::Click);
        }
        assert_eq!(
            session.app.result, "12",
            "Combined preferences at {scale}x must preserve the calculation"
        );
        session.frame(vec![]);
        let width = 480.0 / scale;
        for label in [
            "7",
            "Multiply",
            "Calculate answer",
            "Backspace",
            "Undo editing",
        ] {
            let bounds = session.node(label).bounds().unwrap();
            assert!(
                bounds.x0 >= -0.5 && bounds.x1 <= f64::from(width) + 0.5,
                "{label} exceeds {width}pt at {scale}x/{font_scale} text: {bounds:?}"
            );
            assert!(
                bounds.width() >= 80.0 && bounds.height() >= 80.0,
                "Target shrank: {label}: {bounds:?}"
            );
        }
        assert_eq!(session.status("Answer: 12").live(), Some(Live::Polite));
    }
}

#[test]
fn nonvisual_names_error_recovery_and_focus_are_available_without_audio() {
    let mut session = Session::new(accessible_settings(), egui::vec2(800.0, 900.0));
    session.replace_expression("1/0");
    assert!(session.app.error);
    let error = session.status("Calculation error:");
    assert_eq!(error.live(), Some(Live::Polite));
    assert!(error.is_live_atomic());
    assert_eq!(
        session.ctx.memory(|memory| memory.focused()),
        Some(egui::Id::new("expression"))
    );
    session.replace_expression("8+4");
    assert_eq!(session.app.result, "12");
    assert!(!session.app.error);
    assert!(session.status("Answer: 12").is_live_atomic());
    assert!(!session.app.speech.is_active());
}

#[test]
fn keyboard_only_calculate_settings_and_cancel_preserve_editor_work() {
    let mut session = Session::new(accessible_settings(), egui::vec2(800.0, 900.0));
    session.tab_to("Calculation expression");
    session.frame(vec![egui::Event::Text("12/3".into())]);
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    assert_eq!(session.app.result, "4");
    session.command(egui::Key::Comma);
    assert!(session.app.show_settings);
    session.key(egui::Key::Escape, egui::Modifiers::NONE);
    assert!(!session.app.show_settings);
    assert_eq!(session.app.expression, "12/3");
    assert_eq!(
        session.ctx.memory(|memory| memory.focused()),
        Some(egui::Id::new("expression"))
    );
}

#[test]
fn repeat_guard_reports_rejected_activation_and_keeps_later_deliberate_input() {
    let mut settings = accessible_settings();
    settings.access.guard_ms = 500;
    let mut session = Session::new(settings, egui::vec2(800.0, 900.0));
    session.now = 1.0;
    session.action("7", Action::Click);
    session.now = 1.1;
    session.action("8", Action::Click);
    assert_eq!(session.app.expression, "7");
    session.frame(vec![]);
    assert!(
        session
            .status("Repeated activation ignored.")
            .label()
            .unwrap()
            .contains("input guard")
    );
    session.now = 1.7;
    session.action("8", Action::Click);
    assert_eq!(session.app.expression, "78");
}

#[test]
fn dwell_requires_a_full_stationary_hold_and_leave_before_repeating() {
    let mut settings = accessible_settings();
    settings.access.dwell_ms = 600;
    settings.access.target_size = 80.0;
    let mut session = Session::new(settings, egui::vec2(800.0, 900.0));
    let bounds = session.node("7").bounds().unwrap();
    let point = egui::pos2(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    session.now = 1.0;
    session.frame(vec![egui::Event::PointerMoved(point)]);
    session.now = 1.5;
    session.frame(vec![]);
    assert!(session.app.expression.is_empty());
    session.now = 1.7;
    session.frame(vec![]);
    assert_eq!(session.app.expression, "7");
    session.now = 3.0;
    session.frame(vec![]);
    assert_eq!(
        session.app.expression, "7",
        "Remaining over the target must not repeat"
    );
    session.frame(vec![egui::Event::PointerGone]);
    let after_leave = session.app.input_access.dwell_evidence();
    let bounds = session.node("7").bounds().unwrap();
    let point = egui::pos2(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    session.now = 3.2;
    session.frame(vec![egui::Event::PointerMoved(point)]);
    let after_move = session.app.input_access.dwell_evidence();
    session.now = 3.9;
    session.frame(vec![]);
    assert_eq!(
        session.app.expression,
        "77",
        "after_leave={after_leave}; after_move={after_move}; after_hold={}; point={point:?}; seven={:?}; focus={:?}",
        session.app.input_access.dwell_evidence(),
        session.node("7").bounds(),
        session.ctx.memory(|memory| memory.focused())
    );
}

#[test]
fn switch_workflow_starts_paused_selects_a_key_and_pauses_behind_help() {
    let mut settings = accessible_settings();
    settings.access.scan_mode = ScanMode::Automatic;
    settings.access.scan_interval_ms = 600;
    let mut session = Session::new(settings, egui::vec2(800.0, 1000.0));
    assert!(!session.app.input_access.is_running());
    session.key(egui::Key::Space, egui::Modifiers::NONE);
    assert!(session.app.input_access.is_running());
    session.now += 0.7;
    session.frame(vec![]);
    session.key(egui::Key::Space, egui::Modifiers::NONE);
    session.key(egui::Key::Space, egui::Modifiers::NONE);
    assert_eq!(session.app.expression, "7");
    assert!(
        session.app.input_access.is_running(),
        "A selected key must not steal switch focus into the expression"
    );
    session.key(egui::Key::Space, egui::Modifiers::NONE);
    session.key(egui::Key::Space, egui::Modifiers::NONE);
    assert_eq!(
        session.app.expression, "77",
        "The next row/key choice must remain usable"
    );
    session.key(egui::Key::F1, egui::Modifiers::NONE);
    assert!(session.app.a11y.show_shortcuts);
    assert!(!session.app.input_access.is_running());
    session.now += 60.0;
    session.frame(vec![]);
    assert_eq!(session.app.expression, "77");
    session.key(egui::Key::Escape, egui::Modifiers::NONE);
    assert!(!session.app.input_access.is_running());
}

#[test]
fn quiet_result_error_and_clipboard_feedback_have_text_equivalents() {
    let mut settings = accessible_settings();
    settings.access.speech_volume = 0;
    settings.access.captions = true;
    let mut session = Session::new(settings, egui::vec2(800.0, 900.0));
    session.replace_expression("5*6");
    assert_eq!(session.status("Answer: 30").role(), Role::Status);
    session.key(egui::Key::Tab, egui::Modifiers::NONE);
    session.frame(vec![egui::Event::Copy]);
    assert!(
        session
            .output
            .platform_output
            .commands
            .iter()
            .any(|command| matches!(command, egui::OutputCommand::CopyText(text) if text == "30"))
    );
    assert!(!session.app.speech.is_active());
    session.replace_expression("sqrt(-1)");
    assert!(session.app.error);
    assert!(
        session.status("Calculation error:").label().unwrap().len() > "Calculation error:".len()
    );
}

#[test]
fn reading_spacing_word_labels_and_decimal_conventions_preserve_number_meaning() {
    let mut settings = accessible_settings();
    settings.readable = true;
    settings.access.font_scale = 1.4;
    settings.access.line_spacing = 1.8;
    settings.access.word_labels = true;
    settings.access.decimal_comma = true;
    let mut session = Session::new(settings, egui::vec2(800.0, 1000.0));
    session.replace_expression("1,5+2,5");
    assert_eq!(session.app.answer, 4.0);
    assert_eq!(session.node("Divide").role(), Role::Button);
    assert_eq!(session.node("Square root").role(), Role::Button);
    assert_eq!(
        session.ctx.style().text_styles[&egui::TextStyle::Body].family,
        egui::FontFamily::Monospace
    );
    let paragraph = crate::appearance::paragraph(&session.ctx, "12 and 21");
    let job = egui::WidgetText::from(paragraph).into_layout_job(
        &session.ctx.style(),
        egui::FontSelection::Default,
        egui::Align::Min,
    );
    assert!(job.sections.iter().all(|section| {
        section
            .format
            .line_height
            .is_some_and(|height| height > 40.0)
    }));
}

#[test]
fn low_distraction_practice_does_not_advance_or_score_from_elapsed_time() {
    let mut settings = accessible_settings();
    settings.access.focus_mode = true;
    settings.access.show_scores = false;
    settings.access.show_help_text = false;
    settings.access.one_step = true;
    let mut session = Session::new(settings, egui::vec2(800.0, 900.0));
    session.app.tab = Tab::Practice;
    session.app.practice.set_level(session.app.settings.level);
    session.app.practice.current_session_mut().response = "unfinished answer".into();
    session.frame(vec![]);
    let question = session.app.practice.current_question().problem_text();
    let progress = session.app.practice.current_session().progress;
    session.now += 3600.0;
    session.frame(vec![]);
    assert_eq!(
        session.app.practice.current_question().problem_text(),
        question
    );
    assert_eq!(session.app.practice.current_session().progress, progress);
    assert_eq!(
        session.app.practice.current_session().response,
        "unfinished answer"
    );
}

#[test]
fn glossary_actions_identify_the_term_and_copy_status_is_visible_inside_the_window() {
    let mut session = Session::new(accessible_settings(), egui::vec2(800.0, 1000.0));
    session.app.a11y.glossary_search = "sine".into();
    session.command(egui::Key::G);
    session.frame(vec![]);
    assert_eq!(session.node("Read Sine").role(), Role::Button);
    session.action("Copy Sine", Action::Click);
    assert!(session.output.platform_output.commands.iter().any(|command| matches!(command, egui::OutputCommand::CopyText(text) if text.starts_with("Sine."))));
    session.frame(vec![]);
    assert_eq!(
        session.status("Sine definition copied.").live(),
        Some(Live::Polite)
    );
    assert!(
        session.window_contains_status("Math glossary", "Sine definition copied."),
        "Copy feedback must be inside the glossary, not only the background caption"
    );
    session.key(egui::Key::Escape, egui::Modifiers::NONE);
    assert!(!session.app.a11y.show_glossary);
}

#[test]
fn science_native_units_error_recovery_keyboard_calculate_and_copy_remain_textual() {
    let mut session = Session::new(accessible_settings(), egui::vec2(1000.0, 1200.0));
    session.app.tab = Tab::Toolkit;
    session.app.toolkit.science_open = true;
    session.frame(vec![]);
    session.replace_text("Mass (kg)", "-1");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    let error = session.status("Check science inputs:");
    assert_eq!(error.live(), Some(Live::Polite));
    assert!(error.label().unwrap().contains("Mass"));
    session.replace_text("Mass (kg)", "4");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    let report = session.status("Newton's second law");
    assert!(report.label().unwrap().contains("Net force: 12 N"));
    assert!(report.label().unwrap().contains("Assumptions:"));
    assert!(report.is_live_atomic());
    session.action("Copy science result", Action::Click);
    assert!(session.output.platform_output.commands.iter().any(|command| matches!(command, egui::OutputCommand::CopyText(text) if text.contains("Net force: 12 N"))));
    assert!(!session.app.speech.is_active());
}

#[test]
fn science_inputs_and_actions_fit_combined_four_hundred_percent_preferences() {
    let settings = Settings {
        scale: 4.0,
        contrast: true,
        readable: true,
        access: AccessibilitySettings {
            layout: CalculatorLayout::Accessible,
            font_scale: 1.8,
            line_spacing: 2.0,
            large_controls: true,
            ..Default::default()
        },
        ..accessible_settings()
    };
    let mut session = Session::new(settings, egui::vec2(480.0, 900.0));
    session.app.tab = Tab::Toolkit;
    session.app.toolkit.science_open = true;
    session.frame(vec![]);
    session.action("Choose science tool", Action::Click);
    let search = session.node("Find a science tool").bounds().unwrap();
    assert!(
        search.x0 >= -0.5 && search.x1 <= 120.5,
        "Science search must fit 120 logical points: {search:?}"
    );
    session.action("Choose science tool", Action::Click);
    session.action("Calculate science", Action::Click);
    for label in [
        "Mass (kg)",
        "Acceleration (m/s²)",
        "Calculate science",
        "Copy science result",
        "Math tools",
    ] {
        let bounds = session.node(label).bounds().unwrap();
        assert!(
            bounds.x0 >= -0.5 && bounds.x1 <= 120.5,
            "Science control {label} must fit 120 logical points: {bounds:?}"
        );
    }
    assert!(
        session
            .status("Newton's second law")
            .label()
            .unwrap()
            .contains("Net force: 6 N")
    );
}

#[test]
fn science_narrow_inputs_remain_visible_and_keyboard_browsing_selects_a_tool() {
    let settings = Settings {
        scale: 1.25,
        ..accessible_settings()
    };
    let mut session = Session::new(settings, egui::vec2(360.0, 800.0));
    session.app.tab = Tab::Toolkit;
    session.app.toolkit.science_open = true;
    for _ in 0..3 {
        session.frame(vec![]);
    }
    for label in ["Mass (kg)", "Acceleration (m/s²)", "Calculate science"] {
        let bounds = session.node(label).bounds().unwrap();
        assert!(
            bounds.x0 >= -0.5 && bounds.x1 <= 288.5 && bounds.y0 >= -0.5 && bounds.y1 <= 640.5,
            "Default science input/action must be visible in360x800 at125%: {label}: {bounds:?}"
        );
    }
    assert!(
        !session
            .output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Find a science tool"))
    );
    session.tab_to("Choose science tool");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    session.replace_text("Find a science tool", "Ohm");
    session.tab_to("Ohm's law");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    session.frame(vec![]);
    assert!(
        !session
            .output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Find a science tool")),
        "Selecting a tool must return to its current inputs"
    );
    session.node("Voltage (V)");
    session.tab_to("Resistance (ohm)");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    assert!(session.status("Ohm's law").label().unwrap().contains("2 A"));
}

#[test]
fn notebook_pages_keyboard_text_description_undo_redo_and_svg_need_no_drawing_gesture() {
    let mut session = Session::new(accessible_settings(), egui::vec2(1000.0, 1200.0));
    session.key(egui::Key::Num7, egui::Modifiers::ALT);
    assert_eq!(session.app.tab, Tab::Notebook);
    session.tab_to("Coordinates and elements");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    session.tab_to("Object text");
    session.frame(vec![egui::Event::Text("Area = length × width".into())]);
    session.tab_to("Object description");
    session.frame(vec![egui::Event::Text(
        "Rectangle area relationship".into(),
    )]);
    session.tab_to("Add text");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    let snapshot = session.app.canvas.notebook_snapshot();
    assert_eq!(snapshot.pages[0].elements.len(), 1);
    let summary = snapshot.pages[0].elements[0].summary();
    assert!(
        summary.contains("Area = length × width")
            && summary.contains("Rectangle area relationship")
    );
    assert_eq!(
        session.node(&format!("Object 1: {summary}")).role(),
        Role::Button
    );
    session.command(egui::Key::Z);
    assert!(
        session.app.canvas.notebook_snapshot().pages[0]
            .elements
            .is_empty()
    );
    session.key(
        egui::Key::Z,
        egui::Modifiers {
            ctrl: true,
            command: true,
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(
        session.app.canvas.notebook_snapshot().pages[0]
            .elements
            .len(),
        1
    );
    session.tab_to("Copy page SVG");
    session.frame(vec![egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    assert!(session.output.platform_output.commands.iter().any(|command| matches!(command, egui::OutputCommand::CopyText(text) if text.contains("<svg") && text.contains("Area = length × width") && text.contains("Rectangle area relationship"))));
}

#[test]
fn notebook_pages_controls_and_coordinate_alternatives_fit_combined_magnification() {
    let settings = Settings {
        scale: 4.0,
        contrast: true,
        readable: true,
        access: AccessibilitySettings {
            layout: CalculatorLayout::Accessible,
            font_scale: 1.8,
            line_spacing: 2.0,
            large_controls: true,
            ..Default::default()
        },
        ..accessible_settings()
    };
    let mut session = Session::new(settings, egui::vec2(480.0, 900.0));
    session.app.tab = Tab::Notebook;
    session.frame(vec![]);
    session.action("Page tools", Action::Click);
    session.action("Coordinates and elements", Action::Click);
    for label in [
        "Page title",
        "Add page",
        "Ink",
        "Eraser",
        "Text",
        "Undo",
        "Coordinates and elements",
        "Object text",
        "Object description",
        "Add text",
        "Copy page SVG",
    ] {
        let bounds = session.node(label).bounds().unwrap();
        assert!(
            bounds.x0 >= -0.5 && bounds.x1 <= 120.5,
            "Notebook page control {label} must fit 120 logical points: {bounds:?}"
        );
    }
    assert!(
        session.app.canvas.notebook_snapshot().pages[0]
            .elements
            .is_empty()
    );
}

struct OwnedNotebook(PathBuf);
impl Drop for OwnedNotebook {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn keyboard_save_new_and_open_cancellation_keep_work_and_access_preferences() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file = OwnedNotebook(std::env::temp_dir().join(format!(
        "opencalc-access-{}-{stamp}.opencalc.json",
        std::process::id()
    )));
    let mut settings = accessible_settings();
    settings.readable = true;
    settings.access.word_labels = true;
    let mut session = Session::new(settings, egui::vec2(900.0, 1100.0));
    session.replace_expression("12/3");
    session
        .app
        .save_notebook_to_path(&session.ctx, file.0.clone());
    session.settle_file("Notebook saved");
    session.replace_expression("8*9");
    session.command(egui::Key::S);
    session.settle_file("Notebook saved");
    session.replace_expression("99+1");
    for key in [egui::Key::N, egui::Key::O] {
        session.command(key);
        session.node("Save first");
        session.key(egui::Key::Escape, egui::Modifiers::NONE);
        assert_eq!(session.app.result, "100");
        assert_eq!(session.app.expression, "99+1");
    }
    session.app.open_notebook_path(&session.ctx, file.0.clone());
    session.frame(vec![]);
    session.tab_to("Discard changes");
    session.key(egui::Key::Enter, egui::Modifiers::NONE);
    session.settle_file("Notebook opened");
    assert_eq!(session.app.expression, "8*9");
    assert_eq!(session.app.result, "72");
    assert!(session.app.settings.readable && session.app.settings.access.word_labels);
    assert_eq!(
        session.app.settings.access.layout,
        CalculatorLayout::Accessible
    );
}
