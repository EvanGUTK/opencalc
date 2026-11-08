use crate::ui::{card, primary};
use crate::*;
use egui::{FontId, Pos2, Rect, Stroke, Vec2};

const MIN_TRACE_STEP: f64 = 0.000_000_1;
const MAX_GRAPH_COORDINATE: f64 = 1.0e12;
const MAX_TABLE_ROWS: usize = 100;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GraphAccess {
    pub(crate) trace_x: f64,
    pub(crate) trace_step: f64,
    pub(crate) selected: usize,
    pub(crate) table_start: f64,
    pub(crate) table_step: f64,
    pub(crate) table_rows: usize,
    pub(crate) keyboard_trace: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GraphNotebook {
    pub(crate) range: f64,
    pub(crate) center: [f64; 2],
    pub(crate) surface: bool,
    pub(crate) table: bool,
    pub(crate) trace: GraphAccess,
    pub(crate) drawings: Vec<graph_tools::Drawing>,
}

impl GraphNotebook {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !self.range.is_finite() || !(0.05..=1000.0).contains(&self.range) {
            return Err("Graph range must be between 0.05 and 1000.".into());
        }
        if self
            .center
            .iter()
            .any(|value| !value.is_finite() || value.abs() > MAX_GRAPH_COORDINATE)
        {
            return Err(
                "Graph center coordinates must be finite and between -1e12 and 1e12.".into(),
            );
        }
        if !self.trace.trace_x.is_finite() || self.trace.trace_x.abs() > MAX_GRAPH_COORDINATE {
            return Err("Graph trace x must be finite and between -1e12 and 1e12.".into());
        }
        if !self.trace.trace_step.is_finite()
            || !(MIN_TRACE_STEP..=1.0e6).contains(&self.trace.trace_step)
        {
            return Err("Graph trace step must be between 0.0000001 and 1000000.".into());
        }
        if self.trace.selected >= 10 {
            return Err("Graph trace must select a curve from Y1 to Y10.".into());
        }
        if self.trace.table_step.abs() > MAX_GRAPH_COORDINATE {
            return Err(
                "Graph table step must be between -1e12 and 1e12 and cannot be zero.".into(),
            );
        }
        table_coordinates(
            self.trace.table_start,
            self.trace.table_step,
            self.trace.table_rows,
        )?;
        if self.drawings.len() > graph_tools::MAX_DRAWINGS {
            return Err(format!(
                "Graph notebooks support at most {} drawings.",
                graph_tools::MAX_DRAWINGS
            ));
        }
        for (index, drawing) in self.drawings.iter().enumerate() {
            drawing
                .validate()
                .map_err(|error| format!("Graph drawing {}: {error}", index + 1))?;
        }
        Ok(())
    }
}
impl Default for GraphAccess {
    fn default() -> Self {
        Self {
            trace_x: 0.0,
            trace_step: 0.5,
            selected: 0,
            table_start: -3.0,
            table_step: 1.0,
            table_rows: 7,
            keyboard_trace: false,
        }
    }
}

fn trace_value(expression: &str, x: f64, degrees: bool) -> Result<f64, String> {
    if !x.is_finite() {
        return Err("Enter a finite x coordinate".into());
    }
    math::parse(expression)?.eval(x, 0.0, 0.0, degrees)
}

fn table_coordinates(start: f64, step: f64, rows: usize) -> Result<Vec<f64>, String> {
    if !start.is_finite() || !step.is_finite() || step == 0.0 {
        return Err("Table start and step must be finite; step cannot be zero.".into());
    }
    if !(1..=MAX_TABLE_ROWS).contains(&rows) {
        return Err(format!("Use between 1 and {MAX_TABLE_ROWS} table rows."));
    }
    let values: Vec<_> = (0..rows).map(|row| start + row as f64 * step).collect();
    if values
        .iter()
        .any(|x| !x.is_finite() || x.abs() > MAX_GRAPH_COORDINATE)
    {
        return Err("Table coordinates must be between -1e12 and 1e12.".into());
    }
    Ok(values)
}

fn curve_color(index: usize, contrast: bool) -> C {
    const COLORS: [C; 10] = [
        TEAL,
        C::from_rgb(151, 65, 22),
        C::from_rgb(58, 78, 175),
        C::from_rgb(140, 40, 125),
        C::from_rgb(108, 88, 12),
        C::from_rgb(22, 108, 150),
        C::from_rgb(160, 38, 47),
        C::from_rgb(80, 105, 30),
        C::from_rgb(99, 64, 150),
        C::from_rgb(60, 89, 87),
    ];
    if contrast {
        C::BLACK
    } else {
        COLORS[index % COLORS.len()]
    }
}

fn curve_segment_visible(index: usize, segment: usize, patterned: bool) -> bool {
    if !patterned {
        return true;
    }
    match index % 10 {
        0 => true,
        1 => segment % 20 < 10,
        2 => segment % 6 < 3,
        3 => segment % 24 < 12 || (20..22).contains(&(segment % 24)),
        4 => segment % 24 < 18,
        5 => segment % 12 < 8,
        6 => segment % 10 < 2,
        7 => segment % 20 < 2 || (5..7).contains(&(segment % 20)),
        8 => segment % 32 < 24,
        _ => segment % 9 < 6,
    }
}

fn curve_pattern_name(index: usize) -> &'static str {
    [
        "solid",
        "long dash",
        "short dash",
        "dash-dot",
        "wide dash",
        "medium dash",
        "dots",
        "paired dots",
        "extra-long dash",
        "close dash",
    ][index % 10]
}

impl EqualApp {
    pub(crate) fn capture_graph_notebook(&self) -> GraphNotebook {
        GraphNotebook {
            range: self.range,
            center: self.center,
            surface: self.surface,
            table: self.table,
            trace: self.graph_access.clone(),
            drawings: self.graph_tools.drawings.clone(),
        }
    }

    pub(crate) fn apply_graph_notebook(&mut self, notebook: GraphNotebook) {
        self.range = notebook.range;
        self.center = notebook.center;
        self.surface = notebook.surface;
        self.table = notebook.table;
        self.graph_access = notebook.trace;
        self.graph_tools = graph_tools::GraphTools::default();
        self.graph_tools.drawings = notebook.drawings;
        self.graph_error.clear();
    }

    pub(crate) fn graph_function_count(&self) -> usize {
        10
    }

