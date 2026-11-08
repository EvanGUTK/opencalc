use crate::*;
use egui::{FontId, Pos2, Rect, Stroke, Vec2};

const DEVICE_MIN_SCALE: f32 = 0.9_f32;
const DEVICE_MAX_SCALE: f32 = 1.35_f32;
const MINIMUM_KEY_TARGET: f32 = 24.0_f32;

impl EqualApp {
    pub fn draw(&mut self, ctx: &egui::Context) {
        if self.show_workspace || self.tab != Tab::Calculator {
            self.header(ctx);
            self.sidebar(ctx);
            self.footer(ctx);
            self.content(ctx);
        } else {
            self.handheld(ctx);
        }
        self.settings_dialog(ctx);
        crate::appearance::reading_guide(
            ctx,
            self.settings.access.reading_guide,
            self.a11y.guide_y,
        );
    }

    pub fn handheld(&mut self, ctx: &egui::Context) {
        let palette = crate::appearance::palette(ctx);
        egui::TopBottomPanel::top("handheld-controls")
            .frame(egui::Frame::new().fill(palette.paper).inner_margin(8))
            .show(ctx, |ui| {
                if self.access_overlay_open() {
                    ui.disable();
                }
                ui.horizontal_wrapped(|ui| {
                    ui.label(R::new("OpenCalc").strong());
                    ui.label(LEVELS[self.settings.level]);
                    let settings = ui.button("Settings");
                    ctx.data_mut(|data| {
                        data.insert_temp(egui::Id::new("settings-current-trigger"), settings.id)
                    });
                    if settings.clicked() {
                        self.open_access_settings(ctx, Some(settings.id));
                    }
                    if ui.button("Workspace").clicked() {
                        self.show_workspace = true;
                    }
                    if ui.button("Notebook").clicked() {
                        self.open_notebook_window(ctx);
                    }
                    if ui.button("Read answer").clicked() {
                        self.speak(&self.result.clone());
                    }
                    if ui.button("Stop reading").clicked() {
                        self.message = match self.speech.stop() {
                            Ok(()) => "Reading stopped.".into(),
                            Err(error) => error,
                        };
                    }
                });
                if !self.message.is_empty() {
                    let status = ui.label(&self.message);
                    ui.ctx().accesskit_node_builder(status.id, |node| {
                        node.set_role(egui::accesskit::Role::Status);
                        node.set_label(format!("Status: {}", self.message));
                        node.clear_value();
                        node.set_live(egui::accesskit::Live::Polite);
                        node.set_live_atomic();
                    });
                }
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette.panel).inner_margin(6))
            .show(ctx, |ui| {
                if self.access_overlay_open() {
                    ui.disable();
                }
                self.calculator(ui);
            });
    }

    pub fn theme(&self, ctx: &egui::Context) {
        crate::appearance::apply(ctx, &self.settings);
    }

