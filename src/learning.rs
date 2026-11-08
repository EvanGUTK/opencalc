use crate::*;
use std::io::Read;

const MAX_REFERENCE_IMAGE_BYTES: u64 = 8 * 1024 * 1024;

const TASKS: [&str; 18] = [
    "Keyboard: reach the expression, keys, navigation and Settings with Tab/Shift+Tab; activate controls without a pointer.",
    "Screen reader: identify controls, read 12 + 8 = 20, and find a division-by-zero error without moving focus away.",
    "Magnification: try intermediate and large interface sizes; reach the display, every key and Settings using scrolling.",
    "Appearance: compare light, warm and dark themes and high contrast; read selected controls, formulas and error text.",
    "Color independence: distinguish graph curves using their patterns and names, then read coordinates from the table.",
    "No audio: mute sound and speech; find answers, errors, copied-result feedback and all control states visually.",
    "Speech: choose a comfortable rate and volume, read an expression and result, and stop reading promptly.",
    "Pointer dwell: activate a key after the selected hold time; move away to cancel and confirm a key does not repeat while held.",
    "Repeated input: try the key activation guard; intentional repeated digits still work and accidental rapid activations are filtered.",
    "Switch scanning: start, choose a row and key, pause and restart; opening a dialog must prevent background key activation.",
    "Step-by-step learning: reveal one solution step at a time, then choose all steps when wanted.",
    "Score visibility: hide practice scores, continue a question, and show scores again without changing the recorded attempts.",
    "Reading: choose text size, font and line spacing; read and edit 12, 21, decimal points, fractions and negative signs.",
    "Focus support: try reduced navigation and the reading guide; find how to restore the navigation and leave the current task.",
    "Number formats: enter familiar digits and decimal punctuation; verify decimal commas and semicolon-separated answers are interpreted correctly.",
    "Control names: try word labels and, if used, your system voice-control tool; select a control by its visible name.",
    "Math vocabulary: find a word or symbol in the glossary and read its definition and example with the chosen settings.",
    "Nonvisual graphs: trace with the keyboard, read coordinates, and optionally play/stop graph audio; text remains available without sound.",
];
const RESULTS: [&str; 3] = ["Not tested", "Worked", "Needs change"];

#[derive(Default)]
pub struct LearningSupport {
    pub show_references: bool,
    pub show_access: bool,
    textures: std::collections::HashMap<&'static str, egui::TextureHandle>,
    picture_notices: std::collections::HashMap<&'static str, String>,
    setup: String,
    notes: String,
    outcomes: [usize; TASKS.len()],
    selected_picture: Option<usize>,
}

struct Reference {
    level: &'static str,
    model: &'static str,
    description: &'static str,
    keys: &'static str,
    page: &'static str,
    picture: &'static str,
    screen: Option<&'static str>,
}

const REFERENCES: [Reference; 4] = [
    Reference {
        level: "K–3 · basic arithmetic",
        model: "TI-108",
        description: "A simple four-operation reference for early learners.",
        keys: "Number keys, operation column, large equals key.",
        page: "https://education.ti.com/en/products/calculators/elementary-calculators/ti-108",
        picture: "ti-108.png",
        screen: None,
    },
    Reference {
        level: "4–8 · earlier skills",
        model: "TI-15 Explorer",
        description: "TI positions it for grades 3–6; a reference for fractions and decimals.",
        keys: "Two-line display, fraction controls, arrow keys.",
        page: "https://education.ti.com/en/products/calculators/elementary-calculators/ti-15-explorer",
        picture: "ti-15.png",
        screen: Some("ti-15-screen.png"),
    },
    Reference {
        level: "4–8 · middle school",
        model: "TI-34 MultiView",
        description: "TI's middle-school scientific reference for fractions, powers, and roots.",
        keys: "Four-line display, navigation pad, scientific keys.",
        page: "https://education.ti.com/en/products/calculators/scientific-calculators/ti-34-mv",
        picture: "ti-34.png",
        screen: Some("ti-34-screen.png"),
    },
    Reference {
        level: "9–12 & Advanced · graphing",
        model: "TI-84 Plus CE",
        description: "A graphing reference for algebra through calculus.",
        keys: "Graph screen, navigation pad, scientific rows, light number keys.",
        page: "https://education.ti.com/en/products/calculators/graphing-calculators/ti-84-plus-ce-python",
        picture: "ti-84.png",
        screen: Some("ti-84-screen.png"),
    },
];

