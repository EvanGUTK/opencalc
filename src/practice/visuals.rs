use super::Visual;
use crate::egui;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Rect, Sense, Stroke, Vec2};

const MAX_DIAGRAM_WIDTH: f32 = 620.0;
const MAX_GROUPS: u32 = 12;
const MAX_GROUP_OBJECTS: u32 = 30;
const MAX_FRACTIONS: usize = 8;
const MAX_WHOLE_BARS: u32 = 4;
const MAX_PARTITIONS: u32 = 48;
const MAX_COORDINATE_POINTS: usize = 8;
const MAX_MATRIX_SIZE: usize = 8;

#[derive(Clone)]
struct Palette {
    ink: Color32,
    accent: Color32,
    shade: Color32,
    font: FontId,
}

impl Palette {
    fn new(ctx: &egui::Context, contrast: bool, readable: bool) -> Self {
        let shared = crate::appearance::palette(ctx);
        let ink = if contrast {
            if shared.dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            }
        } else {
            shared.ink
        };
        Self {
            ink,
            accent: if contrast { ink } else { shared.accent },
            shade: if contrast {
                shared.paper
            } else if shared.dark {
                Color32::from_rgb(48, 68, 78)
            } else {
                shared.panel
            },
            font: FontId::new(
                13.0 * crate::appearance::font_scale(ctx),
                if readable {
                    FontFamily::Monospace
                } else {
                    FontFamily::Proportional
                },
            ),
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, visual: &Visual, contrast: bool, readable: bool) {
    if matches!(visual, Visual::None) {
        return;
    }
    let palette = Palette::new(ui.ctx(), contrast, readable);
    // A visual can be called from a wrapped toolbar or a column; keep its own flow vertical.
    ui.scope_builder(
        egui::UiBuilder::new().layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            ui.visuals_mut().override_text_color = None;
            ui.visuals_mut().widgets.noninteractive.fg_stroke.color = palette.ink;
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Body, palette.font.clone());
            match visual {
                Visual::None => {}
                Visual::NumberLine {
                    start,
                    change,
                    min,
                    max,
                } => number_line(ui, *start, *change, *min, *max, &palette),
                Visual::Groups { groups, per_group } => {
                    equal_groups(ui, *groups, *per_group, &palette)
                }
                Visual::FractionBars { parts } => fractions(ui, parts, &palette),
                Visual::PlaceValue { number } => place_value(ui, *number, &palette),
                Visual::Triangle { labels } => triangle(ui, labels, &palette),
                Visual::CoordinatePlot { points, line } => coordinates(ui, points, *line, &palette),
                Visual::Matrix {
                    left,
                    right,
                    operator,
                } => matrices(ui, left, right.as_deref(), operator),
            }
        },
    );
}

fn width(ui: &egui::Ui) -> f32 {
    let available = ui.available_width();
    if available.is_finite() {
        available.clamp(1.0, MAX_DIAGRAM_WIDTH)
    } else {
        300.0
    }
}

fn canvas(ui: &mut egui::Ui, size: Vec2, summary: &str) -> (Rect, egui::Painter) {
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, summary));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_label(summary);
    });
    (rect, ui.painter_at(rect))
}

fn tick_step(span: i64, max_ticks: i64) -> i64 {
    let max_ticks = max_ticks.max(1);
    let target = ((span + max_ticks - 1) / max_ticks).max(1);
    let mut magnitude = 1_i64;
    while magnitude * 10 < target {
        magnitude *= 10;
    }
    for multiplier in [1, 2, 5, 10] {
        let candidate = magnitude * multiplier;
        if candidate >= target {
            return candidate;
        }
    }
    magnitude * 10
}

