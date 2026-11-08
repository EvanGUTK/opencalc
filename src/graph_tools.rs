use crate::{EqualApp, appearance, egui, math};
use egui::{Pos2, Rect, Stroke, Vec2};

const ANALYSIS_SAMPLES: usize = 512;
const INTEGRAL_SAMPLES: usize = 1024;
const ITERATIONS: usize = 96;
const MAX_COORDINATE: f64 = 1.0e12;
pub(crate) const MAX_DRAWINGS: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Analysis {
    #[default]
    Value,
    Zero,
    Minimum,
    Maximum,
    Intersection,
    Derivative,
    Integral,
}
impl Analysis {
    const ALL: [Self; 7] = [
        Self::Value,
        Self::Zero,
        Self::Minimum,
        Self::Maximum,
        Self::Intersection,
        Self::Derivative,
        Self::Integral,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Value => "Value at x",
            Self::Zero => "Zero",
            Self::Minimum => "Minimum",
            Self::Maximum => "Maximum",
            Self::Intersection => "Intersection",
            Self::Derivative => "Derivative at x",
            Self::Integral => "Definite integral",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DrawKind {
    #[default]
    Point,
    Line,
    Circle,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum Drawing {
    Point([f64; 2]),
    Line([f64; 2], [f64; 2]),
    Circle { center: [f64; 2], radius: f64 },
}
impl Drawing {
    pub(crate) fn description(&self) -> String {
        let n = |value| math::format(value, 6);
        match self {
            Self::Point([x, y]) => format!("Point ({}, {})", n(*x), n(*y)),
            Self::Line([x, y], [a, b]) => {
                format!("Line from ({}, {}) to ({}, {})", n(*x), n(*y), n(*a), n(*b))
            }
            Self::Circle {
                center: [x, y],
                radius,
            } => format!(
                "Circle centered at ({}, {}), radius {}",
                n(*x),
                n(*y),
                n(*radius)
            ),
        }
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        let point = |point: &[f64; 2]| {
            point
                .iter()
                .all(|value| value.is_finite() && value.abs() <= MAX_COORDINATE)
        };
        let valid = match self {
            Self::Point(value) => point(value),
            Self::Line(first, second) => point(first) && point(second) && first != second,
            Self::Circle { center, radius } => {
                point(center) && radius.is_finite() && *radius > 0.0 && *radius <= 1.0e6
            }
        };
        if valid {
            Ok(())
        } else {
            Err("Use finite coordinates within ±1e12, distinct line endpoints, and a positive circle radius up to 1,000,000.".into())
        }
    }
}

pub(crate) struct GraphTools {
    operation: Analysis,
    first: usize,
    second: usize,
    left: f64,
    right: f64,
    point: f64,
    output: String,
    summary: String,
    audio_status: String,
    draw_kind: DrawKind,
    draw_values: [f64; 4],
    pub(crate) drawings: Vec<Drawing>,
    undo_drawings: Option<Vec<Drawing>>,
}
impl Default for GraphTools {
    fn default() -> Self {
        Self {
            operation: Analysis::Value,
            first: 0,
            second: 1,
            left: -10.0,
            right: 10.0,
            point: 0.0,
            output: String::new(),
            summary: String::new(),
            audio_status: String::new(),
            draw_kind: DrawKind::Point,
            draw_values: [0.0, 0.0, 1.0, 1.0],
            drawings: Vec::new(),
            undo_drawings: None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AnalysisResult {
    pub(crate) value: f64,
    pub(crate) point: Option<[f64; 2]>,
    pub(crate) description: String,
}

fn bounds(left: f64, right: f64) -> Result<(), String> {
    if !left.is_finite()
        || !right.is_finite()
        || left >= right
        || left.abs() > MAX_COORDINATE
        || right.abs() > MAX_COORDINATE
    {
        Err("Use finite lower and upper bounds within ±1e12, with lower < upper.".into())
    } else {
        Ok(())
    }
}

fn evaluate(expression: &math::Expr, x: f64, degrees: bool) -> Result<f64, String> {
    if !x.is_finite() || x.abs() > MAX_COORDINATE {
        return Err("Use a finite x within ±1e12.".into());
    }
    expression.eval(x, 0.0, 0.0, degrees)
}

fn bisect<F: Fn(f64) -> Result<f64, String>>(
    f: &F,
    mut left: f64,
    mut right: f64,
    mut low: f64,
    high: f64,
    scale: f64,
) -> Result<f64, String> {
    if low == 0.0 {
        return Ok(left);
    }
    if high == 0.0 {
        return Ok(right);
    }
    if low.is_sign_positive() == high.is_sign_positive() {
        return Err("No sign change in these bounds.".into());
    }
    // A huge sample near a pole must not loosen the zero residual elsewhere.
    let tolerance = scale.min(low.abs()).min(high.abs()).max(f64::MIN_POSITIVE) * 1e-10;
    let mut candidate = left + (right - left) / 2.0;
    for _ in 0..ITERATIONS {
        candidate = left + (right - left) / 2.0;
        let value = f(candidate)?;
        if value.abs() <= tolerance {
            return Ok(candidate);
        }
        if candidate == left || candidate == right {
            break;
        }
        if value.is_sign_positive() == low.is_sign_positive() {
            left = candidate;
            low = value;
        } else {
            right = candidate;
        }
    }
    if f(candidate)?.abs() <= tolerance {
        Ok(candidate)
    } else {
        Err(
            "The sign change did not converge to a zero; the interval may contain a discontinuity."
                .into(),
        )
    }
}

fn sampled_zero<F: Fn(f64) -> Result<f64, String>>(
    f: &F,
    left: f64,
    right: f64,
) -> Result<f64, String> {
    bounds(left, right)?;
    let samples: Vec<_> = (0..=ANALYSIS_SAMPLES)
        .map(|index| {
            let x = left + (right - left) * index as f64 / ANALYSIS_SAMPLES as f64;
            (x, f(x))
        })
        .collect();
    let scale = samples
        .iter()
        .filter_map(|(_, y)| y.as_ref().ok().copied())
        .map(f64::abs)
        .fold(0.0_f64, f64::max);
    for pair in samples.windows(2) {
        let [(a, first), (b, second)] = pair else {
            continue;
        };
        let (Ok(first), Ok(second)) = (first, second) else {
            continue;
        };
        if *first == 0.0 {
            return Ok(*a);
        }
        if *second == 0.0 {
            return Ok(*b);
        }
        if first.is_sign_positive() != second.is_sign_positive()
            && let Ok(root) = bisect(f, *a, *b, *first, *second, scale)
        {
            return Ok(root);
        }
    }
    // A tangent zero need not change sign. Require improvement from this local
    // candidate; a huge sample near a pole must not relax its residual check.
    if let Some((start, initial)) = samples
        .iter()
        .filter_map(|(x, y)| y.as_ref().ok().map(|y| (*x, *y)))
        .min_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
    {
        let mut x = start;
        for _ in 0..48 {
            let value = f(x)?;
            if value == 0.0 || value.abs() <= initial.abs().max(f64::MIN_POSITIVE) * 1e-10 {
                return Ok(x);
            }
            let h = (right - left) / 100_000.0;
            let lo = (x - h).max(left);
            let hi = (x + h).min(right);
            if hi <= lo {
                break;
            }
            let slope = (f(hi)? - f(lo)?) / (hi - lo);
            if slope == 0.0 || !slope.is_finite() {
                break;
            }
            let next = x - value / slope;
            if !next.is_finite() || next < left || next > right || next == x {
                break;
            }
            x = next;
        }
    }
    Err("No reliable zero found in the interval. Try narrower bounds around a continuous crossing or tangent.".into())
}

fn extreme<F: Fn(f64) -> Result<f64, String>>(
    f: &F,
    left: f64,
    right: f64,
    maximum: bool,
) -> Result<[f64; 2], String> {
    bounds(left, right)?;
    let mut best = [left, f(left)?];
    let mut best_index = 0;
    let mut previous = best;
    for index in 1..=ANALYSIS_SAMPLES {
        let x = left + (right - left) * index as f64 / ANALYSIS_SAMPLES as f64;
        let y = f(x).map_err(|error| format!("Function undefined inside the interval: {error}"))?;
        if previous[1] != 0.0 && y != 0.0 && previous[1].is_sign_positive() != y.is_sign_positive()
        {
            bisect(f, previous[0], x, previous[1], y, previous[1].abs().max(y.abs()))
                .map_err(|_| "An apparent discontinuity crosses the extrema interval. Use bounds within one continuous part of the function.".to_string())?;
        }
        if if maximum { y > best[1] } else { y < best[1] } {
            best = [x, y];
            best_index = index;
        }
        previous = [x, y];
    }
    let width = (right - left) / ANALYSIS_SAMPLES as f64;
    let mut lo = (left + best_index.saturating_sub(1) as f64 * width).max(left);
    let mut hi = (best[0] + width).min(right);
    for _ in 0..64 {
        let a = lo + (hi - lo) / 3.0;
        let b = hi - (hi - lo) / 3.0;
        let ya = f(a)?;
        let yb = f(b)?;
        for candidate in [[a, ya], [b, yb]] {
            if if maximum {
                candidate[1] > best[1]
            } else {
                candidate[1] < best[1]
            } {
                best = candidate;
            }
        }
        if if maximum { ya < yb } else { ya > yb } {
            lo = a;
        } else {
            hi = b;
        }
    }
    Ok(best)
}

fn integrate(expression: &math::Expr, left: f64, right: f64, degrees: bool) -> Result<f64, String> {
    bounds(left, right)?;
    let f = |x| evaluate(expression, x, degrees);
    let step = (right - left) / INTEGRAL_SAMPLES as f64;
    let mut values = Vec::with_capacity(INTEGRAL_SAMPLES + 1);
    for index in 0..=INTEGRAL_SAMPLES {
        values.push(
            f(left + step * index as f64)
                .map_err(|error| format!("Undefined value in integral interval: {error}"))?,
        );
    }
    let scale = values.iter().copied().map(f64::abs).fold(0.0_f64, f64::max);
    for (index, pair) in values.windows(2).enumerate() {
        if pair[0] != 0.0
            && pair[1] != 0.0
            && pair[0].is_sign_positive() != pair[1].is_sign_positive()
        {
            let a = left + step * index as f64;
            bisect(&f, a, a + step, pair[0], pair[1], scale)
                .map_err(|_| "An apparent discontinuity crosses the integral interval. Split the interval at the discontinuity.".to_string())?;
        }
    }
    // Share the calculator's bounded convergence check. Fixed-grid quadrature
    // alone can report a finite answer for an unresolved even-order pole.
    math::Expr::Call(
        "integral".into(),
        vec![
            expression.clone(),
            math::Expr::Number(left),
            math::Expr::Number(right),
        ],
    )
    .eval(0.0, 0.0, 0.0, degrees)
}

pub(crate) fn analyze(
    operation: Analysis,
    first: &str,
    second: &str,
    left: f64,
    right: f64,
    point: f64,
    degrees: bool,
) -> Result<AnalysisResult, String> {
    let expression = math::parse(first).map_err(|error| format!("First curve: {error}"))?;
    let f = |x| evaluate(&expression, x, degrees);
    let (value, location, description) = match operation {
        Analysis::Value => {
            let y = f(point)?;
            (y, Some([point, y]), "Function value")
        }
        Analysis::Zero => {
            let x = sampled_zero(&f, left, right)?;
            (x, Some([x, f(x)?]), "Approximate zero")
        }
        Analysis::Intersection => {
            let other = math::parse(second).map_err(|error| format!("Second curve: {error}"))?;
            let difference = |x| {
                let value = f(x)? - evaluate(&other, x, degrees)?;
                if value.is_finite() {
                    Ok(value)
                } else {
                    Err(
                        "The difference between the curves exceeds the finite numeric range."
                            .into(),
                    )
                }
            };
            let x = sampled_zero(&difference, left, right)?;
            (x, Some([x, f(x)?]), "Approximate intersection")
        }
        Analysis::Minimum | Analysis::Maximum => {
            let result = extreme(&f, left, right, operation == Analysis::Maximum)?;
            (
                result[1],
                Some(result),
                if operation == Analysis::Maximum {
                    "Sampled maximum, locally refined"
                } else {
                    "Sampled minimum, locally refined"
                },
            )
        }
        Analysis::Derivative => {
            let center = f(point)?;
            let h = 1e-5 * (1.0 + point.abs());
            let below = f(point - h)?;
            let above = f(point + h)?;
            let left_slope = (center - below) / h;
            let right_slope = (above - center) / h;
            if (left_slope - right_slope).abs()
                > 0.001 * left_slope.abs().max(right_slope.abs()).max(1.0)
            {
                return Err("The derivative is not stable here; the function may have a corner or discontinuity.".into());
            }
            (
                (above - below) / (2.0 * h),
                Some([point, center]),
                "Approximate derivative",
            )
        }
        Analysis::Integral => (
            integrate(&expression, left, right, degrees)?,
            None,
            "Approximate signed integral",
        ),
    };
    if !value.is_finite() {
        return Err("The result is not finite.".into());
    }
    Ok(AnalysisResult {
        value,
        point: location,
        description: description.into(),
    })
}

pub(crate) fn sample_summary(source: &str, left: f64, right: f64, degrees: bool) -> String {
    if bounds(left, right).is_err() {
        return "Enter a finite graph window.".into();
    }
    let expression = match math::parse(source) {
        Ok(value) => value,
        Err(error) => return format!("Function cannot be sampled: {error}"),
    };
    let values: Vec<_> = (0..=128)
        .map(|index| {
            evaluate(
                &expression,
                left + (right - left) * index as f64 / 128.0,
                degrees,
            )
            .ok()
        })
        .collect();
    let finite: Vec<_> = values.iter().flatten().copied().collect();
    if finite.is_empty() {
        return "No defined real samples in the current window.".into();
    }
    let min = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let max = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let first = finite[0];
    let last = *finite.last().expect("nonempty samples");
    let trend = if (last - first).abs() <= 1e-8 * first.abs().max(last.abs()).max(1.0) {
        "same sampled endpoint height"
    } else if last > first {
        "higher at the last defined sample"
    } else {
        "lower at the last defined sample"
    };
    format!(
        "129 samples: {} defined, {} undefined. Sampled y range {} to {}; {}. Sampling can miss features and does not prove continuity or global extrema.",
        finite.len(),
        values.len() - finite.len(),
        math::format(min, 6),
        math::format(max, 6),
        trend
    )
}

fn numeric(ui: &mut egui::Ui, label: &str, value: &mut f64, decimal_comma: bool) {
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        let response = ui.add(
            egui::DragValue::new(value)
                .speed(0.1)
                .range(-MAX_COORDINATE..=MAX_COORDINATE)
                .custom_formatter(move |value, _| {
                    if decimal_comma {
                        math::format(value, 6).replace('.', ",")
                    } else {
                        math::format(value, 6)
                    }
                })
                .custom_parser(move |text| {
                    if decimal_comma {
                        text.trim().replace(',', ".").parse().ok()
                    } else {
                        text.trim().parse().ok()
                    }
                }),
        );
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::DragValue, true, label));
    });
}

fn status(ui: &mut egui::Ui, text: &str) {
    let response = ui.label(text);
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_label(text);
        node.clear_value();
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

impl EqualApp {
    pub(crate) fn graph_description_and_audio(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Graph text and sound", |ui| {
            ui.label("Select a Trace curve above. Text and sound use that curve in the visible x window.");
            if ui.button("Describe selected curve").clicked() {
                let index = self.graph_access.selected.min(9);
                self.graph_tools.summary = format!("Y{} = {}. {} mode. x from {} to {}. {}", index + 1, self.graph_function(index), if self.settings.degrees { "Degrees" } else { "Radians" }, math::format(self.center[0] - self.range, 6), math::format(self.center[0] + self.range, 6), sample_summary(self.graph_function(index), self.center[0] - self.range, self.center[0] + self.range, self.settings.degrees));
            }
            if !self.graph_tools.summary.is_empty() {
                status(ui, &self.graph_tools.summary);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Read graph description").clicked() { self.speak(&self.graph_tools.summary.clone()); }
                    if ui.button("Copy graph description").clicked() { ui.ctx().copy_text(self.graph_tools.summary.clone()); }
                });
                ui.label("This description is a snapshot. Describe again after changing the function or window.");
            }
            if ui.checkbox(&mut self.settings.access.graph_sound, "Enable graph sound").changed() && !self.settings.access.graph_sound {
                self.graph_tools.audio_status = match self.graph_audio.stop() { Ok(()) => "Graph sound disabled.".into(), Err(error) => error };
            }
            ui.label("Sound moves left to right through 96 samples. Higher pitch means higher y; undefined samples are silent. Sound uses the speech volume setting and starts only with Play.");
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(self.settings.access.graph_sound, egui::Button::new("Play selected curve")).clicked() {
                    let samples: Vec<_> = (0..96).map(|index| self.graph_value(self.graph_access.selected.min(9), self.center[0] - self.range + 2.0 * self.range * index as f64 / 95.0).ok()).collect();
                    self.graph_tools.audio_status = match self.graph_audio.start(&samples, self.settings.access.speech_volume) {
                        Ok(()) => "Started selected curve sound. Pitch is scaled to the sampled y range; use the table for numeric values.".into(),
                        Err(error) => error,
                    };
                }
                if ui.add_enabled(self.graph_audio.is_playing(), egui::Button::new("Stop graph sound")).clicked() {
                    self.graph_tools.audio_status = match self.graph_audio.stop() { Ok(()) => "Graph sound stopped.".into(), Err(error) => error };
                }
            });
            if !self.graph_tools.audio_status.is_empty() { status(ui, &self.graph_tools.audio_status); }
        });
    }

    pub(crate) fn graph_calculation_menu(&mut self, ui: &mut egui::Ui) {
        ui.strong("Graph calculation");
        egui::ComboBox::from_id_salt("graph-analysis-operation")
            .width(ui.available_width().clamp(70.0, 220.0))
            .selected_text(self.graph_tools.operation.label())
            .show_ui(ui, |ui| {
                for operation in Analysis::ALL {
                    ui.selectable_value(
                        &mut self.graph_tools.operation,
                        operation,
                        operation.label(),
                    );
                }
            })
            .response
            .widget_info(|| egui::WidgetInfo {
                current_text_value: Some(self.graph_tools.operation.label().into()),
                ..egui::WidgetInfo::labeled(
                    egui::WidgetType::ComboBox,
                    true,
                    "Graph calculation operation",
                )
            });
        for (label, selected) in [
            ("First curve", &mut self.graph_tools.first),
            ("Second curve", &mut self.graph_tools.second),
        ] {
            if label == "Second curve" && self.graph_tools.operation != Analysis::Intersection {
                continue;
            }
            ui.horizontal_wrapped(|ui| {
                ui.label(label);
                egui::ComboBox::from_id_salt(label)
                    .width(60.0)
                    .selected_text(format!("Y{}", *selected + 1))
                    .show_ui(ui, |ui| {
                        for index in 0..10 {
                            ui.selectable_value(selected, index, format!("Y{}", index + 1));
                        }
                    })
                    .response
                    .widget_info(|| egui::WidgetInfo {
                        current_text_value: Some(format!("Y{}", *selected + 1)),
                        ..egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, label)
                    });
            });
        }
        let comma = self.settings.access.decimal_comma;
        if matches!(
            self.graph_tools.operation,
            Analysis::Value | Analysis::Derivative
        ) {
            numeric(ui, "At x", &mut self.graph_tools.point, comma);
        } else {
            numeric(ui, "Lower bound", &mut self.graph_tools.left, comma);
            numeric(ui, "Upper bound", &mut self.graph_tools.right, comma);
        }
        if ui.button("Calculate graph result").clicked() {
            let result = analyze(
                self.graph_tools.operation,
                self.graph_function(self.graph_tools.first),
                self.graph_function(self.graph_tools.second),
                self.graph_tools.left,
                self.graph_tools.right,
                self.graph_tools.point,
                self.settings.degrees,
            );
            self.graph_tools.output = match result {
                Ok(result) => {
                    if let Some([x, _]) = result.point {
                        self.graph_access.trace_x = x;
                        self.graph_access.selected = self.graph_tools.first.min(9);
                        self.graph_access.keyboard_trace = true;
                    }
                    let location = result
                        .point
                        .map(|[x, y]| {
                            format!(
                                " at x = {}, y = {}",
                                math::format(x, self.settings.precision),
                                math::format(y, self.settings.precision)
                            )
                        })
                        .unwrap_or_default();
                    format!(
                        "{}: {}{location}",
                        result.description,
                        math::format(result.value, self.settings.precision)
                    )
                }
                Err(error) => error,
            };
        }
        if !self.graph_tools.output.is_empty() {
            status(ui, &self.graph_tools.output);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Read graph result").clicked() {
                    self.speak(&self.graph_tools.output.clone());
                }
                if ui.button("Copy graph result").clicked() {
                    ui.ctx().copy_text(self.graph_tools.output.clone());
                }
            });
        }
        ui.label("Numerical estimates within the entered bounds. Narrow the interval around the feature you need; sampling can miss other features.");
    }

    pub(crate) fn graph_drawing_menu(&mut self, ui: &mut egui::Ui) {
        ui.strong("Graph drawings");
        ui.horizontal_wrapped(|ui| {
            for (kind, label) in [
                (DrawKind::Point, "Point"),
                (DrawKind::Line, "Line"),
                (DrawKind::Circle, "Circle"),
            ] {
                ui.radio_value(&mut self.graph_tools.draw_kind, kind, label);
            }
        });
        let comma = self.settings.access.decimal_comma;
        numeric(
            ui,
            "Drawing x1",
            &mut self.graph_tools.draw_values[0],
            comma,
        );
        numeric(
            ui,
            "Drawing y1",
            &mut self.graph_tools.draw_values[1],
            comma,
        );
        if self.graph_tools.draw_kind == DrawKind::Line {
            numeric(
                ui,
                "Drawing x2",
                &mut self.graph_tools.draw_values[2],
                comma,
            );
            numeric(
                ui,
                "Drawing y2",
                &mut self.graph_tools.draw_values[3],
                comma,
            );
        }
        if self.graph_tools.draw_kind == DrawKind::Circle {
            numeric(
                ui,
                "Circle radius",
                &mut self.graph_tools.draw_values[2],
                comma,
            );
        }
        if ui.button("Add graph drawing").clicked() {
            let [x, y, a, b] = self.graph_tools.draw_values;
            let drawing = match self.graph_tools.draw_kind {
                DrawKind::Point => Drawing::Point([x, y]),
                DrawKind::Line => Drawing::Line([x, y], [a, b]),
                DrawKind::Circle => Drawing::Circle {
                    center: [x, y],
                    radius: a,
                },
            };
            self.graph_tools.output = match drawing.validate() {
                Ok(()) if self.graph_tools.drawings.len() < MAX_DRAWINGS => {
                    self.graph_tools.undo_drawings = Some(self.graph_tools.drawings.clone());
                    let description = drawing.description();
                    self.graph_tools.drawings.push(drawing);
                    format!("Added {description}.")
                }
                Ok(()) => format!(
                    "Use at most {MAX_DRAWINGS} drawings. Remove a drawing before adding another."
                ),
                Err(error) => error,
            };
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !self.graph_tools.drawings.is_empty(),
                    egui::Button::new("Remove last drawing"),
                )
                .clicked()
            {
                self.graph_tools.undo_drawings = Some(self.graph_tools.drawings.clone());
                self.graph_tools.drawings.pop();
                self.graph_tools.output = "Removed the last graph drawing.".into();
            }
            if ui
                .add_enabled(
                    !self.graph_tools.drawings.is_empty(),
                    egui::Button::new("Clear drawings"),
                )
                .clicked()
            {
                self.graph_tools.undo_drawings = Some(self.graph_tools.drawings.clone());
                self.graph_tools.drawings.clear();
                self.graph_tools.output = "Cleared graph drawings. Undo restores them.".into();
            }
            if ui
                .add_enabled(
                    self.graph_tools.undo_drawings.is_some(),
                    egui::Button::new("Undo drawing change"),
                )
                .clicked()
                && let Some(previous) = self.graph_tools.undo_drawings.take()
            {
                let current = std::mem::replace(&mut self.graph_tools.drawings, previous);
                self.graph_tools.undo_drawings = Some(current);
                self.graph_tools.output = "Restored the previous graph drawings.".into();
            }
        });
        for (index, drawing) in self.graph_tools.drawings.iter().enumerate() {
            ui.label(format!("D{}: {}", index + 1, drawing.description()));
        }
        if !self.graph_tools.output.is_empty() {
            status(ui, &self.graph_tools.output);
        }
        ui.label("Drawings use graph coordinates and appear on both graph views. Entering coordinates does not require dragging.");
    }

    pub(crate) fn paint_graph_drawings(&self, painter: &egui::Painter, plot: Rect, range: f64) {
        let palette = appearance::palette(painter.ctx());
        let project = |[x, y]: [f64; 2]| {
            Pos2::new(
                plot.center().x + ((x - self.center[0]) / range) as f32 * plot.width() / 2.0,
                plot.center().y - ((y - self.center[1]) / range) as f32 * plot.height() / 2.0,
            )
        };
        for (index, drawing) in self.graph_tools.drawings.iter().enumerate() {
            let label = format!("D{}", index + 1);
            match drawing {
                Drawing::Point(point) => {
                    let point = project(*point);
                    if point.is_finite() {
                        painter.circle_filled(point, 3.5, palette.ink);
                        painter.text(
                            point + Vec2::new(4.0, -4.0),
                            egui::Align2::LEFT_BOTTOM,
                            label,
                            egui::FontId::monospace(10.0),
                            palette.ink,
                        );
                    }
                }
                Drawing::Line(first, second) => {
                    let points = [project(*first), project(*second)];
                    if points.iter().all(|point| point.is_finite()) {
                        painter.line_segment(points, Stroke::new(2.0_f32, palette.ink));
                        painter.text(
                            points[0],
                            egui::Align2::LEFT_TOP,
                            label,
                            egui::FontId::monospace(10.0),
                            palette.ink,
                        );
                    }
                }
                Drawing::Circle {
                    center: [x, y],
                    radius,
                } => {
                    let points: Vec<_> = (0..=128)
                        .map(|index| {
                            let angle = index as f64 / 128.0 * std::f64::consts::TAU;
                            project([x + radius * angle.cos(), y + radius * angle.sin()])
                        })
                        .collect();
                    if points.iter().all(|point| point.is_finite()) {
                        let label_point = points[0];
                        painter.add(egui::Shape::line(points, Stroke::new(2.0_f32, palette.ink)));
                        painter.text(
                            label_point,
                            egui::Align2::LEFT_TOP,
                            label,
                            egui::FontId::monospace(10.0),
                            palette.ink,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_calculation_inputs_expose_names_and_selected_values() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(crate::Settings::default(), None);
        app.graph_tools.operation = Analysis::Intersection;
        app.graph_tools.first = 9;
        app.graph_tools.second = 0;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 700.0))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.graph_calculation_menu(ui));
            },
        );
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        for (label, value) in [
            ("Graph calculation operation", "Intersection"),
            ("First curve", "Y10"),
            ("Second curve", "Y1"),
        ] {
            assert!(
                nodes
                    .iter()
                    .any(|(_, node)| node.role() == egui::accesskit::Role::ComboBox
                        && node.label() == Some(label)
                        && node.value() == Some(value)),
                "Missing named selector {label} with its selected value"
            );
        }
        for (label, value) in [("Lower bound", -10.0), ("Upper bound", 10.0)] {
            assert!(
                nodes
                    .iter()
                    .any(|(_, node)| node.role() == egui::accesskit::Role::SpinButton
                        && node.label() == Some(label)
                        && node.numeric_value() == Some(value))
            );
        }
    }

    #[test]
    fn poles_cannot_relax_root_residual_or_report_converged_integrals() {
        assert!(
            analyze(
                Analysis::Zero,
                "1/(x-0.1250000001)",
                "",
                -1.0,
                1.0,
                0.0,
                false
            )
            .is_err()
        );
        assert!(
            analyze(
                Analysis::Integral,
                "1/(x-0.1250000001)^2",
                "",
                -1.0,
                1.0,
                0.0,
                false
            )
            .is_err()
        );
        let tangent = analyze(Analysis::Zero, "(x-0.12345)^2", "", -1.0, 1.0, 0.0, false).unwrap();
        assert!((tangent.value - 0.12345).abs() < 1e-7);
        let proper = analyze(
            Analysis::Integral,
            "1/(x-0.125)^2",
            "",
            1.0,
            2.0,
            0.0,
            false,
        )
        .unwrap();
        let expected = 1.0 / (1.0 - 0.125) - 1.0 / (2.0 - 0.125);
        assert!((proper.value - expected).abs() < 1e-8);
    }

    #[test]
    fn graph_calculus_uses_selected_angle_units() {
        let radians = analyze(
            Analysis::Integral,
            "sin(x)",
            "",
            0.0,
            std::f64::consts::PI,
            0.0,
            false,
        )
        .unwrap();
        assert!((radians.value - 2.0).abs() < 1e-8);
        let degrees = analyze(Analysis::Integral, "sin(x)", "", 0.0, 180.0, 0.0, true).unwrap();
        assert!((degrees.value - 360.0 / std::f64::consts::PI).abs() < 1e-7);
        let slope = analyze(Analysis::Derivative, "sin(x)", "", 0.0, 1.0, 30.0, true).unwrap();
        assert!(
            (slope.value - std::f64::consts::PI / 180.0 * (std::f64::consts::PI / 6.0).cos()).abs()
                < 1e-8
        );
    }

    #[test]
    fn notebook_omits_transient_graph_results_audio_status_and_undo() {
        let mut app = EqualApp::with_settings(crate::Settings::default(), None);
        app.graph_tools.output = "Old calculation result".into();
        app.graph_tools.summary = "Cached sample summary".into();
        app.graph_tools.audio_status = "Prior audio playback status".into();
        app.graph_tools.undo_drawings = Some(vec![Drawing::Point([9.0, 9.0])]);
        app.graph_tools.drawings = vec![Drawing::Point([1.0, 2.0])];
        let notebook = app.capture_graph_notebook();
        notebook.validate().unwrap();
        let json = serde_json::to_string(&notebook).unwrap();
        for transient in [
            "Old calculation result",
            "Cached sample summary",
            "Prior audio playback status",
            "undo_drawings",
            "graph_audio",
        ] {
            assert!(!json.contains(transient));
        }
        app.apply_graph_notebook(notebook);
        assert_eq!(app.graph_tools.drawings, vec![Drawing::Point([1.0, 2.0])]);
        assert!(app.graph_tools.output.is_empty());
        assert!(app.graph_tools.summary.is_empty());
        assert!(app.graph_tools.audio_status.is_empty());
        assert!(app.graph_tools.undo_drawings.is_none());
    }

    #[test]
    fn zeros_intersections_and_extrema_match_known_values() {
        let zero = analyze(Analysis::Zero, "x^2-2", "", 0.0, 2.0, 0.0, false).unwrap();
        assert!((zero.value - 2.0_f64.sqrt()).abs() < 1e-6);
        let crossing = analyze(Analysis::Intersection, "x^2", "2*x", 1.0, 3.0, 0.0, false).unwrap();
        assert!((crossing.value - 2.0).abs() < 1e-6);
        let min = analyze(Analysis::Minimum, "(x-1.25)^2+3", "", -2.0, 4.0, 0.0, false).unwrap();
        assert!((min.value - 3.0).abs() < 1e-8);
        assert!((min.point.unwrap()[0] - 1.25).abs() < 1e-6);
        let max = analyze(Analysis::Maximum, "-(x+2)^2+5", "", -4.0, 2.0, 0.0, false).unwrap();
        assert!((max.value - 5.0).abs() < 1e-8);
        assert!(analyze(Analysis::Zero, "1/x", "", -1.0, 1.0, 0.0, false).is_err());
        assert!(analyze(Analysis::Zero, "1/(x-0.1)", "", -1.0, 1.0, 0.0, false).is_err());
        assert!(analyze(Analysis::Zero, "1/x", "", 1e10, 1e11, 0.0, false).is_err());
    }
    #[test]
    fn calculus_respects_units_and_rejects_domain_gaps() {
        assert!(
            (analyze(Analysis::Derivative, "sin(x)", "", 0.0, 1.0, 0.0, true)
                .unwrap()
                .value
                - std::f64::consts::PI / 180.0)
                .abs()
                < 1e-8
        );
        assert!(
            (analyze(Analysis::Integral, "x^2", "", 0.0, 3.0, 0.0, false)
                .unwrap()
                .value
                - 9.0)
                .abs()
                < 1e-8
        );
        assert!(analyze(Analysis::Integral, "1/x", "", -1.0, 1.0, 0.0, false).is_err());
        assert!(analyze(Analysis::Integral, "tan(x)", "", -2.0, 2.0, 0.0, false).is_err());
        assert!(analyze(Analysis::Derivative, "abs(x)", "", -1.0, 1.0, 0.0, false).is_err());
        assert!(analyze(Analysis::Minimum, "sqrt(x)", "", -1.0, 1.0, 0.0, false).is_err());
        assert!(analyze(Analysis::Maximum, "1/(x-0.1)", "", -1.0, 1.0, 0.0, false).is_err());
    }
    #[test]
    fn summaries_and_drawings_are_bounded_and_descriptive() {
        let summary = sample_summary("sqrt(x)", -1.0, 1.0, false);
        assert!(summary.contains("undefined") && summary.contains("Sampled y range"));
        assert!(sample_summary("x", -1.0, 1.0, false).contains("higher"));
        assert!(Drawing::Point([f64::NAN, 0.0]).validate().is_err());
        assert!(Drawing::Line([0.0, 0.0], [0.0, 0.0]).validate().is_err());
        assert!(
            Drawing::Circle {
                center: [0.0, 0.0],
                radius: -1.0
            }
            .validate()
            .is_err()
        );
        assert!(
            Drawing::Circle {
                center: [1.0, 2.0],
                radius: 3.0
            }
            .validate()
            .is_ok()
        );
        assert!(Drawing::Point([1.0, 2.0]).description().contains("(1, 2)"));
    }
}