fn read_picture_from(path: &std::path::Path) -> Result<Option<Vec<u8>>, String> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("The optional local photo could not be opened.".into()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_REFERENCE_IMAGE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "The optional local photo could not be read.".to_string())?;
    if bytes.len() as u64 > MAX_REFERENCE_IMAGE_BYTES {
        return Err("The optional local photo exceeds the 8 MB limit.".into());
    }
    Ok(Some(bytes))
}

fn read_local_picture(filename: &str) -> Result<Option<Vec<u8>>, String> {
    let mut roots = Vec::new();
    if let Ok(executable) = std::env::current_exe()
        && let Some(folder) = executable.parent()
    {
        roots.push(folder.join("assets/ti"));
    }
    if let Ok(folder) = std::env::current_dir() {
        roots.push(folder.join("assets/ti"));
    }
    roots.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ti"));
    for root in roots {
        if let Some(bytes) = read_picture_from(&root.join(filename))? {
            return Ok(Some(bytes));
        }
    }
    Ok(None)
}

fn reference_columns(width: f32) -> usize {
    if width >= 800.0 {
        4
    } else if width >= 440.0 {
        2
    } else {
        1
    }
}

impl LearningSupport {
    pub(crate) fn overlay_open(&self) -> bool {
        self.show_references || self.show_access || self.selected_picture.is_some()
    }
    fn picture(
        &mut self,
        ui: &mut egui::Ui,
        name: &'static str,
        filename: &'static str,
        size: egui::Vec2,
        alt: &str,
    ) -> bool {
        if !self.textures.contains_key(name) {
            if let Some(notice) = self.picture_notices.get(name) {
                ui.label(notice);
                return false;
            }
            let bytes = match read_local_picture(filename) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    let notice = "Optional local photo not installed. Use the source link to view the reference.";
                    self.picture_notices.insert(name, notice.into());
                    ui.label(notice);
                    return false;
                }
                Err(error) => {
                    ui.label(&error);
                    self.picture_notices.insert(name, error);
                    return false;
                }
            };
            match image::load_from_memory(&bytes) {
                Ok(image) => {
                    let rgba = image.into_rgba8();
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [rgba.width() as usize, rgba.height() as usize],
                        rgba.as_raw(),
                    );
                    self.textures.insert(
                        name,
                        ui.ctx()
                            .load_texture(name, image, egui::TextureOptions::LINEAR),
                    );
                }
                Err(error) => {
                    let notice = format!("{alt}: local picture unavailable ({error})");
                    ui.label(&notice);
                    self.picture_notices.insert(name, notice);
                    return false;
                }
            }
        }
        let texture = &self.textures[name];
        ui.add(
            egui::Image::new(texture)
                .fit_to_exact_size(size)
                .maintain_aspect_ratio(true)
                .alt_text(alt)
                .sense(egui::Sense::click()),
        )
        .clicked()
    }

    fn reference_card(&mut self, ui: &mut egui::Ui, index: usize) {
        let reference = &REFERENCES[index];
        ui.push_id(index, |ui| {
            ui.label(R::new(reference.level).strong());
            ui.heading(reference.model);
            let width = ui.available_width();
            if self.picture(
                ui,
                reference.model,
                reference.picture,
                egui::vec2(width, 260.0),
                &format!("{} calculator. Select to enlarge.", reference.model),
            ) {
                self.selected_picture = Some(index);
            }
            ui.label(crate::appearance::paragraph(
                ui.ctx(),
                reference.description,
            ));
            ui.label(crate::appearance::paragraph(ui.ctx(), reference.keys));
            if let Some(filename) = reference.screen {
                ui.strong("Display example");
                self.picture(
                    ui,
                    reference.page,
                    filename,
                    egui::vec2(width, 100.0),
                    &format!(
                        "{} display example. Details are in the source guide.",
                        reference.model
                    ),
                );
            }
            ui.hyperlink_to("TI source and guidebook", reference.page);
        });
    }

    fn report(&self) -> String {
        let mut text = format!(
            "OpenCalc assistive technology session\nSetup (device, OS, assistive technology and versions): {}\nLearning level: record in notes\nThese are session observations, not a compatibility certification.\n\n",
            self.setup
        );
        for (task, outcome) in TASKS.iter().zip(self.outcomes) {
            text.push_str(&format!("[{}] {task}\n", RESULTS[outcome]));
        }
        text.push_str(&format!(
            "\nNotes, date, barriers and requested changes:\n{}\n",
            self.notes
        ));
        text
    }
}