    pub(crate) fn graph_function(&self, index: usize) -> &str {
        match index.min(9) {
            0 => &self.graph_a,
            1 => &self.graph_b,
            index => &self.graph_extra[index - 2],
        }
    }

    pub(crate) fn graph_function_mut(&mut self, index: usize) -> &mut String {
        match index.min(9) {
            0 => &mut self.graph_a,
            1 => &mut self.graph_b,
            index => &mut self.graph_extra[index - 2],
        }
    }

    pub(crate) fn active_graph_indices(&self) -> Vec<usize> {
        if self.surface {
            return vec![0];
        }
        (0..self.graph_function_count())
            .filter(|index| !self.graph_function(*index).trim().is_empty())
            .collect()
    }

    pub(crate) fn graph_value(&self, index: usize, x: f64) -> Result<f64, String> {
        if index >= self.graph_function_count() {
            return Err("Choose a curve from Y1 to Y10.".into());
        }
        let expression = self.graph_function(index);
        if expression.trim().is_empty() {
            return Err(format!("Y{} has no function entered.", index + 1));
        }
        trace_value(expression, x, self.settings.degrees)
    }

    pub(crate) fn graph_table_csv(&self) -> Result<String, String> {
        let coordinates = table_coordinates(
            self.graph_access.table_start,
            self.graph_access.table_step,
            self.graph_access.table_rows,
        )?;
        let active = self.active_graph_indices();
        if active.is_empty() {
            return Err("Enter at least one graph function.".into());
        }
        let names: Vec<_> = active
            .iter()
            .map(|index| {
                if self.surface {
                    "z_at_y_0".into()
                } else {
                    format!("Y{}", index + 1)
                }
            })
            .collect();
        let mut csv = format!("x,{}\n", names.join(","));
        for x in coordinates {
            let values: Vec<_> = active
                .iter()
                .map(|index| {
                    self.graph_value(*index, x)
                        .map(|value| math::format(value, self.settings.precision))
                        .unwrap_or_else(|_| "undefined".into())
                })
                .collect();
            csv.push_str(&format!(
                "{},{}\n",
                math::format(x, self.settings.precision),
                values.join(",")
            ));
        }
        Ok(csv)
    }

    fn cycle_trace_curve(&mut self, forward: bool) {
        let active = self.active_graph_indices();
        if active.is_empty() {
            self.select_trace_curve(0);
            return;
        }
        let position = active
            .iter()
            .position(|index| *index == self.graph_access.selected)
            .unwrap_or(0);
        let position = if forward {
            (position + 1) % active.len()
        } else {
            (position + active.len() - 1) % active.len()
        };
        self.select_trace_curve(active[position]);
    }
    pub(crate) fn trace_text(&self) -> String {
        let selected = if self.surface {
            0
        } else {
            self.graph_access.selected.min(9)
        };
        let expression = self.graph_function(selected);
        let x = self.graph_access.trace_x;
        let prefix = format!(
            "Y{}, x = {}",
            selected + 1,
            math::format(x, self.settings.precision)
        );
        if expression.trim().is_empty() {
            return format!("{prefix}: no function entered.");
        }
        match trace_value(expression, x, self.settings.degrees) {
            Ok(y) => format!(
                "{prefix}, {} = {}",
                if self.surface { "z at y = 0" } else { "y" },
                math::format(y, self.settings.precision)
            ),
            Err(error) => format!("{prefix}: undefined. {error}"),
        }
    }

    pub(crate) fn move_trace(&mut self, delta: f64) {
        let step = if self.graph_access.trace_step.is_finite() {
            self.graph_access
                .trace_step
                .abs()
                .clamp(MIN_TRACE_STEP, 1.0e6)
        } else {
            0.5
        };
        self.graph_access.trace_step = step;
        let old_x = if self.graph_access.trace_x.is_finite() {
            self.graph_access.trace_x
        } else {
            0.0
        };
        if delta.is_finite() {
            self.graph_access.trace_x =
                (old_x + delta * step).clamp(-MAX_GRAPH_COORDINATE, MAX_GRAPH_COORDINATE);
        }
        self.graph_access.keyboard_trace = true;
        // Keep a keyboard trace point visible instead of silently losing it offscreen.
        if (self.graph_access.trace_x - self.center[0]).abs() > self.range {
            self.center[0] = self.graph_access.trace_x;
        }
        if self.settings.speech {
            self.speak(&self.trace_text());
        }
    }

    pub(crate) fn select_trace_curve(&mut self, index: usize) {
        self.graph_access.selected = if self.surface { 0 } else { index.min(9) };
        self.graph_access.keyboard_trace = true;
        if self.settings.speech {
            self.speak(&self.trace_text());
        }
    }

