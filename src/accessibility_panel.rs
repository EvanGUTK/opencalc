use crate::accessibility::{AccessibilitySettings, CalculatorLayout, ColorTheme, ScanMode};
use crate::*;
use std::ops::RangeInclusive;

pub(crate) fn begin_settings_column(ui: &egui::Ui) {
    ui.ctx().data_mut(|data| {
        data.insert_temp(
            egui::Id::new("settings-column-right"),
            ui.max_rect().right(),
        )
    });
}

fn column_width(ui: &egui::Ui) -> f32 {
    let right = ui
        .ctx()
        .data(|data| data.get_temp::<f32>(egui::Id::new("settings-column-right")))
        .unwrap_or(ui.max_rect().right());
    let left = if ui.layout().is_horizontal() && ui.layout().main_wrap() {
        ui.max_rect().left()
    } else {
        ui.next_widget_position().x
    };
    (right - left).min(ui.max_rect().width()).max(1.0)
}

fn wrapped_text(ui: &egui::Ui, text: &str, width: f32) -> std::sync::Arc<egui::Galley> {
    wrapped_text_color(ui, text, width, ui.visuals().text_color())
}

fn wrapped_text_color(
    ui: &egui::Ui,
    text: &str,
    width: f32,
    color: egui::Color32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(
        text.to_string(),
        egui::TextStyle::Button.resolve(ui.style()),
        color,
        width.max(1.0),
    );
    job.wrap.break_anywhere = true;
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

fn access_label(ui: &mut egui::Ui, text: impl AsRef<str>) -> egui::Response {
    let galley = wrapped_text(ui, text.as_ref(), column_width(ui));
    ui.label(galley)
}

pub(crate) fn access_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    access_button_enabled(ui, text, true)
}

pub(crate) fn access_button_enabled(
    ui: &mut egui::Ui,
    text: &str,
    enabled: bool,
) -> egui::Response {
    let width = column_width(ui) - ui.spacing().button_padding.x * 2.0;
    let galley = wrapped_text(ui, text, width);
    ui.add_enabled(enabled, egui::Button::new(galley).wrap())
}

pub(crate) fn access_combo<Content>(
    ui: &mut egui::Ui,
    salt: impl std::hash::Hash,
    max_width: f32,
    selected: impl Into<egui::WidgetText>,
    contents: impl FnOnce(&mut egui::Ui) -> Content,
) -> egui::InnerResponse<Option<Content>> {
    let width = column_width(ui).min(max_width);
    // ComboBox::width is a minimum, and truncation uses its containing Ui's
    // available width. Allocate a bounded child so both obey this column.
    ui.allocate_ui_with_layout(
        egui::vec2(width, ui.spacing().interact_size.y),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            egui::ComboBox::from_id_salt(salt)
                .width(width)
                .truncate()
                .selected_text(selected)
                .show_ui(ui, contents)
        },
    )
    .inner
}

pub(crate) fn access_value<Value: PartialEq>(
    ui: &mut egui::Ui,
    value: &mut Value,
    choice: Value,
    text: impl AsRef<str>,
) -> egui::Response {
    let selected = *value == choice;
    let palette = crate::appearance::palette(ui.ctx());
    let galley = wrapped_text_color(
        ui,
        text.as_ref(),
        column_width(ui) - ui.spacing().button_padding.x * 2.0,
        if selected {
            palette.selected_text
        } else {
            palette.ink
        },
    );
    let mut response = ui.selectable_label(selected, galley);
    if response.clicked() && !selected {
        *value = choice;
        response.mark_changed();
    }
    response
}

fn access_radio<Value: PartialEq>(
    ui: &mut egui::Ui,
    value: &mut Value,
    choice: Value,
    text: &str,
) -> egui::Response {
    let width = column_width(ui) - ui.spacing().icon_width - ui.spacing().icon_spacing;
    let galley = wrapped_text(ui, text, width);
    ui.radio_value(value, choice, galley)
}

#[derive(Clone)]
struct NumberDraft {
    text: String,
    applied: u32,
    error: Option<String>,
}

impl NumberDraft {
    fn new(value: u32) -> Self {
        Self {
            text: value.to_string(),
            applied: value,
            error: None,
        }
    }
}

