use crate::*;
use egui::Stroke;

const SIDEBAR_BREAKPOINT: f32 = 1080.0;
const SIDEBAR_WIDTH: f32 = 286.0;
const MIN_ZOOM_PERCENT: u32 = 90;
const MAX_ZOOM_PERCENT: u32 = 400;
const ZOOM_STEP_PERCENT: u32 = 5;
const ZOOM_PRESETS: [u32; 10] = [90, 100, 110, 125, 150, 175, 200, 250, 300, 400];

#[derive(Clone)]
struct ZoomDraft {
    text: String,
    applied_percent: u32,
    error: Option<String>,
}

impl ZoomDraft {
    fn new(percent: u32) -> Self {
        Self {
            text: percent.to_string(),
            applied_percent: percent,
            error: None,
        }
    }
}

fn uses_sidebar(width: f32) -> bool {
    width >= SIDEBAR_BREAKPOINT
}

impl EqualApp {
    pub(crate) fn open_access_settings(&mut self, ctx: &egui::Context, origin: Option<egui::Id>) {
        let origin = origin
            .or_else(|| ctx.memory(|memory| memory.focused()))
            .or_else(|| {
                ctx.data(|data| {
                    data.get_temp::<egui::Id>(egui::Id::new("settings-current-trigger"))
                })
            });
        ctx.data_mut(|data| {
            let key = egui::Id::new("settings-return-focus");
            if let Some(origin) = origin {
                data.insert_temp(key, origin);
            } else {
                data.remove::<egui::Id>(key);
            }
            data.insert_temp(egui::Id::new("settings-focus-requested"), true);
            data.remove::<bool>(egui::Id::new("settings-dialog-visible"));
        });
        self.show_settings = true;
    }