    pub fn graph_workspace(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.heading("Graph");
            ui.horizontal_wrapped(|ui| {
                if ui.button("Reset view").clicked() {
                    self.range = 10.0;
                    self.center = [0.0, 0.0];
                    self.graph_access.trace_x = 0.0;
                }
                ui.checkbox(&mut self.table, "Value table");
                if self.settings.level == 3 {
                    ui.checkbox(&mut self.surface, "3D surface");
                }
                ui.label(if self.settings.degrees {
                    "Degrees"
                } else {
                    "Radians"
                });
            });
            ui.label(if self.surface {
                "Use x and y. The table samples y = 0."
            } else {
                "Enter functions using x."
            });
            let field_width = ui.available_width().clamp(60.0, 360.0);
            let count = if self.surface { 1 } else { 2 };
            for index in 0..count {
                ui.label(if self.surface {
                    "z =".to_string()
                } else {
                    format!("Y{} =", index + 1)
                });
                ui.add(
                    egui::TextEdit::singleline(self.graph_function_mut(index))
                        .desired_width(field_width)
                        .hint_text(if index == 0 {
                            "sin(x)"
                        } else {
                            "Optional function"
                        })
                        .char_limit(1024),
                )
                .widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::TextEdit,
                        true,
                        format!("Y{} graph function", index + 1),
                    )
                });
            }
            if !self.surface {
                ui.collapsing("More functions (Y3 to Y10)", |ui| {
                    for index in 2..10 {
                        ui.label(format!("Y{} =", index + 1));
                        ui.add(
                            egui::TextEdit::singleline(self.graph_function_mut(index))
                                .desired_width(field_width)
                                .char_limit(1024),
                        )
                        .widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::TextEdit,
                                true,
                                format!("Y{} graph function", index + 1),
                            )
                        });
                    }
                });
            }
            ui.horizontal_wrapped(|ui| {
                ui.label("Examples:");
                for sample in if self.surface {
                    ["sin(x)*cos(y)", "x^2-y^2", "sqrt(x^2+y^2)"]
                } else {
                    ["sin(x)", "x^2", "1/x"]
                } {
                    if ui.small_button(sample).clicked() {
                        self.graph_a = sample.into();
                    }
                }
            });
            ui.add_space(6.0);
            self.draw_graph(ui);
            if !self.surface {
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Zoom out").clicked() {
                        self.range = (self.range * 1.4).min(1000.0);
                    }
                    if ui.button("Zoom in").clicked() {
                        self.range = (self.range / 1.4).max(0.05);
                    }
                    for (label, direction) in [
                        ("Pan left", [-1.0, 0.0]),
                        ("Pan right", [1.0, 0.0]),
                        ("Pan up", [0.0, 1.0]),
                        ("Pan down", [0.0, -1.0]),
                    ] {
                        if ui.button(label).clicked() {
                            self.center[0] = (self.center[0] + direction[0] * self.range / 4.0)
                                .clamp(-MAX_GRAPH_COORDINATE, MAX_GRAPH_COORDINATE);
                            self.center[1] = (self.center[1] + direction[1] * self.range / 4.0)
                                .clamp(-MAX_GRAPH_COORDINATE, MAX_GRAPH_COORDINATE);
                        }
                    }
                });
            } else {
                ui.label("Surface: x and y from -10 to 10; displayed z clipped at +/-20.");
            }
            if !self.graph_error.is_empty() {
                ui.label(R::new(&self.graph_error).strong());
            }
            self.trace_controls(ui);
            if !self.surface {
                ui.collapsing("Calculate zeros, intersections, slopes and area", |ui| {
                    self.graph_calculation_menu(ui)
                });
                ui.collapsing("Draw points, lines and circles", |ui| {
                    self.graph_drawing_menu(ui)
                });
                self.graph_description_and_audio(ui);
            }
            if self.table {
                self.value_table(ui);
            }
            ui.separator();
            ui.collapsing("MATLAB export", |ui| {
                ui.horizontal_wrapped(|ui| {
                    if primary(ui, "Open in MATLAB").clicked() {
                        self.message = match matlab::graph_script(&self.graph_a, &self.graph_b, self.surface, self.center, self.range, self.settings.degrees) {
                            Ok(script) => match matlab::open_graph(&script) {
                                Ok(path) => format!("Graph saved to {}. MATLAB launch requested.", path.display()),
                                Err(error) => error,
                            },
                            Err(error) => error,
                        };
                    }
                    if ui.button("Copy MATLAB script").clicked() {
                        match matlab::graph_script(&self.graph_a, &self.graph_b, self.surface, self.center, self.range, self.settings.degrees) {
                            Ok(script) => { ui.ctx().copy_text(script); self.message = "MATLAB script copied.".into(); }
                            Err(error) => self.message = error,
                        }
                    }
                });
                ui.label("Requires a separate MATLAB installation and license. Exports Y1 and Y2 (or the surface), using the selected angle mode.");
                if !self.message.is_empty() { ui.label(&self.message); }
            });
        });
    }

    fn trace_controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.strong("Trace");
        if !self.surface
            && (self.settings.color_safe
                || self.settings.contrast
                || self.active_graph_indices().len() > 2)
        {
            ui.horizontal_wrapped(|ui| {
                for index in self.active_graph_indices() {
                    ui.label(format!("Y{}: {}", index + 1, curve_pattern_name(index)));
                }
            });
        }
        ui.horizontal_wrapped(|ui| {
            for index in self.active_graph_indices() {
                let selected = self.graph_access.selected == index;
                let palette = crate::appearance::palette(ui.ctx());
                if ui
                    .selectable_label(
                        selected,
                        R::new(format!("Trace Y{}", index + 1)).color(if selected {
                            palette.selected_text
                        } else {
                            palette.ink
                        }),
                    )
                    .clicked()
                {
                    self.select_trace_curve(index);
                }
            }
            if ui.button("Previous point").clicked() {
                self.move_trace(-1.0);
            }
            if ui.button("Next point").clicked() {
                self.move_trace(1.0);
            }
            if ui.button("Read coordinates").clicked() {
                self.speak(&self.trace_text());
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("x:");
            let coordinate = ui.add(
                egui::DragValue::new(&mut self.graph_access.trace_x)
                    .speed(0.1)
                    .range(-MAX_GRAPH_COORDINATE..=MAX_GRAPH_COORDINATE),
            );
            coordinate.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, "Trace x coordinate")
            });
            if coordinate.changed() {
                self.move_trace(0.0);
            }
            ui.label("Step:");
            ui.add(
                egui::DragValue::new(&mut self.graph_access.trace_step)
                    .speed(0.1)
                    .range(MIN_TRACE_STEP..=1.0e6),
            )
            .widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, "Trace step")
            });
        });
        let status = ui.label(self.trace_text());
        ui.ctx().accesskit_node_builder(status.id, |node| {
            node.set_role(egui::accesskit::Role::Status);
            node.set_label(self.trace_text());
            node.clear_value();
            node.set_live(egui::accesskit::Live::Polite);
            node.set_live_atomic();
        });
        ui.label("Tab to the graph. Left/Right: move point. Up/Down: choose curve. Enter: read coordinates.");
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui) {
        let height = (ui.available_width() * 0.65).clamp(190.0, 380.0);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::click_and_drag(),
        );
        self.graph_interaction(ui, &response, true);
        self.paint_graph(ui, rect, false);
        self.graph_accessibility(ui, &response);
    }

    pub(crate) fn embedded_graph(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let response = ui.interact(
            rect,
            ui.id().with("embedded-function-graph"),
            egui::Sense::click_and_drag(),
        );
        self.graph_interaction(ui, &response, false);
        self.paint_graph(ui, rect, true);
        self.graph_accessibility(ui, &response);
        if self.graph_access.keyboard_trace {
            // The visible LCD footer already contains this text. Expose it as a
            // separate status node so coordinate changes are announced without
            // moving focus away from the graph or changing the device layout.
            let status_rect = Rect::from_min_max(
                Pos2::new(rect.left(), (rect.bottom() - 17.0).max(rect.top())),
                rect.right_bottom(),
            );
            let status = ui.interact(
                status_rect,
                ui.id().with("embedded-trace-status"),
                egui::Sense::hover(),
            );
            status.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Label, true, self.trace_text())
            });
            ui.ctx().accesskit_node_builder(status.id, |node| {
                node.set_role(egui::accesskit::Role::Status);
                // egui maps Label text to `value`; a Status needs its announced
                // text as a label after changing the role.
                node.set_label(self.trace_text());
                node.clear_value();
                node.set_live(egui::accesskit::Live::Polite);
                node.set_live_atomic();
            });
        }
    }

    fn graph_accessibility(&self, ui: &mut egui::Ui, response: &egui::Response) {
        let functions = self
            .active_graph_indices()
            .iter()
            .map(|index| format!("Y{} = {}", index + 1, self.graph_function(*index)))
            .collect::<Vec<_>>()
            .join(". ");
        let summary = if self.surface {
            format!(
                "Surface graph: z = {}. {} mode. x and y range -10 to 10. {}. Use the value table for coordinates at y = 0.",
                self.graph_a,
                if self.settings.degrees {
                    "Degrees"
                } else {
                    "Radians"
                },
                self.trace_text()
            )
        } else {
            format!(
                "Function graph. {} mode. {}. {}. x range {} to {}. {}. Left and right arrows trace; up and down select curve; Enter reads coordinates.",
                if self.settings.degrees {
                    "Degrees"
                } else {
                    "Radians"
                },
                functions,
                self.trace_text(),
                math::format(self.center[0] - self.range, 4),
                math::format(self.center[0] + self.range, 4),
                self.graph_tools
                    .drawings
                    .iter()
                    .enumerate()
                    .map(|(index, drawing)| format!("D{}: {}", index + 1, drawing.description()))
                    .collect::<Vec<_>>()
                    .join(". ")
            )
        };
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, &summary));
        if response.has_focus() {
            ui.painter().rect_stroke(
                response.rect.shrink(1.5),
                1.0,
                Stroke::new(3.0_f32, crate::appearance::palette(ui.ctx()).accent),
                egui::StrokeKind::Inside,
            );
        }
    }

    fn graph_interaction(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        allow_pointer_pan: bool,
    ) {
        if self.show_settings || self.show_help || self.show_history || !ui.is_enabled() {
            return;
        }
        if response.clicked() {
            response.request_focus();
            self.graph_access.keyboard_trace = true;
        }
        if response.has_focus() {
            let (left, right, up, down, read) = ui.input_mut(|input| {
                (
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                )
            });
            if left {
                self.move_trace(-1.0);
            }
            if right {
                self.move_trace(1.0);
            }
            if up {
                self.cycle_trace_curve(false);
            }
            if down && !self.surface {
                self.cycle_trace_curve(true);
            }
            if read {
                self.speak(&self.trace_text());
            }
        }
        if allow_pointer_pan && !self.surface {
            if response.dragged() {
                // Pointer delta is per-frame; drag_delta would accumulate the whole drag again.
                let delta = ui.input(|input| input.pointer.delta());
                self.center[0] = (self.center[0]
                    - f64::from(delta.x) * 2.0 * self.range / f64::from(response.rect.width()))
                .clamp(-MAX_GRAPH_COORDINATE, MAX_GRAPH_COORDINATE);
                self.center[1] = (self.center[1]
                    + f64::from(delta.y) * 2.0 * self.range / f64::from(response.rect.height()))
                .clamp(-MAX_GRAPH_COORDINATE, MAX_GRAPH_COORDINATE);
            }
            if response.hovered() {
                let scroll = ui.input(|input| input.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    self.range = (self.range * (1.0 - f64::from(scroll) * 0.002).clamp(0.7, 1.3))
                        .clamp(0.05, 1000.0);
                }
            }
        }
    }

    fn paint_graph(&mut self, ui: &mut egui::Ui, rect: Rect, compact: bool) {
        let painter = ui.painter_at(rect);
        let palette = crate::appearance::palette(ui.ctx());
        painter.rect_filled(rect, 2.0, palette.paper);
        self.graph_error.clear();
        if self.surface {
            self.paint_surface(&painter, rect);
            return;
        }
        let label_height = if compact { 17.0 } else { 24.0 };
        let plot = rect.shrink2(Vec2::new(if compact { 3.0 } else { 12.0 }, label_height));
        if plot.width() <= 0.0 || plot.height() <= 0.0 {
            return;
        }
        let plot_painter = painter.with_clip_rect(plot);
        let range = if self.range.is_finite() {
            self.range.clamp(0.05, 1000.0)
        } else {
            10.0
        };
        let to_pos = |x: f64, y: f64| {
            Pos2::new(
                plot.center().x + ((x - self.center[0]) / range) as f32 * plot.width() / 2.0,
                plot.center().y - ((y - self.center[1]) / range) as f32 * plot.height() / 2.0,
            )
        };
        let grid_color = palette.border;
        let axis_color = palette.ink;
        for axis in [false, true] {
            for index in 0..=10 {
                let value = self.center[usize::from(axis)] - range + index as f64 * range / 5.0;
                let points = if axis {
                    [
                        to_pos(self.center[0] - range, value),
                        to_pos(self.center[0] + range, value),
                    ]
                } else {
                    [
                        to_pos(value, self.center[1] - range),
                        to_pos(value, self.center[1] + range),
                    ]
                };
                plot_painter.line_segment(points, Stroke::new(0.5_f32, grid_color));
            }
        }
        for points in [
            [
                to_pos(0.0, self.center[1] - range),
                to_pos(0.0, self.center[1] + range),
            ],
            [
                to_pos(self.center[0] - range, 0.0),
                to_pos(self.center[0] + range, 0.0),
            ],
        ] {
            plot_painter.line_segment(points, Stroke::new(1.2_f32, axis_color));
        }
        let mut valid = 0;
        let samples = if compact { 300 } else { 800 };
        let active = self.active_graph_indices();
        let patterned = self.settings.color_safe || self.settings.contrast || active.len() > 2;
        for index in active {
            let source = self.graph_function(index);
            if source.trim().is_empty() {
                continue;
            }
            let expression = match math::parse(source) {
                Ok(expression) => expression,
                Err(error) => {
                    self.graph_error = format!("Y{}: {error}", index + 1);
                    continue;
                }
            };
            let color = if self.settings.contrast {
                palette.ink
            } else if index == 0 {
                palette.accent
            } else if palette.dark {
                [
                    C::from_rgb(255, 183, 110),
                    C::from_rgb(150, 180, 255),
                    C::from_rgb(245, 150, 225),
                    C::from_rgb(220, 220, 130),
                    C::from_rgb(130, 220, 250),
                    C::from_rgb(255, 150, 155),
                    C::from_rgb(190, 225, 145),
                    C::from_rgb(200, 165, 255),
                    C::from_rgb(150, 215, 200),
                ][(index - 1) % 9]
            } else {
                curve_color(index, false)
            };
            let mut previous: Option<Pos2> = None;
            for sample in 0..=samples {
                let x = self.center[0] - range + 2.0 * range * sample as f64 / samples as f64;
                match expression.eval(x, 0.0, 0.0, self.settings.degrees) {
                    Ok(y) => {
                        valid += 1;
                        let point = to_pos(x, y);
                        // Do not draw across undefined values or apparent discontinuities.
                        if let Some(last) = previous
                            && point.is_finite()
                            && last.is_finite()
                            && (point.y - last.y).abs() < plot.height() * 0.5
                            && curve_segment_visible(index, sample, patterned)
                        {
                            plot_painter.line_segment(
                                [last, point],
                                Stroke::new(if compact { 1.5_f32 } else { 2.3_f32 }, color),
                            );
                        }
                        previous = Some(point);
                    }
                    Err(_) => previous = None,
                }
            }
        }
        if valid == 0 && self.graph_error.is_empty() {
            self.graph_error = "No real values in this window.".into();
        }
        if self.graph_access.keyboard_trace {
            let source = self.graph_function(self.graph_access.selected.min(9));
            if let Ok(y) = trace_value(source, self.graph_access.trace_x, self.settings.degrees) {
                let point = to_pos(self.graph_access.trace_x, y);
                if point.is_finite() {
                    plot_painter.circle_stroke(
                        point,
                        if compact { 3.0 } else { 5.0 },
                        Stroke::new(2.0_f32, axis_color),
                    );
                    plot_painter.line_segment(
                        [point - Vec2::new(5.0, 0.0), point + Vec2::new(5.0, 0.0)],
                        Stroke::new(1.0_f32, axis_color),
                    );
                    plot_painter.line_segment(
                        [point - Vec2::new(0.0, 5.0), point + Vec2::new(0.0, 5.0)],
                        Stroke::new(1.0_f32, axis_color),
                    );
                }
            }
        }
        self.paint_graph_drawings(&plot_painter, plot, range);
        let font = FontId::monospace(if compact { 10.0 } else { 12.0 });
        painter.text(
            rect.left_top() + Vec2::splat(3.0),
            egui::Align2::LEFT_TOP,
            format!(
                "{} curves / trace Y{} / {}",
                self.active_graph_indices().len(),
                self.graph_access.selected.min(9) + 1,
                if self.settings.degrees { "DEG" } else { "RAD" }
            ),
            font.clone(),
            axis_color,
        );
        let bottom = if !self.graph_error.is_empty() {
            self.graph_error.clone()
        } else if self.graph_access.keyboard_trace {
            self.trace_text()
        } else {
            format!(
                "x: {} to {}",
                math::format(self.center[0] - range, 3),
                math::format(self.center[0] + range, 3)
            )
        };
        let galley = painter.layout(bottom, font, axis_color, rect.width() - 6.0);
        painter.galley(
            rect.left_bottom() + Vec2::new(3.0, -label_height + 1.0),
            galley,
            axis_color,
        );
    }

    fn paint_surface(&mut self, painter: &egui::Painter, rect: Rect) {
        let palette = crate::appearance::palette(painter.ctx());
        let expression = match math::parse(&self.graph_a) {
            Ok(expression) => expression,
            Err(error) => {
                self.graph_error = error;
                return;
            }
        };
        let scale = (rect.width() / 580.0).min(rect.height() / 380.0);
        let project = |x: f64, y: f64, z: f64| {
            Pos2::new(
                rect.center().x + ((x - y) * 13.0) as f32 * scale,
                rect.center().y + ((x + y) * 6.0 - z.clamp(-20.0, 20.0) * 8.0) as f32 * scale,
            )
        };
        let mut valid = 0;
        for axis in [false, true] {
            for index in -10..=10 {
                let mut previous = None;
                for sample in 0..=80 {
                    let value = -10.0 + sample as f64 / 4.0;
                    let (x, y) = if axis {
                        (value, index as f64)
                    } else {
                        (index as f64, value)
                    };
                    match expression.eval(x, y, 0.0, self.settings.degrees) {
                        Ok(z) => {
                            valid += 1;
                            let point = project(x, y, z);
                            if let Some(last) = previous
                                && curve_segment_visible(
                                    usize::from(axis),
                                    sample,
                                    self.settings.color_safe || self.settings.contrast,
                                )
                            {
                                painter.line_segment(
                                    [last, point],
                                    Stroke::new(
                                        1.0_f32,
                                        if self.settings.contrast {
                                            palette.ink
                                        } else if axis {
                                            palette.accent
                                        } else if palette.dark {
                                            C::from_rgb(255, 183, 110)
                                        } else {
                                            curve_color(1, false)
                                        },
                                    ),
                                );
                            }
                            previous = Some(point);
                        }
                        Err(_) => previous = None,
                    }
                }
            }
        }
        if valid == 0 {
            self.graph_error = "No real surface values in this range.".into();
        }
        painter.text(
            rect.left_top() + Vec2::splat(4.0),
            egui::Align2::LEFT_TOP,
            "x, y: -10 to 10; z: -20 to 20",
            FontId::monospace(11.0),
            palette.ink,
        );
    }

    pub(crate) fn value_table(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.strong("Value table");
        if self.surface {
            ui.label("z values at y = 0.");
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("Start:");
            ui.add(
                egui::DragValue::new(&mut self.graph_access.table_start)
                    .speed(0.5)
                    .range(-MAX_GRAPH_COORDINATE..=MAX_GRAPH_COORDINATE),
            )
            .widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, "Table start x")
            });
            ui.label("Step:");
            ui.add(egui::DragValue::new(&mut self.graph_access.table_step).speed(0.1))
                .widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, "Table x step")
                });
            ui.label("Rows:");
            ui.add(
                egui::DragValue::new(&mut self.graph_access.table_rows).range(1..=MAX_TABLE_ROWS),
            )
            .widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, "Number of table rows")
            });
            if ui.button("Use graph window").clicked() {
                self.graph_access.table_start = self.center[0] - self.range;
                self.graph_access.table_step =
                    2.0 * self.range / (self.graph_access.table_rows.max(2) - 1) as f64;
            }
        });
        let coordinates = match table_coordinates(
            self.graph_access.table_start,
            self.graph_access.table_step,
            self.graph_access.table_rows,
        ) {
            Ok(values) => values,
            Err(error) => {
                ui.label(error);
                return;
            }
        };
        let active = self.active_graph_indices();
        if active.is_empty() {
            ui.label("Enter at least one graph function.");
            return;
        }
        for x in coordinates {
            ui.group(|ui| {
                ui.strong(format!("x = {}", math::format(x, self.settings.precision)));
                ui.horizontal_wrapped(|ui| {
                    for index in &active {
                        let value = self
                            .graph_value(*index, x)
                            .map(|value| math::format(value, self.settings.precision))
                            .unwrap_or_else(|_| "undefined".into());
                        let label = if self.surface {
                            "z at y=0".to_string()
                        } else {
                            format!("Y{}", index + 1)
                        };
                        ui.label(format!("{label}: {value}"));
                    }
                });
            });
        }
        if ui.button("Copy table as CSV").clicked() {
            match self.graph_table_csv() {
                Ok(csv) => {
                    ui.ctx().copy_text(csv);
                    self.message = "Graph table copied.".into();
                }
                Err(error) => self.message = error,
            }
        }
    }

    pub fn matrix_workspace(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.label(R::new("Matrices").size(23.0).strong());
            ui.label(
                R::new("Separate columns with spaces and rows with a new line or semicolon.")
                    .size(13.0)
                    .color(MUTED),
            );
            ui.add_space(16.0);
            ui.columns(2, |cols| {
                cols[0].label(R::new("MATRIX A").strong().color(TEAL));
                let w = cols[0].available_width();
                cols[0].add_sized(
                    [w, 130.0],
                    egui::TextEdit::multiline(&mut self.matrix_a).font(egui::TextStyle::Monospace),
                );
                cols[1].label(R::new("MATRIX B").strong().color(TEAL));
                let w = cols[1].available_width();
                cols[1].add_sized(
                    [w, 130.0],
                    egui::TextEdit::multiline(&mut self.matrix_b).font(egui::TextStyle::Monospace),
                );
            });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("matrix-operation")
                    .selected_text(&self.matrix_op)
                    .show_ui(ui, |ui| {
                        for op in [
                            "A × B",
                            "A + B",
                            "Transpose A",
                            "Determinant A",
                            "Inverse A",
                        ] {
                            ui.selectable_value(&mut self.matrix_op, op.to_string(), op);
                        }
                    });
                if primary(ui, "Calculate").clicked() {
                    let result = (|| {
                        let a = math::matrix(&self.matrix_a)?;
                        let b = if self.matrix_op.contains('B') {
                            math::matrix(&self.matrix_b)?
                        } else {
                            a.clone()
                        };
                        math::matrix_op(&a, &b, &self.matrix_op)
                    })();
                    self.matrix_result = match result {
                        Ok(rows) => rows
                            .iter()
                            .map(|r| {
                                r.iter()
                                    .map(|&v| math::format(v, self.settings.precision))
                                    .collect::<Vec<_>>()
                                    .join("    ")
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
                        Err(e) => e,
                    };
                }
                if ui.button("Copy result").clicked() {
                    ui.ctx().copy_text(self.matrix_result.clone());
                }
            });
            ui.add_space(15.0);
            egui::Frame::new()
                .fill(BG)
                .corner_radius(10.0)
                .inner_margin(20)
                .show(ui, |ui| {
                    ui.set_min_height(120.0);
                    ui.label(R::new("RESULT").size(11.0).strong().color(TEAL));
                    ui.add_space(9.0);
                    ui.label(
                        R::new(if self.matrix_result.is_empty() {
                            "Your matrix result will appear here."
                        } else {
                            &self.matrix_result
                        })
                        .monospace()
                        .size(21.0),
                    );
                });
            ui.add_space(15.0);
            ui.label(
                R::new("A numerical workspace for learning. Supports matrices up to 12 × 12.")
                    .size(12.0)
                    .color(MUTED),
            );
        });
    }
    pub fn geometry(&mut self, ctx: &egui::Context) {
        egui::Window::new("Geometry tools")
            .open(&mut self.tool)
            .default_width(430.0)
            .show(ctx, |ui| {
                egui::ComboBox::from_id_salt("geometry")
                    .selected_text(
                        ["Right triangle", "Circle", "Rectangle", "Quadratic roots"]
                            [self.tool_kind],
                    )
                    .show_ui(ui, |ui| {
                        for (i, n) in ["Right triangle", "Circle", "Rectangle", "Quadratic roots"]
                            .iter()
                            .enumerate()
                        {
                            ui.selectable_value(&mut self.tool_kind, i, *n);
                        }
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(if self.tool_kind == 1 { "Radius" } else { "a" });
                    ui.add(egui::DragValue::new(&mut self.tool_a).speed(0.1));
                });
                if self.tool_kind != 1 {
                    ui.horizontal(|ui| {
                        ui.label("b");
                        ui.add(egui::DragValue::new(&mut self.tool_b).speed(0.1));
                    });
                }
                if self.tool_kind == 3 {
                    ui.horizontal(|ui| {
                        ui.label("c");
                        ui.add(egui::DragValue::new(&mut self.tool_c).speed(0.1));
                    });
                }
                ui.add_space(12.0);
                let a = self.tool_a;
                let b = self.tool_b;
                let c = self.tool_c;
                let result = if self.tool_kind != 3 && (a < 0.0 || b < 0.0) {
                    "Lengths must be nonnegative.".into()
                } else {
                    match self.tool_kind {
                        0 => format!(
                            "c = √(a² + b²) = {}\nArea = ab / 2 = {}",
                            math::format(a.hypot(b), 6),
                            math::format(a * b / 2.0, 6)
                        ),
                        1 => format!(
                            "Area = πr² = {}\nCircumference = 2πr = {}",
                            math::format(std::f64::consts::PI * a * a, 6),
                            math::format(2.0 * std::f64::consts::PI * a, 6)
                        ),
                        2 => format!(
                            "Area = ab = {}\nPerimeter = 2(a + b) = {}",
                            math::format(a * b, 6),
                            math::format(2.0 * (a + b), 6)
                        ),
                        _ => {
                            let d = b * b - 4.0 * a * c;
                            if a == 0.0 {
                                "a must be nonzero for a quadratic.".into()
                            } else if d < 0.0 {
                                "No real roots (negative discriminant).".into()
                            } else {
                                format!(
                                    "ax² + bx + c = 0\nx₁ = {}\nx₂ = {}",
                                    math::format((-b + d.sqrt()) / (2.0 * a), 6),
                                    math::format((-b - d.sqrt()) / (2.0 * a), 6)
                                )
                            }
                        }
                    }
                };
                ui.label(R::new(result).size(20.0));
            });
    }
}

