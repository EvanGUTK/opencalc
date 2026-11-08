use crate::*;

struct GlossaryEntry {
    term: &'static str,
    topic: &'static str,
    meaning: &'static str,
    example: &'static str,
    aliases: &'static str,
    level: usize,
}

const GLOSSARY: &[GlossaryEntry] = &[
    GlossaryEntry {
        term: "Add",
        topic: "Arithmetic",
        meaning: "Put amounts together to find their total.",
        example: "3 + 2 = 5",
        aliases: "addition plus + sum",
        level: 0,
    },
    GlossaryEntry {
        term: "Subtract",
        topic: "Arithmetic",
        meaning: "Take an amount away, or find how far apart two amounts are.",
        example: "7 - 3 = 4",
        aliases: "subtraction minus − difference -",
        level: 0,
    },
    GlossaryEntry {
        term: "Equals",
        topic: "Arithmetic",
        meaning: "Both sides have the same value.",
        example: "4 + 1 = 5",
        aliases: "equal = equality",
        level: 0,
    },
    GlossaryEntry {
        term: "Multiply",
        topic: "Arithmetic",
        meaning: "Combine equal groups.",
        example: "3 groups of 4: 3 × 4 = 12",
        aliases: "multiplication times product * ×",
        level: 0,
    },
    GlossaryEntry {
        term: "Divide",
        topic: "Arithmetic",
        meaning: "Share an amount into equal groups, or count how many groups fit.",
        example: "12 shared into 3 equal groups: 12 ÷ 3 = 4",
        aliases: "division quotient / ÷",
        level: 0,
    },
    GlossaryEntry {
        term: "Place value",
        topic: "Arithmetic",
        meaning: "A digit's place tells you its value.",
        example: "In 24, the 2 means 2 tens, or 20.",
        aliases: "ones tens hundreds digit",
        level: 0,
    },
    GlossaryEntry {
        term: "Number line",
        topic: "Arithmetic",
        meaning: "A line showing numbers in order. Greater numbers are farther right.",
        example: "From 2, move 3 places right to reach 5.",
        aliases: "count forward backward",
        level: 0,
    },
    GlossaryEntry {
        term: "Fraction",
        topic: "Arithmetic",
        meaning: "A number written as one quantity divided by another. The bottom number cannot be zero.",
        example: "3/4 means 3 of 4 equal parts.",
        aliases: "numerator denominator /",
        level: 1,
    },
    GlossaryEntry {
        term: "Numerator",
        topic: "Arithmetic",
        meaning: "The top number in a fraction. It counts the parts being used.",
        example: "In 3/4, the numerator is 3.",
        aliases: "fraction top",
        level: 1,
    },
    GlossaryEntry {
        term: "Denominator",
        topic: "Arithmetic",
        meaning: "The bottom number in a fraction. It tells how many equal parts make one whole.",
        example: "In 3/4, the denominator is 4.",
        aliases: "fraction bottom",
        level: 1,
    },
    GlossaryEntry {
        term: "Decimal",
        topic: "Arithmetic",
        meaning: "A way to write place values smaller than one, using a decimal point or decimal comma.",
        example: "One half is 0.5, or 0,5 with a decimal comma.",
        aliases: "decimal point comma tenths hundredths locale",
        level: 1,
    },
    GlossaryEntry {
        term: "Percent",
        topic: "Arithmetic",
        meaning: "An amount out of one hundred.",
        example: "25% = 25/100 = 1/4",
        aliases: "percentage % per cent",
        level: 1,
    },
    GlossaryEntry {
        term: "Negative number",
        topic: "Arithmetic",
        meaning: "A number less than zero.",
        example: "-3 is three places left of 0 on a number line.",
        aliases: "minus sign − signed integer",
        level: 1,
    },
    GlossaryEntry {
        term: "Absolute value",
        topic: "Arithmetic",
        meaning: "A number's distance from zero, which is never negative.",
        example: "|-3| = 3; type abs(-3).",
        aliases: "abs magnitude | |",
        level: 1,
    },
    GlossaryEntry {
        term: "Power",
        topic: "Arithmetic",
        meaning: "An exponent tells how many times to use a number as a factor.",
        example: "2^3 = 2 × 2 × 2 = 8",
        aliases: "exponent squared cubed ^ ² ³",
        level: 1,
    },
    GlossaryEntry {
        term: "Square root",
        topic: "Arithmetic",
        meaning: "The nonnegative number that gives the original number when multiplied by itself.",
        example: "√9 = 3; type sqrt(9).",
        aliases: "root radical sqrt √",
        level: 1,
    },
    GlossaryEntry {
        term: "Order of operations",
        topic: "Arithmetic",
        meaning: "Work inside parentheses first, then powers, then multiplication and division from left to right, then addition and subtraction from left to right.",
        example: "2 + 3 × 4 = 14",
        aliases: "PEMDAS BODMAS BIDMAS parentheses brackets",
        level: 1,
    },
    GlossaryEntry {
        term: "Ratio",
        topic: "Arithmetic",
        meaning: "A comparison of two quantities using division.",
        example: "2 red counters to 3 blue counters is a ratio of 2:3.",
        aliases: "proportion rate :",
        level: 1,
    },
    GlossaryEntry {
        term: "Variable",
        topic: "Algebra",
        meaning: "A letter or symbol standing for a value that can be unknown or can change.",
        example: "In x + 3 = 7, x = 4.",
        aliases: "unknown x y z",
        level: 2,
    },
    GlossaryEntry {
        term: "Expression",
        topic: "Algebra",
        meaning: "Numbers, variables and operations that describe a value.",
        example: "2*x + 3 is an expression.",
        aliases: "formula evaluate",
        level: 1,
    },
    GlossaryEntry {
        term: "Equation",
        topic: "Algebra",
        meaning: "A statement that two expressions have the same value. Solving finds values that make the statement true.",
        example: "2*x + 3 = 11 has the solution x = 4.",
        aliases: "solve equals =",
        level: 2,
    },
    GlossaryEntry {
        term: "Coefficient",
        topic: "Algebra",
        meaning: "A number multiplying a variable or other expression.",
        example: "In 5*x, the coefficient of x is 5.",
        aliases: "factor term",
        level: 2,
    },
    GlossaryEntry {
        term: "Function",
        topic: "Algebra",
        meaning: "A rule that assigns exactly one output to each allowed input.",
        example: "For f(x) = 2*x, f(3) = 6.",
        aliases: "f(x) input output graph",
        level: 2,
    },
    GlossaryEntry {
        term: "Domain",
        topic: "Algebra",
        meaning: "The inputs allowed for a function or expression.",
        example: "For 1/x, x cannot be 0.",
        aliases: "undefined allowed input",
        level: 2,
    },
    GlossaryEntry {
        term: "Slope",
        topic: "Algebra",
        meaning: "Change in vertical position divided by change in horizontal position. A vertical line has no finite slope.",
        example: "A rise of 6 and a run of 3 gives slope 6/3 = 2.",
        aliases: "gradient rise run line",
        level: 2,
    },
    GlossaryEntry {
        term: "Intercept",
        topic: "Algebra",
        meaning: "A point where a graph crosses an axis.",
        example: "y = 2*x + 3 crosses the y-axis at (0, 3).",
        aliases: "x intercept y intercept axis",
        level: 2,
    },
    GlossaryEntry {
        term: "Quadratic",
        topic: "Algebra",
        meaning: "A polynomial whose highest power of the variable is 2.",
        example: "x^2 - 5*x + 6 = 0 has roots 2 and 3.",
        aliases: "parabola square roots polynomial",
        level: 2,
    },
    GlossaryEntry {
        term: "Mean",
        topic: "Statistics",
        meaning: "Add the values, then divide by how many values there are.",
        example: "The mean of 2, 4 and 9 is 15/3 = 5.",
        aliases: "average arithmetic data",
        level: 1,
    },
    GlossaryEntry {
        term: "Median",
        topic: "Statistics",
        meaning: "The middle value after sorting. For an even number of values, use the mean of the two middle values.",
        example: "The median of 1, 3 and 8 is 3.",
        aliases: "middle average data",
        level: 1,
    },
    GlossaryEntry {
        term: "Mode",
        topic: "Statistics",
        meaning: "A value that occurs most often. A data set can have more than one mode.",
        example: "The mode of 2, 2, 3 and 5 is 2.",
        aliases: "frequency most common data",
        level: 1,
    },
    GlossaryEntry {
        term: "Range",
        topic: "Statistics",
        meaning: "For a data set, the greatest value minus the least value. A function's range means its possible outputs.",
        example: "The data range of 2, 4 and 9 is 9 - 2 = 7.",
        aliases: "spread maximum minimum",
        level: 1,
    },
    GlossaryEntry {
        term: "Standard deviation",
        topic: "Statistics",
        meaning: "A measure of how spread out values are around their mean. Sample and population formulas differ.",
        example: "Equal values have a standard deviation of 0.",
        aliases: "stdev sigma σ sample population spread",
        level: 2,
    },
    GlossaryEntry {
        term: "Probability",
        topic: "Statistics",
        meaning: "A number from 0 to 1 describing how likely an event is.",
        example: "A fair coin has probability 1/2 of landing heads.",
        aliases: "chance likelihood random",
        level: 1,
    },
    GlossaryEntry {
        term: "Pi",
        topic: "Trigonometry",
        meaning: "The ratio of a circle's circumference to its diameter.",
        example: "π is about 3.14159; type pi.",
        aliases: "π circle circumference diameter",
        level: 2,
    },
    GlossaryEntry {
        term: "Degree",
        topic: "Trigonometry",
        meaning: "An angle unit. A full turn is 360 degrees.",
        example: "A right angle is 90°.",
        aliases: "degrees ° angle unit DEG",
        level: 2,
    },
    GlossaryEntry {
        term: "Radian",
        topic: "Trigonometry",
        meaning: "An angle unit based on a circle's radius. A full turn is 2*pi radians.",
        example: "pi radians = 180°.",
        aliases: "radians rad angle unit",
        level: 2,
    },
    GlossaryEntry {
        term: "Sine",
        topic: "Trigonometry",
        meaning: "In a right triangle, the opposite side divided by the hypotenuse for the chosen angle.",
        example: "sin(30°) = 1/2; calculator angle mode matters.",
        aliases: "sin opposite hypotenuse",
        level: 2,
    },
    GlossaryEntry {
        term: "Cosine",
        topic: "Trigonometry",
        meaning: "In a right triangle, the adjacent side divided by the hypotenuse for the chosen angle.",
        example: "cos(60°) = 1/2; calculator angle mode matters.",
        aliases: "cos adjacent hypotenuse",
        level: 2,
    },
    GlossaryEntry {
        term: "Tangent",
        topic: "Trigonometry",
        meaning: "In a right triangle, the opposite side divided by the adjacent side for the chosen angle.",
        example: "tan(45°) = 1; calculator angle mode matters.",
        aliases: "tan opposite adjacent",
        level: 2,
    },
    GlossaryEntry {
        term: "Inverse sine",
        topic: "Trigonometry",
        meaning: "Finds the principal angle whose sine is a given value. This is different from 1 divided by sine.",
        example: "asin(1/2) = 30° in degree mode.",
        aliases: "arcsin asin sin⁻¹ inverse trig",
        level: 2,
    },
    GlossaryEntry {
        term: "Limit",
        topic: "Calculus",
        meaning: "The value a function approaches as its input approaches a chosen point.",
        example: "As x approaches 2, x + 3 approaches 5.",
        aliases: "lim approaches →",
        level: 3,
    },
    GlossaryEntry {
        term: "Derivative",
        topic: "Calculus",
        meaning: "The instantaneous rate of change of a function, or the slope of its tangent line.",
        example: "For f(x) = x^2, f'(x) = 2*x.",
        aliases: "differentiate differentiation diff dy/dx d/dx slope",
        level: 3,
    },
    GlossaryEntry {
        term: "Integral",
        topic: "Calculus",
        meaning: "A definite integral measures signed accumulation over an interval.",
        example: "The integral of 2 from x = 0 to x = 3 is 6.",
        aliases: "integrate integration ∫ area accumulation",
        level: 3,
    },
    GlossaryEntry {
        term: "Antiderivative",
        topic: "Calculus",
        meaning: "A function whose derivative is the given function. Indefinite integrals include an arbitrary constant.",
        example: "An antiderivative of 2*x is x^2; the general answer is x^2 + C.",
        aliases: "indefinite integral constant C",
        level: 3,
    },
    GlossaryEntry {
        term: "Matrix",
        topic: "Matrices",
        meaning: "A rectangular arrangement of values in rows and columns.",
        example: "[1, 2; 3, 4] has 2 rows and 2 columns.",
        aliases: "matrices array row column",
        level: 3,
    },
    GlossaryEntry {
        term: "Matrix dimensions",
        topic: "Matrices",
        meaning: "The number of rows followed by the number of columns.",
        example: "A 2 × 3 matrix has 2 rows and 3 columns.",
        aliases: "shape size rows columns order",
        level: 3,
    },
    GlossaryEntry {
        term: "Determinant",
        topic: "Matrices",
        meaning: "A value calculated from a square matrix. A zero determinant means the matrix has no inverse.",
        example: "For [a, b; c, d], the determinant is a*d - b*c.",
        aliases: "det |A| square singular",
        level: 3,
    },
    GlossaryEntry {
        term: "Identity matrix",
        topic: "Matrices",
        meaning: "A square matrix with 1 on its main diagonal and 0 elsewhere. It leaves a compatible matrix unchanged when multiplied.",
        example: "The 2 × 2 identity is [1, 0; 0, 1].",
        aliases: "I identity diagonal",
        level: 3,
    },
    GlossaryEntry {
        term: "Inverse matrix",
        topic: "Matrices",
        meaning: "A matrix that multiplies with the original square matrix to give the identity matrix. Some matrices have no inverse.",
        example: "A * inverse(A) = I when the inverse exists.",
        aliases: "invert A⁻¹ nonsingular",
        level: 3,
    },
    GlossaryEntry {
        term: "Transpose",
        topic: "Matrices",
        meaning: "Swap the rows and columns of a matrix.",
        example: "The row [1, 2] becomes a column with 1 above 2.",
        aliases: "transposed Aᵀ row column",
        level: 3,
    },
];

