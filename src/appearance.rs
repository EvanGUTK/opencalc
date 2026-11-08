use crate::accessibility::{AccessibilitySettings, ColorTheme};
use crate::{Settings, egui};
use egui::{Color32, FontFamily, FontId, Stroke, Vec2};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Palette {
    pub paper: Color32,
    pub panel: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub border: Color32,
    pub selected_text: Color32,
    pub dark: bool,
    pub contrast: bool,
}

impl Palette {
    fn new(theme: ColorTheme, contrast: bool) -> Self {
        let mut palette = match theme {
            ColorTheme::Light => Self {
                paper: Color32::WHITE,
                panel: crate::BG,
                ink: crate::INK,
                muted: crate::MUTED,
                accent: crate::TEAL,
                border: crate::LINE,
                selected_text: Color32::WHITE,
                dark: false,
                contrast,
            },
            ColorTheme::Warm => Self {
                paper: Color32::from_rgb(255, 250, 233),
                panel: Color32::from_rgb(247, 240, 216),
                ink: Color32::from_rgb(42, 37, 28),
                muted: Color32::from_rgb(91, 78, 52),
                accent: Color32::from_rgb(35, 89, 105),
                border: Color32::from_rgb(160, 147, 112),
                selected_text: Color32::WHITE,
                dark: false,
                contrast,
            },
            ColorTheme::Dark => Self {
                paper: Color32::from_rgb(29, 35, 43),
                panel: Color32::from_rgb(18, 23, 30),
                ink: Color32::from_rgb(244, 246, 250),
                muted: Color32::from_rgb(190, 201, 213),
                accent: Color32::from_rgb(111, 211, 185),
                border: Color32::from_rgb(147, 161, 179),
                selected_text: Color32::BLACK,
                dark: true,
                contrast,
            },
        };
        if contrast {
            palette.ink = if palette.dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            palette.muted = palette.ink;
            palette.border = palette.ink;
            palette.accent = palette.ink;
            palette.selected_text = if palette.dark {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            if palette.dark {
                palette.paper = Color32::BLACK;
                palette.panel = Color32::BLACK;
            } else if theme == ColorTheme::Light {
                palette.paper = Color32::WHITE;
                palette.panel = Color32::WHITE;
            }
        }
        palette
    }
}

fn palette_id() -> egui::Id {
    egui::Id::new("equal-appearance-palette")
}
fn options_id() -> egui::Id {
    egui::Id::new("equal-appearance-options")
}

pub(crate) fn palette(ctx: &egui::Context) -> Palette {
    ctx.data(|data| data.get_temp::<Palette>(palette_id()))
        .unwrap_or_else(|| Palette::new(ColorTheme::Light, false))
}

pub(crate) fn font_scale(ctx: &egui::Context) -> f32 {
    ctx.data(|data| data.get_temp::<AccessibilitySettings>(options_id()))
        .map_or(1.0, |options| {
            finite_range(options.font_scale, 0.9, 1.8, 1.0)
        })
}

pub(crate) fn paragraph(ctx: &egui::Context, text: impl Into<String>) -> egui::RichText {
    let options = ctx.data(|data| data.get_temp::<AccessibilitySettings>(options_id()));
    let spacing = options.map_or(1.0, |options| {
        finite_range(options.line_spacing, 1.0, 2.0, 1.0)
    });
    let text = egui::RichText::new(text);
    if spacing > 1.0 {
        text.line_height(Some(18.0 * font_scale(ctx) * spacing))
    } else {
        text
    }
}

fn finite_range(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

pub(crate) fn apply(ctx: &egui::Context, settings: &Settings) {
    let palette = Palette::new(settings.access.theme, settings.contrast);
    let mut style = egui::Style {
        visuals: if palette.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        },
        ..Default::default()
    };
    let font_scale = finite_range(settings.access.font_scale, 0.9, 1.8, 1.0);
    let line_spacing = finite_range(settings.access.line_spacing, 1.0, 2.0, 1.0);
    style.visuals.panel_fill = palette.panel;
    style.visuals.window_fill = palette.paper;
    style.visuals.extreme_bg_color = palette.paper;
    style.visuals.faint_bg_color = palette.panel;
    style.visuals.code_bg_color = palette.panel;
    style.visuals.override_text_color = None;
    style.visuals.hyperlink_color = palette.accent;
    style.visuals.window_stroke = Stroke::new(
        if palette.contrast { 2.0_f32 } else { 1.0_f32 },
        palette.border,
    );
    style.visuals.widgets.noninteractive.fg_stroke.color = palette.ink;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, palette.border);
    let inactive = if palette.contrast {
        palette.paper
    } else {
        match settings.access.theme {
            ColorTheme::Light => Color32::from_rgb(210, 223, 209),
            ColorTheme::Warm => Color32::from_rgb(235, 225, 195),
            ColorTheme::Dark => Color32::from_rgb(46, 57, 69),
        }
    };
    let hovered = if palette.contrast {
        palette.paper
    } else {
        match settings.access.theme {
            ColorTheme::Light => Color32::from_rgb(231, 240, 231),
            ColorTheme::Warm => Color32::from_rgb(244, 230, 190),
            ColorTheme::Dark => Color32::from_rgb(51, 68, 81),
        }
    };
    style.visuals.widgets.inactive.bg_fill = inactive;
    style.visuals.widgets.inactive.weak_bg_fill = palette.paper;
    style.visuals.widgets.inactive.fg_stroke.color = palette.ink;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(
        if palette.contrast { 2.0_f32 } else { 1.0_f32 },
        palette.border,
    );
    style.visuals.widgets.hovered.bg_fill = hovered;
    style.visuals.widgets.hovered.weak_bg_fill = hovered;
    style.visuals.widgets.hovered.fg_stroke.color = palette.ink;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0_f32, palette.accent);
    style.visuals.widgets.active.fg_stroke.color = palette.ink;
    style.visuals.widgets.active.bg_stroke = Stroke::new(3.0_f32, palette.accent);
    style.visuals.widgets.open.fg_stroke.color = palette.ink;
    style.visuals.selection.bg_fill = palette.accent;
    style.visuals.selection.stroke = Stroke::new(1.5_f32, palette.selected_text);
    style.visuals.error_fg_color = if palette.dark {
        Color32::from_rgb(255, 177, 163)
    } else {
        Color32::from_rgb(155, 35, 25)
    };
    style.visuals.warn_fg_color = if palette.dark {
        Color32::from_rgb(255, 219, 142)
    } else {
        Color32::from_rgb(117, 76, 0)
    };
    let family = if settings.readable {
        FontFamily::Monospace
    } else {
        FontFamily::Proportional
    };
    for (kind, size) in [
        (egui::TextStyle::Body, 15.0),
        (egui::TextStyle::Button, 15.0),
        (egui::TextStyle::Small, 12.0),
        (egui::TextStyle::Heading, 22.0),
        (egui::TextStyle::Monospace, 15.0),
    ] {
        let mono = kind == egui::TextStyle::Monospace;
        style.text_styles.insert(
            kind,
            FontId::new(
                size * font_scale,
                if mono {
                    FontFamily::Monospace
                } else {
                    family.clone()
                },
            ),
        );
    }
    style.spacing.item_spacing = Vec2::new(
        if settings.readable { 10.0_f32 } else { 8.0_f32 },
        8.0 * line_spacing,
    );
    style.spacing.button_padding = Vec2::new(10.0, 7.0);
    style.spacing.interact_size.y = if settings.access.large_controls {
        44.0_f32.max(30.0 * font_scale)
    } else {
        30.0 * font_scale
    };
    if settings.access.reduced_motion {
        style.animation_time = 0.0;
    }
    ctx.set_style(style);
    ctx.set_zoom_factor(finite_range(settings.scale, 0.9, 4.0, 1.0));
    ctx.data_mut(|data| {
        data.insert_temp(palette_id(), palette);
        data.insert_temp(options_id(), settings.access.clone());
        data.insert_temp(egui::Id::new("handheld-readable"), settings.readable);
        data.insert_temp(egui::Id::new("handheld-contrast"), settings.contrast);
    });
}

