//! Explicit local notebook actions; disk access and native pickers run off the UI thread.
use crate::{EqualApp, Settings, egui, notebook::Notebook, notebook_files};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

const DIRTY_INTERVAL: f64 = 0.5;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    New,
    Open(Option<PathBuf>),
    Close,
}

enum FileResult {
    Saved {
        path: PathBuf,
        fingerprint: u64,
        after: Option<Action>,
    },
    Opened {
        path: PathBuf,
        document: Box<Notebook>,
    },
    Cancelled,
}

pub(crate) struct NotebookState {
    pub(crate) show: bool,
    pub(crate) title: String,
    pub(crate) notes: String,
    pub(crate) include_practice: bool,
    path: Option<PathBuf>,
    path_draft: String,
    notice: String,
    pending: Option<Receiver<Result<FileResult, String>>>,
    confirm: Option<Action>,
    clean_fingerprint: Option<u64>,
    dirty: bool,
    next_dirty_check: f64,
    allow_close: bool,
    focus_panel: bool,
    focus_confirm: bool,
}

impl Default for NotebookState {
    fn default() -> Self {
        Self {
            show: false,
            title: "Untitled".into(),
            notes: String::new(),
            include_practice: false,
            path: None,
            path_draft: String::new(),
            notice: String::new(),
            pending: None,
            confirm: None,
            clean_fingerprint: None,
            dirty: false,
            next_dirty_check: 0.0,
            allow_close: false,
            focus_panel: false,
            focus_confirm: false,
        }
    }
}

impl NotebookState {
    pub(crate) fn overlay_open(&self) -> bool {
        self.show || self.confirm.is_some() || self.pending.is_some()
    }
}

