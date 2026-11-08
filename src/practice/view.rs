use super::{Answer, Difficulty, FeedbackKind, TopicId, Visual, topics};
use crate::*;

fn practice_status(ui: &mut egui::Ui, text: &str, color: C) {
    let response = ui.label(crate::appearance::paragraph(ui.ctx(), text).color(color));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_label(text);
        node.clear_value();
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

impl EqualApp {
    pub(crate) fn practice_workspace(&mut self, ui: &mut egui::Ui) {
        let palette = crate::appearance::palette(ui.ctx());
        self.practice.set_level(self.settings.level);
        let level = self.settings.level.min(3);
        ui.horizontal_wrapped(|ui| {
            ui.heading("Practice");
            ui.label(LEVELS[level]);
        });

        let available: Vec<_> = topics(level).collect();
        let mut selected = self.practice.selected_topic();
        let selected_title = available
            .iter()
            .find(|topic| topic.id == selected)
            .map_or("Choose topic", |topic| topic.title);
        ui.label("Topic");
        let width = (ui.available_width() - 12.0).clamp(60.0, 260.0);
        egui::ComboBox::from_id_salt("practice-topic")
            .width(width)
            .selected_text(selected_title)
            .show_ui(ui, |ui| {
                ui.set_max_width(width);
                for topic in &available {
                    let color = if selected == topic.id {
                        palette.selected_text
                    } else {
                        palette.ink
                    };
                    ui.selectable_value(&mut selected, topic.id, R::new(topic.title).color(color));
                }
            })
            .response
            .widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, "Practice topic")
            });
        if selected != self.practice.selected_topic() {
            self.practice.select_topic(selected);
        }
        if self.settings.access.show_help_text
            && let Some(topic) = available.iter().find(|topic| topic.id == selected)
        {
            ui.label(crate::appearance::paragraph(ui.ctx(), topic.description));
        }

        let mut difficulty = self.practice.difficulty();
        ui.label("Difficulty");
        ui.horizontal_wrapped(|ui| {
            for value in Difficulty::ALL {
                ui.radio_value(&mut difficulty, value, value.label());
            }
        });
        if difficulty != self.practice.difficulty() {
            self.practice.select_difficulty(difficulty);
        }

        let mut question = self.practice.current_question();
        if self.settings.access.decimal_comma {
            question.prompt = question
                .prompt
                .replace("separated by a comma", "separated by semicolons (;)")
                .replace("separated by commas", "separated by semicolons (;)");
        }
        let essential_givens = matches!(&question.visual, Visual::Matrix { .. });
        ui.separator();
        let margin = if ui.available_width() < 350.0 { 10 } else { 16 };
        egui::Frame::new()
            .fill(palette.paper)
            .stroke(egui::Stroke::new(1.0_f32, palette.border))
            .corner_radius(8.0_f32)
            .inner_margin(margin)
            .show(ui, |ui| {
                if self.settings.access.show_scores { ui.label(format!(
                    "Question {}",
                    self.practice.current_session().index.saturating_add(1)
                )); }
                ui.label(crate::appearance::paragraph(ui.ctx(), &question.prompt).size(21.0 * self.settings.access.font_scale).strong());
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Read problem").clicked() {
                        self.speak(&question.problem_text());
                    }
                    if !essential_givens && !matches!(question.visual, Visual::None) {
                        ui.checkbox(
                            &mut self.practice.current_session_mut().show_visual,
                            "Show visual aid",
                        );
                    }
                });
                if essential_givens || self.practice.current_session().show_visual {
                    super::visuals::show(
                        ui,
                        &question.visual,
                        self.settings.contrast,
                        self.settings.readable,
                    );
                }
                ui.add_space(6.0);
                ui.label("Answer");
                match &question.answer {
                    Answer::Number { .. } => {
                        if self.settings.access.show_help_text {
                        let guidance = match level {
                            0 => "Enter a number.",
                            1 => "Fractions and decimals are accepted.",
                            _ => "Fractions, decimals, and constant expressions are accepted.",
                        };
                        let help = ui.label(R::new(guidance).small());
                        if level >= 2 {
                            help.on_hover_text("Trigonometric expressions use radians.");
                        }
                        }
                    }
                    Answer::Values { ordered, .. } => {
                        let separator = if self.settings.access.decimal_comma { "semicolons (;)" } else { "commas" };
                        ui.label(
                            R::new(if *ordered {
                                format!("Separate answers with {separator}, in the requested order.")
                            } else {
                                format!("Separate answers with {separator}, in any order.")
                            })
                            .small(),
                        );
                        if self.settings.access.show_help_text { ui.label(
                            R::new("Fractions and constant expressions are accepted.").small(),
                        )
                        .on_hover_text("Trigonometric expressions use radians."); }
                    }
                }
                let typed_answer = self.practice.current_session().response.to_ascii_lowercase();
                if selected == TopicId::Trigonometry || ["sin", "cos", "tan"].iter().any(|name| typed_answer.contains(name)) {
                    ui.label(R::new("Trigonometric expressions use radians.").small());
                }
                if self.settings.access.decimal_comma {
                    ui.label(if level == 0 { "Use a comma for decimals, such as 1,5." } else { "Use a decimal comma (1,5). Separate function inputs and multiple answers with semicolons (;)." });
                }
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.practice.current_session_mut().response)
                        .id_salt("practice-answer")
                        .desired_width(ui.available_width())
                        .char_limit(1024)
                        .hint_text("Enter your answer"),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Practice answer")
                });
                if response.changed() {
                    let session = self.practice.current_session_mut();
                    session.feedback.clear();
                    session.feedback_kind = FeedbackKind::None;
                }
                let enter = (response.has_focus() || response.lost_focus())
                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Check answer").clicked() || enter {
                        if self.settings.access.decimal_comma { self.practice.check_answer_with_locale(true); }
                        else { self.practice.check_answer(); }
                        if self.settings.speech {
                            self.speak(&self.practice.current_session().feedback.clone());
                        }
                    }
                    if ui.button("Show a hint").clicked() {
                        self.practice.show_hint();
                        if self.settings.speech {
                            self.speak(&question.hint);
                        }
                    }
                    if ui.button("Reveal answer & method").clicked() {
                        self.practice.reveal_answer_with_pacing(self.settings.access.one_step);
                    }
                });
                if self.practice.current_session().hint_shown {
                    ui.strong("Hint");
                    practice_status(ui, &question.hint, palette.ink);
                }
                let session = self.practice.current_session();
                if !session.feedback.is_empty() && session.feedback != question.hint {
                    let color = if session.feedback_kind == FeedbackKind::Correct
                        && !self.settings.contrast
                    {
                        palette.accent
                    } else {
                        palette.ink
                    };
                    practice_status(ui, &session.feedback, color);
                }
                ui.horizontal_wrapped(|ui| {
                    let feedback = self.practice.current_session().feedback.clone();
                    if !feedback.is_empty() && ui.button("Read feedback").clicked() {
                        self.speak(&feedback);
                    }
                    if ui.button("Next question").clicked() {
                        self.practice.next_question();
                    }
                });
            });

        if self.practice.current_session().show_steps {
            ui.add_space(10.0);
            ui.heading("Worked solution");
            let count = self
                .practice
                .current_session()
                .revealed_steps
                .max(1)
                .min(question.steps.len());
            for (index, step) in question.steps.iter().take(count).enumerate() {
                ui.group(|ui| {
                    ui.strong(format!("{}. {}", index + 1, step.title));
                    ui.label(R::new(&step.calculation).monospace());
                    ui.label(crate::appearance::paragraph(ui.ctx(), &step.explanation));
                });
            }
            if count < question.steps.len() {
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Next solution step").clicked() {
                        self.practice.current_session_mut().revealed_steps = count + 1;
                    }
                    if ui.button("Show all solution steps").clicked() {
                        self.practice.current_session_mut().revealed_steps = question.steps.len();
                    }
                });
            } else {
                ui.strong(format!("Answer: {}", question.answer.display()));
                ui.label(&question.check);
            }
        }

        ui.horizontal_wrapped(|ui| {
            if ui.button("Copy worked solution").clicked() {
                self.practice
                    .reveal_answer_with_pacing(self.settings.access.one_step);
                ui.ctx().copy_text(question.solution_text());
            }
            if ui.button("Explain in Learn").clicked() {
                self.practice
                    .reveal_answer_with_pacing(self.settings.access.one_step);
                self.tutor.load_practice_lesson(&question);
                self.tutor.set_pacing(self.settings.access.one_step);
                self.tab = Tab::Tutor;
                self.show_workspace = true;
            }
            if let Some(expression) = &question.calculator
                && ui.button("Copy calculator expression").clicked()
            {
                ui.ctx().copy_text(expression.clone());
            }
        });
        if self.settings.access.show_scores {
            let session = self.practice.current_session();
            ui.label(format!(
                "First-attempt score: {} correct / {} attempted for this topic and difficulty",
                session.progress.correct, session.progress.attempted
            ));
            let total = self.practice.total_progress_for_level();
            ui.label(format!(
                "{} total: {} correct / {} attempted this session",
                LEVELS[level], total.correct, total.attempted
            ));
            ui.label(
                R::new(
                    "Showing a solution counts as an attempt. Progress is kept for this session.",
                )
                .small()
                .color(palette.muted),
            );
            if ui
                .button("Reset practice progress")
                .on_hover_text("Reset this topic and difficulty only, then start a new question.")
                .clicked()
            {
                self.practice.reset_current_progress();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(
        app: &mut EqualApp,
        ctx: &egui::Context,
        width: f32,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        app.theme(ctx);
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 1800.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.practice_workspace(ui));
            },
        )
    }

    fn control(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
        let node = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("accessibility update")
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("Missing {label}"));
        let bounds = node.1.bounds().expect("control bounds");
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }

    fn click(app: &mut EqualApp, ctx: &egui::Context, label: &str) -> egui::FullOutput {
        let snapshot = frame(app, ctx, 800.0, vec![]);
        let pos = control(&snapshot, label);
        frame(
            app,
            ctx,
            800.0,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame(
            app,
            ctx,
            800.0,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
    }

    fn rendered_text(output: &egui::FullOutput) -> String {
        output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.text()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn hiding_scores_removes_progress_presentation_without_changing_attempts() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.practice.set_level(app.settings.level);
        app.practice.current_session_mut().response =
            app.practice.current_question().answer.display();
        app.practice.check_answer();
        let progress = app.practice.current_session().progress;
        assert_eq!(
            progress,
            super::super::Progress {
                attempted: 1,
                correct: 1
            }
        );
        app.settings.access.show_scores = false;
        frame(&mut app, &ctx, 800.0, vec![]);
        let output = frame(&mut app, &ctx, 800.0, vec![]);
        let text = rendered_text(&output);
        for hidden in [
            "First-attempt score",
            "total:",
            "Reset practice progress",
            "Showing a solution counts",
            "Question 1",
        ] {
            assert!(
                !text.contains(hidden),
                "Unexpected score presentation: {hidden}"
            );
        }
        assert!(text.contains("Correct."));
        assert_eq!(app.practice.current_session().progress, progress);
        app.settings.access.show_scores = true;
        let output = frame(&mut app, &ctx, 800.0, vec![]);
        assert!(rendered_text(&output).contains("First-attempt score: 1 correct / 1 attempted"));
        assert_eq!(app.practice.current_session().progress, progress);
    }

    #[test]
    fn hidden_help_preserves_answer_conventions_and_errors() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(
            Settings {
                level: 2,
                ..Settings::default()
            },
            None,
        );
        app.settings.access.show_help_text = false;
        app.settings.access.decimal_comma = true;
        app.practice.set_level(2);
        app.practice.select_topic(TopicId::Quadratics);
        app.practice.current_session_mut().response = "sin(1".into();
        app.practice.check_answer_with_locale(true);
        frame(&mut app, &ctx, 800.0, vec![]);
        let output = frame(&mut app, &ctx, 800.0, vec![]);
        let text = rendered_text(&output);
        assert!(text.contains("Separate answers with semicolons (;), in any order."));
        assert!(text.contains("Trigonometric expressions use radians."));
        assert!(text.contains("Use a decimal comma (1,5)."));
        assert!(!text.contains("separated by a comma"));
        assert!(text.contains(&app.practice.current_session().feedback));
        let description = topics(2)
            .find(|topic| topic.id == TopicId::Quadratics)
            .unwrap()
            .description;
        assert!(!text.contains(description));
        assert!(!text.contains("Fractions and constant expressions are accepted."));
    }

    #[test]
    fn practice_selection_and_answer_controls_fit_narrow_windows() {
        for level in 0..4 {
            for width in [320.0_f32, 480.0_f32] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = EqualApp::with_settings(
                    Settings {
                        level,
                        ..Settings::default()
                    },
                    None,
                );
                frame(&mut app, &ctx, width, vec![]);
                let output = frame(&mut app, &ctx, width, vec![]);
                let tree = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .expect("accessibility update");
                for label in [
                    "Practice topic",
                    "Basic",
                    "Standard",
                    "Challenge",
                    "Practice answer",
                    "Check answer",
                    "Next question",
                ] {
                    let node = tree
                        .nodes
                        .iter()
                        .find(|(_, node)| node.label() == Some(label))
                        .unwrap_or_else(|| panic!("Missing {label} at level {level}"));
                    let bounds = node.1.bounds().expect("control bounds");
                    assert!(
                        bounds.x0 >= 0.0 && bounds.x1 <= f64::from(width) + 1.0,
                        "{label} overflows {width}-point width at level {level}: {bounds:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn copying_a_worked_solution_counts_as_revealing_and_does_not_inflate_score() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        frame(&mut app, &ctx, 800.0, vec![]);
        let question = app.practice.current_question();
        let output = click(&mut app, &ctx, "Copy worked solution");
        assert!(output.platform_output.commands.iter().any(|command| matches!(command, egui::OutputCommand::CopyText(text) if text == &question.solution_text())));
        assert_eq!(app.practice.current_session().progress.attempted, 1);
        assert_eq!(app.practice.current_session().progress.correct, 0);
        click(&mut app, &ctx, "Copy worked solution");
        assert_eq!(app.practice.current_session().progress.attempted, 1);
        app.practice.current_session_mut().response = question.answer.display();
        click(&mut app, &ctx, "Check answer");
        assert_eq!(app.practice.current_session().progress.correct, 0);
        let output = frame(&mut app, &ctx, 800.0, vec![]);
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
                    && node.label() == Some("Correct."))
        );
    }

    #[test]
    fn elementary_answer_guidance_does_not_include_trigonometry() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(
            Settings {
                level: 0,
                ..Settings::default()
            },
            None,
        );
        frame(&mut app, &ctx, 480.0, vec![]);
        let output = frame(&mut app, &ctx, 480.0, vec![]);
        let text: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.text()),
                _ => None,
            })
            .collect();
        assert!(text.contains(&"Enter a number."));
        assert!(
            !text
                .iter()
                .any(|text| text.to_lowercase().contains("radian")
                    || text.to_lowercase().contains("trig"))
        );
    }

    #[test]
    fn matrix_givens_remain_accessible_when_optional_visual_aids_are_hidden() {
        for topic in [
            super::super::TopicId::Matrices,
            super::super::TopicId::Determinants,
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = EqualApp::with_settings(
                Settings {
                    level: 3,
                    ..Settings::default()
                },
                None,
            );
            app.practice.set_level(3);
            app.practice.select_topic(topic);
            app.settings.access.show_help_text = false;
            app.practice.current_session_mut().show_visual = false;
            frame(&mut app, &ctx, 480.0, vec![]);
            let output = frame(&mut app, &ctx, 480.0, vec![]);
            let tree = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("accessible givens");
            let Visual::Matrix { left, right, .. } = app.practice.current_question().visual else {
                panic!("matrix question must include its givens")
            };
            assert!(
                !tree
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Show visual aid")),
                "Essential givens must not have a hide control"
            );
            for (title, matrix) in
                std::iter::once(("Matrix A", left)).chain(right.map(|matrix| ("Matrix B", matrix)))
            {
                let heading = format!(
                    "{title}: {} rows, {} columns",
                    matrix.len(),
                    matrix[0].len()
                );
                assert!(
                    tree.nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(heading.as_str())
                            || node.value() == Some(heading.as_str())),
                    "Missing {heading}"
                );
                for (index, row) in matrix.iter().enumerate() {
                    let text = format!(
                        "Row {}: [ {} ]",
                        index + 1,
                        row.iter()
                            .map(|value| math::format(*value, 6))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    assert!(
                        tree.nodes
                            .iter()
                            .any(|(_, node)| node.label() == Some(text.as_str())
                                || node.value() == Some(text.as_str())),
                        "Missing matrix entries: {text}"
                    );
                }
            }
        }
    }
}