fn number_line(ui: &mut egui::Ui, start: i32, change: i32, min: i32, max: i32, palette: &Palette) {
    if min >= max {
        ui.label("The number line needs a lower minimum than maximum.");
        return;
    }
    let movement = i64::from(change).abs();
    let direction = if change < 0 { "backward" } else { "forward" };
    let summary = format!(
        "Number line from {min} to {max}. Start at {start}; move {movement} spaces {direction}."
    );
    ui.label(&summary);
    let diagram_width = width(ui);
    let (rect, painter) = canvas(ui, Vec2::new(diagram_width, 112.0), &summary);
    let left = rect.left() + (diagram_width * 0.08).min(20.0);
    let right = rect.right() - (diagram_width * 0.08).min(20.0);
    let axis_y = rect.top() + 66.0;
    let span = i64::from(max) - i64::from(min);
    let x = |value: i64| {
        left + ((value - i64::from(min)) as f64 / span as f64).clamp(0.0, 1.0) as f32
            * (right - left)
    };
    painter.line_segment(
        [Pos2::new(left, axis_y), Pos2::new(right, axis_y)],
        Stroke::new(1.5_f32, palette.ink),
    );
    let max_ticks = ((diagram_width - 20.0) / 48.0).floor().clamp(2.0, 10.0) as i64;
    let step = tick_step(span, max_ticks);
    let mut tick = i64::from(min).div_euclid(step) * step;
    if tick < i64::from(min) {
        tick += step;
    }
    while tick <= i64::from(max) {
        let at = Pos2::new(x(tick), axis_y);
        painter.line_segment(
            [at - Vec2::new(0.0, 5.0), at + Vec2::new(0.0, 5.0)],
            Stroke::new(1.0_f32, palette.ink),
        );
        painter.text(
            at + Vec2::new(0.0, 12.0),
            Align2::CENTER_TOP,
            tick.to_string(),
            palette.font.clone(),
            palette.ink,
        );
        tick += step;
    }
    let endpoint = i64::from(start) + i64::from(change);
    let from = Pos2::new(x(i64::from(start)), axis_y - 23.0);
    let to = Pos2::new(x(endpoint), axis_y - 23.0);
    if from.distance(to) > 0.1 {
        painter.arrow(from, to - from, Stroke::new(2.0_f32, palette.accent));
    } else {
        painter.circle_stroke(from, 5.0, Stroke::new(2.0_f32, palette.accent));
    }
    painter.circle_filled(Pos2::new(x(i64::from(start)), axis_y), 4.0, palette.ink);
    if i64::from(start) < i64::from(min)
        || i64::from(start) > i64::from(max)
        || endpoint < i64::from(min)
        || endpoint > i64::from(max)
    {
        ui.label("Some movement is outside the displayed number line.");
    }
}

fn equal_groups(ui: &mut egui::Ui, count: u32, per_group: u32, palette: &Palette) {
    let summary = format!("{count} groups of {per_group} objects each.");
    ui.label(&summary);
    if count == 0 || per_group == 0 {
        ui.label("There are no objects to draw.");
        return;
    }
    let group_width = width(ui).min(112.0);
    let columns = (((group_width - 18.0) / 15.0).floor() as u32).clamp(1, 6);
    let shown_objects = per_group.min(MAX_GROUP_OBJECTS);
    let rows = shown_objects.div_ceil(columns);
    let group_height = 32.0 + rows as f32 * 15.0;
    ui.horizontal_wrapped(|ui| {
        for index in 0..count.min(MAX_GROUPS) {
            let label = format!("Group {}: {per_group} objects", index + 1);
            let (rect, painter) = canvas(ui, Vec2::new(group_width, group_height), &label);
            painter.rect_stroke(
                rect.shrink(2.0),
                4.0,
                Stroke::new(1.5_f32, palette.ink),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center_top() + Vec2::new(0.0, 7.0),
                Align2::CENTER_TOP,
                (index + 1).to_string(),
                palette.font.clone(),
                palette.ink,
            );
            for item in 0..shown_objects {
                let point = rect.left_top()
                    + Vec2::new(
                        12.0 + (item % columns) as f32 * 15.0,
                        32.0 + (item / columns) as f32 * 15.0,
                    );
                painter.circle_filled(point, 4.0, palette.accent);
            }
        }
    });
    if count > MAX_GROUPS || per_group > MAX_GROUP_OBJECTS {
        ui.label(format!("Drawing limited to the first {} groups and {} objects per group. The stated counts above are unchanged.", count.min(MAX_GROUPS), shown_objects));
    }
}