    pub(crate) fn calculator(&mut self, ui: &mut egui::Ui) {
        if self.access_overlay_open() && ui.is_enabled() {
            ui.disable();
        }
        if self.settings.access.layout == crate::accessibility::CalculatorLayout::Accessible {
            self.accessible_calculator(ui);
            return;
        }
        // Fit the physical window once; the UI zoom then enlarges the complete
        // device instead of immediately fitting the enlarged keys down again.
        let available = ui.available_size() * self.settings.scale;
        let design_height = if self.settings.level == 0 {
            476.0_f32
        } else {
            657.0_f32
        };
        let scale = device_scale(available, design_height);
        egui::ScrollArea::both()
            .id_salt("calculator-device-scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let width = (323.0_f32 * scale).max(ui.available_width());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0_f32;
                    ui.add_space(((width - 323.0_f32 * scale) / 2.0_f32).max(0.0_f32));
                    if self.settings.level >= 2 {
                        self.ti_calculator(ui, scale);
                    } else {
                        self.school_calculator(ui, scale);
                    }
                });
            });
    }

    fn ti_calculator(&mut self, ui: &mut egui::Ui, scale: f32) {
        let (face, _) = ui.allocate_exact_size(
            Vec2::new(323.0_f32, 657.0_f32) * scale,
            egui::Sense::hover(),
        );
        let area = |x: f32, y: f32, w: f32, h: f32| {
            Rect::from_min_size(face.min + Vec2::new(x, y) * scale, Vec2::new(w, h) * scale)
        };
        handheld::draw_face(ui.painter(), face, scale);
        self.device_display(ui, area(53.0_f32, 52.0_f32, 214.0_f32, 149.0_f32), scale);
        let settings_label = "Learning, settings and accessibility";
        let settings_id = ui.id().with(("calculator-key", settings_label));
        ui.ctx().data_mut(|data| {
            data.insert_temp(egui::Id::new("settings-current-trigger"), settings_id)
        });
        if self.face_key(
            ui,
            area(89.0_f32, 10.0_f32, 145.0_f32, 30.0_f32),
            settings_label,
        ) {
            self.open_access_settings(ui.ctx(), Some(settings_id));
        }
        let mut pressed = None;
        for (col, key) in ["y=", "window", "zoom", "trace", "graph"]
            .iter()
            .enumerate()
        {
            if self.face_key(
                ui,
                area(
                    43.0_f32 + col as f32 * 50.0_f32,
                    246.0_f32,
                    35.0_f32,
                    20.0_f32,
                ),
                key,
            ) {
                pressed = Some(*key);
            }
        }
        for (key, x, y) in [
            ("2nd", 43.0_f32, 284.0_f32),
            ("mode", 93.0_f32, 284.0_f32),
            ("delete", 143.0_f32, 284.0_f32),
            ("alpha", 43.0_f32, 320.0_f32),
            ("x,t", 93.0_f32, 320.0_f32),
            ("stat", 143.0_f32, 320.0_f32),
            ("cursor left", 201.0_f32, 302.0_f32),
            ("cursor right", 244.0_f32, 302.0_f32),
            ("previous calculation", 218.0_f32, 283.0_f32),
            ("cursor end", 218.0_f32, 326.0_f32),
        ] {
            if self.face_key(
                ui,
                area(
                    x,
                    y,
                    if key.contains("cursor") || key == "previous calculation" {
                        23.0_f32
                    } else {
                        35.0_f32
                    },
                    if key.contains("cursor") || key == "previous calculation" {
                        21.0_f32
                    } else {
                        24.0_f32
                    },
                ),
                key,
            ) {
                pressed = Some(key);
            }
        }
        for (y, keys) in [
            (356.0_f32, ["math", "apps", "prgm", "vars", "clear"]),
            (392.0_f32, ["1/x", "sin", "cos", "tan", "^"]),
            (428.0_f32, ["x²", ",", "(", ")", "÷"]),
            (464.0_f32, ["log", "7", "8", "9", "×"]),
            (505.0_f32, ["ln", "4", "5", "6", "−"]),
            (545.0_f32, ["sto", "1", "2", "3", "+"]),
        ] {
            for (col, key) in keys.iter().enumerate() {
                let y = match *key {
                    "ln" | "−" => 500.0_f32,
                    "sto" | "+" => 535.0_f32,
                    _ => y,
                };
                if self.face_key(
                    ui,
                    area(43.0_f32 + col as f32 * 50.0_f32, y, 35.0_f32, 25.0_f32),
                    key,
                ) {
                    pressed = Some(*key);
                }
            }
        }
        for (key, x, y) in [
            ("on", 43.0_f32, 570.0_f32),
            ("0", 93.0_f32, 586.0_f32),
            (".", 143.0_f32, 586.0_f32),
            ("±", 193.0_f32, 586.0_f32),
            ("enter", 243.0_f32, 570.0_f32),
        ] {
            if self.face_key(ui, area(x, y, 35.0_f32, 26.0_f32), key) {
                pressed = Some(key);
            }
        }
        if let Some(key) = pressed {
            self.activate_access_key(ui.ctx(), key);
        }
    }

    fn face_key(&self, ui: &mut egui::Ui, rect: Rect, key: &str) -> bool {
        let active = (key == "2nd" && self.second) || (key == "alpha" && self.device.alpha);
        let target = rect.expand2(Vec2::new(
            ((MINIMUM_KEY_TARGET - rect.width()) / 2.0_f32).max(0.0_f32),
            ((MINIMUM_KEY_TARGET - rect.height()) / 2.0_f32).max(0.0_f32),
        ));
        let response = ui.interact(
            target,
            ui.id().with(("calculator-key", key)),
            egui::Sense::click(),
        );
        let label = device_key_label(key);
        response.widget_info(|| {
            if matches!(key, "2nd" | "alpha") {
                egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, active, label)
            } else {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label)
            }
        });
        handheld::draw_key(
            ui.painter(),
            rect,
            key,
            response.is_pointer_button_down_on(),
        );
        if active || response.hovered() || response.has_focus() {
            ui.painter().rect_stroke(
                rect.expand(1.0_f32),
                4.0_f32,
                Stroke::new(
                    if active || response.has_focus() {
                        2.5_f32
                    } else {
                        1.5_f32
                    },
                    if self.settings.contrast {
                        C::WHITE
                    } else {
                        C::from_rgb(255, 220, 80)
                    },
                ),
                egui::StrokeKind::Outside,
            );
        }
        if active {
            ui.painter().text(
                rect.right_top() + Vec2::new(2.0_f32, -2.0_f32),
                egui::Align2::RIGHT_BOTTOM,
                "ON",
                FontId::monospace(8.0_f32),
                C::WHITE,
            );
        }
        let clicked = response.clicked()
            || (response.has_focus()
                && ui
                    .input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)));
        response.on_hover_text(label);
        clicked
    }

    fn school_calculator(&mut self, ui: &mut egui::Ui, scale: f32) {
        let basic = self.settings.level == 0;
        let height = if basic { 476.0_f32 } else { 657.0_f32 };
        let (face, _) =
            ui.allocate_exact_size(Vec2::new(323.0_f32, height) * scale, egui::Sense::hover());
        let area = |x: f32, y: f32, w: f32, h: f32| {
            Rect::from_min_size(face.min + Vec2::new(x, y) * scale, Vec2::new(w, h) * scale)
        };
        let shell = if self.settings.contrast {
            C::BLACK
        } else if basic {
            C::from_rgb(12, 99, 145)
        } else {
            C::from_rgb(39, 120, 157)
        };
        let p = ui.painter();
        p.rect_filled(
            face.shrink(3.0_f32 * scale)
                .translate(Vec2::new(2.0_f32, 3.0_f32) * scale),
            27.0_f32 * scale,
            C::from_gray(180),
        );
        p.rect_filled(
            face.shrink(5.0_f32 * scale),
            26.0_f32 * scale,
            if basic { shell } else { C::from_gray(225) },
        );
        let body = if basic {
            face.shrink(8.0_f32 * scale)
        } else {
            area(20.0_f32, 13.0_f32, 283.0_f32, 626.0_f32)
        };
        p.rect_filled(body, 24.0_f32 * scale, shell);
        p.text(
            area(28.0_f32, height - 28.0_f32, 0.0_f32, 0.0_f32).min,
            egui::Align2::LEFT_TOP,
            LEVELS[self.settings.level],
            FontId::monospace(11.0_f32 * scale),
            C::WHITE,
        );
        if basic {
            // The inset LCD and red memory/function keys mirror the elementary handheld.
            p.rect_filled(
                area(20.0_f32, 67.0_f32, 283.0_f32, 123.0_f32),
                6.0_f32 * scale,
                C::from_rgb(24, 39, 44),
            );
            self.device_display(ui, area(26.0_f32, 73.0_f32, 271.0_f32, 111.0_f32), scale);
            for (row, keys) in [
                ["±", "√", "%", "off"],
                ["MRC", "M-", "M+", "÷"],
                ["7", "8", "9", "×"],
                ["4", "5", "6", "−"],
                ["1", "2", "3", "+"],
                ["on", "0", ".", "enter"],
            ]
            .iter()
            .enumerate()
            {
                self.school_key_row(
                    ui,
                    face.min + Vec2::new(27.0_f32, 207.0_f32 + row as f32 * 37.0_f32) * scale,
                    62.0_f32 * scale,
                    31.0_f32 * scale,
                    keys,
                    basic,
                );
            }
        } else {
            p.rect_filled(
                area(35.0_f32, 66.0_f32, 253.0_f32, 159.0_f32),
                9.0_f32 * scale,
                C::from_rgb(29, 35, 37),
            );
            self.device_display(ui, area(41.0_f32, 72.0_f32, 241.0_f32, 147.0_f32), scale);
            for (row, keys) in [
                ["2nd", "mode", "delete", "data", "stat"],
                ["fraction", "convert", "1/x", "x²", "^"],
                ["π", "sin", "cos", "tan", "Ans"],
                ["log", "ln", "(", ")", "sto"],
                ["MRC", "M-", "M+", "√", "%"],
                ["7", "8", "9", "÷", "clear"],
                ["4", "5", "6", "×", "cursor left"],
                ["1", "2", "3", "−", "cursor right"],
                ["on", "0", ".", "+", "enter"],
            ]
            .iter()
            .enumerate()
            {
                self.school_key_row(
                    ui,
                    face.min + Vec2::new(32.0_f32, 247.0_f32 + row as f32 * 40.0_f32) * scale,
                    47.0_f32 * scale,
                    31.0_f32 * scale,
                    keys,
                    basic,
                );
            }
        }
    }

    fn school_key_row(
        &mut self,
        ui: &mut egui::Ui,
        start: Pos2,
        width: f32,
        height: f32,
        keys: &[&str],
        basic: bool,
    ) {
        let scale = width / if basic { 62.0_f32 } else { 47.0_f32 };
        for (index, key) in keys.iter().enumerate() {
            let rect = Rect::from_min_size(
                start + Vec2::new(index as f32 * (width + 7.0_f32 * scale), 0.0_f32),
                Vec2::new(width, height),
            );
            let target = rect.expand2(Vec2::new(
                0.0_f32,
                ((MINIMUM_KEY_TARGET - height) / 2.0_f32).max(0.0_f32),
            ));
            let response = ui.interact(
                target,
                ui.id().with(("school-key", *key)),
                egui::Sense::click(),
            );
            let label = device_key_label(key);
            let selected = *key == "2nd" && self.second;
            response.widget_info(|| {
                if *key == "2nd" {
                    egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, selected, label)
                } else {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label)
                }
            });
            let number = key.chars().all(|c| c.is_ascii_digit()) || *key == ".";
            let function = matches!(
                *key,
                "±" | "√"
                    | "%"
                    | "off"
                    | "on"
                    | "MRC"
                    | "M-"
                    | "M+"
                    | "enter"
                    | "÷"
                    | "×"
                    | "−"
                    | "+"
            );
            let fill = if self.settings.contrast {
                if number { C::WHITE } else { C::BLACK }
            } else if basic && function {
                C::from_rgb(186, 31, 40)
            } else if number {
                C::from_rgb(246, 247, 243)
            } else if *key == "2nd" {
                C::from_rgb(32, 58, 84)
            } else {
                C::from_rgb(32, 69, 87)
            };
            let text = if number { C::BLACK } else { C::WHITE };
            let p = ui.painter();
            p.rect_filled(
                rect.translate(Vec2::new(0.0_f32, 2.0_f32 * scale)),
                4.0_f32 * scale,
                C::from_gray(17),
            );
            p.rect_filled(
                rect,
                4.0_f32 * scale,
                if response.is_pointer_button_down_on() {
                    fill.gamma_multiply(0.75_f32)
                } else {
                    fill
                },
            );
            p.rect_stroke(
                rect,
                4.0_f32 * scale,
                Stroke::new(
                    if self.settings.contrast {
                        1.5_f32
                    } else {
                        0.7_f32
                    },
                    if number { C::from_gray(120) } else { C::WHITE },
                ),
                egui::StrokeKind::Inside,
            );
            let printed = match *key {
                "on" if basic => "ON/C",
                "on" => "on",
                "off" => "OFF",
                "enter" if basic => "=",
                "fraction" => "n/d",
                "convert" => "F↔D",
                "delete" => "del",
                "cursor left" => "<",
                "cursor right" => ">",
                _ => key,
            };
            let size = if number || matches!(*key, "+" | "−" | "×" | "÷") {
                21.0_f32
            } else if key.len() > 5 {
                11.0_f32
            } else {
                13.5_f32
            };
            p.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                printed,
                if self.settings.readable {
                    FontId::monospace(size * scale)
                } else {
                    FontId::proportional(size * scale)
                },
                text,
            );
            if response.hovered() || response.has_focus() || selected {
                p.rect_stroke(
                    rect.expand(2.0_f32),
                    4.0_f32,
                    Stroke::new(
                        2.5_f32,
                        if self.settings.contrast {
                            C::WHITE
                        } else {
                            C::from_rgb(255, 223, 50)
                        },
                    ),
                    egui::StrokeKind::Outside,
                );
            }
            if selected {
                p.text(
                    rect.right_top(),
                    egui::Align2::RIGHT_TOP,
                    "ON",
                    FontId::monospace(7.0_f32),
                    C::WHITE,
                );
            }
            if response.clicked()
                || (response.has_focus()
                    && ui.input(|i| {
                        i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)
                    }))
            {
                self.activate_access_key(ui.ctx(), key);
            }
            response.on_hover_text(label);
        }
    }
}

