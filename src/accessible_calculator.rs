//! Native large-target calculator controls and opt-in switch/pointer access.
use crate::accessibility::{CalculatorLayout, ScanMode};
use crate::*;
use egui::{Id, Key, Vec2};
use std::time::Duration;

const MINIMUM_TARGET: f32 = 40.0;
const MAXIMUM_TARGET: f32 = 80.0;
const PAUSE_SCAN: &str = "pause-scan";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ScanStage {
    #[default]
    Rows,
    Keys,
}

#[derive(Clone, Copy)]
struct KeyCell {
    id: Id,
    enabled: bool,
}

#[derive(Default)]
pub(crate) struct InputAccess {
    scan_running: bool,
    stage: ScanStage,
    row: usize,
    column: usize,
    next_scan_at: Option<f64>,
    rows: Vec<Vec<&'static str>>,
    cells: Vec<Vec<KeyCell>>,
    scan_controls: Vec<Id>,
    second: bool,
    alpha: bool,
    automatic_enabled: bool,
    focus_start: bool,
    last_scan_announcement: String,
    dwell_key: Option<&'static str>,
    dwell_since: Option<f64>,
    dwell_fired: bool,
    dwell_wait_for_leave: bool,
    last_activation: Option<f64>,
    notice: String,
}

impl InputAccess {
    pub(crate) fn is_running(&self) -> bool {
        self.scan_running
    }

    fn owns_focus(&self, focused: Option<Id>) -> bool {
        focused.is_some_and(|focused| {
            self.scan_controls.contains(&focused)
                || self.cells.iter().flatten().any(|cell| cell.id == focused)
        })
    }

    pub(crate) fn pause(&mut self) {
        let was_running = self.scan_running;
        self.scan_running = false;
        if was_running {
            self.focus_start = self.automatic_enabled;
        }
        self.stage = ScanStage::Rows;
        self.column = 0;
        self.next_scan_at = None;
    }

    fn start(&mut self, now: f64, interval_ms: u64) {
        if self.rows.is_empty() || !now.is_finite() {
            return;
        }
        self.scan_running = true;
        self.focus_start = false;
        self.stage = ScanStage::Rows;
        self.row = 0;
        self.column = 0;
        self.next_scan_at = Some(now + scan_interval(interval_ms));
        self.last_scan_announcement.clear();
        self.notice.clear();
        self.suspend_dwell();
    }

    fn suspend_dwell(&mut self) {
        self.dwell_key = None;
        self.dwell_since = None;
        self.dwell_fired = false;
        self.dwell_wait_for_leave = true;
    }

    fn suspend(&mut self) {
        if self.scan_running {
            self.notice = "Scanning paused. Choose Start scanning when ready.".into();
        }
        self.pause();
        self.suspend_dwell();
    }

    fn update_rows(&mut self, rows: Vec<Vec<&'static str>>) {
        if self.rows != rows {
            if self.scan_running {
                self.notice = "Scanning paused because the keypad layout changed.".into();
            }
            self.pause();
            self.row = 0;
            self.column = 0;
            self.rows = rows;
            self.cells.clear();
            self.suspend_dwell();
        }
    }

    fn tick(&mut self, now: f64, interval_ms: u64) {
        if !self.scan_running || self.rows.is_empty() || !now.is_finite() {
            return;
        }
        let deadline = self
            .next_scan_at
            .get_or_insert_with(|| now + scan_interval(interval_ms));
        if now < *deadline {
            return;
        }
        // Advance once after a delayed frame instead of racing past several choices.
        match self.stage {
            ScanStage::Rows => self.row = (self.row + 1) % self.rows.len(),
            ScanStage::Keys => {
                self.column = (self.column + 1) % self.rows[self.row].len();
            }
        }
        self.next_scan_at = Some(now + scan_interval(interval_ms));
    }