impl EqualApp {
    pub fn learning_windows(&mut self, ctx: &egui::Context) {
        let viewport = ctx.content_rect();
        let guide_width = (viewport.width() - 40.0).clamp(120.0, 890.0);
        let guide_height = (viewport.height() - 70.0).max(80.0);
        let mut open = self.learning.show_references;
        egui::Window::new("TI calculator picture guide")
            .open(&mut open)
            .default_width(guide_width)
            .min_width(120.0).max_width(guide_width)
            .min_height(80.0).max_height(guide_height)
            .constrain_to(viewport.shrink(6.0))
            .vscroll(true)
            .show(ctx, |ui| {
                ui.label("Calculator references by learning level. Optional local photos can be selected to enlarge them; source links work without installed photos.");
                if ui.button("Reload local photos").clicked() {
                    self.learning.textures.clear();
                    self.learning.picture_notices.clear();
                }
                ui.add_space(8.0);
                let count = reference_columns(ui.available_width());
                for first in (0..REFERENCES.len()).step_by(count) {
                    ui.columns(count, |columns| {
                        for (offset, column) in columns.iter_mut().enumerate() {
                            let index = first + offset;
                            if index < REFERENCES.len() {
                                self.learning.reference_card(column, index);
                            }
                        }
                    });
                    ui.separator();
                }
                ui.label("Models overlap across grades. Photos and display examples are references; OpenCalc does not run TI firmware.");
                ui.label(R::new("Reference photos are not bundled with the open-source app. Optional local photos retain their owners' rights and are not covered by OpenCalc's MIT license. OpenCalc is independent and is not endorsed by TI.").small());
            });
        self.learning.show_references = open;

        if let Some(index) = self.learning.selected_picture {
            let reference = &REFERENCES[index];
            let mut open = true;
            let width = (viewport.width() - 40.0).clamp(120.0, 370.0);
            egui::Window::new(format!("{} reference", reference.model))
                .open(&mut open)
                .default_width(width)
                .min_width(120.0)
                .max_width(width)
                .min_height(80.0)
                .max_height(guide_height)
                .constrain_to(viewport.shrink(6.0))
                .vscroll(true)
                .show(ctx, |ui| {
                    self.learning.picture(
                        ui,
                        reference.model,
                        reference.picture,
                        egui::vec2(
                            ui.available_width(),
                            (viewport.height() - 170.0).clamp(100.0, 560.0),
                        ),
                        reference.keys,
                    );
                    ui.label(reference.keys);
                    ui.hyperlink_to("Open TI reference", reference.page);
                });
            if !open {
                self.learning.selected_picture = None;
            }
        }

        let mut open = self.learning.show_access;
        egui::Window::new("Assistive technology test").open(&mut open)
            .default_width((viewport.width() - 40.0).clamp(120.0, 680.0))
            .min_width(120.0).max_width((viewport.width() - 40.0).clamp(120.0, 680.0))
            .min_height(80.0).max_height(guide_height)
            .constrain_to(viewport.shrink(6.0))
            .vscroll(true).show(ctx, |ui| {
                ui.heading("Assistive technology test");
                ui.label(crate::appearance::paragraph(ui.ctx(), "Use this checklist to record what works with your setup. These observations are not a compatibility certification."));
                ui.label(crate::appearance::paragraph(ui.ctx(), "Choose your settings and try the tasks that matter to you. Leave untried tasks as Not tested. No diagnosis or personal details are needed."));
                ui.separator();
                ui.label("Device, Windows version, assistive technology and version (no learner names)");
                ui.add(egui::TextEdit::singleline(&mut self.learning.setup).desired_width(f32::INFINITY)
                    .hint_text("For example: Windows 11, NVDA version, keyboard, 150% Windows scaling"))
                    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Assistive technology and device setup"));
                for (i, task) in TASKS.iter().enumerate() {
                    ui.add_space(6.0);
                    ui.label(crate::appearance::paragraph(ui.ctx(), format!("{}. {task}", i + 1)));
                    ui.horizontal_wrapped(|ui| {
                        for (value, label) in RESULTS.iter().enumerate() {
                            let response = ui.selectable_value(&mut self.learning.outcomes[i], value, *label);
                            response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, self.learning.outcomes[i] == value, format!("Task {}: {label}", i + 1)));
                        }
                    });
                }
                ui.separator();
                ui.label("Session date, learning level, barriers and requested changes");
                ui.add(egui::TextEdit::multiline(&mut self.learning.notes).desired_width(f32::INFINITY).desired_rows(3)
                    .hint_text("Describe what was hard, what worked, and what should change."))
                    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Session notes and barriers"));
                ui.label("Close this checklist to try a task. Reopen it from Settings to record the result. Notes last for this app session; copy the report to keep it.");
                if ui.button("Copy test report").clicked() {
                    ui.ctx().copy_text(self.learning.report());
                    self.message = "Test report copied. Paste it into a document to keep your session notes.".into();
                }
                if !self.message.is_empty() { ui.label(&self.message); }
            });
        self.learning.show_access = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_optional_photos_keep_a_text_fallback_and_source_metadata() {
        let ctx = egui::Context::default();
        let mut support = LearningSupport::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                assert!(!support.picture(
                    ui,
                    "missing-photo-test",
                    "equal-missing-photo-regression.png",
                    egui::vec2(100.0, 200.0),
                    "Calculator reference",
                ));
            });
        });
        assert!(support.textures.is_empty());
        assert!(support.picture_notices.contains_key("missing-photo-test"));
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Text(text) if text.galley.text().contains("Optional local photo not installed"))));
        for reference in &REFERENCES {
            assert!(reference.page.starts_with("https://education.ti.com/"));
            assert!(!reference.description.is_empty() && !reference.keys.is_empty());
            assert!(!reference.picture.contains(['/', '\\']));
        }
    }

    #[test]
    fn learning_dialogs_and_enlarged_pictures_block_background_input() {
        let mut support = LearningSupport::default();
        assert!(!support.overlay_open());
        support.show_access = true;
        assert!(support.overlay_open());
        support.show_access = false;
        support.show_references = true;
        assert!(support.overlay_open());
        support.show_references = false;
        support.selected_picture = Some(0);
        assert!(support.overlay_open());
    }

    #[test]
    fn checklist_covers_input_reading_senses_and_locale_without_collecting_diagnoses() {
        let report = LearningSupport::default().report();
        for task in [
            "Keyboard:",
            "Screen reader:",
            "Magnification:",
            "No audio:",
            "Pointer dwell:",
            "Switch scanning:",
            "Reading:",
            "Number formats:",
            "Control names:",
        ] {
            assert!(report.contains(task), "{task}");
        }
        assert_eq!(report.matches("[Not tested]").count(), TASKS.len());
        assert!(!report.contains("[Worked]"));
        assert!(report.contains("not a compatibility certification"));
    }

    #[test]
    fn report_never_marks_untried_tasks_as_verified() {
        let mut support = LearningSupport::default();
        assert_eq!(
            support.report().matches("[Not tested]").count(),
            TASKS.len()
        );
        support.outcomes[0] = 1;
        support.outcomes[1] = 2;
        support.notes = "Focus was hard to see.".into();
        let report = support.report();
        assert_eq!(report.matches("[Not tested]").count(), TASKS.len() - 2);
        assert!(report.contains("[Worked]") && report.contains("[Needs change]"));
        assert!(report.contains("Focus was hard to see."));
    }
}