fn device_scale(available: Vec2, height: f32) -> f32 {
    let by_width = available.x / 323.0_f32;
    let by_height = if available.y.is_finite() {
        available.y / height
    } else {
        DEVICE_MAX_SCALE
    };
    by_width
        .min(by_height)
        .clamp(DEVICE_MIN_SCALE, DEVICE_MAX_SCALE)
}

fn device_key_label(key: &str) -> &str {
    match key {
        "Learning, settings and accessibility" => "Learning, settings and accessibility",
        "apps" => "Apps: open the learning toolkit",
        "prgm" => "Programs",
        "vars" => "Variables",
        "alpha" => "Toggle alphabet keys",
        "x,t" => "Insert variable x",
        "stat" => "Statistics: data and regression tools",
        "mode" => "Calculator mode",
        "math" => "Math functions",
        "sto" => "Store value in a variable",
        "on" => "Power on or clear",
        "off" => "Power off",
        "±" => "Negate expression; 2nd recalls Ans",
        "enter" => "Calculate answer",
        "delete" => "Delete last character",
        "2nd" => "Toggle second functions",
        "y=" => "Edit graph functions",
        "window" => "Graph window settings",
        "zoom" => "Graph zoom menu",
        "trace" => "Trace graph on calculator",
        "graph" => "Graph on calculator",
        "MRC" => "Recall or clear memory",
        "M+" => "Add to memory",
        "M-" => "Subtract from memory",
        "fraction" => "Enter fraction",
        "convert" => "Convert fraction or decimal",
        "data" => "Edit statistical data",
        "√" => "Square root",
        "÷" => "Divide",
        "×" => "Multiply",
        "−" => "Subtract",
        "+" => "Add",
        _ => key,
    }
}