fn fingerprint(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn notebook_path(path: &Path) -> Result<(), String> {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_none_or(|name| !name.to_ascii_lowercase().ends_with(".opencalc.json"))
    {
        return Err("Choose a filename ending in .opencalc.json.".into());
    }
    Ok(())
}

fn status(ui: &mut egui::Ui, text: &str) {
    let response = label(ui, text);
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_label(text);
        node.clear_value();
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

fn text_galley(ui: &egui::Ui, text: &str, width: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(
        text.to_string(),
        egui::TextStyle::Body.resolve(ui.style()),
        ui.visuals().text_color(),
        width.max(1.0),
    );
    job.wrap.break_anywhere = width < 180.0 || text.contains(['/', '\\']);
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

fn label(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let galley = text_galley(ui, text, ui.max_rect().width());
    ui.label(galley)
}

fn button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let width = ui.max_rect().width().max(1.0);
    let mut job = egui::text::LayoutJob::simple(
        text.to_string(),
        egui::TextStyle::Button.resolve(ui.style()),
        ui.visuals().text_color(),
        (width - ui.spacing().button_padding.x * 2.0).max(1.0),
    );
    job.wrap.break_anywhere = true;
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let response = ui.add(egui::Button::new(galley).wrap());
    if response.has_focus() && !ui.is_rect_visible(response.rect) {
        response.scroll_to_me(None);
    }
    response
}

impl EqualApp {
    pub(crate) fn open_notebook_window(&mut self, ctx: &egui::Context) {
        self.show_settings = false;
        self.show_help = false;
        self.show_history = false;
        self.a11y.show_glossary = false;
        self.a11y.show_shortcuts = false;
        self.notebook.show = true;
        self.notebook.focus_panel = true;
        self.input_access.pause();
        egui::Popup::close_all(ctx);
    }

    fn notebook_document(&self) -> Notebook {
        self.capture_notebook(
            &self.notebook.title,
            &self.notebook.notes,
            self.notebook.include_practice,
        )
    }

    fn refresh_notebook_dirty(&mut self, ctx: &egui::Context, force: bool) {
        let now = ctx.input(|input| input.time);
        if !force && now < self.notebook.next_dirty_check {
            return;
        }
        self.notebook.next_dirty_check = now + DIRTY_INTERVAL;
        match self.notebook_document().content_bytes() {
            Ok(bytes) => {
                let current = fingerprint(&bytes);
                let clean = self.notebook.clean_fingerprint.get_or_insert(current);
                self.notebook.dirty = current != *clean;
            }
            Err(_) => self.notebook.dirty = true,
        }
    }

    fn notebook_notice(&mut self, text: String) {
        self.message = text.clone();
        self.notebook.notice = text.clone();
        if self.settings.access.captions {
            self.a11y.latest_announcement = text;
        }
    }

    fn request_notebook_action(&mut self, ctx: &egui::Context, action: Action) {
        if self.notebook.pending.is_some() || self.notebook.confirm.is_some() {
            return;
        }
        self.refresh_notebook_dirty(ctx, true);
        self.input_access.pause();
        if self.notebook.dirty {
            self.notebook.confirm = Some(action);
            self.notebook.focus_confirm = true;
            egui::Popup::close_all(ctx);
        } else {
            self.perform_notebook_action(ctx, action);
        }
    }

    fn perform_notebook_action(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::New => {
                let mut settings = Settings {
                    level: self.settings.level,
                    degrees: self.settings.degrees,
                    precision: self.settings.precision,
                    ..Default::default()
                };
                settings.access.decimal_comma = self.settings.access.decimal_comma;
                let mut blank = EqualApp::with_settings(settings, None);
                blank.graph_a.clear();
                blank.graph_b.clear();
                blank.tutor.problem.clear();
                let document = blank.capture_notebook("Untitled", "", false);
                match self.apply_notebook(document) {
                    Ok(()) => {
                        let restore_notice = self.message.clone();
                        self.notebook = NotebookState::default();
                        self.refresh_notebook_dirty(ctx, true);
                        let mut notice =
                            "New notebook. Your access settings were kept.".to_string();
                        if restore_notice != "Notebook opened." {
                            notice.push(' ');
                            notice.push_str(&restore_notice);
                        }
                        self.notebook_notice(notice);
                    }
                    Err(error) => self.notebook_notice(error),
                }
            }
            Action::Open(path) => self.start_notebook_open(path),
            Action::Close => {
                self.notebook.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn start_notebook_open(&mut self, chosen: Option<PathBuf>) {
        if self.notebook.pending.is_some() {
            return;
        }
        let current = self.notebook.path.clone();
        let (sender, receiver) = mpsc::channel();
        self.notebook.pending = Some(receiver);
        self.notebook_notice("Opening notebook…".into());
        std::thread::spawn(move || {
            let result = (|| {
                let path = match chosen {
                    Some(path) => Some(path),
                    None => notebook_files::choose_notebook_file(false, current.as_deref())?,
                };
                let Some(path) = path else {
                    return Ok(FileResult::Cancelled);
                };
                notebook_path(&path)?;
                let bytes = notebook_files::read_notebook(&path)?;
                let document = Notebook::decode(&bytes)?;
                Ok(FileResult::Opened {
                    path,
                    document: Box::new(document),
                })
            })();
            let _ = sender.send(result);
        });
    }

    fn start_notebook_save(
        &mut self,
        save_as: bool,
        chosen: Option<PathBuf>,
        after: Option<Action>,
    ) {
        if self.notebook.pending.is_some() {
            return;
        }
        let document = self.notebook_document();
        let save_data = document.to_bytes().and_then(|bytes| {
            document
                .content_bytes()
                .map(|content| (bytes, fingerprint(&content)))
        });
        let (bytes, saved_fingerprint) = match save_data {
            Ok(save_data) => save_data,
            Err(error) => {
                self.notebook.show = true;
                self.notebook.focus_panel = true;
                self.notebook_notice(error);
                return;
            }
        };
        let current = self.notebook.path.clone();
        let (sender, receiver) = mpsc::channel();
        self.notebook.pending = Some(receiver);
        self.input_access.pause();
        self.notebook_notice("Saving notebook…".into());
        std::thread::spawn(move || {
            let result = (|| {
                let path = if let Some(chosen) = chosen {
                    Some(chosen)
                } else if !save_as && current.is_some() {
                    current
                } else {
                    notebook_files::choose_notebook_file(true, current.as_deref())?
                };
                let Some(path) = path else {
                    return Ok(FileResult::Cancelled);
                };
                notebook_path(&path)?;
                notebook_files::write_notebook(&path, &bytes)?;
                Ok(FileResult::Saved {
                    path,
                    fingerprint: saved_fingerprint,
                    after,
                })
            })();
            let _ = sender.send(result);
        });
    }

    pub(crate) fn save_notebook_to_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.refresh_notebook_dirty(ctx, true);
        self.start_notebook_save(true, Some(path), None);
    }

    pub(crate) fn open_notebook_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.request_notebook_action(ctx, Action::Open(Some(path)));
    }

    pub(crate) fn notebook_shortcuts(&mut self, ctx: &egui::Context) -> bool {
        let action = ctx.input_mut(|input| {
            let command = egui::Modifiers::COMMAND;
            let shifted = egui::Modifiers {
                shift: true,
                command: true,
                ..Default::default()
            };
            if input.consume_key(shifted, egui::Key::S) {
                Some(3)
            } else if input.consume_key(command, egui::Key::S) {
                Some(2)
            } else if input.consume_key(command, egui::Key::O) {
                Some(1)
            } else if input.consume_key(command, egui::Key::N) {
                Some(0)
            } else {
                None
            }
        });
        let Some(action) = action else {
            return false;
        };
        if self.notebook.pending.is_none() && self.notebook.confirm.is_none() {
            egui::Popup::close_all(ctx);
            match action {
                0 => self.request_notebook_action(ctx, Action::New),
                1 => self.request_notebook_action(ctx, Action::Open(None)),
                2 => self.start_notebook_save(false, None, None),
                _ => self.start_notebook_save(true, None, None),
            }
        }
        true
    }

    pub(crate) fn poll_notebook(&mut self, ctx: &egui::Context) {
        let result =
            self.notebook
                .pending
                .as_ref()
                .and_then(|receiver| {
                    match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(Err(
                    "The notebook file operation ended unexpectedly. Your current work was kept."
                        .into(),
                )),
            }
                });
        if let Some(result) = result {
            self.notebook.pending = None;
            match result {
                Ok(FileResult::Saved {
                    path,
                    fingerprint,
                    after,
                }) => {
                    self.notebook.path_draft = path.to_string_lossy().into_owned();
                    self.notebook.path = Some(path);
                    self.notebook.clean_fingerprint = Some(fingerprint);
                    self.refresh_notebook_dirty(ctx, true);
                    self.notebook_notice("Notebook saved.".into());
                    if let Some(action) = after {
                        if self.notebook.dirty {
                            self.notebook.confirm = Some(action);
                            self.notebook.focus_confirm = true;
                            self.notebook_notice(
                                "Notebook saved. Additional changes need a save before continuing."
                                    .into(),
                            );
                        } else {
                            self.perform_notebook_action(ctx, action);
                        }
                    }
                }
                Ok(FileResult::Opened { path, document }) => {
                    let title = document.title.clone();
                    let notes = document.notes.clone();
                    let include_practice = document.practice.is_some();
                    match self.apply_notebook(*document) {
                        Ok(()) => {
                            let restore_notice = self.message.clone();
                            self.notebook.title = title;
                            self.notebook.notes = notes;
                            self.notebook.include_practice = include_practice;
                            self.notebook.path_draft = path.to_string_lossy().into_owned();
                            self.notebook.path = Some(path);
                            self.notebook.show = false;
                            self.notebook.clean_fingerprint = None;
                            self.refresh_notebook_dirty(ctx, true);
                            self.notebook_notice(format!(
                                "{restore_notice} Number notation follows the saved work."
                            ));
                        }
                        Err(error) => {
                            self.notebook.show = true;
                            self.notebook.focus_panel = true;
                            self.notebook_notice(error);
                        }
                    }
                }
                Ok(FileResult::Cancelled) => {
                    self.notebook_notice(
                        "Notebook operation cancelled. Your current work was kept.".into(),
                    );
                }
                Err(error) => {
                    self.notebook.show = true;
                    self.notebook.focus_panel = true;
                    self.notebook_notice(format!("{error} Your current work was kept."));
                }
            }
        }
        self.refresh_notebook_dirty(ctx, false);
        let name = if self.notebook.title.trim().is_empty() {
            "Untitled"
        } else {
            self.notebook.title.trim()
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "OpenCalc — {name}{}",
            if self.notebook.dirty { " *" } else { "" }
        )));
        if self.screenshot.is_none()
            && ctx.input(|input| input.viewport().close_requested())
            && !self.notebook.allow_close
        {
            self.refresh_notebook_dirty(ctx, true);
            if self.notebook.dirty
                || self.notebook.pending.is_some()
                || self.notebook.confirm.is_some()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if self.notebook.pending.is_none() && self.notebook.confirm.is_none() {
                    self.notebook.confirm = Some(Action::Close);
                    self.notebook.focus_confirm = true;
                    self.input_access.pause();
                }
            }
        }
        ctx.request_repaint_after(if self.notebook.pending.is_some() {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(500)
        });
    }

    pub(crate) fn notebook_windows(&mut self, ctx: &egui::Context) {
        if self.notebook.show
            && self.notebook.pending.is_none()
            && self.notebook.confirm.is_none()
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            self.notebook.show = false;
        }
        if self.notebook.show {
            let rect = ctx.content_rect();
            let width = (rect.width() - 32.0).clamp(40.0, 620.0);
            let busy = self.notebook.pending.is_some() || self.notebook.confirm.is_some();
            let mut action = None;
            let mut save = None;
            egui::Window::new("Notebook")
                .id(egui::Id::new("notebook-window"))
                .title_bar(false)
                .default_width(width)
                .min_width(40.0)
                .max_width(width)
                .min_height(40.0)
                .max_height((rect.height() - 32.0).max(40.0))
                .constrain_to(rect.shrink(4.0))
                .show(ctx, |ui| {
                    ui.set_max_width(width);
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    ui.horizontal_wrapped(|ui| {
                        ui.heading(if width < 150.0 { "File" } else { "Notebook" });
                        let close = button(ui, "Close");
                        close.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Close notebook panel"));
                        if self.notebook.focus_panel && !busy {
                            close.request_focus();
                            self.notebook.focus_panel = false;
                        }
                        if close.clicked() {
                            self.notebook.show = false;
                        }
                    });
                    egui::ScrollArea::vertical().id_salt("notebook-body")
                        .animated(!self.settings.access.reduced_motion)
                        .max_height((rect.height() - 120.0).max(40.0))
                        .min_scrolled_height((rect.height() - 180.0).clamp(40.0, 580.0))
                        .auto_shrink([false, true]).show(ui, |ui| {
                    if !self.notebook.notice.is_empty() {
                        status(ui, &self.notebook.notice);
                    }
                    ui.add_enabled_ui(!busy, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if button(ui, "New").clicked() {
                                action = Some(Action::New);
                            }
                            if button(ui, "Open").clicked() {
                                action = Some(Action::Open(None));
                            }
                            if button(ui, "Save").clicked() {
                                save = Some((false, None));
                            }
                            if button(ui, "Save as").clicked() {
                                save = Some((true, None));
                            }
                        });
                        status(ui, if self.notebook.dirty { "Unsaved changes" } else { "No unsaved changes" });
                        label(ui,"Title");
                        let title = ui.add(egui::TextEdit::singleline(&mut self.notebook.title)
                            .id_salt("notebook-title").desired_width(ui.available_width()).char_limit(200));
                        title.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Notebook title"));
                        label(ui,"Notes");
                        let notes = ui.add(egui::TextEdit::multiline(&mut self.notebook.notes)
                            .id_salt("notebook-notes").desired_width(ui.available_width()).desired_rows(6).char_limit(65_536));
                        notes.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Notebook notes"));
                        let practice_label=text_galley(ui,"Include practice progress and current answers",ui.max_rect().width()-ui.spacing().icon_width-ui.spacing().icon_spacing);
                        ui.checkbox(&mut self.notebook.include_practice, practice_label);
                        label(ui,"Saves calculations, graphs, matrices, local lessons and notes. AI chats and attachments are not included. Your access preferences stay on this computer; saved number notation follows the notebook.");
                        if let Some(path) = &self.notebook.path {
                            label(ui,&format!("File: {}", path.display()));
                        } else {
                            label(ui,"This notebook has not been saved to a file.");
                        }
                        let id=ui.make_persistent_id("notebook-typed-path");
                        let mut disclosure=egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(),id,false);
                        let toggle=button(ui, if disclosure.is_open(){"− Type a file path"}else{"+ Type a file path"});
                        toggle.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader,true,"Type a file path"));
                        if toggle.clicked(){disclosure.toggle(ui);}
                        ui.ctx().accesskit_node_builder(toggle.id,|node|node.set_expanded(disclosure.is_open()));
                        disclosure.show_body_unindented(ui, |ui| {
                            label(ui,"Notebook file ending in .opencalc.json");
                            let path = ui.add(egui::TextEdit::singleline(&mut self.notebook.path_draft)
                                .id_salt("notebook-path").desired_width(ui.available_width()).hint_text("homework.opencalc.json"));
                            path.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Notebook file path"));
                            ui.horizontal_wrapped(|ui| {
                                if button(ui, "Open typed path").clicked() {
                                    action = Some(Action::Open(Some(PathBuf::from(self.notebook.path_draft.trim()))));
                                }
                                if button(ui, "Save to typed path").clicked() {
                                    save = Some((true, Some(PathBuf::from(self.notebook.path_draft.trim()))));
                                }
                            });
                            label(ui,"Saving to a typed path replaces an existing OpenCalc notebook at that path.");
                        });
                    });
                    });
                });
            if let Some(action) = action {
                if let Action::Open(Some(path)) = action {
                    self.open_notebook_path(ctx, path);
                } else {
                    self.request_notebook_action(ctx, action);
                }
            }
            if let Some((save_as, path)) = save {
                if let Some(path) = path {
                    self.save_notebook_to_path(ctx, path);
                } else {
                    self.refresh_notebook_dirty(ctx, true);
                    self.start_notebook_save(save_as, None, None);
                }
            }
        }
        if self.notebook.pending.is_some() && !self.notebook.show {
            let viewport = ctx.content_rect();
            let _progress = egui::Modal::new(egui::Id::new("notebook-file-progress"))
                .frame(egui::Frame::popup(&ctx.style()).inner_margin(6))
                .show(ctx, |ui| {
                    ui.set_width((viewport.width() - 24.0).clamp(24.0, 360.0));
                    let heading = label(ui,"Notebook file");
                    let height = (viewport.height() - heading.rect.height() - 36.0).max(12.0);
                    egui::ScrollArea::vertical().id_salt("notebook-progress-body")
                        .animated(!self.settings.access.reduced_motion)
                        .max_height(height).min_scrolled_height(height.min(100.0))
                        .show(ui,|ui| {
                            status(ui,&self.notebook.notice);
                            label(ui,"A file action is in progress. If a file dialog is open, Cancel keeps your current work.");
                        });
                });
            #[cfg(test)]
            ctx.data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("notebook-progress-rect"),
                    _progress.response.rect,
                )
            });
        }
        if let Some(action) = self.notebook.confirm.clone() {
            let mut choice = None;
            let viewport = ctx.content_rect();
            let response = egui::Modal::new(egui::Id::new("notebook-unsaved"))
                .frame(egui::Frame::popup(&ctx.style()).inner_margin(6))
                .show(ctx, |ui| {
                    ui.set_width((viewport.width() - 24.0).clamp(24.0, 420.0));
                    let header = ui.vertical(|ui| {
                        label(ui, "Unsaved work");
                        if button(ui, "Cancel").clicked() {
                            choice = Some(2);
                        }
                    });
                    let height =
                        (viewport.height() - header.response.rect.height() - 36.0).max(12.0);
                    egui::ScrollArea::vertical()
                        .id_salt("notebook-confirm-body")
                        .animated(!self.settings.access.reduced_motion)
                        .max_height(height)
                        .min_scrolled_height(height.min(140.0))
                        .show(ui, |ui| {
                            label(ui, "Save changes before continuing, or discard them.");
                            ui.horizontal_wrapped(|ui| {
                                let first = button(ui, "Save first");
                                if self.notebook.focus_confirm {
                                    first.request_focus();
                                    self.notebook.focus_confirm = false;
                                }
                                if first.clicked() {
                                    choice = Some(0);
                                }
                                if button(ui, "Discard changes").clicked() {
                                    choice = Some(1);
                                }
                            });
                        });
                });
            #[cfg(test)]
            ctx.data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("notebook-confirm-rect"),
                    response.response.rect,
                )
            });
            if response.should_close() {
                choice = Some(2);
            }
            match choice {
                Some(0) => {
                    self.notebook.confirm = None;
                    self.start_notebook_save(false, None, Some(action));
                }
                Some(1) => {
                    self.notebook.confirm = None;
                    self.perform_notebook_action(ctx, action);
                }
                Some(_) => {
                    self.notebook.confirm = None;
                    self.notebook_notice("Cancelled. Your current work was kept.".into());
                }
                None => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tab;

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
                    egui::vec2(480.0, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                app.poll_notebook(ctx);
                app.notebook_shortcuts(ctx);
                app.draw(ctx);
                app.notebook_windows(ctx);
            },
        )
    }

    fn app() -> EqualApp {
        EqualApp::with_settings(Settings::default(), None)
    }

    fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn action(output: &egui::FullOutput, label: &str) -> egui::Event {
        let tree = output.platform_output.accesskit_update.as_ref().unwrap();
        let target = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("Missing {label}"))
            .0;
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target,
            data: None,
        })
    }

    #[test]
    fn dirty_new_cancel_keeps_work_and_discard_preserves_access_preferences() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app();
        frame(&mut app, &ctx, vec![]);
        app.expression = "12+8".into();
        app.evaluate();
        app.settings.access.word_labels = true;
        app.request_notebook_action(&ctx, Action::New);
        assert_eq!(app.notebook.confirm, Some(Action::New));
        let output = frame(&mut app, &ctx, vec![]);
        let cancel = action(&output, "Cancel");
        frame(&mut app, &ctx, vec![cancel]);
        assert!(app.notebook.confirm.is_none());
        assert_eq!(app.expression, "12+8");
        assert_eq!(app.result, "20");
        app.request_notebook_action(&ctx, Action::New);
        let output = frame(&mut app, &ctx, vec![]);
        let discard = action(&output, "Discard changes");
        frame(&mut app, &ctx, vec![discard]);
        assert!(app.expression.is_empty());
        assert!(app.history.is_empty());
        assert!(app.settings.access.word_labels);
        assert!(!app.notebook.dirty);
    }

    #[test]
    fn notebook_shortcuts_work_in_an_editor_without_inserting_letters() {
        let ctx = egui::Context::default();
        let mut app = app();
        frame(&mut app, &ctx, vec![]);
        app.expression = "8+8".into();
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("expression")));
        frame(
            &mut app,
            &ctx,
            vec![key(egui::Key::N, egui::Modifiers::COMMAND)],
        );
        assert_eq!(app.notebook.confirm, Some(Action::New));
        assert_eq!(app.expression, "8+8");
        assert!(app.notebook.overlay_open());
    }

    #[test]
    fn focus_menu_and_workspace_navigation_do_not_count_as_unsaved_work() {
        let ctx = egui::Context::default();
        let mut app = app();
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![]);
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("expression")));
        frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            vec![key(egui::Key::ArrowLeft, egui::Modifiers::NONE)],
        );
        app.refresh_notebook_dirty(&ctx, true);
        assert!(
            !app.notebook.dirty,
            "Focusing and moving a caret must not dirty work"
        );
        app.press_device_key(&ctx, "y=");
        frame(&mut app, &ctx, vec![]);
        app.press_device_key(&ctx, "math");
        frame(&mut app, &ctx, vec![]);
        app.refresh_notebook_dirty(&ctx, true);
        assert!(
            !app.notebook.dirty,
            "Opening a function editor and insertion menu must not dirty work"
        );
        app.tab = Tab::Graph;
        app.show_workspace = true;
        frame(&mut app, &ctx, vec![]);
        app.refresh_notebook_dirty(&ctx, true);
        assert!(
            !app.notebook.dirty,
            "Viewing another workspace must not dirty work"
        );
        app.expression = "3+4".into();
        app.evaluate();
        app.refresh_notebook_dirty(&ctx, true);
        assert!(app.notebook.dirty, "A calculation is saved content");
        app.notebook.clean_fingerprint = Some(fingerprint(
            &app.notebook_document().content_bytes().unwrap(),
        ));
        app.notebook.notes = "Record the units.".into();
        app.refresh_notebook_dirty(&ctx, true);
        assert!(app.notebook.dirty, "Notes are saved content");
        app.notebook.clean_fingerprint = Some(fingerprint(
            &app.notebook_document().content_bytes().unwrap(),
        ));
        app.device.variables.insert('A', 42.0);
        app.refresh_notebook_dirty(&ctx, true);
        assert!(app.notebook.dirty, "Stored variables are saved content");
    }

    #[test]
    fn notebook_path_requires_the_specific_extension() {
        assert!(notebook_path(Path::new("work.OPENCALC.JSON")).is_ok());
        for path in ["Cargo.toml", "notes.json", ".env", ""] {
            assert!(notebook_path(Path::new(path)).is_err());
        }
    }

    #[test]
    fn notebook_controls_fit_narrow_windows_and_keep_close_visible() {
        for width in [160.0, 360.0, 480.0] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = app();
            app.notebook.show = true;
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
                    |ctx| app.notebook_windows(ctx),
                ));
            }
            let tree = output.unwrap().platform_output.accesskit_update.unwrap();
            for (_, node) in &tree.nodes {
                if matches!(
                    node.role(),
                    egui::accesskit::Role::Button
                        | egui::accesskit::Role::CheckBox
                        | egui::accesskit::Role::TextInput
                ) && let Some(bounds) = node.bounds()
                {
                    assert!(
                        bounds.x0 >= -0.5 && bounds.x1 <= f64::from(width) + 0.5,
                        "{} overflows {width}: {bounds:?}",
                        node.label().unwrap_or("Notebook control")
                    );
                }
            }
            let close = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Close notebook panel"))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(close.y0 >= 0.0 && close.y1 < 900.0);
            assert!(
                tree.nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Notebook title"))
            );
            assert!(
                tree.nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Notebook notes"))
            );
        }
    }

    #[test]
    fn changes_after_a_save_snapshot_require_another_confirmation() {
        let ctx = egui::Context::default();
        let mut app = app();
        frame(&mut app, &ctx, vec![]);
        let bytes = app.notebook_document().content_bytes().unwrap();
        let (sender, receiver) = mpsc::channel();
        app.notebook.pending = Some(receiver);
        app.expression = "New work while a save was pending".into();
        sender
            .send(Ok(FileResult::Saved {
                path: PathBuf::from("work.opencalc.json"),
                fingerprint: fingerprint(&bytes),
                after: Some(Action::New),
            }))
            .unwrap();
        frame(&mut app, &ctx, vec![]);
        assert!(app.notebook.dirty);
        assert_eq!(app.notebook.confirm, Some(Action::New));
        assert_eq!(app.expression, "New work while a save was pending");
    }

    #[test]
    fn confirmation_and_progress_fit_short_magnified_viewports() {
        for size in [
            egui::vec2(80.0, 200.0),
            egui::vec2(120.0, 225.0),
            egui::vec2(360.0, 480.0),
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = app();
            app.notebook.confirm = Some(Action::New);
            app.notebook.focus_confirm = true;
            let render = |app: &mut EqualApp, events| {
                app.theme(&ctx);
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        events,
                        ..Default::default()
                    },
                    |ctx| app.notebook_windows(ctx),
                )
            };
            for _ in 0..4 {
                let _ = render(&mut app, vec![]);
            }
            let output = render(&mut app, vec![]);
            let rect = ctx
                .data(|data| data.get_temp::<egui::Rect>(egui::Id::new("notebook-confirm-rect")))
                .unwrap();
            assert!(
                rect.min.x >= -0.5
                    && rect.min.y >= -0.5
                    && rect.max.x <= size.x + 0.5
                    && rect.max.y <= size.y + 0.5,
                "Confirmation overflows {size:?}: {rect:?}"
            );
            let tree = output.platform_output.accesskit_update.unwrap();
            let cancel = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Cancel"))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                cancel.y0 >= 0.0 && cancel.y1 <= f64::from(size.y),
                "Cancel must stay pinned: {cancel:?}"
            );
            let first = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Save first"))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                first.y0 >= 0.0 && first.y1 <= f64::from(size.y),
                "Focused Save first must scroll into view: {first:?}"
            );
            app.notebook.confirm = None;
            let (_sender, receiver) = mpsc::channel();
            app.notebook.pending = Some(receiver);
            app.notebook.notice = "Opening the notebook file…".into();
            for _ in 0..4 {
                let _ = render(&mut app, vec![]);
            }
            let rect = ctx
                .data(|data| data.get_temp::<egui::Rect>(egui::Id::new("notebook-progress-rect")))
                .unwrap();
            assert!(
                rect.min.x >= -0.5
                    && rect.min.y >= -0.5
                    && rect.max.x <= size.x + 0.5
                    && rect.max.y <= size.y + 0.5,
                "Progress overflows {size:?}: {rect:?}"
            );
        }
    }

    fn wait_for_job(app: &mut EqualApp, ctx: &egui::Context) {
        let until = std::time::Instant::now() + Duration::from_secs(5);
        while app.notebook.pending.is_some() && std::time::Instant::now() < until {
            frame(app, ctx, vec![]);
            std::thread::yield_now();
        }
        assert!(app.notebook.pending.is_none(), "Notebook worker timed out");
    }

    #[test]
    fn async_save_open_and_malformed_file_preserve_the_right_work() {
        let ctx = egui::Context::default();
        let mut app = app();
        frame(&mut app, &ctx, vec![]);
        let path = std::env::temp_dir().join(format!(
            "opencalc-ui-{}-{}.opencalc.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        app.expression = "6*7".into();
        app.evaluate();
        app.notebook.title = "Homework".into();
        app.notebook.notes = "Keep the units.".into();
        app.save_notebook_to_path(&ctx, path.clone());
        wait_for_job(&mut app, &ctx);
        assert!(path.is_file());
        assert!(!app.notebook.dirty);
        assert!(app.notebook.notice.contains("saved"));
        app.expression = "999".into();
        app.open_notebook_path(&ctx, path.clone());
        assert!(app.notebook.confirm.is_some());
        let action = app.notebook.confirm.take().unwrap();
        app.perform_notebook_action(&ctx, action);
        wait_for_job(&mut app, &ctx);
        assert_eq!(app.expression, "6*7");
        assert_eq!(app.result, "42");
        assert_eq!(app.notebook.notes, "Keep the units.");
        assert_eq!(app.tab, Tab::Calculator);
        std::fs::write(&path, b"{not a notebook}").unwrap();
        app.open_notebook_path(&ctx, path.clone());
        wait_for_job(&mut app, &ctx);
        assert_eq!(app.expression, "6*7");
        assert_eq!(app.result, "42");
        assert!(app.notebook.notice.contains("current work was kept"));
        std::fs::remove_file(path).unwrap();
    }
}