fn fractions(ui: &mut egui::Ui, parts: &[(i32, u32)], palette: &Palette) {
    if parts.is_empty() {
        ui.label("No fractions were supplied.");
        return;
    }
    let bar_width = width(ui);
    for &(numerator, denominator) in parts.iter().take(MAX_FRACTIONS) {
        if denominator == 0 {
            ui.label("Cannot draw a fraction with denominator zero.");
            continue;
        }
        let negative = numerator < 0;
        let label = format!(
            "Fraction {numerator}/{denominator}{}",
            if negative {
                ": negative; bars show its magnitude."
            } else {
                ". Hatched parts are selected."
            }
        );
        ui.label(&label);
        let magnitude = numerator.unsigned_abs();
        let bars = magnitude.div_ceil(denominator).max(1);
        let shown_bars = bars.min(MAX_WHOLE_BARS);
        for unit in 0..shown_bars {
            let remaining = magnitude.saturating_sub(unit.saturating_mul(denominator));
            let filled = remaining.min(denominator);
            let summary = format!(
                "{label} Bar {}: {filled} of {denominator} parts selected.",
                unit + 1
            );
            let (rect, painter) = canvas(ui, Vec2::new(bar_width, 34.0), &summary);
            let bar = rect.shrink2(Vec2::new(2.0, 4.0));
            let selected_width = bar.width() * (f64::from(filled) / f64::from(denominator)) as f32;
            let selected = Rect::from_min_size(bar.min, Vec2::new(selected_width, bar.height()));
            painter.rect_filled(selected, 0.0, palette.shade);
            if selected_width > 0.0 {
                let hatch = painter.with_clip_rect(selected);
                let mut offset = -bar.height();
                while offset < selected_width {
                    hatch.line_segment(
                        [
                            selected.left_bottom() + Vec2::new(offset, 0.0),
                            selected.left_top() + Vec2::new(offset + bar.height(), 0.0),
                        ],
                        Stroke::new(1.0_f32, palette.accent),
                    );
                    offset += 8.0;
                }
            }
            if denominator <= MAX_PARTITIONS {
                for part in 1..denominator {
                    let at = bar.left() + bar.width() * part as f32 / denominator as f32;
                    painter.line_segment(
                        [Pos2::new(at, bar.top()), Pos2::new(at, bar.bottom())],
                        Stroke::new(1.0_f32, palette.ink),
                    );
                }
            }
            painter.rect_stroke(
                bar,
                0.0,
                Stroke::new(1.5_f32, palette.ink),
                egui::StrokeKind::Inside,
            );
        }
        if bars > MAX_WHOLE_BARS {
            ui.label(format!(
                "Showing {shown_bars} of {bars} whole-size bars for {numerator}/{denominator}."
            ));
        }
        if denominator > MAX_PARTITIONS {
            ui.label(format!(
                "Partition lines are omitted because the denominator is {denominator}."
            ));
        }
    }
    if parts.len() > MAX_FRACTIONS {
        ui.label(format!(
            "Showing the first {MAX_FRACTIONS} of {} supplied fractions.",
            parts.len()
        ));
    }
}

fn place_value(ui: &mut egui::Ui, number: u32, palette: &Palette) {
    ui.label(format!("Place value for {number}."));
    if number > 999 {
        ui.label("Place-value blocks are available for numbers from 0 to 999.");
        return;
    }
    let hundreds = number / 100;
    let tens = number / 10 % 10;
    let ones = number % 10;
    ui.label(format!(
        "{hundreds} hundreds, {tens} tens, and {ones} ones."
    ));
    for (name, count, value) in [
        ("Hundreds", hundreds, 100),
        ("Tens", tens, 10),
        ("Ones", ones, 1),
    ] {
        ui.strong(format!("{name}: {count}"));
        if count == 0 {
            ui.label("None");
            continue;
        }
        let block_width = width(ui).min(if value == 100 { 54.0 } else { 30.0 });
        ui.horizontal_wrapped(|ui| {
            for block in 0..count {
                let summary = format!("{name}, block {} of {count}, worth {value}", block + 1);
                let (rect, painter) = canvas(ui, Vec2::new(block_width, 54.0), &summary);
                if value == 1 {
                    painter.circle_filled(rect.center(), 7.0, palette.accent);
                } else {
                    let shape = rect.shrink(3.0);
                    painter.rect_filled(shape, 2.0, palette.shade);
                    painter.rect_stroke(
                        shape,
                        2.0,
                        Stroke::new(1.5_f32, palette.ink),
                        egui::StrokeKind::Inside,
                    );
                    if value == 10 {
                        for tick in 1..10 {
                            let y = shape.top() + shape.height() * tick as f32 / 10.0;
                            painter.line_segment(
                                [Pos2::new(shape.left(), y), Pos2::new(shape.right(), y)],
                                Stroke::new(0.8_f32, palette.ink),
                            );
                        }
                    } else {
                        painter.text(
                            shape.center(),
                            Align2::CENTER_CENTER,
                            "100",
                            palette.font.clone(),
                            palette.ink,
                        );
                    }
                }
            }
        });
    }
}