    fn select(&mut self, now: f64, interval_ms: u64) -> Option<&'static str> {
        if !self.scan_running || self.rows.is_empty() {
            return None;
        }
        self.row = self.row.min(self.rows.len() - 1);
        self.next_scan_at = Some(now + scan_interval(interval_ms));
        match self.stage {
            ScanStage::Rows => {
                self.stage = ScanStage::Keys;
                self.column = 0;
                None
            }
            ScanStage::Keys => {
                let key = self.rows[self.row][self.column.min(self.rows[self.row].len() - 1)];
                self.stage = ScanStage::Rows;
                self.column = 0;
                Some(key)
            }
        }
    }

    fn scan_description(&self) -> String {
        if !self.scan_running || self.rows.is_empty() {
            return "Scanning paused. Start scanning, then press Space to choose a row and a key."
                .into();
        }
        let row = self.row.min(self.rows.len() - 1);
        match self.stage {
            ScanStage::Rows => format!(
                "Row {} of {}: {}. Space chooses this row.",
                row + 1,
                self.rows.len(),
                self.rows[row]
                    .iter()
                    .map(|key| effective_key_name(key, self.second, self.alpha))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ScanStage::Keys => format!(
                "Row {}, key {} of {}: {}. Space activates this key.",
                row + 1,
                self.column + 1,
                self.rows[row].len(),
                effective_key_name(
                    self.rows[row][self.column.min(self.rows[row].len() - 1)],
                    self.second,
                    self.alpha
                )
            ),
        }
    }

    fn spoken_selection(&self) -> String {
        if !self.scan_running || self.rows.is_empty() {
            return "Scanning paused".into();
        }
        let row = self.row.min(self.rows.len() - 1);
        match self.stage {
            ScanStage::Rows => format!(
                "Row {}. {}",
                row + 1,
                self.rows[row]
                    .iter()
                    .map(|key| effective_key_name(key, self.second, self.alpha))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ScanStage::Keys => effective_key_name(
                self.rows[row][self.column.min(self.rows[row].len() - 1)],
                self.second,
                self.alpha,
            )
            .into(),
        }
    }

    fn accept_activation(&mut self, now: f64, guard_ms: u64) -> bool {
        if !now.is_finite() {
            self.notice = "The input timer is unavailable. Use the keyboard controls.".into();
            return false;
        }
        let guard = guard_ms.min(1500) as f64 / 1000.0;
        if let Some(previous) = self.last_activation
            && now >= previous
            && now - previous < guard
        {
            self.notice = "Repeated activation ignored. The input guard is active.".into();
            return false;
        }
        self.last_activation = Some(now);
        self.notice.clear();
        true
    }

    fn dwell_candidate(
        &mut self,
        hovered: Option<&'static str>,
        now: f64,
        dwell_ms: u64,
        pointer_down: bool,
    ) -> Option<&'static str> {
        if hovered.is_none() {
            self.dwell_key = None;
            self.dwell_since = None;
            self.dwell_fired = false;
            self.dwell_wait_for_leave = false;
            return None;
        }
        if dwell_ms == 0 || !now.is_finite() || self.dwell_wait_for_leave {
            return None;
        }
        if self.dwell_key != hovered {
            self.dwell_key = hovered;
            self.dwell_since = Some(now);
            self.dwell_fired = false;
        }
        if pointer_down {
            self.dwell_fired = true;
            return None;
        }
        let delay = dwell_ms.clamp(600, 4000) as f64 / 1000.0;
        if !self.dwell_fired && now - self.dwell_since.unwrap_or(now) >= delay {
            hovered
        } else {
            None
        }
    }

    fn focused_neighbor(&self, focused: Id, direction: Key) -> Option<Id> {
        let (row, column) = self.cells.iter().enumerate().find_map(|(row, cells)| {
            cells
                .iter()
                .position(|cell| cell.id == focused)
                .map(|column| (row, column))
        })?;
        let mut next_row = row;
        let mut next_column = column;
        for _ in 0..self.cells.iter().map(Vec::len).sum::<usize>() {
            match direction {
                Key::ArrowLeft => {
                    next_column =
                        (next_column + self.cells[next_row].len() - 1) % self.cells[next_row].len();
                }
                Key::ArrowRight => {
                    next_column = (next_column + 1) % self.cells[next_row].len();
                }
                Key::ArrowUp => {
                    next_row = (next_row + self.cells.len() - 1) % self.cells.len();
                    next_column = next_column.min(self.cells[next_row].len() - 1);
                }
                Key::ArrowDown => {
                    next_row = (next_row + 1) % self.cells.len();
                    next_column = next_column.min(self.cells[next_row].len() - 1);
                }
                _ => return None,
            }
            let next = self.cells[next_row][next_column];
            if next.enabled {
                return Some(next.id);
            }
        }
        None
    }
}

fn scan_interval(interval_ms: u64) -> f64 {
    interval_ms.clamp(600, 5000) as f64 / 1000.0
}

fn keypad_rows(
    level: usize,
    columns: usize,
    left_handed: bool,
    scan: bool,
) -> Vec<Vec<&'static str>> {
    let mut rows: Vec<Vec<&'static str>> = vec![
        vec!["7", "8", "9", "÷"],
        vec!["4", "5", "6", "×"],
        vec!["1", "2", "3", "−"],
        vec!["0", ".", "±", "+"],
        vec!["clear", "backspace", "enter"],
        vec![
            "cursor left",
            "previous calculation",
            "cursor end",
            "cursor right",
        ],
        vec!["undo", "redo", "on", "off"],
        vec!["MRC", "M-", "M+", "%"],
        vec!["√"],
    ];
    if level >= 1 {
        rows.extend([
            vec!["2nd", "mode", "fraction", "convert"],
            vec!["x²", "1/x", "^", "π"],
            vec!["sin", "cos", "tan", "Ans"],
            vec!["log", "ln", "(", ")"],
            vec!["sto", "data", "stat", "delete_forward"],
        ]);
    }
    if level >= 2 {
        rows.extend([
            vec!["y=", "window", "zoom", "trace", "graph"],
            vec!["alpha", "x,t", "math", "apps"],
            vec!["prgm", "vars", ","],
        ]);
    }
    if left_handed {
        for row in &mut rows {
            if row
                .last()
                .is_some_and(|key| matches!(*key, "÷" | "×" | "−" | "+" | "enter"))
            {
                row.rotate_right(1);
            }
        }
    }
    let mut result = Vec::new();
    if scan {
        result.push(vec![PAUSE_SCAN]);
    }
    for row in rows {
        result.extend(row.chunks(columns.max(1)).map(<[_]>::to_vec));
    }
    result
}