fn status(ui: &mut egui::Ui, message: &str) {
    let response = access_label(ui, message);
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_label(message);
        node.clear_value();
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

fn section(ui: &mut egui::Ui, title: &str, salt: &str, body: impl FnOnce(&mut egui::Ui)) {
    let id = ui.make_persistent_id(salt);
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    let marker = if state.is_open() { "−" } else { "+" };
    let response = access_button(ui, &format!("{marker} {title}"));
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, true, title));
    if response.clicked() {
        state.toggle(ui);
    }
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_expanded(state.is_open()));
    state.show_body_unindented(ui, body);
}

fn access_checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str) -> egui::Response {
    if column_width(ui) >= 180.0 {
        return ui.checkbox(value, label);
    }
    // A native checkbox reserves space for its icon. At high zoom a long word
    // can exceed the remaining text column, so allow it to wrap within the row.
    let text_width =
        (column_width(ui) - ui.spacing().icon_width - ui.spacing().icon_spacing).max(1.0);
    let galley = wrapped_text(ui, label, text_width);
    #[cfg(test)]
    {
        let geometry = (
            ui.available_width(),
            column_width(ui),
            ui.spacing().icon_width,
            ui.spacing().icon_spacing,
            galley.size(),
        );
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new(("settings-checkbox-geometry", label)),
                geometry,
            )
        });
    }
    ui.checkbox(value, galley)
}

// Stage values until Enter or Set so changing typography cannot move an active editor.
fn integer_option(
    ui: &mut egui::Ui,
    label: &str,
    value: u32,
    range: RangeInclusive<u32>,
    unit: &str,
) -> Option<u32> {
    let id = ui.id().with(("access-number", label));
    let mut draft = ui
        .ctx()
        .data(|data| data.get_temp::<NumberDraft>(id))
        .unwrap_or_else(|| NumberDraft::new(value));
    if draft.applied != value {
        draft = NumberDraft::new(value);
    }
    access_label(ui, label);
    let mut apply = false;
    ui.horizontal_wrapped(|ui| {
        let width = (ui.available_width() - 4.0).clamp(26.0, 58.0);
        let response = ui.add(
            egui::TextEdit::singleline(&mut draft.text)
                .desired_width(width)
                .char_limit(7)
                .id_salt(("access-value", label)),
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label));
        if response.changed() {
            draft.error = None;
        }
        apply = (response.has_focus() || response.lost_focus())
            && ui.input(|input| input.key_pressed(egui::Key::Enter));
        if !unit.is_empty() {
            access_label(ui, unit);
        }
        let button = access_button(ui, "Set");
        button.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("Set {label}"))
        });
        apply |= button.clicked();
    });
    let result = if apply {
        match draft.text.trim().parse::<u32>() {
            Ok(parsed) if range.contains(&parsed) => {
                draft = NumberDraft::new(parsed);
                Some(parsed)
            }
            _ => {
                draft.error = Some(format!("Enter {} to {}.", range.start(), range.end()));
                None
            }
        }
    } else {
        None
    };
    if let Some(error) = &draft.error {
        status(ui, error);
    }
    ui.ctx().data_mut(|data| data.insert_temp(id, draft));
    result
}

#[derive(Clone, Copy)]
enum Setup {
    LargePrint,
    LargeTargets,
    Reading,
    Focus,
    Quiet,
}

impl Setup {
    fn title(self) -> &'static str {
        match self {
            Self::LargePrint => "Large print",
            Self::LargeTargets => "Large targets",
            Self::Reading => "Reading support",
            Self::Focus => "Low distraction",
            Self::Quiet => "Quiet",
        }
    }
}

fn apply_setup(settings: &mut Settings, setup: Setup) {
    let access = &mut settings.access;
    match setup {
        Setup::LargePrint => {
            access.layout = CalculatorLayout::Accessible;
            access.target_size = access.target_size.max(64.0);
            access.font_scale = access.font_scale.max(1.15);
            access.large_controls = true;
            settings.scale = settings.scale.max(1.25);
            settings.contrast = true;
        }
        Setup::LargeTargets => {
            access.layout = CalculatorLayout::Accessible;
            access.target_size = 72.0;
            access.key_spacing = 12.0;
            access.large_controls = true;
        }
        Setup::Reading => {
            settings.readable = true;
            access.font_scale = access.font_scale.max(1.1);
            access.line_spacing = access.line_spacing.max(1.4);
            access.word_labels = true;
            access.show_help_text = true;
            access.one_step = true;
        }
        Setup::Focus => {
            access.focus_mode = true;
            access.show_scores = false;
            access.one_step = true;
            access.reduced_motion = true;
            access.show_help_text = false;
        }
        Setup::Quiet => {
            settings.speech = false;
            access.speak_keys = false;
            access.speak_focus = false;
            access.graph_sound = false;
            access.captions = true;
            access.reduced_motion = true;
        }
    }
    access.sanitize();
}