#[cfg(test)]
mod graph_tests {
    use super::*;

    #[test]
    fn graph_notebook_roundtrip_preserves_sparse_functions_and_preferences() {
        let mut source = EqualApp::with_settings(Settings::default(), None);
        for index in 0..10 {
            source.graph_function_mut(index).clear();
        }
        *source.graph_function_mut(0) = "x^2".into();
        *source.graph_function_mut(9) = "sin(x)+10".into();
        source.range = 21.5;
        source.center = [12.0, -7.5];
        source.surface = false;
        source.table = true;
        source.graph_access = GraphAccess {
            trace_x: 1.25,
            trace_step: 0.125,
            selected: 9,
            table_start: -2.5,
            table_step: -0.25,
            table_rows: 13,
            keyboard_trace: true,
        };
        source.graph_tools.drawings = vec![
            graph_tools::Drawing::Point([1.0, 2.0]),
            graph_tools::Drawing::Line([-2.0, 3.0], [4.0, -5.0]),
            graph_tools::Drawing::Circle {
                center: [2.0, 4.0],
                radius: 1.5,
            },
        ];
        let snapshot = source.capture_graph_notebook();
        snapshot.validate().unwrap();
        let json = serde_json::to_string(&snapshot).unwrap();
        let decoded: GraphNotebook = serde_json::from_str(&json).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded, snapshot);
        let encoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        for separately_owned in [
            "functions",
            "graph_a",
            "graph_b",
            "graph_extra",
            "settings",
            "graph_audio",
        ] {
            assert!(
                encoded.get(separately_owned).is_none(),
                "Functions and preferences have separate notebook owners"
            );
        }