pub(crate) fn reading_guide(ctx: &egui::Context, enabled: bool, guide_y: f32) {
    if !enabled {
        return;
    }
    let screen = ctx.content_rect();
    let y = if guide_y.is_finite() && guide_y > 0.0 {
        guide_y
    } else {
        ctx.pointer_hover_pos()
            .map_or(screen.center().y, |point| point.y)
    }
    .clamp(screen.top(), screen.bottom());
    let palette = palette(ctx);
    let height = 28.0 * font_scale(ctx);
    let rect = egui::Rect::from_min_max(
        egui::pos2(screen.left(), y - height / 2.0),
        egui::pos2(screen.right(), y + height / 2.0),
    );
    // Only an outline is drawn: text remains unobscured and the guide never takes input.
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("reading-guide"),
    ))
    .rect_stroke(
        rect.intersect(screen),
        0.0_f32,
        Stroke::new(1.5_f32, palette.accent),
        egui::StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn luminance(color: Color32) -> f64 {
        let linear = |channel: u8| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    }
    fn contrast(first: Color32, second: Color32) -> f64 {
        let first = luminance(first);
        let second = luminance(second);
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }
    #[test]
    fn all_palette_text_and_selected_text_have_readable_contrast() {
        for theme in [ColorTheme::Light, ColorTheme::Warm, ColorTheme::Dark] {
            for high_contrast in [false, true] {
                let palette = Palette::new(theme, high_contrast);
                assert!(contrast(palette.ink, palette.paper) >= 4.5, "{theme:?}");
                assert!(contrast(palette.muted, palette.paper) >= 4.5, "{theme:?}");
                assert!(
                    contrast(palette.selected_text, palette.accent) >= 4.5,
                    "{theme:?}"
                );
            }
        }
    }
    #[test]
    fn typography_targets_and_reduced_motion_apply_without_changing_zoom() {
        let ctx = egui::Context::default();
        let mut settings = Settings {
            scale: 1.25,
            ..Default::default()
        };
        settings.access.font_scale = 1.5;
        settings.access.line_spacing = 1.5;
        settings.access.large_controls = true;
        settings.access.theme = ColorTheme::Dark;
        apply(&ctx, &settings);
        let style = ctx.style();
        assert_eq!(style.text_styles[&egui::TextStyle::Body].size, 22.5);
        assert_eq!(style.spacing.item_spacing.y, 12.0);
        assert!(style.spacing.interact_size.y >= 44.0);
        assert_eq!(style.animation_time, 0.0);
        assert!(palette(&ctx).dark);
    }

    #[test]
    fn reading_guide_does_not_take_focus_or_block_pointer_activation() {
        let ctx = egui::Context::default();
        let mut rect = egui::Rect::NOTHING;
        let mut activations = 0;
        for phase in 0..3 {
            let events = if phase == 0 {
                vec![]
            } else {
                let pos = rect.center();
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: phase == 1,
                        modifiers: egui::Modifiers::NONE,
                    },
                ]
            };
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(320.0, 240.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let response = ui.button("Underlying control");
                        rect = response.rect;
                        if response.clicked() {
                            activations += 1;
                        }
                    });
                    reading_guide(ctx, true, rect.center().y);
                },
            );
        }
        assert_eq!(activations, 1);
    }
}