fn triangle(ui: &mut egui::Ui, labels: &[String; 3], palette: &Palette) {
    ui.label("Right triangle; not to scale.");
    for (name, label) in ["Base", "Vertical side", "Diagonal side"]
        .into_iter()
        .zip(labels)
    {
        ui.label(format!("{name}: {label}"));
    }
    let summary = format!(
        "Right triangle. Base: {}. Vertical side: {}. Diagonal side: {}. Not to scale.",
        labels[0], labels[1], labels[2]
    );
    let diagram_width = width(ui).min(360.0);
    let height = (diagram_width * 0.6).clamp(100.0, 210.0);
    let (rect, painter) = canvas(ui, Vec2::new(diagram_width, height), &summary);
    let margin = (diagram_width * 0.13).min(28.0);
    let a = rect.left_bottom() + Vec2::new(margin, -24.0);
    let b = rect.right_bottom() + Vec2::new(-margin, -24.0);
    let c = rect.left_top() + Vec2::new(margin, 16.0);
    painter.add(egui::Shape::closed_line(
        vec![a, b, c],
        Stroke::new(2.0_f32, palette.ink),
    ));
    let square = 9.0;
    painter.line_segment(
        [a + Vec2::new(square, 0.0), a + Vec2::new(square, -square)],
        Stroke::new(1.0_f32, palette.ink),
    );
    painter.line_segment(
        [a + Vec2::new(square, -square), a + Vec2::new(0.0, -square)],
        Stroke::new(1.0_f32, palette.ink),
    );
    for (at, text) in [
        ((a + b.to_vec2()) / 2.0 + Vec2::new(0.0, 14.0), "base"),
        ((a + c.to_vec2()) / 2.0 + Vec2::new(9.0, 0.0), "side"),
        ((b + c.to_vec2()) / 2.0 + Vec2::new(8.0, -5.0), "diagonal"),
    ] {
        painter.text(
            at,
            Align2::CENTER_CENTER,
            text,
            palette.font.clone(),
            palette.ink,
        );
    }
}

fn number(value: f64) -> String {
    if value == 0.0 {
        "0".into()
    } else if value.abs() < 0.000_001 || value.abs() >= 1_000_000.0 {
        format!("{value:e}")
    } else {
        value.to_string()
    }
}