fn key_name(key: &str) -> &str {
    match key {
        "÷" => "Divide",
        "×" => "Multiply",
        "−" => "Subtract",
        "+" => "Add",
        "enter" => "Calculate answer",
        "clear" => "Clear entry",
        "backspace" => "Backspace",
        "delete_forward" => "Delete at cursor",
        "±" => "Change sign",
        "." => "Decimal point",
        "√" => "Square root",
        "x²" => "Square",
        "1/x" => "Reciprocal",
        "^" => "Power",
        "π" => "Pi",
        "%" => "Percent",
        "fraction" => "Enter fraction",
        "convert" => "Fraction or decimal",
        "MRC" => "Recall or clear memory",
        "M-" => "Subtract from memory",
        "M+" => "Add to memory",
        "2nd" => "Second functions",
        "alpha" => "Alphabet keys",
        "mode" => "Calculator mode",
        "y=" => "Graph functions",
        "window" => "Graph window",
        "zoom" => "Graph zoom",
        "trace" => "Graph trace",
        "graph" => "Draw graph",
        "x,t" => "Variable x",
        "math" => "Math functions",
        "apps" => "Calculator apps",
        "prgm" => "Programs",
        "vars" => "Variables",
        "sto" => "Store in variable",
        "data" => "Data lists",
        "stat" => "Statistics",
        "Ans" => "Last answer",
        "on" => "Power on",
        "off" => "Power off",
        "undo" => "Undo editing",
        "redo" => "Redo editing",
        "sin" => "Sine",
        "cos" => "Cosine",
        "tan" => "Tangent",
        "log" => "Logarithm base ten",
        "ln" => "Natural logarithm",
        "(" => "Open parenthesis",
        ")" => "Close parenthesis",
        "," => "Comma",
        "cursor left" => "Calculator left",
        "cursor right" => "Calculator right",
        "previous calculation" => "Calculator up or previous entry",
        "cursor end" => "Calculator down or end of entry",
        PAUSE_SCAN => "Pause scanning",
        _ => key,
    }
}

fn effective_key_name(key: &str, second: bool, alpha: bool) -> &str {
    if matches!(key, "2nd" | PAUSE_SCAN) {
        return key_name(key);
    }
    if alpha {
        let letter = match key {
            "math" => "A",
            "apps" => "B",
            "prgm" => "C",
            "1/x" => "D",
            "sin" => "E",
            "cos" => "F",
            "tan" => "G",
            "^" => "H",
            "x²" => "I",
            "," => "J",
            "(" => "K",
            ")" => "L",
            "÷" => "M",
            "log" => "N",
            "7" => "O",
            "8" => "P",
            "9" => "Q",
            "×" => "R",
            "ln" => "S",
            "4" => "T",
            "5" => "U",
            "6" => "V",
            "−" => "W",
            "sto" => "X",
            "1" => "Y",
            "2" => "Z",
            _ => "",
        };
        if !letter.is_empty() {
            return letter;
        }
    }
    if second {
        return match key {
            "alpha" => "Lock alphabet keys",
            "mode" => "Return to calculator",
            "graph" => "Value table",
            "window" => "Table settings",
            "zoom" => "Graph format",
            "trace" => "Graph analysis",
            "y=" | "stat" => "Statistics",
            "apps" => "Calculator mode",
            "1/x" | "×" | "−" => "Matrices",
            "vars" => "Distributions",
            "prgm" => "Graph drawing",
            "math" => "Comparison tests",
            "+" => "Calculator memory",
            "0" => "Function catalog",
            "on" => "Power off",
            "enter" => "Recall previous entry",
            "sto" => "Recall variable",
            "sin" => "Inverse sine",
            "cos" => "Inverse cosine",
            "tan" => "Inverse tangent",
            "x²" => "Square root",
            "log" => "Ten to a power",
            "ln" => "Exponential",
            "^" => "Pi",
            "÷" => "Euler's number",
            "±" => "Last answer",
            "," => "Scientific exponent",
            "." => "Imaginary unit",
            "x,t" => "Transfer calculator data",
            "1" => "List one",
            "2" => "List two",
            "3" => "List three",
            "4" => "List four",
            "5" => "List five",
            "6" => "List six",
            "7" => "Sequence u",
            "8" => "Sequence v",
            "9" => "Sequence w",
            "(" => "Open list",
            ")" => "Close list",
            _ => key_name(key),
        };
    }
    key_name(key)
}

fn printed_key(key: &str, words: bool) -> &str {
    if words {
        return key_name(key);
    }
    match key {
        "enter" => "=",
        "backspace" => "Backspace",
        "delete_forward" => "Delete",
        "fraction" => "n/d",
        "convert" => "F ↔ D",
        "x,t" => "x",
        "cursor left" => "Left",
        "cursor right" => "Right",
        "previous calculation" => "Up / previous",
        "cursor end" => "Down / end",
        PAUSE_SCAN => "Pause scanning",
        _ => key,
    }
}

impl EqualApp {
    fn access_input_blocked(&self, ctx: &egui::Context) -> bool {
        self.access_overlay_open() || egui::Popup::is_any_open(ctx)
    }