        let functions: [String; 10] =
            std::array::from_fn(|index| source.graph_function(index).into());
        let mut restored = EqualApp::with_settings(Settings::default(), None);
        for (index, function) in functions.iter().enumerate() {
            *restored.graph_function_mut(index) = function.clone();
        }
        restored.settings.access.decimal_comma = true;
        restored.settings.contrast = true;
        restored.settings.level = 2;
        restored.settings.degrees = false;
        restored.settings.precision = 4;
        restored.graph_error = "Old graph error".into();
        restored.apply_graph_notebook(decoded);
        assert_eq!(restored.capture_graph_notebook(), snapshot);
        assert_eq!(restored.active_graph_indices(), vec![0, 9]);
        for (index, function) in functions.iter().enumerate() {
            assert_eq!(restored.graph_function(index), function);
        }
        assert!(restored.settings.access.decimal_comma && restored.settings.contrast);
        assert_eq!(restored.settings.level, 2);
        assert!(!restored.settings.degrees);
        assert_eq!(restored.settings.precision, 4);
        assert!(restored.graph_error.is_empty());
    }

    #[test]
    fn graph_notebook_rejects_invalid_numeric_view_trace_and_table_state() {
        let snapshot = EqualApp::with_settings(Settings::default(), None).capture_graph_notebook();
        let invalid = |change: fn(&mut GraphNotebook)| {
            let mut candidate = snapshot.clone();
            change(&mut candidate);
            assert!(
                candidate.validate().is_err(),
                "Invalid state must be rejected before apply"
            );
        };
        for change in [
            (|value: &mut GraphNotebook| value.range = f64::NAN) as fn(&mut GraphNotebook),
            |value| value.range = 0.049,
            |value| value.range = 1000.1,
            |value| value.center[0] = f64::INFINITY,
            |value| value.center[1] = -1.1e12,
            |value| value.trace.trace_x = f64::NAN,
            |value| value.trace.trace_x = 1.1e12,
            |value| value.trace.trace_step = 0.0,
            |value| value.trace.trace_step = -1.0,
            |value| value.trace.trace_step = 1e-8,
            |value| value.trace.trace_step = f64::INFINITY,
            |value| value.trace.trace_step = 1.1e6,
            |value| value.trace.selected = 10,
            |value| value.trace.table_start = f64::NAN,
            |value| value.trace.table_start = 1.1e12,
            |value| value.trace.table_step = 0.0,
            |value| value.trace.table_step = f64::INFINITY,
            |value| value.trace.table_step = 1.1e12,
            |value| value.trace.table_rows = 0,
            |value| value.trace.table_rows = 101,
            |value| {
                value.trace.table_start = 1.0e12;
                value.trace.table_step = 1.0;
                value.trace.table_rows = 2;
            },
        ] {
            invalid(change);
        }
        let mut encoded = serde_json::to_value(snapshot).unwrap();
        encoded["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<GraphNotebook>(encoded).is_err());
    }

    #[test]
    fn graph_notebook_rejects_invalid_and_excessive_drawings() {
        let mut snapshot =
            EqualApp::with_settings(Settings::default(), None).capture_graph_notebook();
        for drawing in [
            graph_tools::Drawing::Point([f64::NAN, 0.0]),
            graph_tools::Drawing::Point([1.1e12, 0.0]),
            graph_tools::Drawing::Line([1.0, 2.0], [1.0, 2.0]),
            graph_tools::Drawing::Circle {
                center: [0.0, 0.0],
                radius: 0.0,
            },
            graph_tools::Drawing::Circle {
                center: [0.0, 0.0],
                radius: 1.1e6,
            },
            graph_tools::Drawing::Circle {
                center: [0.0, f64::INFINITY],
                radius: 1.0,
            },
        ] {
            snapshot.drawings = vec![drawing];
            assert!(snapshot.validate().is_err());
        }
        snapshot.drawings =
            vec![graph_tools::Drawing::Point([0.0, 0.0]); graph_tools::MAX_DRAWINGS];
        snapshot.validate().unwrap();
        snapshot
            .drawings
            .push(graph_tools::Drawing::Point([1.0, 1.0]));
        assert!(snapshot.validate().is_err());
    }

    #[test]
    fn trace_evaluates_live_functions_and_angle_mode() {
        assert!((trace_value("sin(x)", 90.0, true).unwrap() - 1.0).abs() < 1.0e-10);
        assert!(
            (trace_value("sin(x)", std::f64::consts::FRAC_PI_2, false).unwrap() - 1.0).abs()
                < 1.0e-10
        );
        assert_eq!(trace_value("x^2+1", 3.0, false).unwrap(), 10.0);
        assert!(trace_value("sqrt(x)", -1.0, false).is_err());
        assert!(trace_value("1/x", 0.0, false).is_err());
        assert!(trace_value("x", f64::NAN, false).is_err());
    }

    #[test]
    fn table_uses_custom_coordinates_and_validates_bounds() {
        assert_eq!(
            table_coordinates(10.0, 0.25, 3).unwrap(),
            vec![10.0, 10.25, 10.5]
        );
        assert_eq!(
            table_coordinates(2.0, -0.5, 3).unwrap(),
            vec![2.0, 1.5, 1.0]
        );
        for (start, step, rows) in [
            (0.0, 0.0, 7),
            (0.0, 1.0, 0),
            (0.0, 1.0, 101),
            (f64::NAN, 1.0, 7),
            (1.0e12, 1.0, 2),
            (0.0, f64::INFINITY, 1),
        ] {
            assert!(table_coordinates(start, step, rows).is_err());
        }
    }

    #[test]
    fn high_contrast_curves_remain_distinguishable_without_color() {
        assert_eq!(curve_color(0, true), curve_color(1, true));
        assert!((0..40).all(|index| curve_segment_visible(0, index, true)));
        assert!((0..40).any(|index| !curve_segment_visible(1, index, true)));
        assert!((0..40).any(|index| curve_segment_visible(1, index, true)));
        let patterns: Vec<_> = (0..10)
            .map(|curve| {
                (0..192)
                    .map(|segment| curve_segment_visible(curve, segment, true))
                    .collect::<Vec<_>>()
            })
            .collect();
        for first in 0..10 {
            for second in first + 1..10 {
                assert_ne!(
                    patterns[first], patterns[second],
                    "Every curve needs a distinct noncolor pattern"
                );
            }
        }
    }

    #[test]
    fn ten_curves_share_live_trace_and_table_data() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        for index in 0..app.graph_function_count() {
            *app.graph_function_mut(index) = format!("x+{}", index + 1);
        }
        assert_eq!(app.active_graph_indices(), (0..10).collect::<Vec<_>>());
        app.graph_access.trace_x = 2.0;
        app.select_trace_curve(9);
        assert_eq!(app.trace_text(), "Y10, x = 2, y = 12");
        app.cycle_trace_curve(true);
        assert_eq!(app.graph_access.selected, 0);
        app.cycle_trace_curve(false);
        assert_eq!(app.graph_access.selected, 9);
        app.graph_access.table_start = 2.0;
        app.graph_access.table_step = 0.5;
        app.graph_access.table_rows = 2;
        let csv = app.graph_table_csv().unwrap();
        assert!(csv.starts_with("x,Y1,Y2,Y3,Y4,Y5,Y6,Y7,Y8,Y9,Y10\n"));
        assert!(csv.contains("2,3,4,5,6,7,8,9,10,11,12\n"));
        *app.graph_function_mut(4) = "sqrt(-1)".into();
        assert!(app.graph_table_csv().unwrap().contains("undefined"));
        app.graph_function_mut(9).clear();
        assert!(!app.active_graph_indices().contains(&9));
        assert!(app.graph_value(10, 0.0).is_err());
    }

    #[test]
    fn trace_moves_without_leaving_calculator_and_reports_undefined_points() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.graph_a = "x^2".into();
        app.graph_b = "sqrt(x)".into();
        app.graph_access.trace_step = 1.25;
        app.move_trace(2.0);
        assert_eq!(app.graph_access.trace_x, 2.5);
        assert_eq!(app.trace_text(), "Y1, x = 2.5, y = 6.25");
        assert!(app.tab == Tab::Calculator);
        app.select_trace_curve(1);
        app.move_trace(-4.0);
        assert!(app.trace_text().contains("Y2, x = -2.5: undefined"));
        app.move_trace(20.0);
        assert_eq!(app.center[0], app.graph_access.trace_x);
    }

    #[test]
    fn embedded_graph_is_keyboard_operable_and_exposes_live_coordinates() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.graph_a = "x^2".into();
        app.graph_access.trace_step = 2.0;
        let frame = |app: &mut EqualApp, events: Vec<egui::Event>| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 700.0))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(300.0, 180.0), egui::Sense::hover());
                        ui.memory_mut(|memory| {
                            memory.request_focus(ui.id().with("embedded-function-graph"))
                        });
                        app.embedded_graph(ui, rect);
                    });
                },
            )
        };
        frame(&mut app, vec![]);
        let output = frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(app.graph_access.trace_x, 2.0);
        assert!(app.tab == Tab::Calculator);
        assert!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node
                    .label()
                    .is_some_and(|label| label.contains("Degrees mode")
                        && label.contains("Y1 = x^2")
                        && label.contains("x = 2, y = 4")))
        );
        assert!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Status
                    && node.live() == Some(egui::accesskit::Live::Polite)
                    && node.is_live_atomic()
                    && node.label() == Some("Y1, x = 2, y = 4"))
        );
        app.show_settings = true;
        frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(
            app.graph_access.trace_x, 2.0,
            "Settings must block graph shortcuts"
        );
        app.show_settings = false;
        app.graph_extra[7] = "x+10".into();
        app.graph_tools
            .drawings
            .push(crate::graph_tools::Drawing::Point([1.0, 2.0]));
        app.select_trace_curve(9);
        let output = frame(&mut app, vec![]);
        let nodes = &output.platform_output.accesskit_update.unwrap().nodes;
        assert!(
            nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Status
                    && node.label() == Some("Y10, x = 2, y = 12"))
        );
        assert!(
            nodes
                .iter()
                .any(|(_, node)| node.label().is_some_and(
                    |label| label.contains("Y10 = x+10") && label.contains("D1: Point (1, 2)")
                ))
        );
    }
}