fn glossary_entries(query: &str, level: usize, all_topics: bool) -> Vec<&'static GlossaryEntry> {
    let query = query.trim().to_lowercase();
    GLOSSARY
        .iter()
        .filter(|entry| {
            if query.is_empty() {
                return all_topics || entry.level <= level;
            }
            [
                entry.term,
                entry.topic,
                entry.meaning,
                entry.example,
                entry.aliases,
            ]
            .iter()
            .any(|text| text.to_lowercase().contains(&query))
        })
        .collect()
}

fn help_window(
    ctx: &egui::Context,
    title: &'static str,
    preferred_width: f32,
) -> egui::Window<'static> {
    let viewport = ctx.content_rect();
    let width = (viewport.width() - 24.0).max(40.0).min(preferred_width);
    egui::Window::new(title)
        .title_bar(false)
        .default_width(width)
        .min_width(40.0)
        .max_width(width)
        .min_height(40.0)
        .max_height((viewport.height() - 24.0).max(40.0))
        .constrain_to(viewport.shrink(4.0))
        .vscroll(true)
}

fn shortcut(ui: &mut egui::Ui, keys: &str, action: &str) {
    ui.label(crate::appearance::paragraph(
        ui.ctx(),
        format!("{keys}: {action}"),
    ));
}

