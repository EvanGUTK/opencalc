//! Bounded vector notebook pages. Device pointers and physical assistive tools need separate testing.
use crate::{Settings, appearance, egui};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

const PAGE_WIDTH: f32 = 720.0;
const PAGE_HEIGHT: f32 = 1000.0;
const MAX_PAGES: usize = 64;
const MAX_ELEMENTS: usize = 4096;
const MAX_PAGE_ELEMENTS: usize = 512;
const MAX_STROKE_POINTS: usize = 4096;
const MAX_POINTS: usize = 100_000;
const MAX_TEXT_BYTES: usize = 1024 * 1024;
const MAX_DESCRIPTION_BYTES: usize = 512 * 1024;
const MAX_UNDO: usize = 32;
const MAX_UNDO_BYTES: usize = 16 * 1024 * 1024;
const MAX_EXPORT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Paper {
    #[default]
    Blank,
    Ruled,
    Grid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InkColor {
    #[default]
    Ink,
    Teal,
    Blue,
    Red,
    Gold,
}

impl InkColor {
    fn label(self) -> &'static str {
        match self {
            Self::Ink => "Ink",
            Self::Teal => "Teal",
            Self::Blue => "Blue",
            Self::Red => "Red",
            Self::Gold => "Gold",
        }
    }

    fn rgb(self, dark: bool) -> [u8; 3] {
        match (self, dark) {
            (Self::Ink, false) => [31, 47, 44],
            (Self::Ink, true) => [244, 246, 250],
            (Self::Teal, false) => [16, 105, 91],
            (Self::Teal, true) => [111, 211, 185],
            (Self::Blue, false) => [37, 77, 163],
            (Self::Blue, true) => [147, 186, 255],
            (Self::Red, false) => [155, 46, 56],
            (Self::Red, true) => [255, 166, 172],
            (Self::Gold, false) => [116, 84, 0],
            (Self::Gold, true) => [235, 212, 133],
        }
    }

    fn paint(self, palette: appearance::Palette) -> Color32 {
        if palette.contrast {
            palette.ink
        } else {
            let [r, g, b] = self.rgb(palette.dark);
            Color32::from_rgb(r, g, b)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InkPoint {
    pub(crate) position: [f32; 2],
    pub(crate) pressure: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum CanvasElement {
    Ink {
        points: Vec<InkPoint>,
        width: f32,
        color: InkColor,
        description: String,
    },
    Line {
        start: [f32; 2],
        end: [f32; 2],
        width: f32,
        color: InkColor,
        description: String,
    },
    Rectangle {
        bounds: [f32; 4],
        width: f32,
        color: InkColor,
        description: String,
    },
    Ellipse {
        bounds: [f32; 4],
        width: f32,
        color: InkColor,
        description: String,
    },
    Text {
        bounds: [f32; 4],
        text: String,
        font_size: f32,
        color: InkColor,
        description: String,
    },
}

fn text_ok(text: &str, max: usize) -> bool {
    text.len() <= max
        && text.chars().all(|c| {
            (!c.is_control() || matches!(c, '\n' | '\r' | '\t'))
                && !matches!(c, '\u{fffe}' | '\u{ffff}')
        })
}

fn point_ok(point: [f32; 2]) -> bool {
    point
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
}

fn bounds_ok([x, y, w, h]: [f32; 4]) -> bool {
    point_ok([x, y])
        && w.is_finite()
        && h.is_finite()
        && w > 0.0
        && h > 0.0
        && x + w <= 1.000_001
        && y + h <= 1.000_001
}

impl CanvasElement {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let style = |width: f32| width.is_finite() && (0.5..=24.0).contains(&width);
        let valid = match self {
            Self::Ink { points, width, .. } => {
                !points.is_empty()
                    && points.len() <= MAX_STROKE_POINTS
                    && points.iter().all(|p| {
                        point_ok(p.position)
                            && p.pressure.is_finite()
                            && (0.0..=1.0).contains(&p.pressure)
                    })
                    && style(*width)
            }
            Self::Line {
                start, end, width, ..
            } => point_ok(*start) && point_ok(*end) && start != end && style(*width),
            Self::Rectangle { bounds, width, .. } | Self::Ellipse { bounds, width, .. } => {
                bounds_ok(*bounds) && style(*width)
            }
            Self::Text {
                bounds,
                text,
                font_size,
                ..
            } => {
                bounds_ok(*bounds)
                    && !text.trim().is_empty()
                    && text_ok(text, 16 * 1024)
                    && font_size.is_finite()
                    && (8.0..=96.0).contains(font_size)
            }
        };
        if !valid || !text_ok(self.description(), 4096) {
            Err("A canvas object has invalid coordinates, text, pressure or stroke size.".into())
        } else {
            Ok(())
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Ink { description, .. }
            | Self::Line { description, .. }
            | Self::Rectangle { description, .. }
            | Self::Ellipse { description, .. }
            | Self::Text { description, .. } => description,
        }
    }

    fn description_mut(&mut self) -> &mut String {
        match self {
            Self::Ink { description, .. }
            | Self::Line { description, .. }
            | Self::Rectangle { description, .. }
            | Self::Ellipse { description, .. }
            | Self::Text { description, .. } => description,
        }
    }

    fn color(&self) -> InkColor {
        match self {
            Self::Ink { color, .. }
            | Self::Line { color, .. }
            | Self::Rectangle { color, .. }
            | Self::Ellipse { color, .. }
            | Self::Text { color, .. } => *color,
        }
    }

    fn color_mut(&mut self) -> &mut InkColor {
        match self {
            Self::Ink { color, .. }
            | Self::Line { color, .. }
            | Self::Rectangle { color, .. }
            | Self::Ellipse { color, .. }
            | Self::Text { color, .. } => color,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Ink { .. } => "Ink stroke",
            Self::Line { .. } => "Line",
            Self::Rectangle { .. } => "Rectangle",
            Self::Ellipse { .. } => "Ellipse",
            Self::Text { .. } => "Text",
        }
    }

    fn bounds(&self) -> [f32; 4] {
        match self {
            Self::Ink { points, .. } => point_bounds(points.iter().map(|p| p.position)),
            Self::Line { start, end, .. } => point_bounds([*start, *end]),
            Self::Rectangle { bounds, .. }
            | Self::Ellipse { bounds, .. }
            | Self::Text { bounds, .. } => *bounds,
        }
    }

    fn transform(&mut self, next: [f32; 4]) {
        let old = self.bounds();
        let position = |p: &mut [f32; 2]| {
            for axis in 0..2 {
                let t = if old[axis + 2] <= f32::EPSILON {
                    0.5
                } else {
                    (p[axis] - old[axis]) / old[axis + 2]
                };
                p[axis] = (next[axis] + t * next[axis + 2]).clamp(0.0, 1.0);
            }
        };
        match self {
            Self::Ink { points, .. } => {
                for point in points {
                    position(&mut point.position);
                }
            }
            Self::Line { start, end, .. } => {
                position(start);
                position(end);
            }
            Self::Rectangle { bounds, .. }
            | Self::Ellipse { bounds, .. }
            | Self::Text { bounds, .. } => *bounds = next,
        }
    }

    pub(crate) fn summary(&self) -> String {
        let [x, y, w, h] = self.bounds();
        let content = match self {
            Self::Text { text, .. } => text.chars().take(160).collect::<String>(),
            Self::Ink { points, .. } => format!("{} points", points.len()),
            _ => String::new(),
        };
        let description = if self.description().trim().is_empty() {
            "No description supplied"
        } else {
            self.description()
        };
        format!(
            "{}: {content}. X {:.1}%, Y {:.1}%, width {:.1}%, height {:.1}%. {description}",
            self.kind(),
            x * 100.0,
            y * 100.0,
            w * 100.0,
            h * 100.0
        )
    }
}

fn point_bounds(points: impl IntoIterator<Item = [f32; 2]>) -> [f32; 4] {
    let mut lo = [1.0_f32; 2];
    let mut hi = [0.0_f32; 2];
    for point in points {
        for axis in 0..2 {
            lo[axis] = lo[axis].min(point[axis]);
            hi[axis] = hi[axis].max(point[axis]);
        }
    }
    [lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasPage {
    pub(crate) title: String,
    pub(crate) background: Paper,
    pub(crate) elements: Vec<CanvasElement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasNotebook {
    pub(crate) pages: Vec<CanvasPage>,
    pub(crate) selected_page: usize,
    pub(crate) zoom: f32,
    pub(crate) pan: [f32; 2],
}

impl Default for CanvasNotebook {
    fn default() -> Self {
        Self {
            pages: vec![CanvasPage {
                title: "Page 1".into(),
                background: Paper::Blank,
                elements: Vec::new(),
            }],
            selected_page: 0,
            zoom: 1.0,
            pan: [0.0; 2],
        }
    }
}

impl CanvasNotebook {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.pages.is_empty()
            || self.pages.len() > MAX_PAGES
            || self.selected_page >= self.pages.len()
        {
            return Err("A canvas notebook needs 1–64 pages and a valid selected page.".into());
        }
        if !self.zoom.is_finite()
            || !(0.25..=4.0).contains(&self.zoom)
            || self
                .pan
                .iter()
                .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
        {
            return Err("Canvas zoom or pan is outside its supported range.".into());
        }
        let mut elements = 0_usize;
        let mut points = 0_usize;
        let mut text = 0_usize;
        let mut descriptions = 0_usize;
        for page in &self.pages {
            if page.title.chars().count() > 100
                || page.title.chars().any(char::is_control)
                || !text_ok(&page.title, 400)
            {
                return Err("Use canvas page titles of at most 100 characters on one line.".into());
            }
            if page.elements.len() > MAX_PAGE_ELEMENTS {
                return Err("A canvas page can contain at most 512 objects.".into());
            }
            elements += page.elements.len();
            for element in &page.elements {
                element.validate()?;
                descriptions += element.description().len();
                match element {
                    CanvasElement::Ink { points: stroke, .. } => points += stroke.len(),
                    CanvasElement::Text { text: value, .. } => text += value.len(),
                    _ => {}
                }
            }
        }
        if elements > MAX_ELEMENTS
            || points > MAX_POINTS
            || text > MAX_TEXT_BYTES
            || descriptions > MAX_DESCRIPTION_BYTES
        {
            return Err("Canvas limit reached: 4,096 objects, 100,000 ink points, 1 MiB text or 512 KiB descriptions across the notebook.".into());
        }
        Ok(())
    }

    fn memory_cost(&self) -> usize {
        self.pages
            .iter()
            .map(|page| {
                page.title.len()
                    + page
                        .elements
                        .iter()
                        .map(|element| {
                            std::mem::size_of::<CanvasElement>()
                                + element.description().len()
                                + match element {
                                    CanvasElement::Ink { points, .. } => {
                                        points.len() * std::mem::size_of::<InkPoint>()
                                    }
                                    CanvasElement::Text { text, .. } => text.len(),
                                    _ => 0,
                                }
                        })
                        .sum::<usize>()
            })
            .sum()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tool {
    #[default]
    Select,
    Ink,
    Eraser,
    Text,
    Line,
    Rectangle,
    Ellipse,
    Hand,
}

impl Tool {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Ink => "Ink",
            Self::Eraser => "Eraser",
            Self::Text => "Text",
            Self::Line => "Line",
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Hand => "Hand",
        }
    }
}

#[derive(Clone)]
struct ObjectDraft {
    x: String,
    y: String,
    width: String,
    height: String,
    text: String,
    description: String,
    stroke_width: String,
    font_size: String,
}

impl Default for ObjectDraft {
    fn default() -> Self {
        Self {
            x: "10".into(),
            y: "10".into(),
            width: "35".into(),
            height: "15".into(),
            text: String::new(),
            description: String::new(),
            stroke_width: "2".into(),
            font_size: "18".into(),
        }
    }
}

enum Gesture {
    Draw {
        start: [f32; 2],
        end: [f32; 2],
        points: Vec<InkPoint>,
        tool: Tool,
        color: InkColor,
        width: f32,
        description: String,
    },
    Move {
        before: CanvasNotebook,
        changed: bool,
    },
    Erase {
        before: CanvasNotebook,
        changed: bool,
    },
    Pan,
}

pub(crate) struct CanvasEditor {
    document: CanvasNotebook,
    tool: Tool,
    color: InkColor,
    selected: Option<usize>,
    draft: ObjectDraft,
    coordinate_points: Vec<InkPoint>,
    undo: Vec<CanvasNotebook>,
    redo: Vec<CanvasNotebook>,
    gesture: Option<Gesture>,
    confirm_delete: bool,
    inspector_open: bool,
    page_tools_open: bool,
    notice: String,
}

impl Default for CanvasEditor {
    fn default() -> Self {
        Self {
            document: CanvasNotebook::default(),
            tool: Tool::Select,
            color: InkColor::Ink,
            selected: None,
            draft: ObjectDraft::default(),
            coordinate_points: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            gesture: None,
            confirm_delete: false,
            inspector_open: false,
            page_tools_open: false,
            notice: String::new(),
        }
    }
}

fn number(text: &str, settings: &Settings) -> Result<f32, String> {
    let text = crate::math::normalize_input(text.trim(), settings.access.decimal_comma)?;
    text.parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| "Enter finite numbers in the canvas coordinate fields.".into())
}

impl ObjectDraft {
    fn position(&self, settings: &Settings) -> Result<[f32; 2], String> {
        let point = [
            number(&self.x, settings)? / 100.0,
            number(&self.y, settings)? / 100.0,
        ];
        if point_ok(point) {
            Ok(point)
        } else {
            Err("Canvas X and Y must be from 0 to 100 percent.".into())
        }
    }

    fn bounds(&self, settings: &Settings) -> Result<[f32; 4], String> {
        let [x, y] = self.position(settings)?;
        let bounds = [
            x,
            y,
            number(&self.width, settings)? / 100.0,
            number(&self.height, settings)? / 100.0,
        ];
        if bounds_ok(bounds) {
            Ok(bounds)
        } else {
            Err(
                "Use positive width and height that stay within the page's 100 percent edges."
                    .into(),
            )
        }
    }

    fn line(&self, settings: &Settings) -> Result<[[f32; 2]; 2], String> {
        let start = self.position(settings)?;
        let end = [
            start[0] + number(&self.width, settings)? / 100.0,
            start[1] + number(&self.height, settings)? / 100.0,
        ];
        if point_ok(end) && start != end {
            Ok([start, end])
        } else {
            Err("A line needs distinct endpoints inside the page. Width and height are signed offsets; use zero on one axis for a horizontal or vertical line.".into())
        }
    }
}

impl CanvasEditor {
    fn element_from_draft(&self, tool: Tool, settings: &Settings) -> Result<CanvasElement, String> {
        let description = self.draft.description.clone();
        let width = number(&self.draft.stroke_width, settings)?;
        let color = self.color;
        let element = match tool {
            Tool::Text => CanvasElement::Text {
                bounds: self.draft.bounds(settings)?,
                text: self.draft.text.clone(),
                font_size: number(&self.draft.font_size, settings)?,
                color,
                description,
            },
            Tool::Line => {
                let [start, end] = self.draft.line(settings)?;
                CanvasElement::Line {
                    start,
                    end,
                    width,
                    color,
                    description,
                }
            }
            Tool::Rectangle => CanvasElement::Rectangle {
                bounds: self.draft.bounds(settings)?,
                width,
                color,
                description,
            },
            Tool::Ellipse => CanvasElement::Ellipse {
                bounds: self.draft.bounds(settings)?,
                width,
                color,
                description,
            },
            _ => {
                return Err(
                    "Choose Text, Line, Rectangle or Ellipse to add an object by coordinates."
                        .into(),
                );
            }
        };
        element.validate()?;
        Ok(element)
    }

    fn apply_draft(&mut self, settings: &Settings) -> bool {
        let Some(index) = self
            .selected
            .filter(|index| *index < self.page().elements.len())
        else {
            return false;
        };
        let result = (|| {
            let mut element = self.page().elements[index].clone();
            if let CanvasElement::Line { start, end, .. } = &mut element {
                let [next_start, next_end] = self.draft.line(settings)?;
                *start = next_start;
                *end = next_end;
            } else {
                element.transform(self.draft.bounds(settings)?);
            }
            *element.description_mut() = self.draft.description.clone();
            *element.color_mut() = self.color;
            match &mut element {
                CanvasElement::Text {
                    text, font_size, ..
                } => {
                    *text = self.draft.text.clone();
                    *font_size = number(&self.draft.font_size, settings)?;
                }
                CanvasElement::Ink { width, .. }
                | CanvasElement::Line { width, .. }
                | CanvasElement::Rectangle { width, .. }
                | CanvasElement::Ellipse { width, .. } => {
                    *width = number(&self.draft.stroke_width, settings)?
                }
            }
            element.validate()?;
            Ok::<_, String>(element)
        })();
        match result {
            Ok(element) => {
                let changed =
                    self.change(|doc| doc.pages[doc.selected_page].elements[index] = element);
                if changed {
                    self.notice = "Object coordinates and text updated.".into();
                }
                changed
            }
            Err(error) => {
                self.notice = error;
                false
            }
        }
    }

    fn add_coordinate_point(&mut self, settings: &Settings) {
        match self.draft.position(settings) {
            Ok(position) if self.coordinate_points.len() < MAX_STROKE_POINTS => {
                let total: usize = self
                    .document
                    .pages
                    .iter()
                    .flat_map(|p| &p.elements)
                    .map(|e| match e {
                        CanvasElement::Ink { points, .. } => points.len(),
                        _ => 0,
                    })
                    .sum();
                if total + self.coordinate_points.len() >= MAX_POINTS {
                    self.notice = "The notebook ink point limit is reached.".into();
                } else {
                    self.coordinate_points.push(InkPoint {
                        position,
                        pressure: 1.0,
                    });
                    self.notice = format!(
                        "{} pending ink points. Choose Finish ink stroke to add them to the page.",
                        self.coordinate_points.len()
                    );
                }
            }
            Ok(_) => {
                self.notice = "Finish this ink stroke before adding more than 4,096 points.".into()
            }
            Err(error) => self.notice = error,
        }
    }

    fn finish_coordinate_ink(&mut self, settings: &Settings) -> bool {
        if self.coordinate_points.is_empty() {
            return false;
        }
        let width = match number(&self.draft.stroke_width, settings) {
            Ok(width) => width,
            Err(error) => {
                self.notice = error;
                return false;
            }
        };
        let changed = self.add_element(CanvasElement::Ink {
            points: self.coordinate_points.clone(),
            width,
            color: self.color,
            description: self.draft.description.clone(),
        });
        if changed {
            self.coordinate_points.clear();
        }
        changed
    }

    fn erase_at(&mut self, point: [f32; 2]) -> bool {
        let radius = 12.0;
        let before = self.page().elements.len();
        self.page_mut().elements.retain(|element| {
            let CanvasElement::Ink { points, width, .. } = element else {
                return true;
            };
            !ink_hit(points, point, radius + *width / 2.0)
        });
        before != self.page().elements.len()
    }

    fn hit(&self, point: [f32; 2]) -> Option<usize> {
        self.page()
            .elements
            .iter()
            .rposition(|element| match element {
                CanvasElement::Ink { points, width, .. } => {
                    ink_hit(points, point, 8.0 + *width / 2.0)
                }
                CanvasElement::Line {
                    start, end, width, ..
                } => segment_distance(point, *start, *end) < 8.0 + *width / 2.0,
                _ => {
                    let [x, y, w, h] = element.bounds();
                    point[0] >= x && point[0] <= x + w && point[1] >= y && point[1] <= y + h
                }
            })
    }

    fn start_pointer(&mut self, point: [f32; 2], pressure: f32, settings: &Settings) {
        match self.tool {
            Tool::Ink | Tool::Line | Tool::Rectangle | Tool::Ellipse => {
                let width = match number(&self.draft.stroke_width, settings) {
                    Ok(value) => value,
                    Err(error) => {
                        self.notice = error;
                        return;
                    }
                };
                let probe = CanvasElement::Ink {
                    points: vec![InkPoint {
                        position: point,
                        pressure,
                    }],
                    width,
                    color: self.color,
                    description: self.draft.description.clone(),
                };
                let mut document = self.document.clone();
                document.pages[document.selected_page].elements.push(probe);
                if let Err(error) = document.validate() {
                    self.notice = error;
                    return;
                }
                self.gesture = Some(Gesture::Draw {
                    start: point,
                    end: point,
                    points: vec![InkPoint {
                        position: point,
                        pressure,
                    }],
                    tool: self.tool,
                    color: self.color,
                    width,
                    description: self.draft.description.clone(),
                });
            }
            Tool::Select => {
                self.select(self.hit(point));
                if self.selected.is_some() {
                    self.gesture = Some(Gesture::Move {
                        before: self.document.clone(),
                        changed: false,
                    });
                }
            }
            Tool::Eraser => {
                let before = self.document.clone();
                let changed = self.erase_at(point);
                self.selected = None;
                self.gesture = Some(Gesture::Erase { before, changed });
            }
            Tool::Hand => self.gesture = Some(Gesture::Pan),
            Tool::Text => {
                self.draft.x = format!("{:.2}", point[0] * 100.0);
                self.draft.y = format!("{:.2}", point[1] * 100.0);
                self.draft.width = format!("{:.2}", ((1.0 - point[0]) * 100.0).clamp(0.1, 35.0));
                self.draft.height = format!("{:.2}", ((1.0 - point[1]) * 100.0).clamp(0.1, 15.0));
                self.inspector_open = true;
                self.notice =
                    "Enter Object text in Coordinates and elements, then choose Add text.".into();
            }
        }
    }

    fn move_pointer(
        &mut self,
        point: [f32; 2],
        pressure: f32,
        delta: [f32; 2],
        viewport_delta: [f32; 2],
    ) {
        let Some(mut gesture) = self.gesture.take() else {
            return;
        };
        match &mut gesture {
            Gesture::Draw {
                end, points, tool, ..
            } => {
                *end = point;
                if *tool == Tool::Ink
                    && points.last().is_none_or(|last| {
                        segment_distance(point, last.position, last.position) > 0.7
                    })
                {
                    let total: usize = self
                        .document
                        .pages
                        .iter()
                        .flat_map(|p| &p.elements)
                        .map(|e| match e {
                            CanvasElement::Ink { points, .. } => points.len(),
                            _ => 0,
                        })
                        .sum();
                    if points.len() < MAX_STROKE_POINTS && total + points.len() < MAX_POINTS {
                        points.push(InkPoint {
                            position: point,
                            pressure,
                        });
                    } else {
                        self.notice =
                            "Ink point limit reached. Finish this stroke or remove older ink."
                                .into();
                    }
                }
            }
            Gesture::Move { changed, .. } => {
                if let Some(index) = self
                    .selected
                    .filter(|index| *index < self.page().elements.len())
                {
                    let mut bounds = self.page().elements[index].bounds();
                    bounds[2] = bounds[2].max(0.001);
                    bounds[3] = bounds[3].max(0.001);
                    bounds[0] = (bounds[0] + delta[0]).clamp(0.0, (1.0 - bounds[2]).max(0.0));
                    bounds[1] = (bounds[1] + delta[1]).clamp(0.0, (1.0 - bounds[3]).max(0.0));
                    if delta != [0.0; 2] {
                        self.page_mut().elements[index].transform(bounds);
                        *changed = true;
                    }
                }
            }
            Gesture::Erase { changed, .. } => *changed |= self.erase_at(point),
            Gesture::Pan => {
                for (axis, shift) in viewport_delta.into_iter().enumerate() {
                    self.document.pan[axis] = (self.document.pan[axis] + shift).clamp(-1.0, 1.0);
                }
            }
        }
        self.gesture = Some(gesture);
    }
}

fn segment_distance(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
    let p = Vec2::new(point[0] * PAGE_WIDTH, point[1] * PAGE_HEIGHT);
    let a = Vec2::new(start[0] * PAGE_WIDTH, start[1] * PAGE_HEIGHT);
    let b = Vec2::new(end[0] * PAGE_WIDTH, end[1] * PAGE_HEIGHT);
    let length = (b - a).length_sq();
    let t = if length <= f32::EPSILON {
        0.0
    } else {
        ((p - a).dot(b - a) / length).clamp(0.0, 1.0)
    };
    (p - (a + (b - a) * t)).length()
}

fn ink_hit(points: &[InkPoint], point: [f32; 2], radius: f32) -> bool {
    points
        .first()
        .is_some_and(|first| segment_distance(point, first.position, first.position) <= radius)
        || points
            .windows(2)
            .any(|pair| segment_distance(point, pair[0].position, pair[1].position) <= radius)
}

fn screen_point(page: Rect, [x, y]: [f32; 2]) -> Pos2 {
    page.min + Vec2::new(x * page.width(), y * page.height())
}
fn screen_bounds(page: Rect, [x, y, w, h]: [f32; 4]) -> Rect {
    Rect::from_min_size(
        screen_point(page, [x, y]),
        Vec2::new(w * page.width(), h * page.height()),
    )
}

fn paint_element(
    painter: &egui::Painter,
    page: Rect,
    element: &CanvasElement,
    palette: appearance::Palette,
) {
    let scale = page.width() / PAGE_WIDTH;
    let color = element.color().paint(palette);
    match element {
        CanvasElement::Ink { points, width, .. } => {
            if points.len() == 1 {
                painter.circle_filled(
                    screen_point(page, points[0].position),
                    (*width * scale / 2.0).max(0.6),
                    color,
                );
            } else {
                for chunk in points.chunks(16) {
                    let pressure =
                        chunk.iter().map(|p| p.pressure).sum::<f32>() / chunk.len() as f32;
                    let line: Vec<_> = chunk
                        .iter()
                        .map(|p| screen_point(page, p.position))
                        .collect();
                    if line.len() == 1 {
                        painter.circle_filled(line[0], (*width * scale / 2.0).max(0.6), color);
                    } else {
                        painter.line(
                            line,
                            Stroke::new(
                                (*width * scale * (0.35 + pressure * 0.65)).max(0.6),
                                color,
                            ),
                        );
                    }
                }
                // Join chunk boundaries without omitting any stored point.
                for index in (16..points.len()).step_by(16) {
                    painter.line_segment(
                        [
                            screen_point(page, points[index - 1].position),
                            screen_point(page, points[index].position),
                        ],
                        Stroke::new((*width * scale).max(0.6), color),
                    );
                }
            }
        }
        CanvasElement::Line {
            start, end, width, ..
        } => {
            painter.line_segment(
                [screen_point(page, *start), screen_point(page, *end)],
                Stroke::new((*width * scale).max(0.6), color),
            );
        }
        CanvasElement::Rectangle { bounds, width, .. } => {
            painter.rect_stroke(
                screen_bounds(page, *bounds),
                0.0,
                Stroke::new((*width * scale).max(0.6), color),
                egui::StrokeKind::Inside,
            );
        }
        CanvasElement::Ellipse { bounds, width, .. } => {
            let rect = screen_bounds(page, *bounds);
            painter.add(egui::Shape::ellipse_stroke(
                rect.center(),
                rect.size() / 2.0,
                Stroke::new((*width * scale).max(0.6), color),
            ));
        }
        CanvasElement::Text {
            bounds,
            text,
            font_size,
            ..
        } => {
            let rect = screen_bounds(page, *bounds);
            let mut job = egui::text::LayoutJob::simple(
                text.clone(),
                FontId::proportional((*font_size * scale).max(1.0)),
                color,
                rect.width(),
            );
            job.wrap.max_rows = 128;
            let galley = painter.fonts_mut(|fonts| fonts.layout_job(job));
            painter
                .with_clip_rect(rect.intersect(painter.clip_rect()))
                .galley(rect.min, galley, color);
        }
    }
}

impl CanvasEditor {
    pub(crate) fn notebook_snapshot(&self) -> CanvasNotebook {
        let mut document = self.document.clone();
        if let Some(element) = self.pending_element() {
            document.pages[document.selected_page]
                .elements
                .push(element);
        }
        document
    }

    pub(crate) fn apply_notebook(&mut self, notebook: CanvasNotebook) {
        *self = Self {
            document: notebook,
            ..Self::default()
        };
    }

    pub(crate) fn overlay_open(&self) -> bool {
        self.confirm_delete
    }

    fn page(&self) -> &CanvasPage {
        &self.document.pages[self.document.selected_page]
    }
    fn page_mut(&mut self) -> &mut CanvasPage {
        &mut self.document.pages[self.document.selected_page]
    }

    fn push_undo(&mut self, before: CanvasNotebook) {
        self.undo.push(before);
        self.redo.clear();
        while self.undo.len() > MAX_UNDO
            || self
                .undo
                .iter()
                .map(CanvasNotebook::memory_cost)
                .sum::<usize>()
                > MAX_UNDO_BYTES
        {
            self.undo.remove(0);
        }
    }

    fn change(&mut self, edit: impl FnOnce(&mut CanvasNotebook)) -> bool {
        let before = self.document.clone();
        edit(&mut self.document);
        if let Err(error) = self.document.validate() {
            self.document = before;
            self.notice = error;
            return false;
        }
        if self.document == before {
            return false;
        }
        self.push_undo(before);
        true
    }

    fn add_element(&mut self, element: CanvasElement) -> bool {
        if let Err(error) = element.validate() {
            self.notice = error;
            return false;
        }
        let index = self.page().elements.len();
        let changed = self.change(|doc| doc.pages[doc.selected_page].elements.push(element));
        if changed {
            self.select(Some(index));
            self.notice = "Object added. Save the notebook to keep it.".into();
        }
        changed
    }

    fn undo(&mut self) -> bool {
        self.finish_gesture();
        if let Some(previous) = self.undo.pop() {
            self.redo
                .push(std::mem::replace(&mut self.document, previous));
            self.selected = None;
            self.notice = "Canvas change undone.".into();
            true
        } else {
            false
        }
    }

    fn redo(&mut self) -> bool {
        self.finish_gesture();
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.document, next));
            self.selected = None;
            self.notice = "Canvas change restored.".into();
            true
        } else {
            false
        }
    }

    fn select(&mut self, index: Option<usize>) {
        self.selected = index.filter(|index| *index < self.page().elements.len());
        if let Some(element) = self
            .selected
            .and_then(|index| self.page().elements.get(index))
            .cloned()
        {
            let [x, y, w, h] = element.bounds();
            self.draft.x = format!("{:.2}", x * 100.0);
            self.draft.y = format!("{:.2}", y * 100.0);
            self.draft.width = format!("{:.2}", (w * 100.0).max(0.1));
            self.draft.height = format!("{:.2}", (h * 100.0).max(0.1));
            self.draft.description = element.description().to_string();
            self.color = element.color();
            if let CanvasElement::Line { start, end, .. } = &element {
                self.draft.x = format!("{:.2}", start[0] * 100.0);
                self.draft.y = format!("{:.2}", start[1] * 100.0);
                self.draft.width = format!("{:.2}", (end[0] - start[0]) * 100.0);
                self.draft.height = format!("{:.2}", (end[1] - start[1]) * 100.0);
            }
            match element {
                CanvasElement::Text {
                    text, font_size, ..
                } => {
                    self.draft.text = text;
                    self.draft.font_size = font_size.to_string();
                }
                CanvasElement::Ink { width, .. }
                | CanvasElement::Line { width, .. }
                | CanvasElement::Rectangle { width, .. }
                | CanvasElement::Ellipse { width, .. } => {
                    self.draft.stroke_width = width.to_string()
                }
            }
        }
    }

    fn remove_selected(&mut self) -> bool {
        let Some(index) = self
            .selected
            .filter(|index| *index < self.page().elements.len())
        else {
            return false;
        };
        let changed = self.change(|doc| {
            doc.pages[doc.selected_page].elements.remove(index);
        });
        if changed {
            self.selected = None;
            self.notice = "Selected object deleted. Undo restores it.".into();
        }
        changed
    }

    fn pending_element(&self) -> Option<CanvasElement> {
        let Gesture::Draw {
            start,
            end,
            points,
            tool,
            color,
            width,
            description,
        } = self.gesture.as_ref()?
        else {
            return None;
        };
        let element = match tool {
            Tool::Ink => CanvasElement::Ink {
                points: points.clone(),
                width: *width,
                color: *color,
                description: description.clone(),
            },
            Tool::Line => CanvasElement::Line {
                start: *start,
                end: *end,
                width: *width,
                color: *color,
                description: description.clone(),
            },
            Tool::Rectangle => CanvasElement::Rectangle {
                bounds: point_bounds([*start, *end]),
                width: *width,
                color: *color,
                description: description.clone(),
            },
            Tool::Ellipse => CanvasElement::Ellipse {
                bounds: point_bounds([*start, *end]),
                width: *width,
                color: *color,
                description: description.clone(),
            },
            _ => return None,
        };
        element.validate().ok()?;
        Some(element)
    }

    fn finish_gesture(&mut self) -> bool {
        let pending = self.pending_element();
        match self.gesture.take() {
            Some(Gesture::Draw { .. }) => pending.is_some_and(|element| self.add_element(element)),
            Some(Gesture::Move { before, changed }) | Some(Gesture::Erase { before, changed })
                if changed =>
            {
                self.push_undo(before);
                true
            }
            _ => false,
        }
    }
}

fn xml_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            '\u{fffe}' | '\u{ffff}' => escaped.push('\u{fffd}'),
            c if c.is_control() && !matches!(c, '\n' | '\r' | '\t') => escaped.push('\u{fffd}'),
            c => escaped.push(c),
        }
    }
    escaped
}