    pub fn header(&mut self, ctx: &egui::Context) {
        let palette = crate::appearance::palette(ctx);
        let tiny = ctx.content_rect().width() < 180.0;
        let minimal = ctx.content_rect().width() < 140.0;
        let background_disabled = self.access_overlay_open();
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(palette.paper)
                    .inner_margin(if tiny { 4 } else { 8 })
                    .stroke(Stroke::new(1.0_f32, palette.border)),
            )
            .show(ctx, |ui| {
                if background_disabled {
                    ui.disable();
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = if minimal { 4.0 } else { 7.0 };
                    if tiny {
                        ui.spacing_mut().button_padding.x = if minimal { 2.0 } else { 4.0 };
                    }
                    if !minimal {
                        ui.label(
                            R::new("OpenCalc")
                                .size(if tiny { 16.0 } else { 20.0 })
                                .strong()
                                .color(palette.ink),
                        );
                        let old_level = self.settings.level;
                        egui::ComboBox::from_id_salt("header-learning-level")
                            .selected_text(if tiny && self.settings.level == 3 {
                                "Adv"
                            } else {
                                LEVELS[self.settings.level]
                            })
                            .width(if tiny { 40.0 } else { 76.0 })
                            .truncate()
                            .show_ui(ui, |ui| {
                                for (i, level) in LEVELS.iter().enumerate() {
                                    ui.selectable_value(&mut self.settings.level, i, *level);
                                }
                            })
                            .response
                            .widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::ComboBox,
                                    true,
                                    "Learning level",
                                )
                            });
                        if old_level != self.settings.level {
                            self.apply_learning_level();
                        }
                    }
                    let settings = ui.button(if tiny { "Access" } else { "Settings" });
                    settings.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Settings")
                    });
                    ctx.data_mut(|data| {
                        data.insert_temp(egui::Id::new("settings-current-trigger"), settings.id)
                    });
                    if settings.clicked() {
                        self.open_access_settings(ctx, Some(settings.id));
                    }
                    ui.menu_button("More", |ui| {
                        ui.set_max_width((ctx.content_rect().width() - 20.0).clamp(40.0, 240.0));
                        if ui.add(egui::Button::new("Quick guide").wrap()).clicked() {
                            self.show_help = true;
                            ui.close();
                        }
                        if ui
                            .add(egui::Button::new("Keyboard shortcuts").wrap())
                            .clicked()
                        {
                            self.a11y.show_shortcuts = true;
                            ui.close();
                        }
                        if ui
                            .add(egui::Button::new("Math word guide").wrap())
                            .clicked()
                        {
                            self.a11y.show_glossary = true;
                            ui.close();
                        }
                        if ui
                            .add(egui::Button::new("TI picture guide").wrap())
                            .clicked()
                        {
                            self.learning.show_references = true;
                            ui.close();
                        }
                        if ui.add(egui::Button::new("Handheld view").wrap()).clicked() {
                            self.show_workspace = false;
                            self.tab = Tab::Calculator;
                            ui.close();
                        }
                    });
                });
            });
    }

    fn apply_learning_level(&mut self) {
        self.set_device_screen(device::DeviceScreen::Home);
        self.device.alpha = false;
        self.second = false;
        if self.settings.level < 3 {
            self.surface = false;
        }
        if self.tab == Tab::Matrix && self.settings.level < 3
            || self.tab == Tab::Graph && self.settings.level < 2
        {
            self.tab = Tab::Calculator;
        }
    }

    pub fn sidebar(&mut self, ctx: &egui::Context) {
        if !uses_sidebar(ctx.content_rect().width()) || self.settings.access.focus_mode {
            return;
        }
        let palette = crate::appearance::palette(ctx);
        let background_disabled = self.access_overlay_open();
        egui::SidePanel::right("settings")
            .exact_width(SIDEBAR_WIDTH)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(palette.paper)
                    .inner_margin(16)
                    .stroke(Stroke::new(1.0_f32, palette.border)),
            )
            .show(ctx, |ui| {
                if background_disabled {
                    ui.disable();
                }
                egui::ScrollArea::vertical()
                    .id_salt("sidebar-settings-scroll")
                    .show(ui, |ui| self.settings_contents(ui));
            });
    }

    pub(crate) fn settings_contents(&mut self, ui: &mut egui::Ui) {
        crate::accessibility_panel::begin_settings_column(ui);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        let control_height = if self.settings.access.large_controls {
            44.0
        } else {
            32.0
        };
        ui.spacing_mut().interact_size.y = ui.spacing().interact_size.y.max(control_height);
        if ui.available_width() < 180.0 {
            ui.spacing_mut().button_padding.x = 4.0;
        }
        ui.strong("Learning level");
        if ui.available_width() < 240.0 {
            let old = self.settings.level;
            let level_name = if self.settings.level == 3 && ui.available_width() < 100.0 {
                "Adv"
            } else {
                LEVELS[self.settings.level]
            };
            crate::accessibility_panel::access_combo(
                ui,
                "settings-learning-level",
                110.0,
                level_name,
                |ui| {
                    for (index, level) in LEVELS.iter().enumerate() {
                        crate::accessibility_panel::access_value(
                            ui,
                            &mut self.settings.level,
                            index,
                            *level,
                        );
                    }
                },
            )
            .response
            .widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, "Learning level")
            });
            if old != self.settings.level {
                self.apply_learning_level();
            }
        } else {
            ui.horizontal_wrapped(|ui| {
                for (i, level) in LEVELS.iter().enumerate() {
                    if crate::accessibility_panel::access_value(
                        ui,
                        &mut self.settings.level,
                        i,
                        *level,
                    )
                    .clicked()
                    {
                        self.apply_learning_level();
                    }
                }
            });
        }
        ui.label(DESCS[self.settings.level]);
        ui.separator();
        self.zoom_controls(ui);
        ui.separator();
        self.accessibility_controls(ui);
        ui.separator();
        ui.strong("Calculator settings");
        ui.label("Angle unit");
        ui.horizontal_wrapped(|ui| {
            crate::accessibility_panel::access_value(
                ui,
                &mut self.settings.degrees,
                true,
                "Degrees",
            );
            crate::accessibility_panel::access_value(
                ui,
                &mut self.settings.degrees,
                false,
                "Radians",
            );
        });
        ui.label("Decimal places");
        crate::accessibility_panel::access_combo(
            ui,
            "precision",
            70.0,
            self.settings.precision.to_string(),
            |ui| {
                for places in [2, 4, 6, 8, 10] {
                    crate::accessibility_panel::access_value(
                        ui,
                        &mut self.settings.precision,
                        places,
                        places.to_string(),
                    );
                }
            },
        )
        .response
        .widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, "Decimal places")
        });
        ui.separator();
        if crate::accessibility_panel::access_button(ui, "Assistive technology check").clicked() {
            self.learning.show_access = true;
            self.show_settings = false;
        }
        if crate::accessibility_panel::access_button(ui, "Reset preferences").clicked() {
            self.settings = Settings::default();
            self.apply_learning_level();
        }
    }

    fn zoom_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Interface zoom");
        let draft_id = egui::Id::new("interface-zoom-draft");
        let current = ((self.settings.scale * 100.0).round() as u32)
            .clamp(MIN_ZOOM_PERCENT, MAX_ZOOM_PERCENT);
        let mut percent = current;
        let mut draft = ui
            .ctx()
            .data(|data| data.get_temp::<ZoomDraft>(draft_id))
            .unwrap_or_else(|| ZoomDraft::new(current));
        if draft.applied_percent != current {
            draft = ZoomDraft::new(current);
        }
        ui.horizontal_wrapped(|ui| {
            let smaller = crate::accessibility_panel::access_button_enabled(
                ui,
                "-",
                percent > MIN_ZOOM_PERCENT,
            );
            smaller.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    percent > MIN_ZOOM_PERCENT,
                    "Decrease interface zoom",
                )
            });
            if smaller.on_hover_text("Smaller by 5%").clicked() {
                percent = percent
                    .saturating_sub(ZOOM_STEP_PERCENT)
                    .max(MIN_ZOOM_PERCENT);
            }
            crate::accessibility_panel::access_combo(
                ui,
                "zoom-presets",
                76.0,
                format!("{percent}%"),
                |ui| {
                    for preset in ZOOM_PRESETS {
                        crate::accessibility_panel::access_value(
                            ui,
                            &mut percent,
                            preset,
                            format!("{preset}%"),
                        );
                    }
                },
            )
            .response
            .widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::ComboBox,
                    true,
                    "Interface zoom presets",
                )
            });
            let larger = crate::accessibility_panel::access_button_enabled(
                ui,
                "+",
                percent < MAX_ZOOM_PERCENT,
            );
            larger.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    percent < MAX_ZOOM_PERCENT,
                    "Increase interface zoom",
                )
            });
            if larger.on_hover_text("Larger by 5%").clicked() {
                percent = (percent + ZOOM_STEP_PERCENT).min(MAX_ZOOM_PERCENT);
            }
            let reset = crate::accessibility_panel::access_button(ui, "Reset");
            reset.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    true,
                    "Reset interface zoom to 100%",
                )
            });
            if reset.clicked() {
                percent = 100;
                draft = ZoomDraft::new(100);
            }
        });
        if percent != current {
            self.settings.scale = percent as f32 / 100.0;
            draft = ZoomDraft::new(percent);
        }
        let mut apply = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Custom");
            let response = ui.add(
                egui::TextEdit::singleline(&mut draft.text)
                    .desired_width(ui.available_width().clamp(26.0, 54.0))
                    .char_limit(6)
                    .id_salt("custom-zoom-percent"),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::TextEdit,
                    true,
                    "Custom zoom percentage",
                )
            });
            if response.changed() {
                draft.error = None;
            }
            if (response.has_focus() || response.lost_focus())
                && ui.input(|input| input.key_pressed(egui::Key::Enter))
            {
                apply = true;
            }
            ui.label("%");
            let button = crate::accessibility_panel::access_button(ui, "Apply");
            button.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Apply interface zoom")
            });
            apply |= button.clicked();
        });
        // Commit after editing, so magnification cannot move the field under the pointer.
        if apply {
            let value = draft
                .text
                .trim()
                .trim_end_matches('%')
                .trim()
                .parse::<u32>();
            match value {
                Ok(value) if (MIN_ZOOM_PERCENT..=MAX_ZOOM_PERCENT).contains(&value) => {
                    self.settings.scale = value as f32 / 100.0;
                    draft = ZoomDraft::new(value);
                }
                _ => draft.error = Some("Enter a whole percentage from 90 to 400.".into()),
            }
        }
        if let Some(error) = &draft.error {
            let response = ui.label(error);
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_role(egui::accesskit::Role::Status);
                node.set_label(error.as_str());
                node.clear_value();
                node.set_live(egui::accesskit::Live::Polite);
            });
        }
        ui.ctx().data_mut(|data| data.insert_temp(draft_id, draft));
    }

    pub(crate) fn settings_dialog(&mut self, ctx: &egui::Context) {
        let seen_id = egui::Id::new("settings-dialog-visible");
        if !self.show_settings {
            ctx.data_mut(|data| data.remove::<bool>(seen_id));
            return;
        }
        let first_frame = !ctx.data(|data| data.get_temp::<bool>(seen_id).unwrap_or(false));
        if first_frame
            && !ctx.data(|data| {
                data.get_temp::<bool>(egui::Id::new("settings-focus-requested"))
                    .unwrap_or(false)
            })
        {
            self.open_access_settings(ctx, None);
        }
        ctx.data_mut(|data| data.remove::<bool>(egui::Id::new("settings-focus-requested")));
        ctx.data_mut(|data| data.insert_temp(seen_id, true));
        let wheel_id = egui::Id::new("settings-wheel-ready");
        let wheel_ready = !first_frame
            && (ctx.data(|data| data.get_temp::<bool>(wheel_id).unwrap_or(false))
                || ctx.input(|input| {
                    input.smooth_scroll_delta == egui::Vec2::ZERO
                        || input
                            .events
                            .iter()
                            .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
                }));
        ctx.data_mut(|data| data.insert_temp(wheel_id, wheel_ready));
        let rect = ctx.content_rect();
        let width = (rect.width() - 32.0).clamp(36.0, 340.0);
        let response = egui::Modal::new(egui::Id::new("settings-dialog"))
            .frame(egui::Frame::popup(&ctx.style()).inner_margin(12))
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.set_max_width(width);
                #[cfg(test)]
                let mut close_rect = egui::Rect::NOTHING;
                let header = ui.horizontal_wrapped(|ui| {
                    if width < 100.0 {
                        ui.strong("Setup");
                    } else {
                        ui.heading("Settings");
                    }
                    let close = crate::accessibility_panel::access_button(ui, "Close");
                    #[cfg(test)]
                    {
                        close_rect = close.rect;
                    }
                    close.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Close settings")
                    });
                    if first_frame {
                        close.request_focus();
                    }
                    if close.clicked() {
                        self.show_settings = false;
                    }
                });
                let outer_margin = if rect.height() < 240.0 { 8.0 } else { 24.0 };
                let body_height = (rect.height()
                    - header.response.rect.height()
                    - ui.spacing().item_spacing.y
                    - 24.0
                    - outer_margin)
                    .max(16.0);
                let scroll_id = ui.make_persistent_id(egui::Id::new("dialog-settings-scroll"));
                if first_frame {
                    egui::scroll_area::State::default().store(ctx, scroll_id);
                }
                let _scroll = egui::ScrollArea::vertical()
                    .id_salt("dialog-settings-scroll")
                    // egui keeps a smoothed wheel tail in InputState, outside
                    // ScrollArea state. Do not carry an earlier scroll into a
                    // newly opened dialog; a new gesture enables it immediately.
                    .scroll_source(egui::scroll_area::ScrollSource {
                        mouse_wheel: wheel_ready,
                        ..Default::default()
                    })
                    .min_scrolled_height(body_height)
                    .max_height(body_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.settings_contents(ui));
                #[cfg(test)]
                ctx.data_mut(|data| {
                    data.insert_temp(
                        egui::Id::new("settings-scroll-geometry"),
                        (
                            close_rect,
                            _scroll.inner_rect,
                            _scroll.state.offset,
                            _scroll.content_size,
                        ),
                    )
                });
            });
        if response.should_close() {
            self.show_settings = false;
        }
        if !self.show_settings {
            ctx.data_mut(|data| data.remove::<bool>(seen_id));
            if let Some(id) =
                ctx.data(|data| data.get_temp::<egui::Id>(egui::Id::new("settings-return-focus")))
            {
                let current = if ctx.read_response(id).is_some() {
                    Some(id)
                } else {
                    ctx.data(|data| {
                        data.get_temp::<egui::Id>(egui::Id::new("settings-current-trigger"))
                    })
                };
                if let Some(current) = current {
                    ctx.memory_mut(|memory| memory.request_focus(current));
                }
            }
        }
    }

    pub fn footer(&self, ctx: &egui::Context) {
        let captions = self.settings.access.captions && !self.a11y.latest_announcement.is_empty();
        let help_visible =
            self.settings.access.show_help_text && ctx.content_rect().height() >= 500.0;
        if !captions && !help_visible {
            return;
        }
        let palette = crate::appearance::palette(ctx);
        egui::TopBottomPanel::bottom("footer")
            .frame(egui::Frame::new().fill(palette.paper).inner_margin(6))
            .show(ctx, |ui| {
                if captions {
                    ui.strong("Latest announcement");
                    egui::ScrollArea::vertical()
                        .id_salt("announcement-caption-scroll")
                        .max_height((ctx.content_rect().height() * 0.2).clamp(20.0, 64.0))
                        .show(ui, |ui| {
                            let caption = ui.label(&self.a11y.latest_announcement);
                            ui.ctx().accesskit_node_builder(caption.id, |node| {
                                node.set_role(egui::accesskit::Role::Status);
                                node.set_label(self.a11y.latest_announcement.as_str());
                                node.clear_value();
                                node.set_live(egui::accesskit::Live::Polite);
                            });
                        });
                }
                if !help_visible {
                    return;
                }
                let help = match self.tab {
                    Tab::Practice => "Enter: check answer / Tab: next control",
                    Tab::Graph => "Arrow keys: trace when the graph is focused",
                    Tab::Toolkit => "Enter: calculate tool / Tab: next control",
                    Tab::Tutor | Tab::Matrix => "Tab: next control / Shift+Tab: previous control",
                    Tab::Calculator => "Enter: calculate / Esc: clear",
                };
                ui.label(R::new(help).small());
            });
    }

    pub fn content(&mut self, ctx: &egui::Context) {
        let palette = crate::appearance::palette(ctx);
        let background_disabled = self.access_overlay_open();
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette.panel).inner_margin(8))
            .show(ctx, |ui| {
                if background_disabled {
                    ui.disable();
                }
                ui.spacing_mut().item_spacing.y = 6.0;
                self.workspace_navigation(ui);
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("workspace-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.tab {
                        Tab::Calculator => self.calculator_workspace(ui),
                        Tab::Graph => self.graph_workspace(ui),
                        Tab::Matrix => self.matrix_workspace(ui),
                        Tab::Tutor => self.tutor_workspace(ui),
                        Tab::Toolkit => self.toolkit_workspace(ui),
                        Tab::Practice => self.practice_workspace(ui),
                    });
            });
    }

    fn workspace_enabled(&self, tab: Tab) -> bool {
        matches!(
            tab,
            Tab::Calculator | Tab::Tutor | Tab::Toolkit | Tab::Practice
        ) || self.settings.level >= 2 && (tab != Tab::Matrix || self.settings.level == 3)
    }

    fn workspace_navigation(&mut self, ui: &mut egui::Ui) {
        let palette = crate::appearance::palette(ui.ctx());
        let compact = ui.available_width() < 330.0;
        let focus_hidden = self.settings.access.focus_mode && !self.a11y.focus_navigation;
        const TABS: [(Tab, &str, &str); 6] = [
            (Tab::Calculator, "Calculator", "Calc"),
            (Tab::Graph, "Graph", "Graph"),
            (Tab::Matrix, "Matrices", "Matrix"),
            (Tab::Tutor, "Learn", "Learn"),
            (Tab::Toolkit, "Tools", "Tools"),
            (Tab::Practice, "Practice", "Study"),
        ];
        if focus_hidden || compact && !self.a11y.focus_navigation {
            let current = TABS
                .iter()
                .find(|(tab, _, _)| *tab == self.tab)
                .map(|(_, full, short)| if compact { *short } else { *full })
                .unwrap_or("Calc");
            let available = ui.available_width();
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if available < 180.0 {
                    ui.spacing_mut().button_padding.x = 4.0;
                }
                let history_label = if available < 180.0 { "Log" } else { "History" };
                let history_font = egui::TextStyle::Button.resolve(ui.style());
                let history_width = ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(history_label.into(), history_font, palette.ink)
                        .size()
                        .x
                }) + ui.spacing().button_padding.x * 2.0;
                let width =
                    (available - history_width - ui.spacing().item_spacing.x).clamp(28.0, 150.0);
                let response = ui
                    .allocate_ui_with_layout(
                        egui::vec2(width, ui.spacing().interact_size.y),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::ComboBox::from_id_salt("workspace-picker")
                                .width(width)
                                .truncate()
                                .selected_text(current)
                                .show_ui(ui, |ui| {
                                    ui.set_max_width(width);
                                    for (tab, full, short) in TABS {
                                        let enabled = self.workspace_enabled(tab);
                                        ui.add_enabled_ui(enabled, |ui| {
                                            let label = if compact { short } else { full };
                                            let selected = self.tab == tab;
                                            let response = ui.selectable_label(
                                                selected,
                                                R::new(label).color(if selected {
                                                    palette.selected_text
                                                } else {
                                                    palette.ink
                                                }),
                                            );
                                            response.widget_info(|| {
                                                egui::WidgetInfo::labeled(
                                                    egui::WidgetType::SelectableLabel,
                                                    enabled,
                                                    full,
                                                )
                                            });
                                            if response.clicked() {
                                                self.tab = tab;
                                            }
                                        });
                                    }
                                })
                        },
                    )
                    .inner
                    .response;
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, "Workspace")
                });
                let history = ui.button(history_label);
                history.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "History")
                });
                if history.clicked() {
                    self.show_history = true;
                }
            });
            if self.settings.access.focus_mode
                && ui.ctx().content_rect().height() >= 360.0
                && ui
                    .add(egui::Button::new("Show navigation").wrap())
                    .clicked()
            {
                self.a11y.focus_navigation = true;
            }
            return;
        }
        if self.settings.access.focus_mode
            && ui
                .add(egui::Button::new("Hide navigation").wrap())
                .clicked()
        {
            self.a11y.focus_navigation = false;
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if compact {
                ui.spacing_mut().button_padding.x = 4.0;
            }
            for (tab, full, short) in TABS {
                let enabled = self.workspace_enabled(tab);
                let selected = self.tab == tab;
                let button = ui.add_enabled(
                    enabled,
                    egui::Button::new(R::new(if compact { short } else { full }).strong().color(
                        if selected {
                            palette.selected_text
                        } else {
                            palette.ink
                        },
                    ))
                    .selected(selected)
                    .wrap(),
                );
                button.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, full)
                });
                if button
                    .on_hover_text(if enabled {
                        "Open workspace"
                    } else if tab == Tab::Matrix {
                        "Select Advanced for matrices"
                    } else {
                        "Select 9-12 or Advanced for graphing"
                    })
                    .clicked()
                {
                    self.tab = tab;
                }
            }
            if ui.button("History").clicked() {
                self.show_history = true;
            }
        });
    }

    fn calculator_workspace(&mut self, ui: &mut egui::Ui) {
        self.calculator(ui);
        ui.horizontal_wrapped(|ui| {
            let explain = ui.button("Explain");
            explain.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    true,
                    "Explain this calculation",
                )
            });
            if explain.clicked() {
                self.tutor.problem = self.expression.clone();
                self.tutor
                    .solve_local(self.settings.level, self.settings.degrees);
                self.tab = Tab::Tutor;
            }
            let read = ui.button("Read answer");
            read.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Read answer aloud")
            });
            if read.clicked() {
                self.speak(&self.result.clone());
            }
            if ui.button("Stop audio").clicked() {
                self.stop_audio();
            }
            if ui.button("Copy answer").clicked() {
                ui.ctx().copy_text(self.result.clone());
                self.message = "Answer copied.".into();
            }
        });
        if !self.message.is_empty() {
            ui.label(&self.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_frame(
        app: &mut EqualApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        app.theme(ctx);
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 6000.0) / ctx.zoom_factor(),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.settings_contents(ui));
            },
        )
    }

    #[test]
    fn zoom_percentage_editor_preserves_intermediate_values() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        settings_frame(&mut app, &ctx, vec![]);
        let output = settings_frame(&mut app, &ctx, vec![]);
        let (_, field) = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Custom zoom percentage"))
            .expect("zoom percentage field");
        let bounds = field.bounds().unwrap();
        let pos = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        for pressed in [true, false] {
            settings_frame(
                &mut app,
                &ctx,
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
        settings_frame(
            &mut app,
            &ctx,
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
                egui::Event::Text("125".into()),
            ],
        );
        assert_eq!(
            app.settings.scale, 1.0,
            "Typing must not resize the interface before applying"
        );
        settings_frame(
            &mut app,
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(
            (app.settings.scale - 1.25).abs() < 0.001,
            "125% must remain an intermediate size, got {}%",
            app.settings.scale * 100.0
        );
    }

    fn click_setting(app: &mut EqualApp, ctx: &egui::Context, label: &str) {
        let output = settings_frame(app, ctx, vec![]);
        let node = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("Missing {label}"));
        let bounds = node.1.bounds().unwrap();
        let pos = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        for pressed in [true, false] {
            settings_frame(
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

    fn type_zoom(app: &mut EqualApp, ctx: &egui::Context, text: &str) {
        let previous = app.settings.scale;
        click_setting(app, ctx, "Custom zoom percentage");
        settings_frame(
            app,
            ctx,
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
        assert_eq!(
            app.settings.scale, previous,
            "Typing must keep the layout stable"
        );
        settings_frame(
            app,
            ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }

    #[test]
    fn zoom_buttons_and_presets_reach_intermediate_sizes() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        settings_frame(&mut app, &ctx, vec![]);
        type_zoom(&mut app, &ctx, "117");
        assert!((app.settings.scale - 1.17).abs() < 0.001);
        for (label, expected) in [
            ("Increase interface zoom", 1.22),
            ("Decrease interface zoom", 1.17),
            ("Reset interface zoom to 100%", 1.0),
        ] {
            click_setting(&mut app, &ctx, label);
            assert!((app.settings.scale - expected).abs() < 0.001, "{label}");
        }
        click_setting(&mut app, &ctx, "Interface zoom presets");
        click_setting(&mut app, &ctx, "150%");
        assert_eq!(app.settings.scale, 1.5);
    }

    #[test]
    fn invalid_zoom_leaves_size_unchanged_and_percent_suffix_is_accepted() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        settings_frame(&mut app, &ctx, vec![]);
        type_zoom(&mut app, &ctx, "500");
        assert_eq!(app.settings.scale, 1.0);
        let output = settings_frame(&mut app, &ctx, vec![]);
        assert!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Status
                    && node.label() == Some("Enter a whole percentage from 90 to 400."))
        );
        type_zoom(&mut app, &ctx, "90%");
        assert!((app.settings.scale - 0.9).abs() < 0.001);
        type_zoom(&mut app, &ctx, "200");
        assert_eq!(app.settings.scale, 2.0);
        type_zoom(&mut app, &ctx, "300");
        assert_eq!(app.settings.scale, 3.0);
        type_zoom(&mut app, &ctx, "400");
        assert_eq!(app.settings.scale, 4.0);
    }

    #[test]
    fn navigation_wraps_with_distinct_history_and_tools_controls() {
        for width in [360.0, 480.0, 640.0, 960.0] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(Settings::default(), None);
            app.show_workspace = true;
            let mut output = None;
            for _ in 0..3 {
                app.theme(&ctx);
                output = Some(ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 900.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw(ctx),
                ));
            }
            let output = output.expect("rendered frame");
            let tree = output
                .platform_output
                .accesskit_update
                .expect("accessibility tree");
            let bounds = |label: &str| {
                tree.nodes
                    .iter()
                    .find_map(|(_, node)| {
                        (node.label() == Some(label))
                            .then(|| node.bounds())
                            .flatten()
                    })
                    .unwrap_or_else(|| panic!("Missing navigation control: {label}"))
            };
            for label in [
                "Calculator",
                "Graph",
                "Matrices",
                "Learn",
                "Tools",
                "Practice",
                "History",
                "Settings",
            ] {
                let rect = bounds(label);
                assert!(
                    rect.x0 >= 0.0 && rect.x1 <= f64::from(width) + 1.0,
                    "{label} must fit the {width}-point window: {rect:?}"
                );
            }
            let history = bounds("History");
            let tools = bounds("Tools");
            assert!(
                history.x0 >= tools.x1 || history.y0 >= tools.y1,
                "History must not overlap Tools at width {width}"
            );
        }
    }

    #[test]
    fn compact_focus_navigation_keeps_settings_history_and_workspace_at_four_hundred_percent() {
        for scale in [2.0, 4.0] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let settings = Settings {
                scale,
                access: crate::accessibility::AccessibilitySettings {
                    focus_mode: true,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut app = EqualApp::with_settings(settings, None);
            let width = 400.0 / scale;
            let mut output = None;
            for _ in 0..3 {
                app.theme(&ctx);
                output = Some(ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 1500.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        app.header(ctx);
                        egui::CentralPanel::default().show(ctx, |ui| app.workspace_navigation(ui));
                    },
                ));
            }
            let tree = output.unwrap().platform_output.accesskit_update.unwrap();
            for label in ["Settings", "Workspace", "History", "Show navigation"] {
                let rect = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(label))
                    .unwrap_or_else(|| panic!("Missing {label} at {scale}x"))
                    .1
                    .bounds()
                    .unwrap();
                assert!(
                    rect.x0 >= -0.5 && rect.x1 <= f64::from(width) + 0.5,
                    "{label} must fit at {scale}x: {rect:?}"
                );
            }
        }
    }

    #[test]
    fn settings_dialog_returns_keyboard_focus_to_each_opening_origin() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let input = egui::Id::new("access-focus-input");
        let mut text = "2+2".to_string();
        let mut trigger = egui::Id::NULL;
        let mut frame = |app: &mut EqualApp, events: Vec<egui::Event>| {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(640.0, 900.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add(egui::TextEdit::singleline(&mut text).id(input));
                        trigger = ui.button("Settings").id;
                        ctx.data_mut(|data| {
                            data.insert_temp(egui::Id::new("settings-current-trigger"), trigger)
                        });
                    });
                    app.settings_dialog(ctx);
                },
            );
        };
        frame(&mut app, vec![]);
        for explicit in [false, true] {
            let origin = if explicit {
                ctx.data(|data| {
                    data.get_temp::<egui::Id>(egui::Id::new("settings-current-trigger"))
                })
                .unwrap()
            } else {
                input
            };
            ctx.memory_mut(|memory| memory.request_focus(origin));
            app.open_access_settings(&ctx, explicit.then_some(origin));
            frame(&mut app, vec![]);
            assert!(app.show_settings);
            assert_ne!(ctx.memory(|memory| memory.focused()), Some(origin));
            frame(
                &mut app,
                vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            assert!(!app.show_settings);
            assert_eq!(ctx.memory(|memory| memory.focused()), Some(origin));
            frame(&mut app, vec![]);
        }
    }

    #[test]
    fn opening_settings_starts_at_learning_level_after_previous_scrolling() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let frame = |app: &mut EqualApp, events: Vec<egui::Event>| {
            app.theme(&ctx);
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(480.0, 900.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| app.settings_dialog(ctx),
            )
        };
        for _ in 0..2 {
            app.open_access_settings(&ctx, None);
            let mut output = frame(&mut app, vec![]);
            for _ in 0..2 {
                output = frame(&mut app, vec![]);
            }
            let tree = output.platform_output.accesskit_update.unwrap();
            let bounds = |label: &str| {
                tree.nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(label))
                    .unwrap_or_else(|| panic!("Missing {label}"))
                    .1
                    .bounds()
                    .unwrap()
            };
            let learning = bounds(LEVELS[0]);
            let close = bounds("Close settings");
            let zoom = bounds("Interface zoom presets");
            let geometry = ctx.data(|data| {
                data.get_temp::<(egui::Rect, egui::Rect, egui::Vec2, egui::Vec2)>(egui::Id::new(
                    "settings-scroll-geometry",
                ))
            });
            assert!(
                learning.y0 >= close.y1 && learning.y1 < zoom.y0,
                "The first settings control must be visible below Close: grade={learning:?}, close={close:?}, zoom={zoom:?}; (close, viewport, offset, content size)={geometry:?}"
            );
            let _ = frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(egui::pos2(240.0, 500.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -300.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            let _ = frame(&mut app, vec![]);
            app.show_settings = false;
            let _ = frame(&mut app, vec![]);
        }
    }
}