    /// Call before global calculator shortcuts and before drawing native buttons.
    pub(crate) fn handle_access_input(&mut self, ctx: &egui::Context) -> bool {
        self.input_access.second = self.second;
        self.input_access.alpha = self.device.alpha;
        if self.settings.access.layout != CalculatorLayout::Accessible
            || self.tab != Tab::Calculator
            || self.access_input_blocked(ctx)
        {
            self.input_access.suspend();
            return false;
        }
        if self.settings.access.scan_mode != ScanMode::Automatic {
            self.input_access.pause();
        }
        if ctx.wants_keyboard_input()
            && !self
                .input_access
                .owns_focus(ctx.memory(|memory| memory.focused()))
        {
            self.input_access.suspend();
            return false;
        }
        let now = ctx.input(|input| input.time);
        if self.input_access.is_running() {
            if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape)) {
                self.input_access.pause();
                self.input_access.notice = "Scanning paused.".into();
                return true;
            }
            let choose = ctx.input(|input| {
                input.events.iter().any(|event| {
                    matches!(event, egui::Event::Key { key: Key::Space, pressed: true, repeat: false, modifiers, .. } if *modifiers == egui::Modifiers::NONE)
                })
            });
            let consumed =
                ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Space));
            if choose {
                if let Some(key) = self
                    .input_access
                    .select(now, self.settings.access.scan_interval_ms)
                {
                    self.activate_access_key(ctx, key);
                }
            } else if self.settings.access.speak_focus && self.speech.is_active() {
                self.input_access.next_scan_at =
                    Some(now + scan_interval(self.settings.access.scan_interval_ms));
            } else {
                self.input_access
                    .tick(now, self.settings.access.scan_interval_ms);
            }
            ctx.request_repaint_after(Duration::from_millis(50));
            self.announce_scan_selection();
            return consumed;
        }
        if let Some(focused) = ctx.memory(|memory| memory.focused()) {
            for direction in [
                Key::ArrowLeft,
                Key::ArrowRight,
                Key::ArrowUp,
                Key::ArrowDown,
            ] {
                if ctx.input(|input| input.key_pressed(direction))
                    && let Some(next) = self.input_access.focused_neighbor(focused, direction)
                {
                    ctx.input_mut(|input| {
                        input.consume_key(egui::Modifiers::NONE, direction);
                    });
                    ctx.memory_mut(|memory| {
                        // egui queues arrow traversal before events reach this handler.
                        // Cancel it so our grid move is not applied a second time at end_pass.
                        memory.move_focus(egui::FocusDirection::None);
                        memory.request_focus(next);
                    });
                    return true;
                }
            }
        }
        false
    }

    fn announce_scan_selection(&mut self) {
        let description = self.input_access.scan_description();
        if description != self.input_access.last_scan_announcement {
            if self.settings.access.speak_focus && self.speech.is_active() {
                return;
            }
            self.input_access.last_scan_announcement = description;
            if self.settings.access.speak_focus {
                self.speak(&self.input_access.spoken_selection());
            }
        }
    }

    /// Shared pointer/key activation gate; the controller itself retains all calculator behavior.
    pub(crate) fn activate_access_key(&mut self, ctx: &egui::Context, key: &str) {
        if self.access_input_blocked(ctx) {
            self.input_access.suspend();
            return;
        }
        if key == PAUSE_SCAN {
            self.input_access.pause();
            self.input_access.notice = "Scanning paused.".into();
            return;
        }
        let now = ctx.input(|input| input.time);
        if !self
            .input_access
            .accept_activation(now, self.settings.access.guard_ms)
        {
            return;
        }
        // Announce first, so a key label cannot cancel the answer spoken by Enter.
        if self.settings.access.speak_keys && !(key == "enter" && self.settings.speech) {
            self.speak(effective_key_name(key, self.second, self.device.alpha));
        }
        self.press_device_key(ctx, key);
        self.input_access.second = self.second;
        self.input_access.alpha = self.device.alpha;
    }

    pub(crate) fn accessible_calculator(&mut self, ui: &mut egui::Ui) {
        let palette = crate::appearance::palette(ui.ctx());
        self.input_access.second = self.second;
        self.input_access.alpha = self.device.alpha;
        let blocked = self.access_input_blocked(ui.ctx()) || !ui.is_enabled();
        if blocked {
            self.input_access.suspend();
        }
        let automatic = self.settings.access.scan_mode == ScanMode::Automatic;
        if automatic != self.input_access.automatic_enabled {
            self.input_access.automatic_enabled = automatic;
            self.input_access.focus_start = automatic;
            self.input_access.pause();
        }
        let target = self
            .settings
            .access
            .target_size
            .clamp(MINIMUM_TARGET, MAXIMUM_TARGET);
        let gap = self.settings.access.key_spacing.clamp(4.0, 24.0);
        let width = ui.available_width().max(MINIMUM_TARGET);
        let columns = (((width + gap) / (target + gap)).floor() as usize).clamp(1, 4);
        self.input_access.update_rows(keypad_rows(
            self.settings.level,
            columns,
            self.settings.access.left_handed,
            automatic,
        ));
        let rows = self.input_access.rows.clone();
        let now = ui.input(|input| input.time);
        if automatic {
            self.input_access.scan_controls.clear();
            ui.horizontal_wrapped(|ui| {
                let toggle = ui.button(if self.input_access.scan_running {
                    "Pause scanning"
                } else {
                    "Start scanning"
                });
                self.input_access.scan_controls.push(toggle.id);
                if self.input_access.focus_start && !blocked {
                    toggle.request_focus();
                    self.input_access.focus_start = false;
                }
                if toggle.clicked() && !blocked {
                    if self.input_access.scan_running {
                        self.input_access.pause();
                        self.input_access.notice = "Scanning paused.".into();
                    } else {
                        self.input_access
                            .start(now, self.settings.access.scan_interval_ms);
                    }
                }
                let restart = ui.button("Restart scanning");
                self.input_access.scan_controls.push(restart.id);
                if restart.clicked() && !blocked {
                    self.input_access
                        .start(now, self.settings.access.scan_interval_ms);
                }
            });
            self.announce_scan_selection();
            access_status(ui, &self.input_access.scan_description());
        }
        if !self.input_access.notice.is_empty() {
            access_status(ui, &self.input_access.notice);
        }
        if self.settings.access.dwell_ms > 0 && self.settings.access.show_help_text {
            ui.label(format!(
                "Pointer dwell: hold over a key for {:.1} seconds. Move away before repeating.",
                self.settings.access.dwell_ms.clamp(600, 4000) as f64 / 1000.0
            ));
        }
        egui::ScrollArea::vertical()
            .id_salt("accessible-calculator-scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
                ui.visuals_mut().override_text_color = None;
                ui.visuals_mut().widgets.noninteractive.fg_stroke.color = palette.ink;
                let display_height = (ui.available_width() * 0.48).clamp(190.0, 310.0);
                let (display, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), display_height),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(display, 4.0_f32, palette.paper);
                self.device_display(ui, display.shrink(6.0_f32), 1.0);
                let mut cells = Vec::new();
                let mut clicked = None;
                let mut hovered = None;
                let mut hovered_rect = None;
                for (row_index, row) in rows.iter().enumerate() {
                    let key_width = ((ui.available_width()
                        - gap * (columns.saturating_sub(1)) as f32)
                        / columns as f32)
                        .max(MINIMUM_TARGET);
                    let mut row_cells = Vec::new();
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for (column, &key) in row.iter().enumerate() {
                            let scan_selected = self.input_access.scan_running
                                && self.input_access.row == row_index
                                && (self.input_access.stage == ScanStage::Rows
                                    || self.input_access.column == column);
                            let modifier = (key == "2nd" && self.second)
                                || (key == "alpha" && self.device.alpha);
                            let selected = scan_selected || modifier;
                            let enabled =
                                !blocked && (key != PAUSE_SCAN || self.input_access.scan_running);
                            let name = effective_key_name(key, self.second, self.device.alpha);
                            let label = match key {
                                "cursor left" => "Left",
                                "cursor right" => "Right",
                                "previous calculation" => "Up",
                                "cursor end" => "Down",
                                _ if self.settings.access.word_labels => name,
                                _ => printed_key(key, false),
                            };
                            let response = ui
                                .push_id(("accessible-key", key), |ui| {
                                    ui.add_enabled_ui(enabled, |ui| {
                                        ui.add_sized(
                                            Vec2::new(key_width, target),
                                            egui::Button::new(
                                                R::new(label)
                                                    .size(16.0 * self.settings.access.font_scale)
                                                    .color(if selected {
                                                        palette.selected_text
                                                    } else {
                                                        palette.ink
                                                    }),
                                            )
                                            .wrap()
                                            .min_size(Vec2::new(key_width, target))
                                            .fill(if selected {
                                                palette.accent
                                            } else {
                                                palette.paper
                                            })
                                            .stroke(
                                                egui::Stroke::new(
                                                    if selected { 3.0_f32 } else { 1.0_f32 },
                                                    if selected {
                                                        palette.accent
                                                    } else {
                                                        palette.border
                                                    },
                                                ),
                                            ),
                                        )
                                    })
                                    .inner
                                })
                                .inner;
                            response.widget_info(|| {
                                if matches!(key, "2nd" | "alpha") {
                                    egui::WidgetInfo::selected(
                                        egui::WidgetType::Checkbox,
                                        enabled,
                                        modifier,
                                        name,
                                    )
                                } else {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        enabled,
                                        name,
                                    )
                                }
                            });
                            if response.has_focus() {
                                ui.painter().rect_stroke(
                                    response.rect.expand(2.0_f32),
                                    4.0_f32,
                                    egui::Stroke::new(3.0_f32, palette.ink),
                                    egui::StrokeKind::Outside,
                                );
                                response.scroll_to_me(None);
                            }
                            if scan_selected && self.input_access.stage == ScanStage::Keys {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if response.gained_focus()
                                && self.settings.access.speak_focus
                                && !self.input_access.scan_running
                            {
                                self.speak(name);
                            }
                            // Scan Space is consumed before draw; additionally ignore native
                            // keyboard clicks while scanning so a focused key cannot also fire.
                            let pointer_click = response.clicked_by(egui::PointerButton::Primary);
                            if response.clicked()
                                && (!self.input_access.scan_running || pointer_click)
                            {
                                clicked = Some(key);
                            }
                            if response.hovered() && enabled {
                                hovered = Some(key);
                                hovered_rect = Some(response.rect);
                            }
                            row_cells.push(KeyCell {
                                id: response.id,
                                enabled,
                            });
                        }
                    });
                    cells.push(row_cells);
                }
                self.input_access.cells = cells;
                if blocked {
                    self.input_access.suspend_dwell();
                } else if let Some(key) = clicked {
                    if self.input_access.scan_running {
                        self.input_access.pause();
                    }
                    self.input_access.dwell_fired = true;
                    self.activate_access_key(ui.ctx(), key);
                } else if self.input_access.scan_running {
                    self.input_access.suspend_dwell();
                } else {
                    let pointer_down = ui.input(|input| input.pointer.any_down());
                    if let Some(key) = self.input_access.dwell_candidate(
                        hovered,
                        now,
                        self.settings.access.dwell_ms,
                        pointer_down,
                    ) {
                        let previous = self.input_access.last_activation;
                        self.activate_access_key(ui.ctx(), key);
                        if self.input_access.last_activation != previous {
                            self.input_access.dwell_fired = true;
                        }
                    }
                    if self.settings.access.dwell_ms > 0
                        && hovered.is_some()
                        && !self.input_access.dwell_fired
                        && !self.input_access.dwell_wait_for_leave
                    {
                        ui.ctx().request_repaint_after(Duration::from_millis(50));
                        if let Some(rect) = hovered_rect {
                            ui.painter().rect_stroke(
                                rect.shrink(2.0_f32),
                                3.0_f32,
                                egui::Stroke::new(3.0_f32, palette.accent),
                                egui::StrokeKind::Inside,
                            );
                        }
                        let elapsed = now - self.input_access.dwell_since.unwrap_or(now);
                        let delay = self.settings.access.dwell_ms.clamp(600, 4000) as f64 / 1000.0;
                        ui.label(format!(
                            "Dwell hold: {:.0}%",
                            (elapsed / delay * 100.0).clamp(0.0, 100.0)
                        ));
                    }
                }
            });
        if self.input_access.scan_running {
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
    }
}