fn svg_write(output: &mut String, arguments: std::fmt::Arguments<'_>) -> Result<(), String> {
    output
        .write_fmt(arguments)
        .map_err(|_| "Could not format notebook SVG.".to_string())?;
    if output.len() > MAX_EXPORT_BYTES {
        return Err(
            "SVG export exceeds 16 MiB. Export an individual page or remove older content.".into(),
        );
    }
    Ok(())
}

fn svg_text_lines(text: &str, bounds: [f32; 4], font_size: f32) -> Vec<String> {
    let columns = ((bounds[2] * PAGE_WIDTH) / (font_size * 0.6))
        .floor()
        .max(1.0) as usize;
    let rows = ((bounds[3] * PAGE_HEIGHT) / (font_size * 1.25))
        .floor()
        .clamp(1.0, 128.0) as usize;
    let mut result = Vec::new();
    for source in text.lines() {
        let mut line = String::new();
        let mut count = 0;
        for character in source.chars() {
            line.push(character);
            count += 1;
            if count == columns {
                result.push(std::mem::take(&mut line));
                count = 0;
                if result.len() >= rows {
                    return result;
                }
            }
        }
        if !line.is_empty() || source.is_empty() {
            result.push(line);
            if result.len() >= rows {
                return result;
            }
        }
    }
    result
}