impl EqualApp {
    pub(crate) fn accessibility_controls(&mut self, ui: &mut egui::Ui) {
        ui.strong("Accessibility");
        access_label(ui, "Calculator layout");
        ui.horizontal_wrapped(|ui| {
            access_radio(
                ui,
                &mut self.settings.access.layout,
                CalculatorLayout::Familiar,
                "Familiar",
            );
            access_radio(
                ui,
                &mut self.settings.access.layout,
                CalculatorLayout::Accessible,
                "Large keys",
            );
        });
        access_checkbox(ui, &mut self.settings.contrast, "High contrast");
        access_checkbox(ui, &mut self.settings.readable, "Reading-friendly text");
        access_checkbox(ui, &mut self.settings.color_safe, "Color-safe graphs");
        access_checkbox(ui, &mut self.settings.speech, "Read results aloud");

        section(ui, "Quick setups", "access-setups", |ui| {
            access_label(
                ui,
                "Combine a setup with any individual option below. Your calculator work is kept.",
            );
            ui.horizontal_wrapped(|ui| {
                for setup in [
                    Setup::LargePrint,
                    Setup::LargeTargets,
                    Setup::Reading,
                    Setup::Focus,
                    Setup::Quiet,
                ] {
                    if access_button(ui, setup.title()).clicked() {
                        apply_setup(&mut self.settings, setup);
                        self.a11y.focus_navigation = false;
                        let mut notice = format!("{} setup applied.", setup.title());
                        if matches!(setup, Setup::Quiet) {
                            self.stop_audio();
                            notice.push_str(&format!(" {}", self.message));
                        }
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(egui::Id::new("access-setup-notice"), notice)
                        });
                    }
                }
            });
            if let Some(notice) = ui
                .ctx()
                .data(|data| data.get_temp::<String>(egui::Id::new("access-setup-notice")))
            {
                status(ui, &notice);
            }
        });

        section(ui, "Display and reading", "access-display", |ui| {
            access_label(ui, "Color theme");
            ui.horizontal_wrapped(|ui| {
                for (value, name) in [
                    (ColorTheme::Light, "Light"),
                    (ColorTheme::Warm, "Warm"),
                    (ColorTheme::Dark, "Dark"),
                ] {
                    access_value(ui, &mut self.settings.access.theme, value, name);
                }
            });
            if let Some(value) = integer_option(
                ui,
                "Body text size",
                (self.settings.access.font_scale * 100.0).round() as u32,
                90..=180,
                "%",
            ) {
                self.settings.access.font_scale = value as f32 / 100.0;
            }
            if let Some(value) = integer_option(
                ui,
                "Line spacing",
                (self.settings.access.line_spacing * 100.0).round() as u32,
                100..=200,
                "%",
            ) {
                self.settings.access.line_spacing = value as f32 / 100.0;
            }
            access_checkbox(ui, &mut self.settings.access.reading_guide, "Reading guide")
                .on_hover_text("A guide line helps follow the current line of text.");
            access_checkbox(
                ui,
                &mut self.settings.access.reduced_motion,
                "Reduce motion",
            );
            access_label(
                ui,
                "Use interface zoom above to enlarge the whole app. Body text size and spacing adjust reading separately.",
            );
        });

        section(ui, "Keys and pointer", "access-pointer", |ui| {
            access_label(
                ui,
                "Large-key options apply to the Large keys calculator layout.",
            );
            if let Some(value) = integer_option(
                ui,
                "Key target size",
                self.settings.access.target_size.round() as u32,
                40..=80,
                "pt",
            ) {
                self.settings.access.target_size = value as f32;
            }
            if let Some(value) = integer_option(
                ui,
                "Space between keys",
                self.settings.access.key_spacing.round() as u32,
                4..=24,
                "pt",
            ) {
                self.settings.access.key_spacing = value as f32;
            }
            access_checkbox(
                ui,
                &mut self.settings.access.word_labels,
                "Use word labels on keys",
            );
            access_checkbox(
                ui,
                &mut self.settings.access.left_handed,
                "Left-handed key order",
            );
            access_checkbox(
                ui,
                &mut self.settings.access.large_controls,
                "Larger app controls",
            );
            let mut dwell = self.settings.access.dwell_ms != 0;
            if access_checkbox(ui, &mut dwell, "Hover to activate a key").changed() {
                self.settings.access.dwell_ms = if dwell { 1200 } else { 0 };
            }
            if dwell {
                if let Some(value) = integer_option(
                    ui,
                    "Dwell time",
                    self.settings.access.dwell_ms as u32,
                    600..=4000,
                    "ms",
                ) {
                    self.settings.access.dwell_ms = u64::from(value);
                }
                access_label(
                    ui,
                    "Hold the pointer over a key until it activates. Move away before using it again.",
                );
            }
            let mut guard = self.settings.access.guard_ms != 0;
            if access_checkbox(ui, &mut guard, "Ignore rapid repeat activations").changed() {
                self.settings.access.guard_ms = if guard { 300 } else { 0 };
            }
            if guard
                && let Some(value) = integer_option(
                    ui,
                    "Repeat guard",
                    self.settings.access.guard_ms as u32,
                    100..=1500,
                    "ms",
                )
            {
                self.settings.access.guard_ms = u64::from(value);
            }
        });

        section(ui, "Keyboard and switch", "access-switch", |ui| {
            let mut scanning = self.settings.access.scan_mode == ScanMode::Automatic;
            if access_checkbox(ui, &mut scanning, "Enable switch scanning").changed() {
                self.settings.access.scan_mode = if scanning {
                    ScanMode::Automatic
                } else {
                    ScanMode::Off
                };
            }
            if scanning {
                self.settings.access.layout = CalculatorLayout::Accessible;
                if let Some(value) = integer_option(
                    ui,
                    "Scan interval",
                    self.settings.access.scan_interval_ms as u32,
                    600..=5000,
                    "ms",
                ) {
                    self.settings.access.scan_interval_ms = u64::from(value);
                }
                access_label(
                    ui,
                    "Scanning starts paused. Use Start scanning on the calculator. Space selects a row, then a key; Escape pauses.",
                );
            }
            access_checkbox(
                ui,
                &mut self.settings.access.speak_focus,
                "Read focused keys",
            );
            if access_button(ui, "Keyboard shortcuts").clicked() {
                self.a11y.show_shortcuts = true;
                self.show_settings = false;
            }
            access_label(
                ui,
                "Tab and Shift+Tab move between controls. Enter or Space activates a focused button.",
            );
        });

        section(ui, "Speech and visible feedback", "access-speech", |ui| {
            access_checkbox(
                ui,
                &mut self.settings.access.speak_keys,
                "Read activated keys",
            );
            access_checkbox(
                ui,
                &mut self.settings.access.captions,
                "Show announcement text",
            );
            access_label(
                ui,
                "All answers and errors remain visible when audio is off.",
            );
            access_label(ui, "Speech speed");
            let rate_text = |rate: i32| {
                if rate == 0 {
                    "Normal".to_string()
                } else if rate < 0 {
                    format!("Slower {}", -rate)
                } else {
                    format!("Faster {rate}")
                }
            };
            access_combo(
                ui,
                "access-speech-rate",
                110.0,
                rate_text(self.settings.access.speech_rate),
                |ui| {
                    for rate in -5..=5 {
                        access_value(
                            ui,
                            &mut self.settings.access.speech_rate,
                            rate,
                            rate_text(rate),
                        );
                    }
                },
            )
            .response
            .widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, "Speech speed")
            });
            if let Some(value) = integer_option(
                ui,
                "Speech volume",
                self.settings.access.speech_volume,
                0..=100,
                "%",
            ) {
                self.settings.access.speech_volume = value;
            }
            access_label(ui, "Voice");
            let selected = if self.settings.access.speech_voice.is_empty() {
                "System default".to_string()
            } else {
                self.settings.access.speech_voice.clone()
            };
            access_combo(ui, "access-speech-voice", 230.0, selected, |ui| {
                access_value(
                    ui,
                    &mut self.settings.access.speech_voice,
                    String::new(),
                    "System default",
                );
                for voice in &self.a11y.voices {
                    access_value(
                        ui,
                        &mut self.settings.access.speech_voice,
                        voice.clone(),
                        voice,
                    );
                }
            })
            .response
            .widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, "Speech voice")
            });
            if access_button(ui, "Refresh voices").clicked() {
                match speech::installed_voices() {
                    Ok(voices) => {
                        self.a11y.voice_notice =
                            format!("{} voices available on this device.", voices.len());
                        self.a11y.voices = voices;
                    }
                    Err(error) => self.a11y.voice_notice = error,
                }
            }
            if !self.a11y.voice_notice.is_empty() {
                status(ui, &self.a11y.voice_notice);
            }
            access_checkbox(
                ui,
                &mut self.settings.access.graph_sound,
                "Enable graph sound",
            );
            access_label(
                ui,
                "Graph sound plays only when you choose Play graph sound. The volume setting applies to playback.",
            );
            ui.horizontal_wrapped(|ui| {
                if access_button(ui, "Read sample").clicked() {
                    self.speak("One half plus one quarter equals three quarters.");
                }
                if access_button(ui, "Stop audio").clicked() {
                    self.stop_audio();
                }
            });
            if self.settings.access.captions && !self.a11y.latest_announcement.is_empty() {
                ui.strong("Latest announcement");
                access_label(ui, &self.a11y.latest_announcement);
                if access_button(ui, "Copy announcement").clicked() {
                    ui.ctx().copy_text(self.a11y.latest_announcement.clone());
                }
            }
        });

        section(ui, "Attention and learning", "access-attention", |ui| {
            if access_checkbox(ui, &mut self.settings.access.focus_mode, "Focus mode")
                .on_hover_text(
                    "Keep the current workspace and hide the full navigation until requested.",
                )
                .changed()
            {
                self.a11y.focus_navigation = false;
            }
            access_checkbox(
                ui,
                &mut self.settings.access.show_help_text,
                "Show extra help text",
            );
            access_checkbox(
                ui,
                &mut self.settings.access.one_step,
                "Show lessons one step at a time",
            );
            access_checkbox(
                ui,
                &mut self.settings.access.show_scores,
                "Show practice scores",
            );
            if access_button(ui, "Math word guide").clicked() {
                self.a11y.show_glossary = true;
                self.show_settings = false;
            }
        });

        section(ui, "Number entry", "access-numbers", |ui| {
            access_checkbox(
                ui,
                &mut self.settings.access.decimal_comma,
                "Use a decimal comma",
            );
            if self.settings.access.decimal_comma {
                access_label(
                    ui,
                    "Write 1,5 for one and a half. Use semicolons between function arguments or several answers: min(1;2).",
                );
            } else {
                access_label(
                    ui,
                    "Write 1.5 for one and a half. Use commas between function arguments or several answers: min(1,2).",
                );
            }
            access_label(
                ui,
                "Supported Unicode decimal digits are normalized when entered; the result uses the calculator's number format.",
            );
        });

        if access_button(ui, "Reset access options")
            .on_hover_text("Keep the grade, angle mode, calculator work, and interface zoom.")
            .clicked()
        {
            self.settings.access = AccessibilitySettings::default();
            self.settings.contrast = false;
            self.settings.readable = false;
            self.settings.color_safe = false;
            self.settings.speech = false;
            self.a11y.focus_navigation = false;
            self.stop_audio();
        }
        self.settings.access.sanitize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number_frame(
        ctx: &egui::Context,
        value: &mut u32,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(180.0, 600.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(next) = integer_option(ui, "Body text size", *value, 90..=180, "%")
                    {
                        *value = next;
                    }
                });
            },
        )
    }

    fn number_click(ctx: &egui::Context, value: &mut u32, label: &str) {
        let frame = number_frame(ctx, value, vec![]);
        let node = frame
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap();
        let rect = node.1.bounds().unwrap();
        let pos = egui::pos2(
            ((rect.x0 + rect.x1) * 0.5) as f32,
            ((rect.y0 + rect.y1) * 0.5) as f32,
        );
        for pressed in [true, false] {
            number_frame(
                ctx,
                value,
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

    #[test]
    fn typography_changes_wait_for_commit_and_reject_out_of_range_values() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut value = 100;
        number_frame(&ctx, &mut value, vec![]);
        for (text, expected) in [("145", 145), ("181", 145)] {
            number_click(&ctx, &mut value, "Body text size");
            let previous = value;
            number_frame(
                &ctx,
                &mut value,
                vec![
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
                ],
            );
            assert_eq!(value, previous, "Typing must keep the editor stable");
            number_click(&ctx, &mut value, "Set Body text size");
            assert_eq!(value, expected);
        }
        let output = number_frame(&ctx, &mut value, vec![]);
        assert!(
            output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Status
                    && node.label() == Some("Enter 90 to 180."))
        );
    }

    #[test]
    fn auxiliary_help_from_settings_closes_settings_and_remains_interactive() {
        for (group, action, close) in [
            (
                "Keyboard and switch",
                "Keyboard shortcuts",
                "Close keyboard help",
            ),
            (
                "Attention and learning",
                "Math word guide",
                "Close math glossary",
            ),
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(Settings::default(), None);
            let frame = |app: &mut EqualApp, events: Vec<egui::Event>| {
                app.theme(&ctx);
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(900.0, 1100.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        app.settings_dialog(ctx);
                        app.accessibility_help_windows(ctx);
                    },
                )
            };
            let click = |output: egui::FullOutput, label: &str| {
                let tree = output.platform_output.accesskit_update.unwrap();
                let target = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(label))
                    .unwrap_or_else(|| panic!("Missing {label}"))
                    .0;
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target,
                        data: None,
                    },
                )]
            };
            app.open_access_settings(&ctx, None);
            let _ = frame(&mut app, vec![]);
            let output = frame(&mut app, vec![]);
            let _ = frame(&mut app, click(output, group));
            let output = frame(&mut app, vec![]);
            let _ = frame(&mut app, click(output, action));
            assert!(
                !app.show_settings,
                "{action} must not leave Settings covering the help window"
            );
            assert!(app.a11y.show_shortcuts || app.a11y.show_glossary);
            let output = frame(&mut app, vec![]);
            let _ = frame(&mut app, click(output, close));
            assert!(
                !app.a11y.show_shortcuts && !app.a11y.show_glossary,
                "The auxiliary close control must remain interactive"
            );
        }
    }

    #[test]
    fn expanded_access_controls_fit_narrow_settings_columns() {
        for width in [100.0, 160.0, 280.0] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(Settings::default(), None);
            let mut output = None;
            for _ in 0..2 {
                app.theme(&ctx);
                output = Some(ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 20_000.0))), ..Default::default() }, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        for salt in ["access-setups", "access-display", "access-pointer", "access-switch", "access-speech", "access-attention", "access-numbers"] {
                            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ctx, ui.make_persistent_id(salt), true);
                            state.set_open(true);
                            state.store(ctx);
                        }
                        app.settings_contents(ui);
                    });
                }));
            }
            let tree = output.unwrap().platform_output.accesskit_update.unwrap();
            for (_, node) in &tree.nodes {
                if matches!(
                    node.role(),
                    egui::accesskit::Role::Button
                        | egui::accesskit::Role::CheckBox
                        | egui::accesskit::Role::RadioButton
                        | egui::accesskit::Role::TextInput
                        | egui::accesskit::Role::ComboBox
                ) && let Some(rect) = node.bounds()
                {
                    let geometry = ctx.data(|data| {
                        data.get_temp::<(f32, f32, f32, f32, egui::Vec2)>(egui::Id::new((
                            "settings-checkbox-geometry",
                            node.label().unwrap_or(""),
                        )))
                    });
                    assert!(
                        rect.x0 >= -0.5 && rect.x1 <= f64::from(width) + 0.5,
                        "{} must fit width {width}: {rect:?}; checkbox geometry (available, column, icon, gap, text): {geometry:?}",
                        node.label().unwrap_or("control")
                    );
                }
            }
            for label in [
                "Body text size",
                "Key target size",
                "Speech voice",
                "Focus mode",
                "Use a decimal comma",
                "Reset access options",
            ] {
                assert!(
                    tree.nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(label)),
                    "Missing {label} at width {width}"
                );
            }
        }
    }

    #[test]
    fn setups_preserve_learning_and_calculator_work() {
        let mut app = EqualApp::with_settings(
            Settings {
                level: 3,
                degrees: false,
                precision: 10,
                ..Settings::default()
            },
            None,
        );
        app.expression = "12/3".into();
        app.result = "4".into();
        for setup in [
            Setup::LargePrint,
            Setup::LargeTargets,
            Setup::Reading,
            Setup::Focus,
            Setup::Quiet,
        ] {
            apply_setup(&mut app.settings, setup);
            assert_eq!(app.settings.level, 3);
            assert!(!app.settings.degrees);
            assert_eq!(app.settings.precision, 10);
            assert_eq!(app.expression, "12/3");
            assert_eq!(app.result, "4");
        }
        assert_eq!(app.settings.access.layout, CalculatorLayout::Accessible);
        assert!(!app.settings.access.show_scores);
        assert!(!app.settings.speech && !app.settings.access.graph_sound);
    }
}
