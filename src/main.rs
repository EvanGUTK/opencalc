mod accessibility;
mod accessibility_help;
mod accessibility_panel;
mod accessible_calculator;
mod appearance;
mod audio_trace;
mod complex;
mod device;
mod graph_tools;
mod handheld;
mod layout;
mod learning;
mod math;
mod matlab;
mod practice;
mod programs;
mod speech;
mod toolkit;
mod tutor;
mod ui;
mod workspaces;
use eframe::egui::{self, Color32 as C, RichText as R};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const TEAL: C = C::from_rgb(21, 105, 91);
const INK: C = C::from_rgb(31, 47, 44);
const MUTED: C = C::from_rgb(78, 91, 85);
const BG: C = C::from_rgb(245, 247, 242);
const LINE: C = C::from_rgb(222, 229, 220);
const HANDHELD_FUNCTIONS: [&str; 2] = ["0.75*(x-0.5)^3-2.25*(x-0.5)+1.15", "-(x-1)^2+3"];
const LEVELS: [&str; 4] = ["K–3", "4–8", "9–12", "Advanced"];
const DESCS: [&str; 4] = [
    "Addition, subtraction, counting, place value, and number lines.",
    "Multiplication, division, fractions, decimals, percentages, and ratios.",
    "Equations, geometry, trigonometry, and functions.",
    "Calculus, matrices, determinants, and linear systems.",
];

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    level: usize,
    degrees: bool,
    contrast: bool,
    readable: bool,
    color_safe: bool,
    speech: bool,
    scale: f32,
    precision: usize,
    access: accessibility::AccessibilitySettings,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            level: 2,
            degrees: true,
            contrast: false,
            readable: false,
            color_safe: false,
            speech: false,
            scale: 1.0,
            precision: 8,
            access: accessibility::AccessibilitySettings::default(),
        }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum Tab {
    Calculator,
    Graph,
    Matrix,
    Tutor,
    Toolkit,
    Practice,
}
enum AppShortcut {
    Help,
    Settings,
    Glossary,
    History,
    Workspace(Tab),
    Undo,
    Redo,
    CopyAnswer,
    StopAudio,
    Guide(f32),
}
pub(crate) fn text_input_focused(ctx: &egui::Context) -> bool {
    ctx.memory(|memory| memory.focused())
        .is_some_and(|id| egui::text_edit::TextEditState::load(ctx, id).is_some())
}
struct EqualApp {
    settings: Settings,
    tab: Tab,
    expression: String,
    answer: f64,
    result: String,
    error: bool,
    history: Vec<(String, String)>,
    show_history: bool,
    show_help: bool,
    learning: learning::LearningSupport,
    tutor: tutor::Tutor,
    toolkit: toolkit::Toolkit,
    practice: practice::Practice,
    graph_a: String,
    graph_b: String,
    graph_error: String,
    surface: bool,
    table: bool,
    range: f64,
    center: [f64; 2],
    matrix_a: String,
    matrix_b: String,
    matrix_op: String,
    matrix_result: String,
    tool: bool,
    tool_kind: usize,
    tool_a: f64,
    tool_b: f64,
    tool_c: f64,
    message: String,
    screenshot: Option<PathBuf>,
    frame_count: u32,
    screenshot_requested: bool,
    second: bool,
    show_workspace: bool,
    handheld_preview: bool,
    show_settings: bool,
    device: device::DeviceState,
    graph_access: workspaces::GraphAccess,
    speech: speech::Speech,
    initial_window_size: Option<[f32; 2]>,
    a11y: accessibility::AccessibilityRuntime,
    input_access: accessible_calculator::InputAccess,
    graph_extra: [String; 8],
    graph_tools: graph_tools::GraphTools,
    graph_audio: audio_trace::GraphAudio,
}
impl EqualApp {
    fn new(cc: &eframe::CreationContext<'_>, screenshot: Option<PathBuf>) -> Self {
        handheld::install_fonts(&cc.egui_ctx);
        let mut settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "equal-settings"))
            .unwrap_or_default();
        if screenshot.is_some() {
            settings = Settings::default();
        }
        settings.level = settings.level.min(3);
        settings.scale = if settings.scale.is_finite() {
            settings.scale.clamp(0.9, 4.0)
        } else {
            1.0
        };
        settings.precision = settings.precision.clamp(2, 10);
        settings.access.sanitize();
        Self::with_settings(settings, screenshot)
    }
    fn with_settings(settings: Settings, screenshot: Option<PathBuf>) -> Self {
        Self {
            settings,
            tab: Tab::Calculator,
            expression: String::new(),
            answer: 0.0,
            result: "0".into(),
            error: false,
            history: vec![],
            show_history: false,
            show_help: false,
            learning: learning::LearningSupport::default(),
            tutor: tutor::Tutor::default(),
            toolkit: toolkit::Toolkit::default(),
            practice: practice::Practice::default(),
            graph_a: HANDHELD_FUNCTIONS[0].into(),
            graph_b: HANDHELD_FUNCTIONS[1].into(),
            graph_error: String::new(),
            surface: false,
            table: false,
            range: 10.0,
            center: [0.0, 0.0],
            matrix_a: "1  2\n3  4".into(),
            matrix_b: "5  6\n7  8".into(),
            matrix_op: "A × B".into(),
            matrix_result: String::new(),
            tool: false,
            tool_kind: 0,
            tool_a: 3.0,
            tool_b: 4.0,
            tool_c: 5.0,
            message: String::new(),
            screenshot,
            frame_count: 0,
            screenshot_requested: false,
            second: false,
            show_workspace: false,
            handheld_preview: true,
            show_settings: false,
            device: device::DeviceState::default(),
            graph_access: workspaces::GraphAccess::default(),
            speech: speech::Speech::default(),
            initial_window_size: None,
            a11y: accessibility::AccessibilityRuntime::default(),
            input_access: accessible_calculator::InputAccess::default(),
            graph_extra: Default::default(),
            graph_tools: graph_tools::GraphTools::default(),
            graph_audio: audio_trace::GraphAudio::default(),
        }
    }
    fn speak(&mut self, text: &str) {
        if self.settings.access.captions {
            self.a11y.latest_announcement =
                speech::describe_math(text).chars().take(2000).collect();
        }
        if let Err(error) = self.speech.configure(
            self.settings.access.speech_rate,
            self.settings.access.speech_volume,
            &self.settings.access.speech_voice,
        ) {
            self.message = error;
            return;
        }
        if let Err(error) = self.speech.speak(text) {
            self.message = error;
        }
    }
    pub(crate) fn access_overlay_open(&self) -> bool {
        self.show_settings
            || self.show_help
            || self.show_history
            || self.learning.overlay_open()
            || self.a11y.show_shortcuts
            || self.a11y.show_glossary
    }
    pub(crate) fn stop_audio(&mut self) {
        let speech = self.speech.stop();
        let graph = self.graph_audio.stop();
        self.message = match (speech, graph) {
            (Ok(()), Ok(())) => "Audio stopped.".into(),
            (Err(error), _) | (_, Err(error)) => error,
        };
        if self.settings.access.captions {
            self.a11y.latest_announcement = self.message.clone();
        }
    }
    fn application_shortcuts(&mut self, ctx: &egui::Context) -> bool {
        let editing = text_input_focused(ctx);
        let overlay_open = self.access_overlay_open() || egui::Popup::is_any_open(ctx);
        let action = ctx.input_mut(|input| {
            let command = egui::Modifiers::COMMAND;
            let shifted = egui::Modifiers {
                shift: true,
                command: true,
                ..Default::default()
            };
            if input.consume_key(egui::Modifiers::NONE, egui::Key::F1) {
                return Some(AppShortcut::Help);
            }
            if input.consume_key(command, egui::Key::Comma) {
                return Some(AppShortcut::Settings);
            }
            if input.consume_key(command, egui::Key::G) {
                return Some(AppShortcut::Glossary);
            }
            if input.consume_key(command, egui::Key::H) {
                return Some(AppShortcut::History);
            }
            if input.consume_key(shifted, egui::Key::Space) {
                return Some(AppShortcut::StopAudio);
            }
            if input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowUp) {
                return Some(AppShortcut::Guide(-20.0));
            }
            if input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowDown) {
                return Some(AppShortcut::Guide(20.0));
            }
            if !editing && !overlay_open && self.tab == Tab::Calculator {
                if input.consume_key(shifted, egui::Key::Z)
                    || input.consume_key(command, egui::Key::Y)
                {
                    return Some(AppShortcut::Redo);
                }
                if input.consume_key(command, egui::Key::Z) {
                    return Some(AppShortcut::Undo);
                }
                // Native Windows Copy arrives as a clipboard event, not Key::C.
                let copy_event = input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Copy));
                if copy_event || input.consume_key(command, egui::Key::C) {
                    input
                        .events
                        .retain(|event| !matches!(event, egui::Event::Copy));
                    return Some(AppShortcut::CopyAnswer);
                }
            }
            if !overlay_open {
                for (key, tab) in [
                    (egui::Key::Num1, Tab::Calculator),
                    (egui::Key::Num2, Tab::Graph),
                    (egui::Key::Num3, Tab::Matrix),
                    (egui::Key::Num4, Tab::Tutor),
                    (egui::Key::Num5, Tab::Toolkit),
                    (egui::Key::Num6, Tab::Practice),
                ] {
                    if input.consume_key(egui::Modifiers::ALT, key) {
                        return Some(AppShortcut::Workspace(tab));
                    }
                }
            }
            None
        });
        let Some(action) = action else {
            return false;
        };
        if matches!(
            action,
            AppShortcut::Help
                | AppShortcut::Settings
                | AppShortcut::Glossary
                | AppShortcut::History
        ) {
            egui::Popup::close_all(ctx);
        }
        match action {
            AppShortcut::Help => {
                self.show_settings = false;
                self.a11y.show_shortcuts = !self.a11y.show_shortcuts;
            }
            AppShortcut::Settings => self.open_access_settings(ctx, None),
            AppShortcut::Glossary => {
                self.show_settings = false;
                self.a11y.show_glossary = !self.a11y.show_glossary;
            }
            AppShortcut::History => {
                self.show_settings = false;
                self.show_history = true;
            }
            AppShortcut::StopAudio => self.stop_audio(),
            AppShortcut::Workspace(tab) => {
                let enabled = !matches!(tab, Tab::Graph | Tab::Matrix)
                    || self.settings.level >= 2 && (tab != Tab::Matrix || self.settings.level == 3);
                if enabled {
                    self.tab = tab;
                    self.show_workspace = true;
                    self.input_access.pause();
                } else {
                    self.message =
                        "Choose 9–12 or Advanced for graphing, or Advanced for matrices.".into();
                }
            }
            AppShortcut::Undo if self.tab == Tab::Calculator => self.press_device_key(ctx, "undo"),
            AppShortcut::Redo if self.tab == Tab::Calculator => self.press_device_key(ctx, "redo"),
            AppShortcut::CopyAnswer if self.tab == Tab::Calculator => {
                ctx.copy_text(self.result.clone());
                self.message = "Answer copied.".into();
            }
            AppShortcut::Guide(delta) => {
                self.settings.access.reading_guide = true;
                let initial = if self.a11y.guide_y > 0.0 {
                    self.a11y.guide_y
                } else {
                    ctx.content_rect().center().y
                };
                self.a11y.guide_y = (initial + delta).clamp(0.0, ctx.content_rect().height());
            }
            _ => {}
        }
        true
    }
    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        // egui releases focus on Escape during begin_pass, before app input runs.
        let escaped_editor = ctx.input(|input| input.key_pressed(egui::Key::Escape))
            && self.a11y.workspace_focus.is_some_and(|id| {
                ctx.memory(|memory| memory.had_focus_last_frame(id))
                    && egui::text_edit::TextEditState::load(ctx, id).is_some()
            });
        if !self.access_overlay_open() && !self.a11y.overlay_was_open {
            self.a11y.workspace_focus = ctx.memory(|memory| memory.focused());
        }
        let shortcut_consumed = self.application_shortcuts(ctx);
        let access_consumed = self.handle_access_input(ctx);
        let overlay_open = self.access_overlay_open() || egui::Popup::is_any_open(ctx);
        if ctx.input(|i| !i.raw.dropped_files.is_empty()) {
            self.tab = Tab::Tutor;
        }
        if self.tab == Tab::Calculator
            && !text_input_focused(ctx)
            && !escaped_editor
            && !overlay_open
            && !shortcut_consumed
            && !access_consumed
            && ctx.input(|i| i.key_pressed(egui::Key::Escape))
        {
            if self.device.screen == device::DeviceScreen::Home {
                self.press_device_key(ctx, "clear");
            } else {
                self.set_device_screen(device::DeviceScreen::Home);
            }
        }
        if !text_input_focused(ctx)
            && self.tab == Tab::Calculator
            && !overlay_open
            && !shortcut_consumed
            && !access_consumed
        {
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Text(text) = event
                    && !text.trim().is_empty()
                    && text.chars().all(|character| !character.is_control())
                    && ctx.input(|input| {
                        !input.modifiers.command && !input.modifiers.ctrl && !input.modifiers.alt
                    })
                {
                    self.activate_access_key(ctx, &text);
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Enter))
                && !ctx.memory(|m| m.focused().is_some())
            {
                self.activate_access_key(ctx, "enter");
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Backspace)) {
                self.activate_access_key(ctx, "backspace");
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
                self.activate_access_key(ctx, "delete_forward");
            }
        }
    }
    fn sync_overlay_focus(&mut self, ctx: &egui::Context) {
        let open = self.access_overlay_open();
        if open && !self.a11y.overlay_was_open {
            self.a11y.overlay_return_focus = self.a11y.workspace_focus;
            self.a11y.overlay_had_settings = self.show_settings;
            egui::Popup::close_all(ctx);
        } else if !open && self.a11y.overlay_was_open {
            let settings_origin = self
                .a11y
                .overlay_had_settings
                .then(|| {
                    ctx.data(|data| {
                        data.get_temp::<egui::Id>(egui::Id::new("settings-return-focus"))
                    })
                })
                .flatten();
            let origin = settings_origin
                .or(self.a11y.overlay_return_focus)
                .or_else(|| {
                    ctx.data(|data| {
                        data.get_temp::<egui::Id>(egui::Id::new("settings-current-trigger"))
                    })
                });
            if let Some(origin) = origin {
                ctx.memory_mut(|memory| memory.request_focus(origin));
            }
            self.a11y.overlay_had_settings = false;
        }
        self.a11y.overlay_had_settings |= self.show_settings;
        self.a11y.overlay_was_open = open;
        if !open {
            self.a11y.workspace_focus = ctx.memory(|memory| memory.focused());
        }
    }
    fn windows(&mut self, ctx: &egui::Context) {
        egui::Window::new("Calculation history")
            .open(&mut self.show_history)
            .default_width(470.0)
            .max_width((ctx.content_rect().width() - 28.0).max(240.0))
            .max_height((ctx.content_rect().height() - 40.0).max(180.0))
            .show(ctx, |ui| {
                if self.history.is_empty() {
                    ui.label("Press Enter to save your first calculation here.");
                }
                let mut reuse = None;
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (expr, result) in self.history.iter().rev() {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(expr);
                                    ui.label(R::new(format!("= {result}")).strong().size(20.0));
                                });
                                if ui.button("Reuse").clicked() {
                                    reuse = Some(expr.clone());
                                }
                            });
                            ui.separator();
                        }
                    });
                if let Some(expr) = reuse {
                    self.expression = expr;
                    self.tab = Tab::Calculator;
                }
                ui.horizontal(|ui| {
                    if ui.button("Copy history").clicked() {
                        ui.ctx().copy_text(
                            self.history
                                .iter()
                                .map(|(a, b)| format!("{a} = {b}"))
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                    }
                    if ui.button("Clear history").clicked() {
                        self.history.clear();
                    }
                });
            });
        egui::Window::new("Quick guide").open(&mut self.show_help).default_width(550.0).max_width((ctx.content_rect().width()-24.0).max(240.0)).max_height((ctx.content_rect().height()-40.0).max(200.0)).vscroll(true).show(ctx,|ui|{
            ui.heading("Calculator controls");ui.label("Choose a learning level in Settings. Click the handheld keys, or click the display and type. Use * for multiplication, / for division, and ^ for a power. Use explicit multiplication: 2*pi and 2*(3+4).");ui.separator();
            ui.label("Keyboard: Tab / Shift+Tab move through controls; Space activates buttons; Enter evaluates while the display is focused; Escape clears. Controls expose labels through AccessKit for system screen readers.");ui.separator();ui.strong("Scientific & calculus examples");
            for s in ["sin(30) in Degrees mode → 0.5","asin(0.5) → inverse sine","diff(x^3, 2) → derivative at x = 2","integral(x^2, 0, 3) → definite integral","root(x^2-2, 1) → numerical root","5! → factorial; 50% → 0.5"]{ui.label(s);}ui.separator();
            ui.label("Graphs use the selected Degrees or Radians mode. Y= edits functions; GRAPH draws on the calculator; TRACE and the arrow keys explore coordinates; 2nd then GRAPH opens the table. The Graph tab opens a larger view. Calculus uses numerical approximations over real numbers.");
            ui.label("Reading-friendly mode uses a monospaced font and extra spacing. High contrast, larger text, patterned graphs, and optional spoken answers can be combined. All feedback is visible.");ui.separator();
            ui.label("Equal is independent open-source software under the MIT license. The TI picture guide shows real reference devices. Open in MATLAB sends graph samples to your installed MATLAB; Equal itself does not execute MATLAB programs. Accessibility needs evaluation with disabled learners and their assistive technology.");
        });
        self.learning_windows(ctx);
        self.geometry(ctx);
    }
}