pub fn primary(ui: &mut egui::Ui, s: &str) -> egui::Response {
    let palette = crate::appearance::palette(ui.ctx());
    ui.add(
        egui::Button::new(R::new(s).color(palette.selected_text))
            .fill(palette.accent)
            .corner_radius(7.0_f32),
    )
}
pub fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    let palette = crate::appearance::palette(ui.ctx());
    egui::Frame::new()
        .fill(palette.paper)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .inner_margin(22)
        .corner_radius(14.0_f32)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_navigation_text_stays_white_in_both_contrast_modes() {
        for contrast in [false, true] {
            let ctx = egui::Context::default();
            let mut app = EqualApp::with_settings(
                Settings {
                    contrast,
                    ..Settings::default()
                },
                None,
            );
            app.show_workspace = true;
            let mut output = None;
            for _ in 0..2 {
                app.theme(&ctx);
                output = Some(ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(960.0_f32, 940.0_f32),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw(ctx),
                ));
            }
            let output = output.expect("workspace frame");
            let text = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.text() == "Calculator" => Some(text),
                    _ => None,
                })
                .expect("selected Calculator tab");
            for section in &text.galley.job.sections {
                let color = text.override_text_color.unwrap_or_else(|| {
                    if section.format.color == C::PLACEHOLDER {
                        text.fallback_color
                    } else {
                        section.format.color
                    }
                });
                assert_eq!(
                    color,
                    C::WHITE,
                    "selected tab is unreadable; contrast={contrast}"
                );
            }
        }
    }

    #[test]
    fn calculator_zoom_enlarges_keys_in_a_fixed_size_window() {
        let physical_key_width = |scale| {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(
                Settings {
                    scale,
                    ..Settings::default()
                },
                None,
            );
            let physical_size = Vec2::new(360.0_f32, 720.0_f32);
            let mut output = None;
            for _ in 0..3 {
                app.theme(&ctx);
                output = Some(ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            physical_size / ctx.zoom_factor(),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame::NONE)
                            .show(ctx, |ui| {
                                app.calculator(ui);
                            });
                    },
                ));
            }
            let output = output.expect("calculator frame");
            let tree = output
                .platform_output
                .accesskit_update
                .expect("accessible controls");
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Edit graph functions"))
                .and_then(|(_, node)| node.bounds())
                .expect("graph function key bounds");
            bounds.width() * f64::from(output.pixels_per_point)
        };
        let normal = physical_key_width(1.0_f32);
        let magnified = physical_key_width(2.0_f32);
        assert!(
            magnified >= normal * 1.9 && magnified <= normal * 2.1,
            "200% zoom must enlarge keys: normal={normal}, magnified={magnified}"
        );
    }

    #[test]
    fn each_school_level_has_its_own_accessible_handheld() {
        for (level, case_width, case_height, key) in [
            (0, 317.0, 470.0, "Recall or clear memory"),
            (1, 317.0, 651.0, "Enter fraction"),
            (2, 307.0, 648.0, "Graph on calculator"),
            (3, 307.0, 648.0, "Graph on calculator"),
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(
                Settings {
                    level,
                    ..Settings::default()
                },
                None,
            );
            app.message = "Speech is unavailable.".into();
            app.theme(&ctx);
            let frame = |app: &mut EqualApp| {
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(360.0_f32, 720.0_f32),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw(ctx),
                )
            };
            frame(&mut app);
            let output = frame(&mut app);
            let case_color = C::from_gray(if level < 2 { 180 } else { 201 });
            assert!(
                output.shapes.iter().any(|shape| match &shape.shape {
                    egui::Shape::Rect(rect) if rect.fill == case_color => {
                        (rect.rect.width() / rect.rect.height() - case_width / case_height).abs()
                            < 0.001
                    }
                    _ => false,
                }),
                "calculator casing changed at level {level}"
            );
            for shape in &output.shapes {
                if let egui::Shape::Text(text) = &shape.shape {
                    let label = text.galley.text().to_ascii_uppercase();
                    assert!(
                        !["TI-", "PLUS CE", "TEXAS", "INSTRUMENTS"]
                            .iter()
                            .any(|brand| label.contains(brand)),
                        "vendor branding remained at level {level}: {label}"
                    );
                }
            }
            assert_eq!(
                output
                    .shapes
                    .iter()
                    .filter(|shape| matches!(
                        &shape.shape,
                        egui::Shape::Text(text) if text.galley.text() == "OpenCalc"
                    ))
                    .count(),
                1,
                "the app name belongs in the toolbar only"
            );
            let tree = output
                .platform_output
                .accesskit_update
                .expect("accessible controls");
            for label in [
                key,
                "Calculate answer",
                "Settings",
                "Status: Speech is unavailable.",
            ] {
                assert!(
                    tree.nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(label)),
                    "missing {label} at level {level}"
                );
            }
            let status = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Status: Speech is unavailable."))
                .expect("speech status announcement");
            assert_eq!(status.1.role(), egui::accesskit::Role::Status);
        }
    }
}