fn svg_element(
    output: &mut String,
    element: &CanvasElement,
    page_index: usize,
    element_index: usize,
) -> Result<(), String> {
    let [r, g, b] = element.color().rgb(false);
    let color = format!("#{r:02x}{g:02x}{b:02x}");
    svg_write(
        output,
        format_args!("<g><title>{}</title>", xml_escape(&element.summary())),
    )?;
    match element {
        CanvasElement::Ink { points, width, .. } => {
            if points.len() == 1 {
                let point = points[0];
                svg_write(
                    output,
                    format_args!(
                        "<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{:.3}\" fill=\"{}\"/>",
                        point.position[0] * PAGE_WIDTH,
                        point.position[1] * PAGE_HEIGHT,
                        width / 2.0,
                        color
                    ),
                )?;
            } else {
                for chunk in points.chunks(16) {
                    let pressure =
                        chunk.iter().map(|point| point.pressure).sum::<f32>() / chunk.len() as f32;
                    svg_write(
                        output,
                        format_args!(
                            "<polyline fill=\"none\" stroke=\"{color}\" stroke-width=\"{:.3}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" points=\"",
                            width * (0.35 + pressure * 0.65)
                        ),
                    )?;
                    for point in chunk {
                        svg_write(
                            output,
                            format_args!(
                                "{:.3},{:.3} ",
                                point.position[0] * PAGE_WIDTH,
                                point.position[1] * PAGE_HEIGHT
                            ),
                        )?;
                    }
                    svg_write(output, format_args!("\"/>"))?;
                }
                for index in (16..points.len()).step_by(16) {
                    let start = points[index - 1].position;
                    let end = points[index].position;
                    svg_write(
                        output,
                        format_args!(
                            "<line x1=\"{:.3}\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"{color}\" stroke-width=\"{width:.3}\" stroke-linecap=\"round\"/>",
                            start[0] * PAGE_WIDTH,
                            start[1] * PAGE_HEIGHT,
                            end[0] * PAGE_WIDTH,
                            end[1] * PAGE_HEIGHT
                        ),
                    )?;
                }
            }
        }
        CanvasElement::Line {
            start, end, width, ..
        } => {
            svg_write(
                output,
                format_args!(
                    "<line x1=\"{:.3}\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"{color}\" stroke-width=\"{width:.3}\" stroke-linecap=\"round\"/>",
                    start[0] * PAGE_WIDTH,
                    start[1] * PAGE_HEIGHT,
                    end[0] * PAGE_WIDTH,
                    end[1] * PAGE_HEIGHT
                ),
            )?;
        }
        CanvasElement::Rectangle {
            bounds: [x, y, w, h],
            width,
            ..
        } => {
            svg_write(
                output,
                format_args!(
                    "<rect x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"{:.3}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{width:.3}\"/>",
                    x * PAGE_WIDTH,
                    y * PAGE_HEIGHT,
                    w * PAGE_WIDTH,
                    h * PAGE_HEIGHT
                ),
            )?;
        }
        CanvasElement::Ellipse {
            bounds: [x, y, w, h],
            width,
            ..
        } => {
            svg_write(
                output,
                format_args!(
                    "<ellipse cx=\"{:.3}\" cy=\"{:.3}\" rx=\"{:.3}\" ry=\"{:.3}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{width:.3}\"/>",
                    (x + w / 2.0) * PAGE_WIDTH,
                    (y + h / 2.0) * PAGE_HEIGHT,
                    w * PAGE_WIDTH / 2.0,
                    h * PAGE_HEIGHT / 2.0
                ),
            )?;
        }
        CanvasElement::Text {
            bounds,
            text,
            font_size,
            ..
        } => {
            let [x, y, w, h] = *bounds;
            let id = format!("text-clip-{page_index}-{element_index}");
            svg_write(
                output,
                format_args!(
                    "<defs><clipPath id=\"{id}\"><rect x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"{:.3}\"/></clipPath></defs><text fill=\"{color}\" font-family=\"sans-serif\" font-size=\"{font_size:.3}\" clip-path=\"url(#{id})\">",
                    x * PAGE_WIDTH,
                    y * PAGE_HEIGHT,
                    w * PAGE_WIDTH,
                    h * PAGE_HEIGHT
                ),
            )?;
            for (row, line) in svg_text_lines(text, *bounds, *font_size)
                .into_iter()
                .enumerate()
            {
                svg_write(
                    output,
                    format_args!(
                        "<tspan x=\"{:.3}\" y=\"{:.3}\">{}</tspan>",
                        x * PAGE_WIDTH,
                        y * PAGE_HEIGHT + font_size + row as f32 * font_size * 1.25,
                        xml_escape(&line)
                    ),
                )?;
            }
            svg_write(output, format_args!("</text>"))?;
        }
    }
    svg_write(output, format_args!("</g>"))
}