impl eframe::App for EqualApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme(ctx);
        if self.initial_window_size.is_some() {
            ctx.request_repaint();
            if self.frame_count >= 2
                && let Some(size) = self.initial_window_size.take()
            {
                // eframe restores older window geometry even with persist_window=false.
                // Apply the launch size once, after the stored zoom has settled.
                let zoom = ctx.zoom_factor();
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(
                    egui::vec2(320.0, 400.0) / zoom,
                ));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                    egui::vec2(size[0], size[1]) / zoom,
                ));
            }
        }
        self.tutor.poll(ctx);
        if let Err(error) = self.speech.poll() {
            self.message = error;
        }
        if let Err(error) = self.graph_audio.poll() {
            self.message = error;
        }
        self.handle_keyboard_input(ctx);
        self.draw(ctx);
        self.windows(ctx);
        self.accessibility_help_windows(ctx);
        self.sync_overlay_focus(ctx);
        self.frame_count += 1;
        if self.screenshot.is_some() {
            ctx.request_repaint();
            if self.frame_count >= 120 && !self.screenshot_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.screenshot_requested = true;
            }
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event
                    && let Some(path) = &self.screenshot
                {
                    if let Some(parent) = path.parent()
                        && let Err(e) = std::fs::create_dir_all(parent)
                    {
                        eprintln!("Screenshot folder: {e}");
                        std::process::exit(1);
                    }
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    match image::save_buffer(
                        path,
                        &bytes,
                        image.width() as u32,
                        image.height() as u32,
                        image::ColorType::Rgba8,
                    ) {
                        Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                        Err(e) => {
                            eprintln!("Screenshot failed: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.screenshot.is_none() {
            eframe::set_value(storage, "equal-settings", &self.settings);
        }
    }
}
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let argument = |name: &str| {
        args.windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| pair[1].clone())
    };
    let size = argument("--size").and_then(|value| {
        let (width, height) = value.split_once('x')?;
        let width: f32 = width.parse().ok()?;
        let height: f32 = height.parse().ok()?;
        (width.is_finite()
            && height.is_finite()
            && (320.0..=4096.0).contains(&width)
            && (400.0..=2160.0).contains(&height))
        .then_some([width, height])
    });
    if argument("--size").is_some() && size.is_none() {
        eprintln!("Use --size WIDTHxHEIGHT, at least 320x400 and at most 4096x2160.");
        std::process::exit(2);
    }
    let level = argument("--level").and_then(|value| match value.as_str() {
        "k3" | "0" => Some(0),
        "4-8" | "1" => Some(1),
        "9-12" | "2" => Some(2),
        "advanced" | "3" => Some(3),
        _ => None,
    });
    let screen = argument("--screen");
    let workspace = args.iter().any(|value| value == "--workspace");
    let start_practice = args.iter().any(|value| value == "--practice");
    let practice_topic = argument("--topic");
    let practice_difficulty = argument("--difficulty");
    let show_solution = args.iter().any(|value| value == "--show-solution");
    let contrast = args.iter().any(|value| value == "--contrast");
    let readable = args.iter().any(|value| value == "--readable");
    let access_layout = argument("--layout");
    let access_theme = argument("--theme");
    let word_labels = args.iter().any(|value| value == "--word-labels");
    let left_handed = args.iter().any(|value| value == "--left-handed");
    let large_controls = args.iter().any(|value| value == "--large-controls");
    let focus_mode = args.iter().any(|value| value == "--focus-mode");
    let reading_guide = args.iter().any(|value| value == "--reading-guide");
    let hide_scores = args.iter().any(|value| value == "--hide-scores");
    let decimal_comma = args.iter().any(|value| value == "--decimal-comma");
    let initial_expression = argument("--expression");
    let complex_mode = args.iter().any(|value| value == "--complex");
    let font_scale = argument("--font-scale")
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite());
    let scale = argument("--scale")
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.9, 4.0));
    let screenshot = args
        .iter()
        .position(|a| a == "--screenshot")
        .map(|i| PathBuf::from(args.get(i + 1).expect("--screenshot needs a PNG filename")));
    let preview = args
        .windows(2)
        .find(|a| a[0] == "--preview")
        .map(|a| a[1].clone());
    let options = eframe::NativeOptions {
        persist_window: false,
        persistence_path: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("preferences.ron")),
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size.unwrap_or([462.0, 840.0]))
            .with_min_inner_size([320.0, 400.0])
            .with_title("Equal Calculator"),
        ..Default::default()
    };
    eframe::run_native(
        "Equal",
        options,
        Box::new(move |cc| {
            let mut app = EqualApp::new(cc, screenshot);
            if let Some(level) = level {
                app.settings.level = level;
            }
            if contrast {
                app.settings.contrast = true;
            }
            if readable {
                app.settings.readable = true;
            }
            if access_layout.as_deref() == Some("accessible") {
                app.settings.access.layout = accessibility::CalculatorLayout::Accessible;
            }
            if let Some(theme) = access_theme.as_deref() {
                app.settings.access.theme = match theme {
                    "dark" => accessibility::ColorTheme::Dark,
                    "warm" => accessibility::ColorTheme::Warm,
                    _ => accessibility::ColorTheme::Light,
                };
            }
            if word_labels {
                app.settings.access.word_labels = true;
            }
            if left_handed {
                app.settings.access.left_handed = true;
            }
            if large_controls {
                app.settings.access.large_controls = true;
            }
            if focus_mode {
                app.settings.access.focus_mode = true;
            }
            if reading_guide {
                app.settings.access.reading_guide = true;
            }
            if hide_scores {
                app.settings.access.show_scores = false;
            }
            if decimal_comma {
                app.settings.access.decimal_comma = true;
            }
            if let Some(size) = font_scale {
                app.settings.access.font_scale = size;
            }
            app.settings.access.sanitize();
            if let Some(scale) = scale {
                app.settings.scale = scale;
            }
            if complex_mode {
                app.device.complex_mode = true;
            }
            if let Some(expression) = initial_expression {
                app.expression = expression;
                app.evaluate();
            }
            app.show_workspace = workspace;
            if start_practice {
                app.tab = Tab::Practice;
                app.show_workspace = true;
            }
            app.set_device_screen(match screen.as_deref() {
                Some("graph") => device::DeviceScreen::Graph,
                Some("trace") => device::DeviceScreen::Trace,
                Some("functions") => device::DeviceScreen::Functions,
                Some("mode") => device::DeviceScreen::Mode,
                Some("table") => device::DeviceScreen::Table,
                Some("lists") => device::DeviceScreen::Lists,
                Some("sequences") => device::DeviceScreen::Sequences,
                Some("programs") => device::DeviceScreen::Programs,
                Some("link") => device::DeviceScreen::Link,
                Some("draw") => device::DeviceScreen::GraphDraw,
                Some("calculate") => device::DeviceScreen::GraphCalculate,
                Some("angles") => device::DeviceScreen::Angles,
                _ => device::DeviceScreen::Home,
            });
            app.graph_access.keyboard_trace = app.device.screen == device::DeviceScreen::Trace;
            if app.screenshot.is_some() {
                match preview.as_deref() {
                    Some("pictures") => app.learning.show_references = true,
                    Some("access") => app.learning.show_access = true,
                    Some("settings") => app.show_settings = true,
                    Some("glossary") => app.a11y.show_glossary = true,
                    Some("shortcuts") => app.a11y.show_shortcuts = true,
                    Some("graph") => app.tab = Tab::Graph,
                    Some("toolkit") => app.tab = Tab::Toolkit,
                    Some("practice") => app.tab = Tab::Practice,
                    Some("tutor") => {
                        app.tab = Tab::Tutor;
                        app.tutor
                            .solve_local(app.settings.level, app.settings.degrees);
                    }
                    Some("surface") => {
                        app.settings.level = 3;
                        app.tab = Tab::Graph;
                        app.surface = true;
                        app.graph_a = "sin(x)*cos(y)".into();
                    }
                    _ => {}
                }
                if app.tab != Tab::Calculator {
                    app.show_workspace = true;
                }
            }
            app.practice.set_level(app.settings.level);
            if let Some(name) = &practice_topic {
                let topic = practice::topics(app.settings.level)
                    .find(|topic| {
                        topic.title.to_lowercase().replace(' ', "-") == name.to_lowercase()
                    })
                    .map(|topic| topic.id);
                if let Some(topic) = topic {
                    app.practice.select_topic(topic);
                }
            }
            if let Some(difficulty) = practice_difficulty
                .as_deref()
                .and_then(|value| match value {
                    "basic" => Some(practice::Difficulty::Basic),
                    "standard" => Some(practice::Difficulty::Standard),
                    "challenge" => Some(practice::Difficulty::Challenge),
                    _ => None,
                })
            {
                app.practice.select_difficulty(difficulty);
            }
            if show_solution && app.tab == Tab::Practice {
                app.practice.reveal_answer();
            }
            app.initial_window_size = Some(size.unwrap_or(if app.tab != Tab::Calculator {
                [1360.0, 940.0]
            } else {
                [462.0, 840.0]
            }));
            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod ui_tests {
    use super::*;
    fn frame(
        app: &mut EqualApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        app.theme(ctx);
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1360.0, 940.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                app.draw(ctx);
            },
        )
    }
    fn click(app: &mut EqualApp, ctx: &egui::Context, pos: egui::Pos2) {
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    fn input_frame(
        app: &mut EqualApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
    ) -> egui::FullOutput {
        app.theme(ctx);
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 900.0),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| {
                app.handle_keyboard_input(ctx);
                app.draw(ctx);
                app.windows(ctx);
                app.accessibility_help_windows(ctx);
                app.sync_overlay_focus(ctx);
            },
        )
    }
    fn key_event(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }
    #[test]
    fn keyboard_shortcuts_preserve_work_and_respect_native_text_editing() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.settings.access.layout = accessibility::CalculatorLayout::Accessible;
        input_frame(&mut app, &ctx, vec![], egui::Modifiers::NONE);
        input_frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text("8+8".into())],
            egui::Modifiers::NONE,
        );
        assert_eq!(app.expression, "8+8");
        app.evaluate();
        let output = input_frame(
            &mut app,
            &ctx,
            vec![egui::Event::Copy],
            egui::Modifiers::COMMAND,
        );
        assert!(
            output.platform_output.commands.iter().any(
                |command| matches!(command, egui::OutputCommand::CopyText(text) if text == "16")
            )
        );
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("expression")));
        let mut consumed = true;
        let _ = ctx.run(
            egui::RawInput {
                events: vec![key_event(egui::Key::Z, egui::Modifiers::COMMAND)],
                modifiers: egui::Modifiers::COMMAND,
                ..Default::default()
            },
            |ctx| {
                assert!(text_input_focused(ctx));
                consumed = app.application_shortcuts(ctx);
            },
        );
        assert!(
            !consumed,
            "Native text editors retain their own Undo command"
        );
        assert_eq!(app.expression, "8+8");
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("expression")));
        input_frame(&mut app, &ctx, vec![], egui::Modifiers::NONE);
        input_frame(
            &mut app,
            &ctx,
            vec![key_event(egui::Key::Escape, egui::Modifiers::NONE)],
            egui::Modifiers::NONE,
        );
        assert_eq!(
            app.expression, "8+8",
            "Escape must not clear a focused editor"
        );
        input_frame(
            &mut app,
            &ctx,
            vec![key_event(egui::Key::Comma, egui::Modifiers::COMMAND)],
            egui::Modifiers::COMMAND,
        );
        assert!(app.show_settings);
        assert_eq!(app.expression, "8+8");
        assert_eq!(app.result, "16");
    }
    #[test]
    fn help_history_and_glossary_prevent_background_expression_edits() {
        for overlay in ["help", "history", "glossary"] {
            let ctx = egui::Context::default();
            let mut app = EqualApp::with_settings(Settings::default(), None);
            app.settings.access.layout = accessibility::CalculatorLayout::Accessible;
            app.expression = "12+8".into();
            app.evaluate();
            input_frame(&mut app, &ctx, vec![], egui::Modifiers::NONE);
            input_frame(&mut app, &ctx, vec![], egui::Modifiers::NONE);
            ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("expression")));
            input_frame(&mut app, &ctx, vec![], egui::Modifiers::NONE);
            match overlay {
                "help" => app.a11y.show_shortcuts = true,
                "history" => app.show_history = true,
                "glossary" => app.a11y.show_glossary = true,
                _ => unreachable!(),
            }
            input_frame(
                &mut app,
                &ctx,
                vec![
                    egui::Event::Text("999".into()),
                    key_event(egui::Key::Enter, egui::Modifiers::NONE),
                ],
                egui::Modifiers::NONE,
            );
            assert_eq!(app.expression, "12+8", "No typing behind {overlay}");
            assert_eq!(app.result, "20", "No calculation behind {overlay}");
            assert_eq!(app.history.len(), 1);
            app.a11y.show_shortcuts = false;
            app.show_history = false;
            app.a11y.show_glossary = false;
            input_frame(&mut app, &ctx, vec![], egui::Modifiers::NONE);
            input_frame(&mut app, &ctx, vec![], egui::Modifiers::NONE);
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                Some(egui::Id::new("expression")),
                "Closing {overlay} restores the expression's keyboard focus"
            );
        }
    }
    #[test]
    fn reference_face_keys_and_tutor_navigation_are_interactive() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.show_workspace = true;
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "clear");
        click(&mut app, &ctx, pos);
        assert_eq!(
            app.expression, "",
            "Clear on the actual face must clear the live input"
        );
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "7");
        click(&mut app, &ctx, pos);
        assert_eq!(app.expression, "7");
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "Calculate answer");
        click(&mut app, &ctx, pos);
        assert_eq!(app.result, "7");
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "Learn");
        click(&mut app, &ctx, pos);
        assert!(app.tab == Tab::Tutor);
    }
    #[test]
    fn accessibility_preserves_the_prominent_reference_calculator() {
        let mut positions = None;
        for (contrast, readable, color_safe) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (true, true, true),
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(
                Settings {
                    contrast,
                    readable,
                    color_safe,
                    ..Settings::default()
                },
                None,
            );
            app.show_workspace = true;
            app.expression = "8+88".into();
            app.handheld_preview = false;
            frame(&mut app, &ctx, vec![]);
            let output = frame(&mut app, &ctx, vec![]);
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
                egui::Shape::Text(text) if text.galley.text() == "TI-84")));
            let seven = control(&output, "7");
            let enter = control(&output, "Calculate answer");
            // These controls span 150 of the face's 323 units: require a face
            // at least 300 points wide instead of the old 271-point thumbnail.
            assert!(enter.x - seven.x > 139.0);
            assert!(enter.x < 1054.0 && enter.y < 900.0);
            if let Some((original_seven, original_enter)) = positions {
                assert!(
                    seven.distance(original_seven) < 2.0,
                    "reading={readable}, contrast={contrast}, original={original_seven:?}, actual={seven:?}"
                );
                assert!(enter.distance(original_enter) < 2.0);
            } else {
                positions = Some((seven, enter));
            }
            click(&mut app, &ctx, enter);
            assert_eq!(app.expression, "8+88");
            assert_eq!(app.result, "96");
        }
    }
    fn control(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
        let tree = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("AccessKit tree");
        let node = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("Missing control: {label}"));
        let rect = node.1.bounds().expect("Control bounds");
        egui::pos2(
            ((rect.x0 + rect.x1) / 2.0) as f32,
            ((rect.y0 + rect.y1) / 2.0) as f32,
        )
    }
    #[test]
    fn handheld_renders_without_photo_textures_at_each_size() {
        for size in [[323.0, 657.0], [462.0, 940.0], [646.0, 1314.0]] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(Settings::default(), None);
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(size[0], size[1]),
                    )),
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            // Font atlas 0 is the only allowed texture: the casing and keys are geometry.
            assert!(
                output
                    .textures_delta
                    .set
                    .iter()
                    .all(|(id, _)| *id == egui::TextureId::Managed(0))
            );
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
                egui::Shape::Text(text) if text.galley.text() == "TI-84")));
            let seven = control(&output, "7");
            assert!(seven.x > 0.0 && seven.x < size[0]);
            assert!(seven.y > 0.0 && seven.y < size[1]);
        }
    }
    #[test]
    fn handheld_calculates_and_opens_settings() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(app.handheld_preview);
        let pos = control(&output, "7");
        click(&mut app, &ctx, pos);
        assert_eq!(app.expression, "7");
        assert!(!app.handheld_preview);
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "Calculate answer");
        click(&mut app, &ctx, pos);
        assert_eq!(app.result, "7");
        for label in ["clear", "8", "Add", "8", "8", "Calculate answer"] {
            let output = frame(&mut app, &ctx, vec![]);
            let pos = control(&output, label);
            click(&mut app, &ctx, pos);
        }
        assert_eq!(app.expression, "8+88");
        assert_eq!(app.result, "96");
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "Settings");
        click(&mut app, &ctx, pos);
        assert!(app.show_settings);
        assert!(
            !app.show_workspace,
            "Settings must preserve the compact view"
        );
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "Close settings");
        click(&mut app, &ctx, pos);
        assert!(!app.show_settings);
        assert!(app.tab == Tab::Calculator);
    }
    #[test]
    fn toolkit_keys_results_clipboard_and_practice_are_interactive() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        click(
            &mut app,
            &ctx,
            control(&output, "Statistics: data and regression tools"),
        );
        assert_eq!(app.device.screen, device::DeviceScreen::Statistics);
        let output = frame(&mut app, &ctx, vec![]);
        click(&mut app, &ctx, control(&output, "More statistics tools"));
        assert!(app.tab == Tab::Toolkit);
        for label in ["Numbers", "Simplify a fraction", "Calculate tool"] {
            let output = frame(&mut app, &ctx, vec![]);
            click(&mut app, &ctx, control(&output, label));
        }
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            output
                .shapes
                .iter()
                .any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text() == "3/4"))
        );
        let pos = control(&output, "Copy result & method");
        // Inspect the release frame because clipboard commands are emitted once.
        frame(
            &mut app,
            &ctx,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        let output = frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(output.platform_output.commands.iter().any(|c| matches!(c, egui::OutputCommand::CopyText(text) if text.contains("3/4") && text.contains("greatest common divisor"))), "copy at {pos:?}: {:?}", output.platform_output.commands);
        for label in ["Practice", "K–3", "Show a hint", "Reveal answer & method"] {
            let output = frame(&mut app, &ctx, vec![]);
            click(&mut app, &ctx, control(&output, label));
        }
        assert!(
            app.tab == Tab::Practice,
            "Switching learning level must keep practice open"
        );
        let first_question = app.practice.current_question();
        let expected_answer = format!("Answer: {}.", first_question.answer.display());
        let output = frame(&mut app, &ctx, vec![]);
        assert!(output.shapes.iter().any(
            |s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text().contains(&expected_answer))
        ));
        assert_eq!(app.practice.current_session().progress.attempted, 1);
        let next = control(&output, "Next question");
        click(&mut app, &ctx, next);
        let output = frame(&mut app, &ctx, vec![]);
        let next_question = app.practice.current_question();
        assert_ne!(first_question.prompt, next_question.prompt);
        assert!(output.shapes.iter().any(
            |s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text() == next_question.prompt)
        ));
    }
    #[test]
    fn physical_graph_keys_edit_and_draw_on_the_calculator_display() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.show_workspace = true;
        frame(&mut app, &ctx, vec![]);
        for label in [
            "Edit graph functions",
            "clear",
            "Insert variable x",
            "^",
            "2",
        ] {
            let output = frame(&mut app, &ctx, vec![]);
            click(&mut app, &ctx, control(&output, label));
        }
        assert_eq!(app.graph_a, "x^2");
        assert!(app.expression.is_empty());
        let output = frame(&mut app, &ctx, vec![]);
        click(&mut app, &ctx, control(&output, "Graph on calculator"));
        assert_eq!(app.device.screen, device::DeviceScreen::Graph);
        assert!(app.tab == Tab::Calculator);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label().is_some_and(|label| label.contains("x^2")))
        );
        let output = frame(&mut app, &ctx, vec![]);
        click(&mut app, &ctx, control(&output, "Graph"));
        assert!(app.tab == Tab::Graph);
        assert_eq!(app.graph_a, "x^2");
    }

    #[test]
    fn worked_lesson_controls_follow_the_learners_pace() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.tab = Tab::Tutor;
        frame(&mut app, &ctx, vec![]);
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "K–3");
        click(&mut app, &ctx, pos);
        assert!(
            app.tab == Tab::Tutor,
            "Changing learning level must keep the tutor open"
        );
        let output = frame(&mut app, &ctx, vec![]);
        let pos = control(&output, "Explain my problem");
        click(&mut app, &ctx, pos);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(output.shapes.len() > 10);
        let next = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find_map(|(_, n)| n.label().filter(|l| l.starts_with("Next step")))
            .unwrap()
            .to_string();
        assert!(
            next.starts_with("Next step · 2"),
            "Lessons start one step at a time"
        );
        let pos = control(&output, &next);
        click(&mut app, &ctx, pos);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, n)| n.label().is_some_and(|l| l.starts_with("Next step · 3")))
        );
        let pos = control(&output, "Show all steps");
        click(&mut app, &ctx, pos);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            !output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node
                    .label()
                    .is_some_and(|label| label.starts_with("Next step"))),
            "Learners can still choose to show all steps"
        );
        let pos = control(&output, "Give me a hint first");
        click(&mut app, &ctx, pos);
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, n)| n.value().is_some_and(|l| l.contains("Hint"))
                    || n.label().is_some_and(|l| l.contains("Hint")))
        );
    }
}