fn coordinates(ui: &mut egui::Ui, points: &[(f64, f64)], line: bool, palette: &Palette) {
    if points.is_empty() {
        ui.label("No coordinate points were supplied.");
        return;
    }
    if points
        .iter()
        .take(MAX_COORDINATE_POINTS)
        .any(|(x, y)| !x.is_finite() || !y.is_finite() || x.abs() > 1e9 || y.abs() > 1e9)
    {
        ui.label("Coordinate points must be finite and within -1,000,000,000 to 1,000,000,000.");
        return;
    }
    if points.len() > MAX_COORDINATE_POINTS {
        ui.label(format!(
            "Showing the first {MAX_COORDINATE_POINTS} of {} supplied coordinate points.",
            points.len()
        ));
    }
    let points = &points[..points.len().min(MAX_COORDINATE_POINTS)];
    let descriptions: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            format!(
                "{}: x={}, y={}",
                char::from(b'A' + index as u8),
                number(*x),
                number(*y)
            )
        })
        .collect();
    let summary = format!(
        "Coordinate plot. {}. {}",
        descriptions.join("; "),
        if line {
            "Consecutive points are joined with straight segments."
        } else {
            "Points are separate."
        }
    );
    for description in &descriptions {
        ui.label(description);
    }
    let diagram_width = width(ui).min(420.0);
    let height = (diagram_width * 0.75).clamp(120.0, 240.0);
    let (rect, painter) = canvas(ui, Vec2::new(diagram_width, height), &summary);
    let inset = (diagram_width * 0.15).min(28.0);
    let plot = rect.shrink2(Vec2::new(inset, 20.0));
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    for &(x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    let x_pad = ((max_x - min_x) * 0.15).max(1.0);
    let y_pad = ((max_y - min_y) * 0.15).max(1.0);
    min_x -= x_pad;
    max_x += x_pad;
    min_y -= y_pad;
    max_y += y_pad;
    let position = |x: f64, y: f64| {
        Pos2::new(
            plot.left() + ((x - min_x) / (max_x - min_x)) as f32 * plot.width(),
            plot.bottom() - ((y - min_y) / (max_y - min_y)) as f32 * plot.height(),
        )
    };
    let origin = position(0.0, 0.0);
    painter.line_segment(
        [
            Pos2::new(plot.left(), origin.y),
            Pos2::new(plot.right(), origin.y),
        ],
        Stroke::new(1.0_f32, palette.ink),
    );
    painter.line_segment(
        [
            Pos2::new(origin.x, plot.top()),
            Pos2::new(origin.x, plot.bottom()),
        ],
        Stroke::new(1.0_f32, palette.ink),
    );
    painter.text(
        plot.right_bottom(),
        Align2::RIGHT_TOP,
        "x",
        palette.font.clone(),
        palette.ink,
    );
    painter.text(
        plot.left_top(),
        Align2::LEFT_TOP,
        "y",
        palette.font.clone(),
        palette.ink,
    );
    let plotted: Vec<_> = points.iter().map(|(x, y)| position(*x, *y)).collect();
    if line && plotted.len() > 1 {
        painter.add(egui::Shape::line(
            plotted.clone(),
            Stroke::new(1.5_f32, palette.accent),
        ));
    }
    for (index, point) in plotted.iter().enumerate() {
        painter.circle_stroke(*point, 4.0, Stroke::new(2.0_f32, palette.ink));
        painter.text(
            *point + Vec2::new(7.0, -7.0),
            Align2::LEFT_BOTTOM,
            char::from(b'A' + index as u8).to_string(),
            palette.font.clone(),
            palette.ink,
        );
    }
}