fn svg_pages(document: &CanvasNotebook, all: bool) -> Result<Vec<u8>, String> {
    document.validate()?;
    let count = if all { document.pages.len() } else { 1 };
    let height = count as f32 * (PAGE_HEIGHT + 32.0) - 32.0;
    let mut output = String::with_capacity(4096);
    svg_write(
        &mut output,
        format_args!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"720\" height=\"{height:.0}\" viewBox=\"0 0 720 {height:.0}\" role=\"img\"><title>OpenCalc notebook pages</title>"
        ),
    )?;
    let pages: &[CanvasPage] = if all {
        &document.pages
    } else {
        &document.pages[document.selected_page..=document.selected_page]
    };
    for (page_index, page) in pages.iter().enumerate() {
        let offset = page_index as f32 * (PAGE_HEIGHT + 32.0);
        svg_write(
            &mut output,
            format_args!(
                "<g transform=\"translate(0 {offset:.0})\"><title>{}</title><rect width=\"720\" height=\"1000\" fill=\"#ffffff\" stroke=\"#a7b6ad\"/>",
                xml_escape(&page.title)
            ),
        )?;
        if page.background != Paper::Blank {
            for y in (40..1000).step_by(32) {
                svg_write(
                    &mut output,
                    format_args!(
                        "<line x1=\"0\" y1=\"{y}\" x2=\"720\" y2=\"{y}\" stroke=\"#d4dfd6\" stroke-width=\"0.75\"/>"
                    ),
                )?;
            }
            if page.background == Paper::Grid {
                for x in (16..720).step_by(32) {
                    svg_write(
                        &mut output,
                        format_args!(
                            "<line x1=\"{x}\" y1=\"0\" x2=\"{x}\" y2=\"1000\" stroke=\"#d4dfd6\" stroke-width=\"0.75\"/>"
                        ),
                    )?;
                }
            }
        }
        svg_write(
            &mut output,
            format_args!(
                "<defs><clipPath id=\"page-clip-{page_index}\"><rect width=\"720\" height=\"1000\"/></clipPath></defs><g clip-path=\"url(#page-clip-{page_index})\">"
            ),
        )?;
        for (element_index, element) in page.elements.iter().enumerate() {
            svg_element(&mut output, element, page_index, element_index)?;
        }
        svg_write(&mut output, format_args!("</g></g>"))?;
    }
    svg_write(&mut output, format_args!("</svg>"))?;
    Ok(output.into_bytes())
}