impl EqualApp {
    pub(crate) fn accessibility_help_windows(&mut self, ctx: &egui::Context) {
        if (self.a11y.show_glossary || self.a11y.show_shortcuts)
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            if self.a11y.show_glossary {
                self.a11y.show_glossary = false;
            } else {
                self.a11y.show_shortcuts = false;
            }
        }
        let palette = crate::appearance::palette(ctx);
        if self.a11y.show_shortcuts {
            help_window(ctx, "Keyboard help", 620.0).show(ctx, |ui| {
                ui.heading("Keyboard help");
                if ui.button("Close keyboard help").clicked() { self.a11y.show_shortcuts = false; }
                ui.label(crate::appearance::paragraph(ctx, "Tab moves to the next control. Shift+Tab moves back. Space activates a focused button or changes a checkbox. Use arrow keys in lists, sliders and menus."));
                ui.separator();
                shortcut(ui, "F1", "open this help");
                shortcut(ui, "Ctrl+,", "open Settings");
                shortcut(ui, "Ctrl+G", "open the math glossary");
                shortcut(ui, "Ctrl+H", "open History");
                shortcut(ui, "Alt+1", "Calculator");
                shortcut(ui, "Alt+2", "Graphing");
                shortcut(ui, "Alt+3", "Matrices");
                shortcut(ui, "Alt+4", "Learn");
                shortcut(ui, "Alt+5", "Tools");
                shortcut(ui, "Alt+6", "Practice");
                ui.label(crate::appearance::paragraph(ctx, "Workspaces follow the chosen learning level. Change the learning level in Settings to use additional workspaces."));
                shortcut(ui, "Enter", "calculate when the calculator input is focused; check a focused practice answer");
                shortcut(ui, "Ctrl+C", "copy the calculator answer when a text field does not have focus");
                shortcut(ui, "Ctrl+Z", "undo calculator input when a text field does not have focus");
                shortcut(ui, "Ctrl+Shift+Z or Ctrl+Y", "redo calculator input when a text field does not have focus");
                ui.label(crate::appearance::paragraph(ctx, "Text fields keep their usual editing and clipboard shortcuts."));
                shortcut(ui, "Ctrl+Shift+Space", "stop speech and graph audio");
                shortcut(ui, "Alt+Up / Alt+Down", "enable and move the reading guide");
                shortcut(ui, "Escape", "close this help or the glossary; clear calculator input when no dialog or editor has focus");
                ui.separator();
                ui.strong("Pointer and switch access");
                ui.label(crate::appearance::paragraph(ctx, "Settings includes larger controls, key spacing, a left-handed layout, a repeated-input guard and pointer dwell. Dwell activates a key after you hold the pointer over it; move away to cancel. Move off the key before activating it again."));
                ui.label(crate::appearance::paragraph(ctx, "Automatic switch scanning is available in the accessible calculator layout. Choose Start scanning. Space chooses a row, then Space chooses a key. Escape pauses scanning. Use Pause scanning or Restart scanning to change course. Scanning starts paused and pauses when a dialog opens."));
                ui.strong("Voice control and sound");
                ui.label(crate::appearance::paragraph(ctx, "Use your system voice-control tool with the visible button names. Word labels in Settings can make symbol keys easier to name. Equal does not require a microphone or spoken answers. Speech and graph audio are optional; answers and errors remain available as text."));
                ui.label(crate::appearance::paragraph(ctx, "Assistive technology behavior varies by setup. Use the assistive technology test in Settings to record what works and what needs a change.").color(palette.muted));
            });
        }
        if self.a11y.show_glossary {
            help_window(ctx, "Math glossary", 620.0).show(ctx, |ui| {
                ui.heading("Math glossary");
                if ui.button("Close math glossary").clicked() { self.a11y.show_glossary = false; }
                ui.label("Search math glossary");
                let response = ui.add(egui::TextEdit::singleline(&mut self.a11y.glossary_search)
                    .desired_width(ui.available_width()).char_limit(100).hint_text("Word, topic or symbol"));
                response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Search math glossary"));
                let mut all_topics = ctx.data(|data| data.get_temp::<bool>(egui::Id::new("glossary-all-topics"))).unwrap_or(false);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Clear search").clicked() { self.a11y.glossary_search.clear(); }
                    if ui.checkbox(&mut all_topics, "All topics").changed() {
                        ctx.data_mut(|data| data.insert_temp(egui::Id::new("glossary-all-topics"), all_topics));
                    }
                });
                if self.settings.access.show_help_text {
                    ui.label(crate::appearance::paragraph(ctx, "With no search, entries follow your learning level. Searching includes every topic. Definitions are stored in the app and work offline."));
                }
                if self.settings.access.decimal_comma {
                    ui.label(crate::appearance::paragraph(ctx, "For calculator entry, use a decimal comma (1,5) and semicolons (;) between function inputs. Glossary examples may show standard mathematical comma lists."));
                }
                let entries = glossary_entries(&self.a11y.glossary_search, self.settings.level, all_topics);
                if entries.is_empty() { ui.label("No matching entries. Try another word, topic or symbol."); }
                for entry in entries {
                    ui.separator();
                    ui.strong(format!("{} · {}", entry.term, entry.topic));
                    ui.label(crate::appearance::paragraph(ctx, entry.meaning));
                    ui.label(crate::appearance::paragraph(ctx, format!("Example: {}", entry.example)));
                    ui.horizontal_wrapped(|ui| {
                        ui.push_id(entry.term, |ui| {
                            let definition = format!("{}. {} Example: {}", entry.term, entry.meaning, entry.example);
                            if ui.button("Read definition").on_hover_text(format!("Read {}", entry.term)).clicked() { self.speak(&definition); }
                            if ui.button("Copy definition").on_hover_text(format!("Copy {}", entry.term)).clicked() {
                                ui.ctx().copy_text(definition);
                                self.message = format!("{} definition copied.", entry.term);
                            }
                        });
                    });
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glossary_search_finds_terms_topics_symbols_and_examples() {
        for (query, expected) in [
            ("DERIVATIVE", "Derivative"),
            ("Matrices", "Determinant"),
            ("√", "Square root"),
            ("denominator", "Fraction"),
            ("dy/dx", "Derivative"),
        ] {
            assert!(
                glossary_entries(query, 0, false)
                    .iter()
                    .any(|entry| entry.term == expected),
                "{query}"
            );
        }
        assert!(glossary_entries("zzzz-unknown", 3, true).is_empty());
    }

    #[test]
    fn default_glossary_follows_learning_level_without_blocking_search() {
        let early = glossary_entries("", 0, false);
        assert!(early.iter().any(|entry| entry.term == "Add"));
        assert!(early.iter().all(|entry| entry.level == 0));
        assert_eq!(glossary_entries("", 0, true).len(), GLOSSARY.len());
        assert!(
            glossary_entries("integral", 0, false)
                .iter()
                .any(|entry| entry.term == "Integral")
        );
    }
}