fn access_status(ui: &mut egui::Ui, text: &str) {
    let response = ui.label(text);
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_label(text);
        node.clear_value();
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dwell_requires_full_hold_leave_and_rearm_after_a_modal() {
        let mut state = InputAccess::default();
        assert_eq!(state.dwell_candidate(Some("7"), 0.0, 1000, false), None);
        assert_eq!(state.dwell_candidate(Some("7"), 0.99, 1000, false), None);
        assert_eq!(
            state.dwell_candidate(Some("7"), 1.0, 1000, false),
            Some("7")
        );
        state.dwell_fired = true;
        assert_eq!(state.dwell_candidate(Some("7"), 4.0, 1000, false), None);
        state.suspend_dwell();
        assert_eq!(state.dwell_candidate(Some("7"), 10.0, 1000, false), None);
        state.dwell_candidate(None, 10.1, 1000, false);
        state.dwell_candidate(Some("7"), 10.2, 1000, false);
        assert_eq!(
            state.dwell_candidate(Some("7"), 11.3, 1000, false),
            Some("7")
        );
    }

    #[test]
    fn pointer_click_cannot_also_complete_a_dwell() {
        let mut state = InputAccess::default();
        state.dwell_candidate(Some("8"), 0.0, 600, false);
        assert_eq!(state.dwell_candidate(Some("8"), 0.7, 600, true), None);
        assert_eq!(state.dwell_candidate(Some("8"), 2.0, 600, false), None);
    }

    #[test]
    fn switch_scanning_starts_paused_and_selects_row_then_key() {
        let mut state = InputAccess::default();
        state.update_rows(vec![vec![PAUSE_SCAN], vec!["7", "8", "9"]]);
        assert!(!state.is_running());
        assert_eq!(state.select(1.0, 1000), None);
        state.start(1.0, 1000);
        state.tick(2.0, 1000);
        assert_eq!(state.row, 1);
        assert_eq!(state.select(2.1, 1000), None);
        state.tick(3.2, 1000);
        assert_eq!(state.select(3.3, 1000), Some("8"));
        assert_eq!(state.stage, ScanStage::Rows);
        state.suspend();
        assert!(!state.is_running());
        state.tick(99.0, 1000);
        assert!(!state.is_running());
    }

    #[test]
    fn activation_guard_rejects_repeats_without_losing_the_next_deliberate_key() {
        let mut state = InputAccess::default();
        assert!(state.accept_activation(1.0, 500));
        assert!(!state.accept_activation(1.2, 500));
        assert!(state.accept_activation(1.6, 500));
        assert!(!state.accept_activation(f64::NAN, 500));
    }

    #[test]
    fn spoken_scan_cues_omit_repeated_instructions() {
        let mut state = InputAccess::default();
        state.update_rows(vec![vec!["7", "8", "9"]]);
        assert_eq!(state.spoken_selection(), "Scanning paused");
        state.start(0.0, 1000);
        assert_eq!(state.spoken_selection(), "Row 1. 7, 8, 9");
        state.select(0.1, 1000);
        assert_eq!(state.spoken_selection(), "7");
        assert!(!state.spoken_selection().contains("Space"));
    }

    #[test]
    fn alternate_function_names_follow_the_controller_modes() {
        assert_eq!(effective_key_name("sin", false, false), "Sine");
        assert_eq!(effective_key_name("sin", true, false), "Inverse sine");
        assert_eq!(effective_key_name("7", false, true), "O");
        assert_eq!(effective_key_name("graph", true, false), "Value table");
        let mut state = InputAccess::default();
        state.update_rows(vec![vec!["sin", "cos", "tan"]]);
        state.start(0.0, 1000);
        state.second = true;
        assert!(state.scan_description().contains("Inverse sine"));
        state.select(0.1, 1000);
        assert_eq!(state.spoken_selection(), "Inverse sine");
    }

    #[test]
    fn reflow_keeps_numbers_and_moves_operators_for_left_handed_layout() {
        let normal = keypad_rows(0, 4, false, false);
        let left = keypad_rows(0, 4, true, false);
        assert_eq!(normal[0], ["7", "8", "9", "÷"]);
        assert_eq!(left[0], ["÷", "7", "8", "9"]);
        assert!(
            keypad_rows(3, 2, true, true)
                .iter()
                .all(|row| row.len() <= 2)
        );
        for level in 0..=3 {
            let rows = keypad_rows(level, 4, false, true);
            for direction in [
                "cursor left",
                "cursor right",
                "previous calculation",
                "cursor end",
            ] {
                assert!(rows.iter().flatten().any(|key| *key == direction));
            }
        }
    }

    fn frame(
        app: &mut EqualApp,
        ctx: &egui::Context,
        time: f64,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        app.theme(ctx);
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(360.0, 2200.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| {
                app.handle_keyboard_input(ctx);
                egui::CentralPanel::default().show(ctx, |ui| app.accessible_calculator(ui));
            },
        )
    }

    fn app() -> EqualApp {
        let mut settings = Settings::default();
        settings.access.layout = CalculatorLayout::Accessible;
        EqualApp::with_settings(settings, None)
    }

    fn switch(pressed: bool) -> egui::Event {
        egui::Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }

    fn key_id(app: &EqualApp, key: &str) -> Id {
        app.input_access
            .rows
            .iter()
            .enumerate()
            .find_map(|(row, keys)| {
                keys.iter()
                    .position(|candidate| *candidate == key)
                    .map(|column| app.input_access.cells[row][column].id)
            })
            .unwrap()
    }

    #[test]
    fn paused_scanning_does_not_pull_focus_back_from_other_controls_or_editors() {
        fn external_frame(app: &mut EqualApp, ctx: &egui::Context, time: f64, editor: bool) -> Id {
            let mut external = None;
            let mut text = String::new();
            app.theme(ctx);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(360.0, 2200.0),
                    )),
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| {
                    app.handle_access_input(ctx);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        external = Some(if editor {
                            ui.add(
                                egui::TextEdit::singleline(&mut text)
                                    .id(Id::new("external-test-editor")),
                            )
                            .id
                        } else {
                            ui.button("Header action").id
                        });
                        app.accessible_calculator(ui);
                    });
                },
            );
            external.unwrap()
        }
        let ctx = egui::Context::default();
        let mut app = app();
        app.settings.access.scan_mode = ScanMode::Automatic;
        let header = external_frame(&mut app, &ctx, 0.0, false);
        assert!(!app.input_access.is_running());
        ctx.memory_mut(|memory| memory.request_focus(header));
        external_frame(&mut app, &ctx, 0.1, false);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(header));
        let editor = external_frame(&mut app, &ctx, 0.2, true);
        ctx.memory_mut(|memory| memory.request_focus(editor));
        external_frame(&mut app, &ctx, 0.3, true);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(editor));
        assert!(!app.input_access.is_running());
    }

    #[test]
    fn arrows_move_grid_focus_and_enter_activates_one_native_key() {
        let ctx = egui::Context::default();
        let mut app = app();
        frame(&mut app, &ctx, 0.0, vec![]);
        frame(&mut app, &ctx, 0.1, vec![]);
        ctx.memory_mut(|memory| memory.request_focus(key_id(&app, "7")));
        frame(
            &mut app,
            &ctx,
            0.2,
            vec![egui::Event::Key {
                key: Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(key_id(&app, "8"))
        );
        frame(
            &mut app,
            &ctx,
            0.3,
            vec![egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(app.expression, "8");
    }

    #[test]
    fn native_space_text_does_not_consume_the_guard_before_the_focused_key() {
        let ctx = egui::Context::default();
        let mut app = app();
        app.settings.access.guard_ms = 500;
        frame(&mut app, &ctx, 0.0, vec![]);
        frame(&mut app, &ctx, 0.1, vec![]);
        ctx.memory_mut(|memory| memory.request_focus(key_id(&app, "7")));
        frame(
            &mut app,
            &ctx,
            1.0,
            vec![switch(true), egui::Event::Text(" ".into())],
        );
        assert_eq!(
            app.expression, "7",
            "Native Space must activate its focused key once"
        );
        assert_eq!(app.input_access.last_activation, Some(1.0));
        frame(&mut app, &ctx, 1.1, vec![switch(false)]);
        ctx.memory_mut(|memory| memory.request_focus(key_id(&app, "8")));
        frame(
            &mut app,
            &ctx,
            1.6,
            vec![switch(true), egui::Event::Text(" ".into())],
        );
        assert_eq!(
            app.expression, "78",
            "The next deliberate key remains usable with the guard enabled"
        );
    }

    #[test]
    fn one_switch_can_start_pause_and_resume_after_an_overlay() {
        let ctx = egui::Context::default();
        let mut app = app();
        app.settings.access.scan_mode = ScanMode::Automatic;
        frame(&mut app, &ctx, 0.0, vec![]);
        frame(&mut app, &ctx, 0.1, vec![]);
        assert!(!app.input_access.is_running());
        assert!(
            app.input_access
                .owns_focus(ctx.memory(|memory| memory.focused()))
        );
        frame(&mut app, &ctx, 0.2, vec![switch(true)]);
        assert!(
            app.input_access.is_running(),
            "Space starts without Tab or a pointer"
        );
        frame(&mut app, &ctx, 0.3, vec![switch(false)]);
        frame(&mut app, &ctx, 0.4, vec![switch(true)]);
        assert_eq!(app.input_access.stage, ScanStage::Keys);
        frame(&mut app, &ctx, 0.5, vec![switch(false)]);
        frame(&mut app, &ctx, 0.6, vec![switch(true)]);
        assert!(
            !app.input_access.is_running(),
            "Pause is reachable inside the scan"
        );
        frame(&mut app, &ctx, 0.7, vec![switch(false)]);
        frame(&mut app, &ctx, 0.8, vec![switch(true)]);
        assert!(
            app.input_access.is_running(),
            "Space resumes after selecting Pause"
        );
        app.a11y.show_glossary = true;
        frame(&mut app, &ctx, 0.9, vec![switch(false)]);
        assert!(!app.input_access.is_running());
        app.a11y.show_glossary = false;
        frame(&mut app, &ctx, 1.0, vec![]);
        frame(&mut app, &ctx, 1.1, vec![switch(true)]);
        assert!(
            app.input_access.is_running(),
            "Space resumes after a dialog closes"
        );
        assert!(app.expression.is_empty());
        egui::Popup::open_id(&ctx, Id::new("test-access-popup"));
        frame(&mut app, &ctx, 1.2, vec![switch(false)]);
        assert!(
            !app.input_access.is_running(),
            "Open menus also shield the keypad"
        );
        egui::Popup::close_all(&ctx);
    }

    #[test]
    fn large_key_targets_fit_a_narrow_window_and_have_voice_control_names() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app();
        app.settings.access.target_size = 64.0;
        app.settings.access.word_labels = true;
        frame(&mut app, &ctx, 0.0, vec![]);
        let output = frame(&mut app, &ctx, 0.1, vec![]);
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        for name in [
            "7",
            "Multiply",
            "Calculate answer",
            "Backspace",
            "Undo editing",
            "Calculator down or end of entry",
            "Calculator up or previous entry",
        ] {
            let (_, node) = nodes
                .iter()
                .find(|(_, node)| node.label() == Some(name))
                .unwrap_or_else(|| panic!("Missing {name}"));
            let bounds = node.bounds().unwrap();
            assert!(bounds.x0 >= 0.0 && bounds.x1 <= 360.0, "{name}: {bounds:?}");
            assert!(
                bounds.width() >= 64.0 && bounds.height() >= 64.0,
                "{name}: {bounds:?}"
            );
        }
    }

    #[test]
    fn scan_space_is_consumed_before_a_different_focused_button_can_fire() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = app();
        app.settings.access.scan_mode = ScanMode::Automatic;
        frame(&mut app, &ctx, 0.0, vec![]);
        frame(&mut app, &ctx, 0.1, vec![]);
        let (row, column) = app
            .input_access
            .rows
            .iter()
            .enumerate()
            .find_map(|(row, keys)| {
                keys.iter()
                    .position(|key| *key == "7")
                    .map(|column| (row, column))
            })
            .unwrap();
        let (focus_row, focus_column) = app
            .input_access
            .rows
            .iter()
            .enumerate()
            .find_map(|(row, keys)| {
                keys.iter()
                    .position(|key| *key == "8")
                    .map(|column| (row, column))
            })
            .unwrap();
        let focused = app.input_access.cells[focus_row][focus_column].id;
        app.input_access.scan_running = true;
        app.input_access.stage = ScanStage::Keys;
        app.input_access.row = row;
        app.input_access.column = column;
        app.input_access.next_scan_at = Some(10.0);
        ctx.memory_mut(|memory| memory.request_focus(focused));
        frame(
            &mut app,
            &ctx,
            0.2,
            vec![egui::Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(
            app.expression, "7",
            "One switch selection must not also activate the focused 8"
        );
        frame(
            &mut app,
            &ctx,
            0.3,
            vec![egui::Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(
            app.expression, "7",
            "Holding the switch cannot add another digit"
        );
        app.show_settings = true;
        frame(
            &mut app,
            &ctx,
            0.4,
            vec![egui::Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(app.expression, "7");
        assert!(!app.input_access.is_running());
    }
}