impl CanvasEditor {
    pub(crate) fn export_current_svg(&self) -> Result<Vec<u8>, String> {
        svg_pages(&self.notebook_snapshot(), false)
    }

    pub(crate) fn export_all_svg(&self) -> Result<Vec<u8>, String> {
        svg_pages(&self.notebook_snapshot(), true)
    }

    fn copy_svg(&mut self, ctx: &egui::Context, all: bool) {
        let result = if all {
            self.export_all_svg()
        } else {
            self.export_current_svg()
        };
        match result.and_then(|bytes| {
            String::from_utf8(bytes).map_err(|_| "Could not encode notebook SVG.".into())
        }) {
            Ok(svg) => {
                ctx.copy_text(svg);
                self.notice = "SVG copied. Paste into a .svg text file to open or print it in a browser. Text wrapping can differ from the notebook.".into();
            }
            Err(error) => self.notice = error,
        }
    }
}

fn canvas_label(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let width = canvas_width(ui);
    let mut job = egui::text::LayoutJob::simple(
        text.into(),
        egui::TextStyle::Body.resolve(ui.style()),
        ui.visuals().text_color(),
        width,
    );
    job.wrap.break_anywhere = width < 180.0;
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    ui.label(galley)
}

fn canvas_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    canvas_choice(ui, text, None)
}

fn canvas_choice(ui: &mut egui::Ui, text: &str, selected: Option<bool>) -> egui::Response {
    let width = (canvas_width(ui) - ui.spacing().button_padding.x * 2.0).max(1.0);
    let mut job = egui::text::LayoutJob::simple(
        text.into(),
        egui::TextStyle::Button.resolve(ui.style()),
        ui.visuals().text_color(),
        width,
    );
    job.wrap.break_anywhere = true;
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let mut button = egui::Button::new(galley).wrap();
    if let Some(selected) = selected {
        button = button.selected(selected);
    }
    let response = ui.add(button);
    if selected.is_some() {
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::SelectableLabel, ui.is_enabled(), text)
        });
    }
    if response.has_focus() && !ui.clip_rect().contains_rect(response.rect) {
        response.scroll_to_me(None);
    }
    response
}

fn canvas_field(ui: &mut egui::Ui, name: &str, value: &mut String, multiline: bool, limit: usize) {
    ui.push_id(name, |ui| {
        canvas_label(ui, name);
        let width = ui.available_width().min(canvas_width(ui)).max(1.0);
        let edit = if multiline {
            egui::TextEdit::multiline(value).desired_rows(2)
        } else {
            egui::TextEdit::singleline(value)
        };
        let response = ui.add_sized([width, 0.0], edit.desired_width(width).char_limit(limit));
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, ui.is_enabled(), name)
        });
        if response.has_focus() && !ui.clip_rect().contains_rect(response.rect) {
            response.scroll_to_me(None);
        }
    });
}

fn canvas_width(ui: &egui::Ui) -> f32 {
    let left = ui.max_rect().left().max(ui.clip_rect().left());
    let right = ui.max_rect().right().min(ui.clip_rect().right());
    (right - left).max(1.0)
}

fn canvas_row<R>(
    ui: &mut egui::Ui,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    if canvas_width(ui) < 360.0 {
        ui.vertical(content)
    } else {
        ui.horizontal_wrapped(content)
    }
}

fn canvas_status(ui: &mut egui::Ui, text: &str) {
    let response = canvas_label(ui, text);
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_label(text);
        node.clear_value();
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

fn page_rect(viewport: Rect, document: &CanvasNotebook) -> Rect {
    let fit = ((viewport.width() - 16.0).max(1.0) / PAGE_WIDTH)
        .min((viewport.height() - 16.0).max(1.0) / PAGE_HEIGHT);
    let size = Vec2::new(PAGE_WIDTH, PAGE_HEIGHT) * fit * document.zoom;
    let offset = Vec2::new(
        document.pan[0] * viewport.width(),
        document.pan[1] * viewport.height(),
    );
    Rect::from_center_size(viewport.center() + offset, size)
}

fn local_point(page: Rect, position: Pos2) -> [f32; 2] {
    [
        ((position.x - page.left()) / page.width()).clamp(0.0, 1.0),
        ((position.y - page.top()) / page.height()).clamp(0.0, 1.0),
    ]
}

fn paint_paper(painter: &egui::Painter, rect: Rect, paper: Paper, palette: appearance::Palette) {
    painter.rect_filled(rect, 0.0, palette.paper);
    let color = if palette.contrast {
        palette.border
    } else {
        palette.border.gamma_multiply(0.6)
    };
    if paper != Paper::Blank {
        for y in (40..1000).step_by(32) {
            painter.line_segment(
                [
                    screen_point(rect, [0.0, y as f32 / PAGE_HEIGHT]),
                    screen_point(rect, [1.0, y as f32 / PAGE_HEIGHT]),
                ],
                Stroke::new(0.75_f32, color),
            );
        }
        if paper == Paper::Grid {
            for x in (16..720).step_by(32) {
                painter.line_segment(
                    [
                        screen_point(rect, [x as f32 / PAGE_WIDTH, 0.0]),
                        screen_point(rect, [x as f32 / PAGE_WIDTH, 1.0]),
                    ],
                    Stroke::new(0.75_f32, color),
                );
            }
        }
    }
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0_f32, palette.border),
        egui::StrokeKind::Inside,
    );
}