fn matrix(ui: &mut egui::Ui, title: &str, values: &[Vec<f64>]) {
    let columns = values.first().map_or(0, Vec::len);
    if values.is_empty() || columns == 0 || values.iter().any(|row| row.len() != columns) {
        ui.label(format!("{title}: supply a nonempty rectangular matrix."));
        return;
    }
    if values.len() > MAX_MATRIX_SIZE || columns > MAX_MATRIX_SIZE {
        ui.label(format!("{title}: {} rows and {columns} columns. Visual tables support at most {MAX_MATRIX_SIZE} rows and columns.", values.len()));
        return;
    }
    if values.iter().flatten().any(|value| !value.is_finite()) {
        ui.label(format!("{title}: entries must be finite numbers."));
        return;
    }
    ui.strong(format!("{title}: {} rows, {columns} columns", values.len()));
    egui::Frame::new()
        .stroke(Stroke::new(1.0_f32, ui.visuals().text_color()))
        .inner_margin(6)
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            for (index, row) in values.iter().enumerate() {
                ui.label(format!(
                    "Row {}: [ {} ]",
                    index + 1,
                    row.iter()
                        .map(|value| number(*value))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        });
}

fn matrices(ui: &mut egui::Ui, left: &[Vec<f64>], right: Option<&[Vec<f64>]>, operator: &str) {
    matrix(ui, "Matrix A", left);
    if let Some(right) = right {
        let operation = match operator {
            "×" | "*" => "Multiply A by B",
            "+" => "Add A and B",
            "−" | "-" => "Subtract B from A",
            _ => operator,
        };
        ui.label(operation);
        matrix(ui, "Matrix B", right);
    } else if !operator.is_empty() {
        ui.label(operator);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_diagram_labels_use_the_shared_light_ink() {
        let ctx = egui::Context::default();
        let mut settings = crate::Settings::default();
        settings.access.theme = crate::accessibility::ColorTheme::Dark;
        crate::appearance::apply(&ctx, &settings);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(360.0, 500.0))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    show(
                        ui,
                        &Visual::NumberLine {
                            start: 0,
                            change: 2,
                            min: 0,
                            max: 4,
                        },
                        false,
                        false,
                    )
                });
            },
        );
        let ink = crate::appearance::palette(&ctx).ink;
        let tick = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text() == "0" => Some(text),
                _ => None,
            })
            .expect("number-line tick");
        for section in &tick.galley.job.sections {
            let color = tick.override_text_color.unwrap_or_else(|| {
                if section.format.color == Color32::PLACEHOLDER {
                    tick.fallback_color
                } else {
                    section.format.color
                }
            });
            assert_eq!(color, ink);
        }
    }

    fn render(visual: &Visual, logical_width: f32) -> egui::FullOutput {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut output = None;
        for _ in 0..2 {
            output = Some(ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(logical_width, 1800.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| show(ui, visual, true, true));
                },
            ));
        }
        output.unwrap()
    }

    #[test]
    fn diagram_descriptions_and_bounds_survive_narrow_content() {
        let cases = [
            (
                Visual::NumberLine {
                    start: 3,
                    change: 4,
                    min: 0,
                    max: 10,
                },
                "Start at 3; move 4 spaces forward",
            ),
            (
                Visual::Groups {
                    groups: 2,
                    per_group: 3,
                },
                "3 objects",
            ),
            (
                Visual::FractionBars {
                    parts: vec![(7, 4), (-1, 3)],
                },
                "7/4",
            ),
            (Visual::PlaceValue { number: 123 }, "worth 100"),
            (
                Visual::Triangle {
                    labels: ["3".into(), "4".into(), "?".into()],
                },
                "Diagonal side: ?",
            ),
            (
                Visual::CoordinatePlot {
                    points: vec![(1.0, 2.0), (3.0, 4.0)],
                    line: true,
                },
                "A: x=1, y=2",
            ),
        ];
        // 160 logical points is the available width of a 320-point window at 200% zoom.
        for logical_width in [320.0, 160.0] {
            for (visual, expected) in &cases {
                let output = render(visual, logical_width);
                let tree = output.platform_output.accesskit_update.as_ref().unwrap();
                assert!(
                    tree.nodes
                        .iter()
                        .any(|(_, node)| node.role() == egui::accesskit::Role::Image
                            && node.label().is_some_and(|label| label.contains(expected))),
                    "Missing description {expected}"
                );
                for (_, node) in tree
                    .nodes
                    .iter()
                    .filter(|(_, node)| node.role() == egui::accesskit::Role::Image)
                {
                    let bounds = node.bounds().unwrap();
                    assert!(
                        bounds.x0 >= 0.0 && bounds.x1 <= f64::from(logical_width),
                        "Diagram overflow: {bounds:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn invalid_visual_data_has_visible_text_instead_of_panicking() {
        for (visual, expected) in [
            (
                Visual::NumberLine {
                    start: 0,
                    change: 1,
                    min: 2,
                    max: 2,
                },
                "lower minimum",
            ),
            (
                Visual::FractionBars {
                    parts: vec![(1, 0)],
                },
                "denominator zero",
            ),
            (
                Visual::CoordinatePlot {
                    points: vec![(f64::NAN, 0.0)],
                    line: false,
                },
                "must be finite",
            ),
            (
                Visual::Matrix {
                    left: vec![],
                    right: None,
                    operator: String::new(),
                },
                "nonempty rectangular",
            ),
        ] {
            let output = render(&visual, 160.0);
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text().contains(expected))), "Missing validation: {expected}");
        }
    }

    #[test]
    fn huge_counts_keep_exact_descriptions_and_bounded_drawing() {
        let output = render(
            &Visual::Groups {
                groups: u32::MAX,
                per_group: u32::MAX,
            },
            320.0,
        );
        assert!(output.shapes.len() < 1000);
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text().contains("4294967295 groups of 4294967295"))));
        let fractions = render(
            &Visual::FractionBars {
                parts: vec![(i32::MIN, 1); 100],
            },
            320.0,
        );
        assert!(fractions.shapes.len() < 3000);
        assert!(fractions.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text().contains("Showing 4 of 2147483648"))));
    }

    #[test]
    fn matrix_values_reflow_as_accessible_rows() {
        let output = render(
            &Visual::Matrix {
                left: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
                right: Some(vec![vec![5.0], vec![6.0]]),
                operator: "×".into(),
            },
            160.0,
        );
        for expected in [
            "Matrix A: 2 rows, 2 columns",
            "Row 1: [ 1, 2 ]",
            "Matrix B: 2 rows, 1 columns",
            "Multiply A by B",
        ] {
            assert!(
                output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.value() == Some(expected)
                        || node.label() == Some(expected)),
                "Missing matrix text {expected}"
            );
        }
    }
}