impl CanvasEditor {
    fn page_controls(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        canvas_row(ui, |ui| {
            canvas_label(ui, "Canvas page");
            let previous = self.document.selected_page;
            let selector = egui::ComboBox::from_id_salt("canvas-page")
                .selected_text(format!("{} · {}", previous + 1, self.page().title))
                .width(ui.available_width().clamp(1.0, 220.0))
                .truncate()
                .show_ui(ui, |ui| {
                    ui.set_max_width((ui.ctx().content_rect().width() - 24.0).max(1.0));
                    for (index, page) in self.document.pages.iter().enumerate() {
                        if canvas_choice(
                            ui,
                            &format!("{} · {}", index + 1, page.title),
                            Some(self.document.selected_page == index),
                        )
                        .clicked()
                        {
                            self.document.selected_page = index;
                        }
                    }
                });
            selector.response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::ComboBox,
                    ui.is_enabled(),
                    "Canvas page",
                )
            });
            if previous != self.document.selected_page {
                self.select(None);
                self.coordinate_points.clear();
            }
            ui.add_enabled_ui(self.document.pages.len() < MAX_PAGES, |ui| {
                if canvas_button(ui, "Add page").clicked() {
                    changed |= self.change(|document| {
                        document.pages.push(CanvasPage {
                            title: format!("Page {}", document.pages.len() + 1),
                            background: Paper::Blank,
                            elements: Vec::new(),
                        });
                        document.selected_page = document.pages.len() - 1;
                        document.pan = [0.0; 2];
                    });
                    self.select(None);
                    self.notice = "Blank page added.".into();
                }
            });
            ui.add_enabled_ui(self.document.pages.len() > 1, |ui| {
                if canvas_button(ui, "Delete page").clicked() {
                    self.confirm_delete = true;
                }
            });
        });
        let mut title = self.page().title.clone();
        canvas_field(ui, "Page title", &mut title, false, 100);
        if title != self.page().title {
            changed |= self.change(|document| document.pages[document.selected_page].title = title);
        }
        canvas_row(ui, |ui| {
            canvas_label(ui, "Paper background");
            for (paper, name) in [
                (Paper::Blank, "Blank"),
                (Paper::Ruled, "Ruled"),
                (Paper::Grid, "Grid"),
            ] {
                if canvas_choice(ui, name, Some(self.page().background == paper)).clicked() {
                    changed |= self.change(|document| {
                        document.pages[document.selected_page].background = paper
                    });
                }
            }
        });
        changed
    }

    fn tool_controls(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        canvas_row(ui, |ui| {
            for tool in [
                Tool::Select,
                Tool::Ink,
                Tool::Eraser,
                Tool::Text,
                Tool::Line,
                Tool::Rectangle,
                Tool::Ellipse,
                Tool::Hand,
            ] {
                if canvas_choice(ui, tool.label(), Some(self.tool == tool)).clicked() {
                    changed |= self.finish_gesture();
                    self.tool = tool;
                    if matches!(
                        tool,
                        Tool::Text | Tool::Line | Tool::Rectangle | Tool::Ellipse
                    ) {
                        self.inspector_open = true;
                    }
                }
            }
        });
        canvas_row(ui, |ui| {
            canvas_label(ui, "Ink color");
            let selector = egui::ComboBox::from_id_salt("canvas-color")
                .selected_text(self.color.label())
                .width(ui.available_width().clamp(1.0, 90.0))
                .truncate()
                .show_ui(ui, |ui| {
                    ui.set_max_width((ui.ctx().content_rect().width() - 24.0).max(1.0));
                    for color in [
                        InkColor::Ink,
                        InkColor::Teal,
                        InkColor::Blue,
                        InkColor::Red,
                        InkColor::Gold,
                    ] {
                        if canvas_choice(ui, color.label(), Some(self.color == color)).clicked() {
                            self.color = color;
                        }
                    }
                });
            selector.response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), "Ink color")
            });
            ui.add_enabled_ui(!self.undo.is_empty(), |ui| {
                if canvas_button(ui, "Undo").clicked() {
                    changed |= self.undo();
                }
            });
            ui.add_enabled_ui(!self.redo.is_empty(), |ui| {
                if canvas_button(ui, "Redo").clicked() {
                    changed |= self.redo();
                }
            });
        });
        canvas_row(ui, |ui| {
            if canvas_button(ui, "Zoom out").clicked() {
                self.document.zoom = (self.document.zoom - 0.1).max(0.25);
            }
            canvas_label(
                ui,
                &format!("Canvas zoom {:.0}%", self.document.zoom * 100.0),
            );
            if canvas_button(ui, "Zoom in").clicked() {
                self.document.zoom = (self.document.zoom + 0.1).min(4.0);
            }
            if canvas_button(ui, "Fit page").clicked() {
                self.document.zoom = 1.0;
                self.document.pan = [0.0; 2];
            }
            for (label, axis, direction) in [
                ("Pan left", 0, -0.1),
                ("Pan right", 0, 0.1),
                ("Pan up", 1, -0.1),
                ("Pan down", 1, 0.1),
            ] {
                if canvas_button(ui, label).clicked() {
                    self.document.pan[axis] =
                        (self.document.pan[axis] + direction).clamp(-1.0, 1.0);
                }
            }
        });
        changed
    }

    fn surface(&mut self, ui: &mut egui::Ui, settings: &Settings) -> bool {
        let mut available_height = ui.available_height();
        if canvas_width(ui) < 360.0 {
            // A narrow notebook commonly lives in a vertical scroll area with unbounded height.
            available_height = available_height.min(ui.clip_rect().bottom() - ui.cursor().top());
        }
        let height = if available_height.is_finite() {
            available_height.clamp(180.0, 560.0)
        } else {
            480.0
        };
        let (viewport, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().min(canvas_width(ui)).max(1.0), height),
            Sense::click_and_drag(),
        );
        let palette = appearance::palette(ui.ctx());
        let rect = page_rect(viewport, &self.document);
        let painter = ui
            .painter()
            .with_clip_rect(viewport.intersect(ui.clip_rect()));
        painter.rect_filled(viewport, 0.0, palette.panel);
        paint_paper(&painter, rect, self.page().background, palette);
        let page_painter =
            painter.with_clip_rect(rect.intersect(viewport).intersect(ui.clip_rect()));
        for element in &self.page().elements {
            paint_element(&page_painter, rect, element, palette);
        }
        if let Some(element) = self.pending_element() {
            paint_element(&page_painter, rect, &element, palette);
        }
        if let Some(element) = self
            .selected
            .and_then(|index| self.page().elements.get(index))
        {
            let selected = screen_bounds(rect, element.bounds()).expand(4.0);
            page_painter.rect_stroke(
                selected,
                0.0,
                Stroke::new(2.0_f32, palette.accent),
                egui::StrokeKind::Outside,
            );
            for corner in [
                selected.left_top(),
                selected.right_top(),
                selected.left_bottom(),
                selected.right_bottom(),
            ] {
                page_painter.circle_filled(corner, 3.0, palette.ink);
            }
        }
        if !rect.intersects(viewport) {
            painter.text(
                viewport.center(),
                Align2::CENTER_CENTER,
                "Use Fit page to return to the page.",
                egui::TextStyle::Body.resolve(ui.style()),
                palette.ink,
            );
        }
        let accessible = format!(
            "Canvas page {}: {}. {} objects. Use Coordinates and elements to create, read, move or delete objects without drawing. Ink descriptions are supplied by the author.",
            self.document.selected_page + 1,
            self.page().title,
            self.page().elements.len()
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Other, ui.is_enabled(), &accessible)
        });
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Canvas);
            node.set_label(accessible);
            node.clear_value();
        });
        if response.has_focus() && !ui.clip_rect().contains_rect(response.rect) {
            response.scroll_to_me(None);
        }
        #[cfg(test)]
        ui.ctx()
            .data_mut(|data| data.insert_temp(egui::Id::new("canvas-page-rect"), rect));
        if !ui.is_enabled() || egui::Popup::is_any_open(ui.ctx()) {
            return self.finish_gesture();
        }
        let pressure = ui.input(|input| {
            input
                .events
                .iter()
                .rev()
                .find_map(|event| match event {
                    egui::Event::Touch {
                        force: Some(force), ..
                    } if force.is_finite() => Some(force.clamp(0.0, 1.0)),
                    _ => None,
                })
                .unwrap_or(1.0)
        });
        let mut changed = false;
        if response.drag_started()
            && let Some(position) = ui
                .input(|input| input.pointer.press_origin())
                .filter(|position| rect.contains(*position))
        {
            self.start_pointer(local_point(rect, position), pressure, settings);
        }
        if response.dragged()
            && let Some(position) = response.interact_pointer_pos()
        {
            let delta = response.drag_delta();
            self.move_pointer(
                local_point(rect, position),
                pressure,
                [delta.x / rect.width(), delta.y / rect.height()],
                [delta.x / viewport.width(), delta.y / viewport.height()],
            );
        }
        if response.clicked()
            && let Some(position) = response
                .interact_pointer_pos()
                .filter(|position| rect.contains(*position))
        {
            self.start_pointer(local_point(rect, position), pressure, settings);
            changed |= self.finish_gesture();
        }
        if response.drag_stopped()
            || (self.gesture.is_some() && !ui.input(|input| input.pointer.primary_down()))
        {
            changed |= self.finish_gesture();
        }
        if response.hovered() && self.gesture.is_none() {
            let zoom = ui.input(|input| input.zoom_delta());
            if zoom.is_finite() && zoom != 1.0 {
                self.document.zoom = (self.document.zoom * zoom).clamp(0.25, 4.0);
            }
        }
        changed
    }

    fn inspector(&mut self, ui: &mut egui::Ui, settings: &Settings) -> bool {
        let mut changed = false;
        let toggle = canvas_choice(ui, "Coordinates and elements", Some(self.inspector_open));
        toggle.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::CollapsingHeader,
                ui.is_enabled(),
                "Coordinates and elements",
            )
        });
        if toggle.clicked() {
            self.inspector_open = !self.inspector_open;
        }
        ui.ctx()
            .accesskit_node_builder(toggle.id, |node| node.set_expanded(self.inspector_open));
        if !self.inspector_open {
            return false;
        }
        canvas_label(
            ui,
            "Coordinates use percent of the page. X increases right; Y increases down. Text and descriptions can be read independently of the drawing.",
        );
        canvas_label(
            ui,
            "For lines, width and height are signed offsets from X and Y. Set one offset to 0 for a horizontal or vertical line. Other objects need positive width and height.",
        );
        canvas_field(ui, "Object X percent", &mut self.draft.x, false, 24);
        canvas_field(ui, "Object Y percent", &mut self.draft.y, false, 24);
        canvas_field(ui, "Object width percent", &mut self.draft.width, false, 24);
        canvas_field(
            ui,
            "Object height percent",
            &mut self.draft.height,
            false,
            24,
        );
        canvas_field(ui, "Object text", &mut self.draft.text, true, 16 * 1024);
        canvas_field(
            ui,
            "Object description",
            &mut self.draft.description,
            true,
            4096,
        );
        canvas_field(ui, "Stroke width", &mut self.draft.stroke_width, false, 24);
        canvas_field(ui, "Text font size", &mut self.draft.font_size, false, 24);
        canvas_row(ui, |ui| {
            if canvas_button(ui, "Add text").clicked() {
                match self.element_from_draft(Tool::Text, settings) {
                    Ok(element) => changed |= self.add_element(element),
                    Err(error) => self.notice = error,
                }
            }
            if canvas_button(ui, "Add shape").clicked() {
                match self.element_from_draft(self.tool, settings) {
                    Ok(element) => changed |= self.add_element(element),
                    Err(error) => self.notice = error,
                }
            }
            ui.add_enabled_ui(self.selected.is_some(), |ui| {
                if canvas_button(ui, "Apply object changes").clicked() {
                    changed |= self.apply_draft(settings);
                }
                if canvas_button(ui, "Delete selected object").clicked() {
                    changed |= self.remove_selected();
                }
            });
        });
        canvas_label(
            ui,
            "Ink by coordinates: set X and Y, add each point, then finish the stroke. Pending points are not saved until you finish.",
        );
        canvas_row(ui, |ui| {
            if canvas_button(ui, "Add ink point").clicked() {
                self.add_coordinate_point(settings);
            }
            ui.add_enabled_ui(!self.coordinate_points.is_empty(), |ui| {
                if canvas_button(ui, "Finish ink stroke").clicked() {
                    changed |= self.finish_coordinate_ink(settings);
                }
                if canvas_button(ui, "Cancel ink points").clicked() {
                    self.coordinate_points.clear();
                    self.notice = "Pending ink points cancelled.".into();
                }
            });
        });
        canvas_label(ui, "Elements");
        if self.page().elements.is_empty() {
            canvas_label(ui, "This page has no objects.");
        }
        let mut select = None;
        for (index, element) in self.page().elements.iter().enumerate() {
            let label = format!("Object {}: {}", index + 1, element.summary());
            let response = canvas_choice(ui, &label, Some(self.selected == Some(index)));
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::SelectableLabel,
                    ui.is_enabled(),
                    &label,
                )
            });
            if response.has_focus() && !ui.clip_rect().contains_rect(response.rect) {
                response.scroll_to_me(None);
            }
            if response.clicked() {
                select = Some(index);
            }
        }
        if let Some(index) = select {
            self.select(Some(index));
            self.tool = Tool::Select;
        }
        changed
    }

    fn deletion_confirmation(&mut self, ctx: &egui::Context, settings: &Settings) -> bool {
        if !self.confirm_delete {
            return false;
        }
        let mut choice = None;
        let viewport = ctx.content_rect();
        let response = egui::Modal::new(egui::Id::new("canvas-delete-page")).frame(egui::Frame::popup(&ctx.style()).inner_margin(6)).show(ctx, |ui| {
            ui.set_width((viewport.width() - 24.0).clamp(1.0, 420.0));
            let header = ui.vertical(|ui| {
                canvas_label(ui, "Delete canvas page?");
                if canvas_button(ui, "Cancel").clicked() { choice = Some(false); }
            });
            let height = (viewport.height() - header.response.rect.height() - 36.0).max(12.0);
            egui::ScrollArea::vertical().id_salt("canvas-delete-body").animated(!settings.access.reduced_motion).max_height(height).min_scrolled_height(height.min(100.0)).show(ui, |ui| {
                canvas_label(ui, &format!("Delete page {}: {} and its {} objects? Undo can restore the page during this session.", self.document.selected_page + 1, self.page().title, self.page().elements.len()));
                if canvas_button(ui, "Delete this page").clicked() { choice = Some(true); }
            });
        });
        if response.should_close() {
            choice = Some(false);
        }
        match choice {
            Some(delete) => {
                self.confirm_delete = false;
                if delete && self.document.pages.len() > 1 {
                    let changed = self.change(|document| {
                        document.pages.remove(document.selected_page);
                        document.selected_page =
                            document.selected_page.min(document.pages.len() - 1);
                    });
                    self.select(None);
                    self.coordinate_points.clear();
                    self.notice = "Page deleted. Undo restores it.".into();
                    changed
                } else {
                    self.notice = "Page kept.".into();
                    false
                }
            }
            None => false,
        }
    }

    /// Returns true for content edits. Navigation and tool settings are excluded.
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui, settings: &Settings) -> bool {
        let mut changed = false;
        if self.confirm_delete || !ui.is_enabled() {
            changed |= self.finish_gesture();
        }
        ui.push_id("canvas-editor", |ui| {
            ui.add_enabled_ui(!self.confirm_delete, |ui| {
                if ui.is_enabled() && !crate::text_input_focused(ui.ctx()) && !egui::Popup::is_any_open(ui.ctx()) {
                    let undo = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
                    let redo = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::Z);
                    let redo_y = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Y);
                    if ui.input_mut(|input| input.consume_shortcut(&redo) || input.consume_shortcut(&redo_y)) { changed |= self.redo(); }
                    else if ui.input_mut(|input| input.consume_shortcut(&undo)) { changed |= self.undo(); }
                    if self.selected.is_some() && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Delete)) { changed |= self.remove_selected(); }
                }
                canvas_label(ui, "Notebook canvas");
                let narrow = canvas_width(ui) < 360.0;
                if narrow {
                    canvas_label(ui, &format!("Page {} of {}: {} · Tool: {}", self.document.selected_page + 1, self.document.pages.len(), self.page().title, self.tool.label()));
                    let toggle = canvas_choice(ui, "Page tools", Some(self.page_tools_open));
                    toggle.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, ui.is_enabled(), "Page tools"));
                    if toggle.clicked() { self.page_tools_open = !self.page_tools_open; }
                    ui.ctx().accesskit_node_builder(toggle.id, |node| node.set_expanded(self.page_tools_open));
                }
                let tools_visible = !narrow || self.page_tools_open;
                if tools_visible {
                    changed |= self.page_controls(ui);
                    changed |= self.tool_controls(ui);
                }
                changed |= self.surface(ui, settings);
                canvas_label(ui, "Draw with Ink or a shape tool. Select moves objects. Eraser removes whole ink strokes; select other objects to delete them. Hand pans the page.");
                if tools_visible {
                    changed |= self.inspector(ui, settings);
                    canvas_row(ui, |ui| {
                        if canvas_button(ui, "Copy page SVG").clicked() { self.copy_svg(ui.ctx(), false); }
                        if canvas_button(ui, "Copy all pages SVG").clicked() { self.copy_svg(ui.ctx(), true); }
                    });
                }
                if !self.notice.is_empty() { canvas_status(ui, &self.notice); }
            });
        });
        changed | self.deletion_confirmation(ui.ctx(), settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest};

    struct Session {
        editor: CanvasEditor,
        settings: Settings,
        ctx: egui::Context,
        output: egui::FullOutput,
        time: f64,
        size: Vec2,
    }

    impl Session {
        fn new() -> Self {
            Self::with_view(Settings::default(), Vec2::new(1200.0, 3200.0))
        }

        fn with_view(settings: Settings, size: Vec2) -> Self {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut session = Self {
                editor: CanvasEditor::default(),
                settings,
                ctx,
                output: egui::FullOutput::default(),
                time: 0.0,
                size,
            };
            session.frame(Vec::new());
            session.frame(Vec::new());
            session
        }

        fn frame(&mut self, events: Vec<egui::Event>) {
            self.time += 1.0 / 60.0;
            appearance::apply(&self.ctx, &self.settings);
            self.output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        self.editor.ui(ui, &self.settings);
                    });
                },
            );
        }

        fn action(&mut self, label: &str, action: Action) {
            let target = self
                .output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("native tree")
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .unwrap_or_else(|| panic!("Native canvas control missing: {label}"))
                .0;
            self.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target,
                data: None,
            })]);
            self.frame(Vec::new());
        }

        fn click(&mut self, label: &str) {
            self.action(label, Action::Click);
        }

        fn replace(&mut self, label: &str, text: &str) {
            self.action(label, Action::Focus);
            self.frame(vec![egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }]);
            self.frame(vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
                egui::Event::Text(text.into()),
            ]);
            self.frame(Vec::new());
        }

        fn pointer(&mut self, position: Pos2, pressed: Option<bool>) {
            let mut events = vec![egui::Event::PointerMoved(position)];
            if let Some(pressed) = pressed {
                events.push(egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            self.frame(events);
        }

        fn page_rect(&self) -> Rect {
            self.ctx
                .data(|data| data.get_temp(egui::Id::new("canvas-page-rect")))
                .unwrap()
        }
    }

    fn text(value: &str) -> CanvasElement {
        CanvasElement::Text {
            bounds: [0.1, 0.1, 0.7, 0.3],
            text: value.into(),
            font_size: 18.0,
            color: InkColor::Blue,
            description: "A typed explanation".into(),
        }
    }

    fn ink(points: usize) -> CanvasElement {
        CanvasElement::Ink {
            points: vec![
                InkPoint {
                    position: [0.25, 0.4],
                    pressure: 0.6
                };
                points
            ],
            width: 2.0,
            color: InkColor::Ink,
            description: "A sketched force arrow".into(),
        }
    }

    #[test]
    fn vector_notebook_round_trip_preserves_all_objects_and_view() {
        let mut editor = CanvasEditor::default();
        for element in [
            ink(3),
            text("F = ma; H₂O"),
            CanvasElement::Line {
                start: [0.1, 0.2],
                end: [0.6, 0.3],
                width: 3.0,
                color: InkColor::Red,
                description: "Velocity".into(),
            },
            CanvasElement::Rectangle {
                bounds: [0.3, 0.4, 0.2, 0.1],
                width: 1.0,
                color: InkColor::Teal,
                description: "System".into(),
            },
            CanvasElement::Ellipse {
                bounds: [0.6, 0.6, 0.2, 0.2],
                width: 1.0,
                color: InkColor::Gold,
                description: "Orbit".into(),
            },
        ] {
            assert!(editor.add_element(element));
        }
        editor.document.pages.push(CanvasPage {
            title: "Second page".into(),
            background: Paper::Grid,
            elements: Vec::new(),
        });
        editor.document.selected_page = 1;
        editor.document.zoom = 1.7;
        editor.document.pan = [0.2, -0.1];
        let snapshot = editor.notebook_snapshot();
        snapshot.validate().unwrap();
        let encoded = serde_json::to_vec(&snapshot).unwrap();
        let decoded: CanvasNotebook = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(snapshot, decoded);
        let mut reopened = CanvasEditor::default();
        reopened.apply_notebook(decoded);
        assert_eq!(reopened.notebook_snapshot(), snapshot);
        assert!(reopened.undo.is_empty());
        assert!(reopened.selected.is_none());
        assert!(!String::from_utf8(encoded).unwrap().contains("undo"));
    }

    #[test]
    fn validation_rejects_out_of_page_nonfinite_and_unknown_data() {
        for bounds in [
            [-0.1, 0.1, 0.2, 0.2],
            [0.9, 0.1, 0.2, 0.2],
            [0.1, 0.1, 0.0, 0.2],
            [f32::NAN, 0.1, 0.2, 0.2],
            [0.1, 0.1, f32::INFINITY, 0.2],
        ] {
            let mut element = text("Test");
            if let CanvasElement::Text { bounds: value, .. } = &mut element {
                *value = bounds;
            }
            assert!(
                element.validate().is_err(),
                "Accepted invalid bounds {bounds:?}"
            );
        }
        let mut document = CanvasNotebook::default();
        document.pages[0].elements.push(ink(0));
        assert!(document.validate().is_err());
        document.pages[0].elements = vec![ink(1)];
        if let CanvasElement::Ink { points, .. } = &mut document.pages[0].elements[0] {
            points[0].pressure = 1.1;
        }
        assert!(document.validate().is_err());
        let mut object = serde_json::to_value(CanvasNotebook::default()).unwrap();
        object["runtime_path"] = serde_json::json!("private");
        assert!(serde_json::from_value::<CanvasNotebook>(object).is_err());
        let mut document = CanvasNotebook {
            zoom: f32::NAN,
            ..CanvasNotebook::default()
        };
        assert!(document.validate().is_err());
        document.zoom = 1.0;
        document.selected_page = 10;
        assert!(document.validate().is_err());
    }

    #[test]
    fn document_limits_apply_across_pages_and_edits_roll_back() {
        let mut document = CanvasNotebook::default();
        document.pages[0].elements =
            vec![ink(MAX_STROKE_POINTS); MAX_POINTS / MAX_STROKE_POINTS + 1];
        assert!(document.validate().is_err());
        document.pages[0].elements =
            vec![text(&"a".repeat(16 * 1024)); MAX_TEXT_BYTES / (16 * 1024)];
        document.pages.push(CanvasPage {
            title: "Overflow".into(),
            background: Paper::Blank,
            elements: vec![text("a")],
        });
        assert!(document.validate().is_err());
        let mut editor = CanvasEditor::default();
        let original = editor.notebook_snapshot();
        assert!(!editor.change(|document| {
            document.pages[0]
                .elements
                .push(text(&"a".repeat(16 * 1024 + 1)))
        }));
        assert_eq!(editor.notebook_snapshot(), original);
        assert!(editor.undo.is_empty());
        assert!(!editor.notice.is_empty());
    }

    #[test]
    fn native_pointer_stroke_shape_eraser_and_undo_redo_work() {
        let mut session = Session::new();
        session.click("Ink");
        let page = session.page_rect();
        let a = screen_point(page, [0.2, 0.3]);
        let b = screen_point(page, [0.4, 0.4]);
        let c = screen_point(page, [0.6, 0.3]);
        session.pointer(a, Some(true));
        session.pointer(b, None);
        session.pointer(c, None);
        session.pointer(c, Some(false));
        assert_eq!(session.editor.page().elements.len(), 1);
        let CanvasElement::Ink { points, .. } = &session.editor.page().elements[0] else {
            panic!("Expected actual pointer ink");
        };
        assert!(points.len() >= 3);
        assert!((points[0].position[0] - 0.2).abs() < 0.01);
        session.click("Rectangle");
        let page = session.page_rect();
        let a = screen_point(page, [0.1, 0.6]);
        let b = screen_point(page, [0.4, 0.8]);
        session.pointer(a, Some(true));
        session.pointer(b, None);
        session.pointer(b, Some(false));
        assert!(matches!(
            session.editor.page().elements[1],
            CanvasElement::Rectangle { .. }
        ));
        session.click("Eraser");
        let target = screen_point(session.page_rect(), [0.4, 0.4]);
        session.pointer(target, Some(true));
        session.pointer(target, Some(false));
        assert_eq!(session.editor.page().elements.len(), 1);
        assert!(matches!(
            session.editor.page().elements[0],
            CanvasElement::Rectangle { .. }
        ));
        session.click("Undo");
        assert_eq!(session.editor.page().elements.len(), 2);
        session.click("Redo");
        assert_eq!(session.editor.page().elements.len(), 1);
        session.editor.notebook_snapshot().validate().unwrap();
    }

    #[test]
    fn native_text_coordinate_shape_and_ink_need_no_drag() {
        let mut session = Session::new();
        session.click("Text");
        session.replace("Object text", "n = N / Nₐ");
        session.replace("Object description", "Amount of substance in moles");
        session.click("Add text");
        assert!(
            matches!(&session.editor.page().elements[0], CanvasElement::Text { text, description, .. } if text == "n = N / Nₐ" && description.contains("moles"))
        );
        session.click("Ellipse");
        session.replace("Object X percent", "40");
        session.replace("Object width percent", "20");
        session.click("Add shape");
        assert!(
            matches!(&session.editor.page().elements[1], CanvasElement::Ellipse { bounds, .. } if (bounds[0] - 0.4).abs() < 0.001)
        );
        session.replace("Object X percent", "50");
        session.click("Apply object changes");
        assert!((session.editor.page().elements[1].bounds()[0] - 0.5).abs() < 0.001);
        session.click("Add ink point");
        session.replace("Object Y percent", "30");
        session.click("Add ink point");
        assert_eq!(session.editor.page().elements.len(), 2);
        session.click("Finish ink stroke");
        assert!(
            matches!(&session.editor.page().elements[2], CanvasElement::Ink { points, .. } if points.len() == 2)
        );
        assert!(session.editor.coordinate_points.is_empty());
        let encoded = serde_json::to_vec(&session.editor.notebook_snapshot()).unwrap();
        let saved: CanvasNotebook = serde_json::from_slice(&encoded).unwrap();
        saved.validate().unwrap();
        assert_eq!(saved.pages[0].elements.len(), 3);
    }

    #[test]
    fn native_page_delete_requires_confirmation_and_can_be_undone() {
        let mut session = Session::new();
        session.click("Add page");
        session.replace("Page title", "Chemistry lab");
        session.click("Grid");
        assert_eq!(session.editor.document.pages.len(), 2);
        assert_eq!(session.editor.page().title, "Chemistry lab");
        session.click("Delete page");
        assert!(session.editor.overlay_open());
        session.click("Cancel");
        assert_eq!(session.editor.document.pages.len(), 2);
        session.click("Delete page");
        session.click("Delete this page");
        assert_eq!(session.editor.document.pages.len(), 1);
        assert!(!session.editor.overlay_open());
        session.click("Undo");
        assert_eq!(session.editor.document.pages.len(), 2);
        assert_eq!(session.editor.page().title, "Chemistry lab");
        assert_eq!(session.editor.page().background, Paper::Grid);
    }

    #[test]
    fn native_coordinate_lines_support_horizontal_vertical_and_signed_offsets() {
        let mut session = Session::new();
        session.click("Line");
        session.replace("Object height percent", "0");
        session.click("Add shape");
        assert!(
            matches!(&session.editor.page().elements[0], CanvasElement::Line { start, end, .. } if start[1] == end[1] && start[0] != end[0])
        );
        session.replace("Object X percent", "50");
        session.replace("Object Y percent", "50");
        session.replace("Object width percent", "0");
        session.replace("Object height percent", "-25");
        session.click("Add shape");
        assert!(
            matches!(&session.editor.page().elements[1], CanvasElement::Line { start, end, .. } if start[0] == end[0] && (end[1] - 0.25).abs() < 0.001)
        );
        session.replace("Object width percent", "-20");
        session.click("Apply object changes");
        assert!(
            matches!(&session.editor.page().elements[1], CanvasElement::Line { start, end, .. } if end[0] < start[0] && end[1] < start[1])
        );
        session.editor.notebook_snapshot().validate().unwrap();
    }

    #[test]
    fn narrow_canvas_starts_visible_and_page_tools_toggle_with_keyboard() {
        let mut session = Session::with_view(
            Settings {
                scale: 1.25,
                ..Settings::default()
            },
            Vec2::new(360.0, 800.0),
        );
        let has_control = |session: &Session, label: &str| {
            session
                .output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some(label))
        };
        assert!(has_control(&session, "Page tools"));
        assert!(!has_control(&session, "Add page"));
        assert!(!has_control(&session, "Coordinates and elements"));
        let viewport = session.ctx.content_rect();
        let page = session.page_rect();
        assert!(
            page.top() < viewport.height() * 0.4,
            "Canvas starts too far down: {page:?} in {viewport:?}"
        );
        assert!(
            viewport.contains_rect(page),
            "The fitted page must be visible before opening tools: {page:?}"
        );
        assert!(page.height() > 200.0);
        session.action("Page tools", Action::Focus);
        for pressed in [true, false] {
            session.frame(vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }]);
        }
        session.frame(Vec::new());
        assert!(session.editor.page_tools_open);
        for label in [
            "Page title",
            "Add page",
            "Ink",
            "Coordinates and elements",
            "Copy page SVG",
        ] {
            assert!(
                has_control(&session, label),
                "Expanded Page tools is missing {label}"
            );
        }
        session.click("Ink");
        session.click("Page tools");
        assert!(!session.editor.page_tools_open);
        assert_eq!(session.editor.tool, Tool::Ink);
        assert!(!has_control(&session, "Add page"));
        assert!(
            session
                .output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Label
                    && node.value().is_some_and(
                        |value| value.contains("Page 1 of 1") && value.contains("Tool: Ink")
                    ))
        );
        assert!(
            session
                .ctx
                .content_rect()
                .contains_rect(session.page_rect())
        );
        assert!(session.editor.page().elements.is_empty());
    }

    #[test]
    fn pending_pointer_ink_is_captured_and_undo_history_is_bounded() {
        let settings = Settings::default();
        let mut editor = CanvasEditor {
            tool: Tool::Ink,
            ..CanvasEditor::default()
        };
        editor.start_pointer([0.2, 0.2], 0.2, &settings);
        editor.move_pointer([0.4, 0.4], 0.9, [0.2, 0.2], [0.0; 2]);
        let pending = editor.notebook_snapshot();
        pending.validate().unwrap();
        assert_eq!(pending.pages[0].elements.len(), 1);
        assert!(editor.page().elements.is_empty());
        assert!(editor.finish_gesture());
        for _ in 0..MAX_UNDO + 4 {
            assert!(editor.add_element(ink(1)));
        }
        assert!(editor.undo.len() <= MAX_UNDO);
        assert!(
            editor
                .undo
                .iter()
                .map(CanvasNotebook::memory_cost)
                .sum::<usize>()
                <= MAX_UNDO_BYTES
        );
    }

    #[test]
    fn svg_is_self_contained_escaped_and_has_bounded_text_layout() {
        let mut editor = CanvasEditor::default();
        let payload = "<script>&\"' 水 H₂O\nF = ma";
        assert!(editor.add_element(text(payload)));
        editor.document.pages[0].title = "A < B & \"notebook\"".into();
        editor.document.pages[0].background = Paper::Grid;
        let svg = String::from_utf8(editor.export_all_svg().unwrap()).unwrap();
        assert!(svg.starts_with("<?xml"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("&lt;script&gt;&amp;&quot;&apos; 水 H₂O"));
        assert!(svg.contains("A &lt; B &amp; &quot;notebook&quot;"));
        assert!(!svg.contains("<script>"));
        assert!(!svg.contains("href="));
        assert!(!svg.contains("https://"));
        assert_eq!(svg.matches("http://").count(), 1); // SVG namespace only.
        assert!(svg.contains("url(#text-clip-0-0)"));
        assert!(svg.len() < MAX_EXPORT_BYTES);
        assert_eq!(xml_escape("a\u{fffe}\u{ffff}\u{0}b"), "a���b");
        assert!(text("\u{fffe}").validate().is_err());
        assert!(text("\u{ffff}").validate().is_err());
        let mut too_large = String::new();
        assert!(
            svg_write(
                &mut too_large,
                format_args!("{}", "x".repeat(MAX_EXPORT_BYTES + 1))
            )
            .is_err()
        );
        eprintln!("CANVAS-SVG-XML-BEGIN\n{svg}\nCANVAS-SVG-XML-END");
    }
}
