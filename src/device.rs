use crate::*;
use egui::{FontId, Rect};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_EDITOR_HISTORY: usize = 100;
const MAX_TRANSFER_BYTES: usize = 160 * 1024;
const MAX_LIST_BYTES: usize = 32_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DeviceScreen {
    #[default]
    Home,
    Functions,
    Window,
    Zoom,
    Graph,
    Trace,
    Table,
    TableSettings,
    Mode,
    Math,
    Apps,
    Statistics,
    Variables,
    Programs,
    Matrices,
    Memory,
    Catalog,
    Tests,
    Distributions,
    Format,
    Information,
    Lists,
    Sequences,
    Link,
    GraphCalculate,
    GraphDraw,
    Angles,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum EditTarget {
    #[default]
    Home,
    FirstFunction,
    SecondFunction,
    WindowX,
    WindowY,
    WindowRange,
    TableStart,
    TableStep,
    Data,
    Program,
    MatrixA,
    MatrixB,
    ExtraFunction(u8),
    List(u8),
    Sequence(u8),
    SequenceIndex,
    ProgramName,
    ProgramInput,
    Transfer,
}

#[derive(Clone)]
struct EditorSnapshot {
    target: EditTarget,
    text: String,
    cursor: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferPacket {
    version: u32,
    variables: BTreeMap<char, f64>,
    complex_variables: BTreeMap<char, crate::complex::Complex>,
    lists: [String; 6],
    sequences: [String; 3],
    functions: Vec<String>,
    programs: BTreeMap<String, String>,
}

const MATH_ITEMS: [(&str, &str); 10] = [
    ("Square root", "sqrt("),
    ("Absolute value", "abs("),
    ("Factorial", "!"),
    ("Minimum", "min("),
    ("Maximum", "max("),
    ("Round down", "floor("),
    ("Round up", "ceil("),
    ("Derivative", "diff("),
    ("Integral", "integral("),
    ("Equation root", "root("),
];
const TEST_ITEMS: [(&str, &str); 6] = [
    ("Equal", "="),
    ("Not equal", "!="),
    ("Less than", "<"),
    ("Less than or equal", "<="),
    ("Greater than", ">"),
    ("Greater than or equal", ">="),
];

pub(crate) struct DeviceState {
    pub screen: DeviceScreen,
    pub alpha: bool,
    pub power: bool,
    pub memory: f64,
    pub variables: BTreeMap<char, f64>,
    fraction: bool,
    display_value: f64,
    just_evaluated: bool,
    recall_pressed: bool,
    pub data: String,
    stats: String,
    program: String,
    program_output: String,
    cursor: Option<usize>,
    input: EditTarget,
    menu_index: usize,
    menu_selection_changed: bool,
    store_target: bool,
    insert_mode: bool,
    alpha_lock: bool,
    window_inputs: [String; 3],
    table_inputs: [String; 2],
    information: String,
    pub(crate) complex_mode: bool,
    complex_answer: crate::complex::Complex,
    complex_variables: BTreeMap<char, crate::complex::Complex>,
    display_complex: bool,
    list_answer: Option<Vec<f64>>,
    lists: [String; 6],
    selected_list: usize,
    sequences: [String; 3],
    sequence_index: String,
    selected_sequence: usize,
    sequence_result: String,
    undo: Vec<EditorSnapshot>,
    redo: Vec<EditorSnapshot>,
    selection: Option<(usize, usize)>,
    program_name: String,
    programs: BTreeMap<String, String>,
    program_execution: Option<(crate::programs::Program, crate::programs::ProgramState)>,
    program_snapshot: String,
    program_input: String,
    program_waiting: bool,
    transfer: String,
    transfer_undo: Option<TransferPacket>,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            screen: DeviceScreen::Home,
            alpha: false,
            power: true,
            memory: 0.0,
            variables: BTreeMap::new(),
            fraction: false,
            display_value: 0.0,
            just_evaluated: false,
            recall_pressed: false,
            data: "2; 4; 4; 5; 7; 9".into(),
            stats: String::new(),
            program: "5→A\nDisp A*2".into(),
            program_output: String::new(),
            cursor: None,
            input: EditTarget::Home,
            menu_index: 0,
            menu_selection_changed: true,
            store_target: false,
            insert_mode: false,
            alpha_lock: false,
            window_inputs: ["0".into(), "0".into(), "10".into()],
            table_inputs: ["0".into(), "1".into()],
            information: String::new(),
            complex_mode: false,
            complex_answer: crate::complex::Complex::from_real(0.0),
            complex_variables: BTreeMap::new(),
            display_complex: false,
            list_answer: None,
            lists: [
                "2; 4; 4; 5; 7; 9".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            selected_list: 0,
            sequences: ["n".into(), "n^2".into(), "2*n+1".into()],
            sequence_index: "1".into(),
            selected_sequence: 0,
            sequence_result: String::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            selection: None,
            program_name: "MAIN".into(),
            programs: BTreeMap::new(),
            program_execution: None,
            program_snapshot: String::new(),
            program_input: String::new(),
            program_waiting: false,
            transfer: String::new(),
            transfer_undo: None,
        }
    }
}

fn resolve_variables(text: &str, variables: &BTreeMap<char, f64>) -> Result<String, String> {
    let mut output = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index].is_ascii_alphabetic() {
            if matches!(chars[index], 'e' | 'E')
                && index > 0
                && (chars[index - 1].is_ascii_digit() || chars[index - 1] == '.')
                && chars
                    .get(index + 1)
                    .is_some_and(|c| c.is_ascii_digit() || matches!(c, '+' | '-'))
            {
                output.push(chars[index]);
                index += 1;
                continue;
            }
            let start = index;
            while index < chars.len() && chars[index].is_ascii_alphabetic() {
                index += 1;
            }
            let token: String = chars[start..index].iter().collect();
            if token.len() == 1 && chars[start].is_ascii_uppercase() {
                let name = chars[start];
                let value = variables.get(&name).ok_or_else(|| {
                    format!("Variable {name} has no value. Store a value with STO first.")
                })?;
                output.push_str(&format!("({value:.16e})"));
            } else {
                output.push_str(&token);
            }
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }
    Ok(output)
}

impl DeviceState {
    fn calculate(&mut self, text: &str, answer: f64, degrees: bool) -> Result<f64, String> {
        let normalized = text.replace("->", "→");
        let (value, target) = if let Some((value, name)) = normalized.split_once('→') {
            let name = name.trim();
            if name.len() != 1 || !name.chars().all(|c| c.is_ascii_uppercase()) {
                return Err("Choose one variable A–Z after STO, then press Enter.".into());
            }
            (value, name.chars().next())
        } else {
            (normalized.as_str(), None)
        };
        let mut value = resolve_variables(value, &self.variables)?;
        // Handhelds close trailing function parentheses when Enter is pressed.
        let missing = value
            .chars()
            .filter(|c| *c == '(')
            .count()
            .saturating_sub(value.chars().filter(|c| *c == ')').count());
        if missing > 48 {
            return Err("Too many open parentheses.".into());
        }
        value.extend(std::iter::repeat_n(')', missing));
        let result = if let Some((left, right, operation)) = comparison_parts(&value) {
            let left = math::calculate(left, answer, degrees)?;
            let right = math::calculate(right, answer, degrees)?;
            let result = match operation {
                "=" => left == right,
                "!=" => left != right,
                "<" => left < right,
                "<=" => left <= right,
                ">" => left > right,
                ">=" => left >= right,
                _ => unreachable!("comparison_parts only returns known operations"),
            };
            f64::from(u8::from(result))
        } else {
            math::calculate(&value, answer, degrees)?
        };
        if let Some(name) = target {
            self.variables.insert(name, result);
            self.complex_variables.remove(&name);
        }
        Ok(result)
    }
}

impl EqualApp {
    pub(crate) fn evaluate(&mut self) {
        let normalized = match math::normalize_input(
            &self.expression.replace("->", "\u{2192}"),
            self.settings.access.decimal_comma,
        ) {
            Ok(value) => value,
            Err(error) => {
                self.device_calculation_error(error);
                return;
            }
        };
        match self.evaluate_list_expression(&normalized) {
            Ok(true) => return,
            Err(error) => {
                self.device_calculation_error(error);
                return;
            }
            Ok(false) => {}
        }
        let (value, target) = normalized
            .split_once('\u{2192}')
            .map_or((normalized.as_str(), None), |(value, target)| {
                (value, Some(target))
            });
        let completed = match close_parentheses(value) {
            Ok(value) => target.map_or_else(
                || value.clone(),
                |target| format!("{value}\u{2192}{target}"),
            ),
            Err(error) => {
                self.device_calculation_error(error);
                return;
            }
        };
        let source = match self.expand_device_functions(&completed) {
            Ok(source) => source,
            Err(error) => {
                self.device_calculation_error(error);
                return;
            }
        };
        if self.device.list_answer.is_some() && has_identifier(&source, "ans") {
            self.device_calculation_error(
                "Ans contains a list. Use a list operation such as sum(ans) or mean(ans).".into(),
            );
            return;
        }
        if self.device.complex_mode
            || has_identifier(&source, "i")
            || self.device.complex_answer.im != 0.0 && has_identifier(&source, "ans")
            || self
                .device
                .complex_variables
                .keys()
                .any(|name| has_identifier(&source, &name.to_string()))
        {
            self.device.complex_mode = true;
            match self.calculate_complex(&source) {
                Ok(value) => {
                    self.device.list_answer = None;
                    self.device.complex_answer = value;
                    self.device.display_complex = value.im != 0.0;
                    self.device.display_value = value.re;
                    self.answer = value.re;
                    self.result = value.format(self.settings.precision);
                    self.finish_device_calculation();
                }
                Err(error) => self.device_calculation_error(error),
            }
        } else {
            match self
                .device
                .calculate(&source, self.answer, self.settings.degrees)
            {
                Ok(value) => {
                    self.device.list_answer = None;
                    self.answer = value;
                    self.device.complex_answer = crate::complex::Complex::from_real(value);
                    self.device.display_value = value;
                    self.device.display_complex = false;
                    self.result = math::format(value, self.settings.precision);
                    self.finish_device_calculation();
                }
                Err(error) => self.device_calculation_error(error),
            }
        }
    }

    fn finish_device_calculation(&mut self) {
        self.error = false;
        self.device.just_evaluated = true;
        self.history
            .push((self.expression.clone(), self.result.clone()));
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        if self.settings.speech {
            self.speak(&format!("{} equals {}", self.expression, self.result));
        }
    }

    fn device_calculation_error(&mut self, error: String) {
        self.result = error;
        self.error = true;
        self.device.just_evaluated = false;
        if self.settings.speech {
            self.speak(&format!("Calculation error: {}", self.result));
        }
    }

    fn calculate_complex(&mut self, text: &str) -> Result<crate::complex::Complex, String> {
        let (expression, target) = split_store(text)?;
        let mut variables: BTreeMap<_, _> = self
            .device
            .variables
            .iter()
            .map(|(name, value)| (*name, crate::complex::Complex::from_real(*value)))
            .collect();
        variables.extend(
            self.device
                .complex_variables
                .iter()
                .map(|(name, value)| (*name, *value)),
        );
        let expression = close_parentheses(expression)?;
        let result = if let Some((left, right, operation)) = comparison_parts(&expression) {
            let left = crate::complex::evaluate(
                left,
                self.device.complex_answer,
                self.settings.degrees,
                &variables,
            )?;
            let right = crate::complex::evaluate(
                right,
                self.device.complex_answer,
                self.settings.degrees,
                &variables,
            )?;
            let equal = left.re == right.re && left.im == right.im;
            let value = match operation {
                "=" => equal,
                "!=" => !equal,
                _ if left.im != 0.0 || right.im != 0.0 => {
                    return Err("Complex numbers support equality tests, not ordering.".into());
                }
                "<" => left.re < right.re,
                "<=" => left.re <= right.re,
                ">" => left.re > right.re,
                ">=" => left.re >= right.re,
                _ => return Err("Unknown comparison.".into()),
            };
            crate::complex::Complex::from_real(f64::from(u8::from(value)))
        } else {
            crate::complex::evaluate(
                &expression,
                self.device.complex_answer,
                self.settings.degrees,
                &variables,
            )?
        };
        if let Some(target) = target {
            self.device.complex_variables.insert(target, result);
            if result.im == 0.0 {
                self.device.variables.insert(target, result.re);
            } else {
                self.device.variables.remove(&target);
            }
        }
        Ok(result)
    }

    fn save_editor_snapshot(&mut self) {
        if !self.device_has_editor() {
            return;
        }
        let snapshot = EditorSnapshot {
            target: self.device.input,
            text: self.device_input_text().clone(),
            cursor: self.device.cursor,
        };
        self.record_editor_snapshot(snapshot);
    }

    fn record_editor_snapshot(&mut self, snapshot: EditorSnapshot) {
        if self.device.undo.last().is_none_or(|previous| {
            previous.target != snapshot.target || previous.text != snapshot.text
        }) {
            self.device.undo.push(snapshot);
            if self.device.undo.len() > MAX_EDITOR_HISTORY {
                self.device.undo.remove(0);
            }
        }
        self.device.redo.clear();
    }

    fn restore_editor_snapshot(&mut self, redo: bool) {
        let snapshot = if redo {
            self.device.redo.pop()
        } else {
            self.device.undo.pop()
        };
        let Some(snapshot) = snapshot else {
            self.message = if redo {
                "Nothing to redo."
            } else {
                "Nothing to undo."
            }
            .into();
            return;
        };
        let current = EditorSnapshot {
            target: self.device.input,
            text: self.device_input_text().clone(),
            cursor: self.device.cursor,
        };
        self.device.input = snapshot.target;
        self.device.screen = screen_for_target(snapshot.target);
        if let EditTarget::List(index) = snapshot.target {
            self.device.selected_list = usize::from(index).min(5);
        }
        if let EditTarget::Sequence(index) = snapshot.target {
            self.device.selected_sequence = usize::from(index).min(2);
        }
        *self.device_input_text() = snapshot.text;
        self.device.cursor = snapshot.cursor;
        self.device.selection = None;
        self.device.just_evaluated = false;
        if redo {
            self.device.undo.push(current);
        } else {
            self.device.redo.push(current);
        }
    }

    fn device_insert(&mut self, text: &str) {
        if !self.device_has_editor() {
            self.set_device_screen(DeviceScreen::Home);
        }
        self.handheld_preview = false;
        self.save_editor_snapshot();
        if self.device.input == EditTarget::Home && self.device.just_evaluated {
            if text
                .chars()
                .next()
                .is_some_and(|c| "+−-×*/÷^!%→=<>".contains(c))
            {
                self.expression = "ans".into();
            } else {
                self.expression.clear();
            }
            self.device.cursor = None;
            self.device.just_evaluated = false;
        }
        let text_length = self.device_input_text().chars().count();
        let mut position = self.device.cursor.unwrap_or(text_length).min(text_length);
        let insert_mode = self.device.insert_mode;
        let limit = editor_limit(self.device.input);
        let selection = self.device.selection.take();
        let target = self.device_input_text();
        if target.len() + text.len() > limit {
            self.message = format!("Input is limited to {limit} characters.");
            return;
        }
        if let Some((first, second)) = selection.filter(|(first, second)| first != second) {
            let start = first.min(second).min(text_length);
            let end = first.max(second).min(text_length);
            let a = character_offset(target, start);
            let b = character_offset(target, end);
            target.replace_range(a..b, "");
            position = start;
        }
        let offset = target
            .char_indices()
            .nth(position)
            .map_or(target.len(), |(i, _)| i);
        if !insert_mode
            && selection.is_none_or(|(a, b)| a == b)
            && position < target.chars().count()
            && text.chars().count() == 1
        {
            target.remove(offset);
        }
        target.insert_str(offset, text);
        self.device.cursor = Some(position + text.chars().count());
    }

    fn device_has_editor(&self) -> bool {
        matches!(
            self.device.screen,
            DeviceScreen::Home
                | DeviceScreen::Functions
                | DeviceScreen::Window
                | DeviceScreen::TableSettings
                | DeviceScreen::Statistics
                | DeviceScreen::Programs
                | DeviceScreen::Matrices
                | DeviceScreen::Lists
                | DeviceScreen::Sequences
                | DeviceScreen::Link
        )
    }

    fn device_input_text(&mut self) -> &mut String {
        self.device_target_text(self.device.input)
    }

    fn device_target_text(&mut self, target: EditTarget) -> &mut String {
        match target {
            EditTarget::Home => &mut self.expression,
            EditTarget::FirstFunction => &mut self.graph_a,
            EditTarget::SecondFunction => &mut self.graph_b,
            EditTarget::WindowX => &mut self.device.window_inputs[0],
            EditTarget::WindowY => &mut self.device.window_inputs[1],
            EditTarget::WindowRange => &mut self.device.window_inputs[2],
            EditTarget::TableStart => &mut self.device.table_inputs[0],
            EditTarget::TableStep => &mut self.device.table_inputs[1],
            EditTarget::Data => &mut self.device.data,
            EditTarget::Program => &mut self.device.program,
            EditTarget::MatrixA => &mut self.matrix_a,
            EditTarget::MatrixB => &mut self.matrix_b,
            EditTarget::ExtraFunction(index) => self.graph_function_mut(usize::from(index)),
            EditTarget::List(index) => &mut self.device.lists[usize::from(index).min(5)],
            EditTarget::Sequence(index) => &mut self.device.sequences[usize::from(index).min(2)],
            EditTarget::SequenceIndex => &mut self.device.sequence_index,
            EditTarget::ProgramName => &mut self.device.program_name,
            EditTarget::ProgramInput => &mut self.device.program_input,
            EditTarget::Transfer => &mut self.device.transfer,
        }
    }

    pub(crate) fn set_device_screen(&mut self, screen: DeviceScreen) {
        let origin = self.device.input;
        let cursor = self.device.cursor;
        let selection = self.device.selection;
        let insertion_menu = matches!(
            screen,
            DeviceScreen::Math
                | DeviceScreen::Catalog
                | DeviceScreen::Tests
                | DeviceScreen::Variables
                | DeviceScreen::Angles
        );
        if self.device.screen == DeviceScreen::Statistics {
            self.device.lists[self.device.selected_list] = self.device.data.clone();
        }
        self.device.screen = screen;
        self.device.menu_index = 0;
        self.device.menu_selection_changed = true;
        self.device.cursor = None;
        self.device.selection = None;
        self.device.information.clear();
        self.device.input = match screen {
            DeviceScreen::Functions => EditTarget::FirstFunction,
            DeviceScreen::Window => {
                self.device.window_inputs = [
                    self.center[0].to_string(),
                    self.center[1].to_string(),
                    self.range.to_string(),
                ];
                EditTarget::WindowX
            }
            DeviceScreen::TableSettings => {
                self.device.table_inputs = [
                    self.graph_access.table_start.to_string(),
                    self.graph_access.table_step.to_string(),
                ];
                EditTarget::TableStart
            }
            DeviceScreen::Statistics => {
                self.device.data = self.device.lists[self.device.selected_list].clone();
                EditTarget::Data
            }
            DeviceScreen::Programs => EditTarget::Program,
            DeviceScreen::Matrices => EditTarget::MatrixA,
            DeviceScreen::Lists => EditTarget::List(self.device.selected_list as u8),
            DeviceScreen::Sequences => EditTarget::Sequence(self.device.selected_sequence as u8),
            DeviceScreen::Link => EditTarget::Transfer,
            DeviceScreen::Math
            | DeviceScreen::Catalog
            | DeviceScreen::Tests
            | DeviceScreen::Variables
            | DeviceScreen::Angles => origin,
            _ => EditTarget::Home,
        };
        if insertion_menu {
            self.device.cursor = cursor;
            self.device.selection = selection;
        }
    }

    fn return_to_device_editor(&mut self) {
        self.device.screen = screen_for_target(self.device.input);
        self.device.menu_index = 0;
        self.device.menu_selection_changed = true;
        self.device.information.clear();
    }

    fn device_clear(&mut self) {
        self.save_editor_snapshot();
        self.device.power = true;
        if let Some((a, b)) = self.device.selection.take().filter(|(a, b)| a != b) {
            let target = self.device_input_text();
            let first = character_offset(target, a.min(b));
            let second = character_offset(target, a.max(b));
            target.replace_range(first..second, "");
            self.device.cursor = Some(a.min(b));
            self.device.just_evaluated = false;
            return;
        } else if self.device_has_editor() && self.device.input != EditTarget::Home {
            self.device_input_text().clear();
        } else {
            self.set_device_screen(DeviceScreen::Home);
            self.expression.clear();
            self.result = "0".into();
            self.device.display_value = 0.0;
            self.device.display_complex = false;
            self.error = false;
        }
        self.device.just_evaluated = false;
        self.device.cursor = None;
        self.handheld_preview = false;
    }

    fn select_device_input(&mut self, target: EditTarget, response: &egui::Response) {
        if response.has_focus() {
            self.device.input = target;
            if let Some(range) = egui::text_edit::TextEditState::load(&response.ctx, response.id)
                .and_then(|state| state.cursor.char_range())
            {
                self.device.cursor = Some(range.primary.index);
                self.device.selection = Some((range.primary.index, range.secondary.index));
            }
        }
    }

    pub(crate) fn press_device_key(&mut self, ctx: &egui::Context, key: &str) {
        if !self.device.power && !matches!(key, "on" | "clear") {
            return;
        }
        if key == "undo" {
            self.restore_editor_snapshot(false);
            self.sync_device_cursor(ctx);
            return;
        }
        if key == "redo" {
            self.restore_editor_snapshot(true);
            self.sync_device_cursor(ctx);
            return;
        }
        if key == "2nd" {
            self.second = !self.second;
            self.device.alpha = false;
            return;
        }
        if key == "alpha" {
            self.device.alpha_lock = self.second;
            self.device.alpha = if self.second {
                true
            } else {
                !self.device.alpha
            };
            self.second = false;
            return;
        }
        if matches!(
            key,
            "previous calculation" | "cursor left" | "cursor right" | "cursor end"
        ) {
            self.second = false;
            self.device_arrow(ctx, key);
            return;
        }
        let shifted = std::mem::take(&mut self.second);
        if self.device.alpha {
            let letter = match key {
                "math" => "A",
                "apps" => "B",
                "prgm" => "C",
                "1/x" => "D",
                "sin" => "E",
                "cos" => "F",
                "tan" => "G",
                "^" => "H",
                "x\u{b2}" => "I",
                "," => "J",
                "0" => " ",
                "." => ":",
                "+" => "\"",
                "(" => "K",
                ")" => "L",
                "\u{f7}" => "M",
                "log" => "N",
                "7" => "O",
                "8" => "P",
                "9" => "Q",
                "\u{d7}" => "R",
                "ln" => "S",
                "4" => "T",
                "5" => "U",
                "6" => "V",
                "\u{2212}" => "W",
                "sto" => "X",
                "1" => "Y",
                "2" => "Z",
                _ => "",
            };
            if !self.device.alpha_lock {
                self.device.alpha = false;
            }
            if !letter.is_empty() {
                self.device_insert(letter);
                self.sync_device_cursor(ctx);
                return;
            }
        }
        if shifted {
            match key {
                "mode" => self.set_device_screen(DeviceScreen::Home),
                "graph" => self.set_device_screen(DeviceScreen::Table),
                "window" => self.set_device_screen(DeviceScreen::TableSettings),
                "zoom" => self.set_device_screen(DeviceScreen::Format),
                "trace" => self.set_device_screen(DeviceScreen::GraphCalculate),
                "y=" | "stat" => self.set_device_screen(DeviceScreen::Statistics),
                "apps" => self.set_device_screen(DeviceScreen::Angles),
                "1/x" => self.set_device_screen(DeviceScreen::Matrices),
                "vars" => self.set_device_screen(DeviceScreen::Distributions),
                "prgm" => self.set_device_screen(DeviceScreen::GraphDraw),
                "math" => self.set_device_screen(DeviceScreen::Tests),
                "+" => self.set_device_screen(DeviceScreen::Memory),
                "0" => self.set_device_screen(DeviceScreen::Catalog),
                "on" => self.device.power = false,
                "enter" => self.recall_calculation(),
                "sto" => {
                    self.device.store_target = false;
                    self.set_device_screen(DeviceScreen::Variables);
                }
                "sin" => self.device_insert("asin("),
                "cos" => self.device_insert("acos("),
                "tan" => self.device_insert("atan("),
                "x\u{b2}" => self.device_insert("sqrt("),
                "log" => self.device_insert("10^("),
                "ln" => self.device_insert("exp("),
                "^" => self.device_insert("pi"),
                "\u{f7}" => self.device_insert("e"),
                "\u{b1}" => self.device_insert("ans"),
                "," => self.device_insert("E"),
                "delete" => self.device.insert_mode = true,
                "." => {
                    self.device.complex_mode = true;
                    self.device_insert("i");
                }
                "x,t" => self.set_device_screen(DeviceScreen::Link),
                "1" | "2" | "3" | "4" | "5" | "6" => {
                    if self.device.screen == DeviceScreen::Lists {
                        self.device.selected_list =
                            key.parse::<usize>().unwrap_or(1).saturating_sub(1).min(5);
                        self.device.input = EditTarget::List(self.device.selected_list as u8);
                        self.device.cursor = None;
                    } else {
                        self.device_insert(&format!("L{key}"));
                    }
                }
                "7" | "8" | "9" => self.device_insert(match key {
                    "7" => "u(",
                    "8" => "v(",
                    _ => "w(",
                }),
                "(" => self.device_insert("{"),
                ")" => self.device_insert("}"),
                "\u{d7}" => self.device_insert("["),
                "\u{2212}" => self.device_insert("]"),
                "clear" => self.device_clear(),
                _ => self.device_insert(key),
            }
            self.sync_device_cursor(ctx);
            return;
        }
        let recalled = key == "MRC";
        match key {
            "on" => {
                self.device.power = true;
                if self.settings.level == 0 {
                    self.device_clear();
                } else {
                    self.set_device_screen(DeviceScreen::Home);
                }
            }
            "off" => self.device.power = false,
            "clear" => self.device_clear(),
            "delete" | "backspace" | "delete_forward" => {
                if self.device_has_editor() {
                    self.save_editor_snapshot();
                    let position = self
                        .device
                        .cursor
                        .unwrap_or(self.device_input_text().chars().count());
                    if let Some((first, second)) =
                        self.device.selection.take().filter(|(a, b)| a != b)
                    {
                        let target = self.device_input_text();
                        let start = first.min(second).min(target.chars().count());
                        let end = first.max(second).min(target.chars().count());
                        let a = character_offset(target, start);
                        let b = character_offset(target, end);
                        target.replace_range(a..b, "");
                        self.device.cursor = Some(start);
                    } else if key == "delete_forward" {
                        let target = self.device_input_text();
                        if let Some((index, _)) = target.char_indices().nth(position) {
                            target.remove(index);
                        }
                    } else if position > 0 {
                        let target = self.device_input_text();
                        if let Some((index, _)) = target.char_indices().nth(position - 1) {
                            target.remove(index);
                        }
                        self.device.cursor = Some(position - 1);
                    }
                    self.device.just_evaluated = false;
                }
            }
            "enter" => self.device_enter(),
            "mode" => self.set_device_screen(DeviceScreen::Mode),
            "y=" => self.set_device_screen(DeviceScreen::Functions),
            "window" => self.set_device_screen(DeviceScreen::Window),
            "zoom" => self.set_device_screen(DeviceScreen::Zoom),
            "trace" => {
                self.set_device_screen(DeviceScreen::Trace);
                self.graph_access.keyboard_trace = true;
            }
            "graph" => {
                if self.device.screen != DeviceScreen::Window || self.apply_device_window() {
                    self.set_device_screen(DeviceScreen::Graph);
                }
            }
            "math" => self.set_device_screen(DeviceScreen::Math),
            "apps" => self.set_device_screen(DeviceScreen::Apps),
            "stat" | "data" => self.set_device_screen(DeviceScreen::Statistics),
            "vars" => {
                self.device.store_target = false;
                self.set_device_screen(DeviceScreen::Variables);
            }
            "prgm" => self.set_device_screen(DeviceScreen::Programs),
            "lists" => self.set_device_screen(DeviceScreen::Lists),
            "sequences" => self.set_device_screen(DeviceScreen::Sequences),
            "link" => self.set_device_screen(DeviceScreen::Link),
            "sto" => {
                if self.device_has_editor() && self.device.input != EditTarget::Home {
                    self.device_insert("\u{2192}");
                } else {
                    let token = if self.expression.is_empty() && !self.device.just_evaluated {
                        "ans\u{2192}"
                    } else {
                        "\u{2192}"
                    };
                    self.device_insert(token);
                    if self.settings.level == 1 {
                        self.device.store_target = true;
                        self.set_device_screen(DeviceScreen::Variables);
                    }
                }
            }
            "fraction" => self.device_insert("/"),
            "convert" => self.device.fraction = !self.device.fraction,
            "M+" | "M-" => {
                let source = if self.expression.is_empty() {
                    "ans"
                } else {
                    &self.expression
                };
                match self
                    .device
                    .calculate(source, self.answer, self.settings.degrees)
                {
                    Ok(value) => {
                        let memory = self.device.memory + if key == "M+" { value } else { -value };
                        if memory.is_finite() {
                            self.device.memory = memory;
                        } else {
                            self.result = "Memory overflow".into();
                            self.error = true;
                        }
                    }
                    Err(error) => {
                        self.result = error;
                        self.error = true;
                    }
                }
            }
            "MRC" => {
                if self.device.recall_pressed {
                    self.device.memory = 0.0;
                }
                self.set_device_screen(DeviceScreen::Home);
                self.save_editor_snapshot();
                self.expression = math::format(self.device.memory, self.settings.precision);
                self.device.just_evaluated = false;
            }
            "\u{b1}" => {
                if self.device.input != EditTarget::Home
                    || self.expression.is_empty() && !self.device.just_evaluated
                {
                    self.device_insert("-");
                } else {
                    self.save_editor_snapshot();
                    let value = if self.device.just_evaluated || self.expression.is_empty() {
                        "ans"
                    } else {
                        &self.expression
                    };
                    self.expression = format!("-({value})");
                    self.device.just_evaluated = false;
                    self.device.cursor = None;
                }
            }
            "\u{221a}" if self.settings.level == 0 => {
                if self.device.input != EditTarget::Home {
                    self.set_device_screen(DeviceScreen::Home);
                }
                self.save_editor_snapshot();
                let value = if self.device.just_evaluated || self.expression.is_empty() {
                    "ans"
                } else {
                    &self.expression
                };
                self.expression = format!("sqrt({value})");
                self.device.just_evaluated = false;
                self.set_device_screen(DeviceScreen::Home);
            }
            "\u{221a}" => self.device_insert("sqrt("),
            "\u{3c0}" => self.device_insert("pi"),
            "x\u{b2}" => self.device_insert("^2"),
            "1/x" => {
                if self.device.input == EditTarget::Home {
                    self.save_editor_snapshot();
                    let value = if self.device.just_evaluated || self.expression.is_empty() {
                        "ans"
                    } else {
                        &self.expression
                    };
                    self.expression = format!("1/({value})");
                    self.device.just_evaluated = false;
                    self.set_device_screen(DeviceScreen::Home);
                } else {
                    self.device_insert("^(-1)");
                }
            }
            "Ans" => self.device_insert("ans"),
            "x,t" => self.device_insert(if self.device.screen == DeviceScreen::Sequences {
                "n"
            } else {
                "x"
            }),
            "sin" | "cos" | "tan" | "log" | "ln" => self.device_insert(&format!("{key}(")),
            _ if key.chars().all(|c| {
                c.is_ascii_digit()
                    || "+-*/().^%,".contains(c)
                    || matches!(c, '\u{d7}' | '\u{f7}' | '\u{2212}')
            }) =>
            {
                self.device_insert(key)
            }
            _ => self.device_insert(key),
        }
        self.device.recall_pressed = recalled;
        self.sync_device_cursor(ctx);
    }

    fn sync_device_cursor(&mut self, ctx: &egui::Context) {
        if !self.device_has_editor() {
            return;
        }
        let count = self.device_input_text().chars().count();
        let position = self.device.cursor.unwrap_or(count).min(count);
        let id = device_input_id(self.device.input);
        let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(position),
            )));
        state.store(ctx, id);
    }

    fn recall_calculation(&mut self) {
        if let Some(expression) = self
            .history
            .last()
            .map(|(expression, _)| expression.clone())
        {
            self.set_device_screen(DeviceScreen::Home);
            self.save_editor_snapshot();
            self.expression = expression;
            self.device.just_evaluated = false;
            self.device.cursor = None;
        }
    }

    fn device_arrow(&mut self, ctx: &egui::Context, key: &str) {
        if matches!(
            self.device.screen,
            DeviceScreen::Trace | DeviceScreen::Graph
        ) {
            match key {
                "cursor left" => self.move_trace(-1.0),
                "cursor right" => self.move_trace(1.0),
                _ => {
                    let active = self.active_graph_indices();
                    if !active.is_empty() {
                        let index = active
                            .iter()
                            .position(|index| *index == self.graph_access.selected)
                            .unwrap_or(0);
                        self.select_trace_curve(
                            active[adjacent_index(index, active.len(), key == "cursor end")],
                        );
                    }
                }
            }
            self.set_device_screen(DeviceScreen::Trace);
            if self.settings.speech {
                self.speak(&self.trace_text());
            }
            return;
        }
        let entries = self.device_menu_entries();
        if !entries.is_empty() {
            let forward = matches!(key, "cursor right" | "cursor end");
            let previous = self.device.menu_index;
            self.device.menu_index = if forward {
                (self.device.menu_index + 1).min(entries.len() - 1)
            } else {
                self.device.menu_index.saturating_sub(1)
            };
            self.device.menu_selection_changed |= previous != self.device.menu_index;
            return;
        }
        if matches!(key, "previous calculation" | "cursor end")
            && self.device.screen != DeviceScreen::Home
        {
            let forward = key == "cursor end";
            if matches!(
                self.device.input,
                EditTarget::Program | EditTarget::Transfer | EditTarget::Data | EditTarget::List(_)
            ) {
                let text = self.device_input_text().clone();
                let position = self.device.cursor.unwrap_or(text.chars().count());
                self.device.cursor = Some(vertical_cursor(&text, position, forward));
                self.device.selection = None;
                self.sync_device_cursor(ctx);
                ctx.memory_mut(|memory| memory.request_focus(device_input_id(self.device.input)));
                return;
            }
            self.device.input = match (self.device.screen, self.device.input) {
                (DeviceScreen::Functions, target) => {
                    let index = match target {
                        EditTarget::SecondFunction => 1,
                        EditTarget::ExtraFunction(index) => usize::from(index).min(9),
                        _ => 0,
                    };
                    graph_edit_target(adjacent_index(index, 10, forward))
                }
                (DeviceScreen::Window, target) => {
                    let index = match target {
                        EditTarget::WindowY => 1,
                        EditTarget::WindowRange => 2,
                        _ => 0,
                    };
                    [
                        EditTarget::WindowX,
                        EditTarget::WindowY,
                        EditTarget::WindowRange,
                    ][adjacent_index(index, 3, forward)]
                }
                (DeviceScreen::TableSettings, EditTarget::TableStart) => EditTarget::TableStep,
                (DeviceScreen::TableSettings, _) => EditTarget::TableStart,
                (DeviceScreen::Matrices, EditTarget::MatrixA) => EditTarget::MatrixB,
                (DeviceScreen::Matrices, _) => EditTarget::MatrixA,
                (DeviceScreen::Sequences, EditTarget::Sequence(_)) => EditTarget::SequenceIndex,
                (DeviceScreen::Sequences, _) => {
                    EditTarget::Sequence(self.device.selected_sequence as u8)
                }
                (DeviceScreen::Programs, _) => EditTarget::Program,
                (_, target) => target,
            };
            self.device.cursor = None;
            self.device.selection = None;
            if self.device_has_editor() {
                self.sync_device_cursor(ctx);
                ctx.memory_mut(|memory| memory.request_focus(device_input_id(self.device.input)));
            }
            return;
        }
        if key == "previous calculation" {
            self.recall_calculation();
            return;
        }
        if !self.device_has_editor() {
            return;
        }
        let count = self.device_input_text().chars().count();
        let position = self.device.cursor.unwrap_or(count);
        let position = match key {
            "cursor left" => position.saturating_sub(1),
            "cursor right" => (position + 1).min(count),
            _ => count,
        };
        self.device.cursor = Some(position);
        self.device.selection = None;
        self.sync_device_cursor(ctx);
        ctx.memory_mut(|m| m.request_focus(device_input_id(self.device.input)));
    }

    fn device_enter(&mut self) {
        if !self.device_menu_entries().is_empty() {
            self.activate_device_menu(self.device.menu_index);
            return;
        }
        match self.device.screen {
            DeviceScreen::Home => self.evaluate(),
            DeviceScreen::Functions => {
                self.device.input = match self.device.input {
                    EditTarget::FirstFunction => EditTarget::SecondFunction,
                    EditTarget::SecondFunction => EditTarget::ExtraFunction(2),
                    EditTarget::ExtraFunction(index) if index < 9 => {
                        EditTarget::ExtraFunction(index + 1)
                    }
                    _ => EditTarget::FirstFunction,
                };
                self.device.cursor = None;
            }
            DeviceScreen::Window => {
                self.device.input = match self.device.input {
                    EditTarget::WindowX => EditTarget::WindowY,
                    EditTarget::WindowY => EditTarget::WindowRange,
                    _ => {
                        if self.apply_device_window() {
                            self.set_device_screen(DeviceScreen::Graph);
                        }
                        return;
                    }
                };
                self.device.cursor = None;
            }
            DeviceScreen::TableSettings => {
                if self.device.input == EditTarget::TableStart {
                    self.device.input = EditTarget::TableStep;
                    self.device.cursor = None;
                } else if self.apply_device_table() {
                    self.set_device_screen(DeviceScreen::Table);
                }
            }
            DeviceScreen::Statistics => {
                self.device.lists[self.device.selected_list] = self.device.data.clone();
                self.device.stats =
                    list_report(&self.device.data, self.settings.access.decimal_comma)
                        .unwrap_or_else(|error| error)
            }
            DeviceScreen::Lists => {
                self.device.stats = list_report(
                    &self.device.lists[self.device.selected_list],
                    self.settings.access.decimal_comma,
                )
                .unwrap_or_else(|error| error)
            }
            DeviceScreen::Sequences => self.evaluate_sequence(),
            DeviceScreen::Link => self.import_device_transfer(),
            DeviceScreen::Programs => self.run_device_program(),
            DeviceScreen::Trace => self.speak(&self.trace_text()),
            _ => self.set_device_screen(DeviceScreen::Home),
        }
    }

    fn device_menu_entries(&self) -> Vec<String> {
        match self.device.screen {
            DeviceScreen::Mode => vec![
                "Degrees",
                "Radians",
                "Toggle fraction answers",
                "Accessibility settings",
                "Return to calculation",
                "Real number mode",
                "Complex number mode (a+bi)",
                "Sequence formulas u, v, w",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            DeviceScreen::Zoom => vec!["Zoom in", "Zoom out", "Standard window"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            DeviceScreen::Math | DeviceScreen::Catalog => {
                MATH_ITEMS.iter().map(|(name, _)| (*name).into()).collect()
            }
            DeviceScreen::Tests => TEST_ITEMS.iter().map(|(name, _)| (*name).into()).collect(),
            DeviceScreen::Angles => [
                "Degrees to radians",
                "Radians to degrees",
                "Insert dms(degrees; minutes; seconds)",
                "Decimal degrees to DMS display",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            DeviceScreen::Apps => vec![
                "Learning toolkit",
                "Step-by-step lessons",
                "Open larger graph",
                "Named lists L1-L6",
                "Sequence formulas u, v, w",
                "Equal text transfer",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            DeviceScreen::Memory => vec!["Recall memory", "Clear memory"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            DeviceScreen::Variables => ('A'..='Z')
                .map(|name| {
                    if self.device.store_target {
                        format!("Store in {name}")
                    } else if let Some(value) = self.device.complex_variables.get(&name) {
                        format!("{name} = {}", value.format(self.settings.precision))
                    } else if let Some(value) = self.device.variables.get(&name) {
                        format!("{name} = {}", math::format(*value, self.settings.precision))
                    } else {
                        format!("{name} (not stored)")
                    }
                })
                .collect(),
            _ => vec![],
        }
    }

    fn activate_device_menu(&mut self, index: usize) {
        self.device.menu_selection_changed |= self.device.menu_index != index;
        self.device.menu_index = index;
        match self.device.screen {
            DeviceScreen::Mode => match index {
                0 => self.settings.degrees = true,
                1 => self.settings.degrees = false,
                2 => self.device.fraction = !self.device.fraction,
                3 => self.show_settings = true,
                5 => self.device.complex_mode = false,
                6 => self.device.complex_mode = true,
                7 => self.set_device_screen(DeviceScreen::Sequences),
                _ => self.set_device_screen(DeviceScreen::Home),
            },
            DeviceScreen::Zoom => {
                match index {
                    0 => self.range = (self.range / 1.4).max(0.05),
                    1 => self.range = (self.range * 1.4).min(1000.0),
                    _ => {
                        self.range = 10.0;
                        self.center = [0.0, 0.0];
                    }
                }
                self.set_device_screen(DeviceScreen::Graph);
            }
            DeviceScreen::Math | DeviceScreen::Catalog => {
                if let Some((_, token)) = MATH_ITEMS.get(index) {
                    self.return_to_device_editor();
                    self.device_insert(token);
                }
            }
            DeviceScreen::Tests => {
                if let Some((_, token)) = TEST_ITEMS.get(index) {
                    self.return_to_device_editor();
                    self.device_insert(token);
                }
            }
            DeviceScreen::Angles => match index {
                0 => self.apply_angle_conversion("rad"),
                1 => self.apply_angle_conversion("deg"),
                2 => {
                    self.return_to_device_editor();
                    self.device_insert("dms(");
                }
                3 => self.display_angle_dms(),
                _ => {}
            },
            DeviceScreen::Apps => {
                match index {
                    3 => {
                        self.set_device_screen(DeviceScreen::Lists);
                        return;
                    }
                    4 => {
                        self.set_device_screen(DeviceScreen::Sequences);
                        return;
                    }
                    5 => {
                        self.set_device_screen(DeviceScreen::Link);
                        return;
                    }
                    _ => {}
                }
                self.tab = match index {
                    0 => Tab::Toolkit,
                    1 => Tab::Tutor,
                    _ => Tab::Graph,
                };
                self.show_workspace = true;
            }
            DeviceScreen::Variables => {
                if let Ok(offset) = u8::try_from(index)
                    && offset < 26
                {
                    let name = char::from(b'A' + offset);
                    self.return_to_device_editor();
                    self.device_insert(&name.to_string());
                    self.device.store_target = false;
                }
            }
            DeviceScreen::Memory => {
                if index == 0 {
                    let value = self.device.memory.to_string();
                    self.set_device_screen(DeviceScreen::Home);
                    self.device_insert(&value);
                } else {
                    self.device.memory = 0.0;
                }
            }
            _ => {}
        }
    }

    fn apply_angle_conversion(&mut self, function: &str) {
        self.return_to_device_editor();
        if self.device.input == EditTarget::Home && self.device.just_evaluated {
            self.device_insert(&format!("{function}(ans)"));
            return;
        }
        let source = self.device_input_text().clone();
        if let Some((first, second)) = self
            .device
            .selection
            .filter(|(first, second)| first != second)
        {
            let start = character_offset(&source, first.min(second));
            let end = character_offset(&source, first.max(second));
            self.device_insert(&format!("{function}({})", &source[start..end]));
        } else if matches!(
            self.device.input,
            EditTarget::Home
                | EditTarget::FirstFunction
                | EditTarget::SecondFunction
                | EditTarget::ExtraFunction(_)
                | EditTarget::WindowX
                | EditTarget::WindowY
                | EditTarget::WindowRange
                | EditTarget::TableStart
                | EditTarget::TableStep
                | EditTarget::Sequence(_)
                | EditTarget::SequenceIndex
                | EditTarget::ProgramInput
        ) && !source.trim().is_empty()
        {
            let converted = format!("{function}({source})");
            if converted.len() > editor_limit(self.device.input) {
                self.device.information = "The converted angle expression is too long.".into();
                return;
            }
            self.save_editor_snapshot();
            *self.device_input_text() = converted;
            self.device.cursor = None;
            self.device.selection = None;
            self.device.just_evaluated = false;
        } else {
            self.device_insert(&format!("{function}("));
        }
    }

    fn display_angle_dms(&mut self) {
        let source = if self.device.input == EditTarget::Home
            && (self.device.just_evaluated || self.expression.trim().is_empty())
        {
            "ans".to_owned()
        } else {
            self.device_input_text().clone()
        };
        let result = if has_identifier(&source, "ans")
            && (self.device.complex_answer.im != 0.0 || self.device.list_answer.is_some())
        {
            Err("DMS display needs a real scalar angle in degrees.".into())
        } else if ["x", "y", "n", "i"]
            .iter()
            .any(|name| has_identifier(&source, name))
        {
            Err("DMS display needs a numeric angle, without graph or sequence coordinates.".into())
        } else {
            math::normalize_input(&source, self.settings.access.decimal_comma)
                .and_then(|source| resolve_variables(&source, &self.device.variables))
                .and_then(|source| math::calculate(&source, self.answer, self.settings.degrees))
                .and_then(|value| format_dms(value, self.settings.precision))
        };
        self.device.information = result.unwrap_or_else(|error| error);
    }

    fn apply_device_window(&mut self) -> bool {
        let values: Result<Vec<f64>, _> = self
            .device
            .window_inputs
            .iter()
            .map(|value| value.trim().parse::<f64>())
            .collect();
        match values {
            Ok(values)
                if values.iter().all(|value| value.is_finite())
                    && values[0].abs() <= 1e6
                    && values[1].abs() <= 1e6
                    && (0.05..=1000.0).contains(&values[2]) =>
            {
                self.center = [values[0], values[1]];
                self.range = values[2];
                self.device.information.clear();
                true
            }
            _ => {
                self.device.information =
                    "Centers must be finite within ±1,000,000; half range 0.05–1,000.".into();
                false
            }
        }
    }

    fn apply_device_table(&mut self) -> bool {
        let values: Result<Vec<f64>, _> = self
            .device
            .table_inputs
            .iter()
            .map(|value| value.trim().parse::<f64>())
            .collect();
        match values {
            Ok(values)
                if values.iter().all(|value| value.is_finite())
                    && values[0].abs() <= 1e6
                    && (0.001..=1e6).contains(&values[1]) =>
            {
                self.graph_access.table_start = values[0];
                self.graph_access.table_step = values[1];
                self.device.information.clear();
                true
            }
            _ => {
                self.device.information =
                    "Start must be finite within ±1,000,000; step 0.001–1,000,000.".into();
                false
            }
        }
    }

    fn device_field(&mut self, ui: &mut egui::Ui, target: EditTarget, label: &str, rows: usize) {
        ui.label(label);
        let width = ui.available_width();
        let limit = editor_limit(target);
        let before = self.device_target_text(target).clone();
        let cursor = self.device.cursor;
        let text = self.device_target_text(target);
        let editor = if rows > 1 {
            egui::TextEdit::multiline(text).desired_rows(rows)
        } else {
            egui::TextEdit::singleline(text)
        };
        let response = ui.add(
            editor
                .id(device_input_id(target))
                .desired_width(width)
                .char_limit(limit),
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label));
        if response.changed() {
            self.record_editor_snapshot(EditorSnapshot {
                target,
                text: before,
                cursor,
            });
        }
        self.select_device_input(target, &response);
    }

    pub(crate) fn device_display(&mut self, ui: &mut egui::Ui, rect: Rect, scale: f32) {
        let palette = crate::appearance::palette(ui.ctx());
        ui.painter().rect_filled(rect, 1.0, palette.paper);
        if !self.device.power {
            ui.put(rect, egui::Label::new("Off — press ON"));
            return;
        }
        if matches!(
            self.device.screen,
            DeviceScreen::Graph | DeviceScreen::Trace
        ) {
            self.embedded_graph(ui, rect);
            return;
        }
        let inner = rect.shrink(4.0 * scale);
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner)
            .layout(egui::Layout::top_down(egui::Align::Min)), |ui| {
            ui.set_clip_rect(inner);
            ui.set_width(inner.width());
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 3.0);
            ui.spacing_mut().button_padding = egui::vec2(4.0, 3.0);
            let font = if self.settings.readable { egui::FontFamily::Monospace } else { egui::FontFamily::Proportional };
            for (style, size) in [(egui::TextStyle::Body, 12.0), (egui::TextStyle::Button, 11.0), (egui::TextStyle::Small, 10.0)] {
                ui.style_mut().text_styles.insert(style, FontId::new((size * scale).max(11.0), font.clone()));
            }
            ui.visuals_mut().override_text_color = Some(palette.ink);
            ui.visuals_mut().widgets.noninteractive.fg_stroke.color = palette.ink;
            let screen = self.device.screen;
            let title = format!("{}{}{}{}", if self.second { "2ND · " } else { "" }, if self.device.alpha { "ALPHA · " } else { "" }, if self.device.memory != 0.0 { "M · " } else { "" }, if self.settings.degrees { "DEG" } else { "RAD" });
            ui.label(R::new(format!("{title} · {}", if self.device.complex_mode { "a+bi" } else { "REAL" })).small().color(palette.ink));
            egui::ScrollArea::vertical().id_salt("device-display-scroll").max_height((inner.height() - 16.0).max(30.0)).show(ui, |ui| {
                let entries = self.device_menu_entries();
                if !entries.is_empty() {
                    ui.strong(match screen {
                        DeviceScreen::Mode => "Calculator mode",
                        DeviceScreen::Zoom => "Zoom",
                        DeviceScreen::Math | DeviceScreen::Catalog => "Math functions",
                        DeviceScreen::Angles => "Angle conversions",
                        DeviceScreen::Tests => "Tests: 1=true, 0=false",
                        DeviceScreen::Apps => "Apps",
                        DeviceScreen::Variables if self.device.store_target => "Choose a storage variable",
                        DeviceScreen::Variables => "Variables",
                        DeviceScreen::Memory => "Memory",
                        _ => "Menu",
                    });
                    if screen == DeviceScreen::Mode {
                        ui.label(if self.device.fraction { "Answers: fractions" } else { "Answers: decimals" });
                    }
                    if screen == DeviceScreen::Angles {
                        ui.label("Conversions keep the angle mode unchanged. DMS example: dms(30;15;0) = 30.25 degrees.");
                    }
                    if screen == DeviceScreen::Memory {
                        ui.label(math::format(self.device.memory, self.settings.precision));
                    }
                    if let Some(selected) = entries.get(self.device.menu_index) {
                        let announcement = format!("Selected: {selected}, {} of {}", self.device.menu_index + 1, entries.len());
                        let status = ui.label(&announcement);
                        ui.ctx().accesskit_node_builder(status.id, |node| {
                            node.set_role(egui::accesskit::Role::Status);
                            node.set_label(announcement.as_str());
                            node.clear_value();
                            node.set_live(egui::accesskit::Live::Polite);
                            node.set_live_atomic();
                        });
                    }
                    for (index, label) in entries.iter().enumerate() {
                        let selected = self.device.menu_index == index;
                        let response = ui.selectable_label(selected, R::new(label).color(if selected { palette.selected_text } else { palette.ink }));
                        if selected && self.device.menu_selection_changed {
                            response.scroll_to_me(Some(egui::Align::Center));
                            self.device.menu_selection_changed = false;
                        }
                        if response.clicked() { self.activate_device_menu(index); }
                    }
                } else {
                    match screen {
                        DeviceScreen::Home => self.device_home(ui, scale),
                        DeviceScreen::Functions => {
                            ui.strong("Y= functions");
                            for index in 0..10 {
                                let target = match index { 0 => EditTarget::FirstFunction, 1 => EditTarget::SecondFunction, _ => EditTarget::ExtraFunction(index as u8) };
                                self.device_field(ui, target, &format!("Y{} function on calculator", index + 1), 1);
                            }
                            if ui.button("Draw graph").clicked() { self.set_device_screen(DeviceScreen::Graph); }
                        }
                        DeviceScreen::Window => {
                            ui.strong("Graph window");
                            self.device_field(ui, EditTarget::WindowX, "X center", 1);
                            self.device_field(ui, EditTarget::WindowY, "Y center", 1);
                            self.device_field(ui, EditTarget::WindowRange, "Half range", 1);
                            if ui.button("Apply window").clicked() && self.apply_device_window() { self.set_device_screen(DeviceScreen::Graph); }
                        }
                        DeviceScreen::Table | DeviceScreen::TableSettings => {
                            ui.strong("Value table");
                            if screen == DeviceScreen::TableSettings {
                                self.device_field(ui, EditTarget::TableStart, "Table start", 1);
                                self.device_field(ui, EditTarget::TableStep, "Table step", 1);
                                if ui.button("Apply table settings").clicked() && self.apply_device_table() { self.set_device_screen(DeviceScreen::Table); }
                            } else {
                                if ui.button("Table settings").clicked() { self.set_device_screen(DeviceScreen::TableSettings); }
                                for row in 0..self.graph_access.table_rows.clamp(1, 20) {
                                    let x = self.graph_access.table_start + row as f64 * self.graph_access.table_step;
                                    let values = self.active_graph_indices().iter().map(|index| format!("Y{}={}", index + 1, self.graph_value(*index,x).map(|value| math::format(value,4)).unwrap_or_else(|_| "undefined".into()))).collect::<Vec<_>>().join("  ");
                                    ui.label(format!("x={}  {values}", math::format(x, 4)));
                                }
                            }
                        }
                        DeviceScreen::Statistics => {
                            ui.strong("Data and statistics");
                            if ui.button("Choose L1-L6").clicked() { self.set_device_screen(DeviceScreen::Lists); }
                            if ui.button("More statistics tools").clicked() { self.toolkit.statistics(); self.tab = Tab::Toolkit; self.show_workspace = true; }
                            self.device_field(ui, EditTarget::Data, &format!("L{}: values separated by ; or spaces", self.device.selected_list+1), 2);
                            if ui.button("1-variable statistics").clicked() { self.device.lists[self.device.selected_list] = self.device.data.clone(); self.device.stats = list_report(&self.device.data,self.settings.access.decimal_comma).unwrap_or_else(|error| error); }
                            ui.label(&self.device.stats);
                        }
                        DeviceScreen::Programs => {
                            ui.strong("Local calculator programs");
                            ui.label("Real numbers: Disp, Input, If/Then/Else, For, While, Repeat, Pause, Stop. One instruction per line.");
                            self.device_field(ui, EditTarget::ProgramName, "Program name", 1);
                            self.device_field(ui, EditTarget::Program, "Program source", 3);
                            if ui.button(if self.device.program_waiting { "Submit input" } else { "Run / continue" }).clicked() { self.run_device_program(); }
                            if self.device.program_waiting { self.device_field(ui, EditTarget::ProgramInput, "Program input value", 1); }
                            if ui.button("Restart program").clicked() { self.device.program_execution = None; self.device.program_waiting = false; self.run_device_program(); }
                            if ui.button("Save program in this session").clicked() {
                                if valid_program_name(&self.device.program_name) && self.device.programs.len() < 32 {
                                    self.device.programs.insert(self.device.program_name.clone(),self.device.program.clone());
                                    self.device.information = "Saved. Use Equal text transfer to keep a copy.".into();
                                } else { self.device.information = "Use a letter followed by up to 15 letters, digits, or underscores. Maximum 32 saved programs.".into(); }
                            }
                            for name in self.device.programs.keys().cloned().collect::<Vec<_>>() {
                                if ui.button(format!("Open {name}")).clicked() {
                                    self.device.program = self.device.programs[&name].clone(); self.device.program_name = name;
                                    self.device.program_execution = None; self.device.program_waiting = false;
                                }
                            }
                            ui.label(&self.device.program_output);
                        }
                        DeviceScreen::Lists => self.device_lists_ui(ui),
                        DeviceScreen::Sequences => self.device_sequences_ui(ui),
                        DeviceScreen::Link => self.device_transfer_ui(ui),
                        DeviceScreen::GraphCalculate => self.graph_calculation_menu(ui),
                        DeviceScreen::GraphDraw => self.graph_drawing_menu(ui),
                        DeviceScreen::Matrices => {
                            ui.strong("Matrices");
                            self.device_field(ui, EditTarget::MatrixA, "Matrix A: rows separated by ;", 1);
                            self.device_field(ui, EditTarget::MatrixB, "Matrix B", 1);
                            for operation in ["A + B", "A \u{d7} B", "Transpose A", "Determinant A", "Inverse A"] {
                                if ui.button(operation).clicked() {
                                    self.matrix_result = math::matrix(&self.matrix_a).and_then(|a| {
                                        let b = if operation.contains('B') { math::matrix(&self.matrix_b)? } else { a.clone() };
                                        math::matrix_op(&a, &b, operation)
                                    }).map(|rows| rows.iter().map(|row| row.iter().map(|value| math::format(*value, 5)).collect::<Vec<_>>().join("  ")).collect::<Vec<_>>().join("\n")).unwrap_or_else(|error| error);
                                }
                            }
                            ui.label(&self.matrix_result);
                        }
                        DeviceScreen::Distributions => {
                            ui.strong("Probability distributions");
                            ui.label("Binomial and normal distribution calculators are in Tools, under Probability.");
                            if ui.button("Open probability tools").clicked() { self.tab = Tab::Toolkit; self.show_workspace = true; }
                        }
                        DeviceScreen::Format => {
                            ui.strong("Graph format");
                            ui.checkbox(&mut self.settings.color_safe, "Patterned graph lines");
                            ui.checkbox(&mut self.settings.contrast, "High contrast");
                            if ui.button("Return to graph").clicked() { self.set_device_screen(DeviceScreen::Graph); }
                        }
                        DeviceScreen::Information => {
                            ui.label(&self.device.information);
                            if ui.button("Return to calculation").clicked() { self.set_device_screen(DeviceScreen::Home); }
                        }
                        DeviceScreen::Graph | DeviceScreen::Trace | DeviceScreen::Mode | DeviceScreen::Math
                        | DeviceScreen::Apps | DeviceScreen::Variables | DeviceScreen::Zoom | DeviceScreen::Memory
                        | DeviceScreen::Catalog | DeviceScreen::Tests | DeviceScreen::Angles => {}
                    }
                }
                if screen != DeviceScreen::Information && !self.device.information.is_empty() {
                    let information=ui.label(&self.device.information);
                    if screen == DeviceScreen::Angles {
                        ui.ctx().accesskit_node_builder(information.id,|node| {
                            node.set_role(egui::accesskit::Role::Status);
                            node.set_label(self.device.information.as_str());
                            node.clear_value();
                            node.set_live(egui::accesskit::Live::Polite);
                            node.set_live_atomic();
                        });
                    }
                }
            });
        });
    }

    fn device_home(&mut self, ui: &mut egui::Ui, scale: f32) {
        self.device.input = EditTarget::Home;
        let before = self.expression.clone();
        let cursor = self.device.cursor;
        let palette = crate::appearance::palette(ui.ctx());
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.expression)
                .id(egui::Id::new("expression"))
                .desired_width(ui.available_width())
                .font(FontId::monospace((17.0 * scale).max(15.0)))
                .text_color(palette.ink)
                .char_limit(1024)
                .hint_text("Enter calculation"),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Calculation expression")
        });
        if response.changed() {
            self.record_editor_snapshot(EditorSnapshot {
                target: EditTarget::Home,
                text: before,
                cursor,
            });
            self.device.just_evaluated = false;
        }
        if response.has_focus() {
            if let Some(state) = egui::text_edit::TextEditState::load(ui.ctx(), response.id) {
                self.device.cursor = state.cursor.char_range().map(|range| range.primary.index);
                self.device.selection = state
                    .cursor
                    .char_range()
                    .map(|range| (range.primary.index, range.secondary.index));
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.evaluate();
            }
        }
        let result = self.device_result_text();
        let label = ui.label(
            R::new(&result)
                .monospace()
                .size((24.0 * scale).max(20.0))
                .color(palette.ink),
        );
        label.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Label,
                true,
                format!(
                    "{}: {result}",
                    if self.error {
                        "Calculation error"
                    } else {
                        "Answer"
                    }
                ),
            )
        });
        ui.ctx().accesskit_node_builder(label.id, |node| {
            node.set_role(egui::accesskit::Role::Status);
            node.set_label(format!(
                "{}: {result}",
                if self.error {
                    "Calculation error"
                } else {
                    "Answer"
                }
            ));
            node.clear_value();
            node.set_live(egui::accesskit::Live::Polite);
            node.set_live_atomic();
        });
        if self.error {
            ui.label("Edit the expression, or press Clear.");
        }
        if !self.device.stats.is_empty() && self.settings.level == 0 {
            ui.label(&self.device.stats);
        }
    }

    fn device_result_text(&self) -> String {
        if self.device.fraction
            && !self.error
            && !self.device.display_complex
            && self.device.list_answer.is_none()
        {
            fraction_answer(self.device.display_value).unwrap_or_else(|| self.result.clone())
        } else {
            self.result.clone()
        }
    }

    fn device_lists_ui(&mut self, ui: &mut egui::Ui) {
        ui.strong("Named data lists");
        ui.horizontal_wrapped(|ui| {
            for index in 0..6 {
                if ui
                    .selectable_label(
                        self.device.selected_list == index,
                        format!("L{}", index + 1),
                    )
                    .clicked()
                {
                    self.device.selected_list = index;
                    self.device.input = EditTarget::List(index as u8);
                    self.device.cursor = None;
                    self.device.selection = None;
                    self.device.stats.clear();
                }
            }
        });
        let index = self.device.selected_list;
        self.device_field(
            ui,
            EditTarget::List(index as u8),
            &format!("L{} values; separate with ; or spaces", index + 1),
            3,
        );
        if ui.button("Calculate list statistics").clicked() {
            self.device.stats = list_report(
                &self.device.lists[index],
                self.settings.access.decimal_comma,
            )
            .unwrap_or_else(|error| error);
        }
        for (name, token) in [
            ("Insert list name", format!("L{}", index + 1)),
            ("Sum", format!("sum(L{})", index + 1)),
            ("Mean", format!("mean(L{})", index + 1)),
            (
                "Sample standard deviation",
                format!("stdDev(L{})", index + 1),
            ),
            (
                "Population standard deviation",
                format!("populationStdDev(L{})", index + 1),
            ),
        ] {
            if ui.button(name).clicked() {
                self.set_device_screen(DeviceScreen::Home);
                self.device_insert(&token);
            }
        }
        ui.label(&self.device.stats);
    }

    fn device_sequences_ui(&mut self, ui: &mut egui::Ui) {
        ui.strong("Sequence formulas");
        ui.label("Closed formulas in n, for integer indices. Example: n^2+1. Recurrences are not evaluated.");
        ui.horizontal_wrapped(|ui| {
            for (index, name) in ["u(n)", "v(n)", "w(n)"].iter().enumerate() {
                if ui
                    .selectable_label(self.device.selected_sequence == index, *name)
                    .clicked()
                {
                    self.device.selected_sequence = index;
                    self.device.input = EditTarget::Sequence(index as u8);
                    self.device.cursor = None;
                    self.device.selection = None;
                }
            }
        });
        self.device_field(
            ui,
            EditTarget::Sequence(self.device.selected_sequence as u8),
            "Formula in n",
            1,
        );
        self.device_field(ui, EditTarget::SequenceIndex, "Integer index n", 1);
        if ui.button("Evaluate sequence term").clicked() {
            self.evaluate_sequence();
        }
        ui.label(&self.device.sequence_result);
        if ui.button("Insert sequence call").clicked() {
            let name = ["u(", "v(", "w("][self.device.selected_sequence];
            self.set_device_screen(DeviceScreen::Home);
            self.device_insert(name);
        }
    }

    fn device_transfer_ui(&mut self, ui: &mut egui::Ui) {
        ui.strong("Equal text transfer");
        ui.label("Copy or paste your Equal variables, lists, sequences, graph functions, and programs. This text format does not communicate with a physical TI calculator or emulate its transfer protocol.");
        if ui.button("Prepare export").clicked() {
            self.export_device_transfer();
        }
        if ui.button("Copy transfer text").clicked() {
            ui.ctx().copy_text(self.device.transfer.clone());
        }
        self.device_field(
            ui,
            EditTarget::Transfer,
            "Equal transfer text: paste here to import",
            5,
        );
        if ui.button("Import transfer text").clicked() {
            self.import_device_transfer();
        }
        if ui
            .add_enabled(
                self.device.transfer_undo.is_some(),
                egui::Button::new("Undo last import"),
            )
            .clicked()
            && let Some(packet) = self.device.transfer_undo.take()
        {
            self.apply_transfer_packet(packet);
            self.device.information = "Restored the data from before the import.".into();
        }
    }

    fn run_device_program(&mut self) {
        use crate::programs::{Program, ProgramState, RunStatus};
        let changed = self.device.program_snapshot != self.device.program;
        let finished = self
            .device
            .program_execution
            .as_ref()
            .is_some_and(|(_, state)| state.status == RunStatus::Finished);
        if changed || finished || self.device.program_execution.is_none() {
            let program = match Program::parse(&self.device.program) {
                Ok(program) => program,
                Err(error) => {
                    self.device.program_output = error;
                    return;
                }
            };
            let mut state = ProgramState::new(self.device.variables.clone());
            state.answer = self.answer;
            self.device.program_execution = Some((program, state));
            self.device.program_snapshot = self.device.program.clone();
            self.device.program_waiting = false;
        }
        let Some((program, state)) = self.device.program_execution.as_mut() else {
            return;
        };
        if self.device.program_waiting {
            let input = match math::normalize_input(
                &self.device.program_input,
                self.settings.access.decimal_comma,
            ) {
                Ok(input) => input,
                Err(error) => {
                    self.device.program_output = error;
                    return;
                }
            };
            if let Err(error) = program.provide_input(state, &input, self.settings.degrees) {
                self.device.program_output = error;
                return;
            }
            self.device.program_input.clear();
        }
        let status = program.run(state, self.settings.degrees);
        self.device.variables = state.variables.clone();
        self.device
            .complex_variables
            .retain(|name, _| !self.device.variables.contains_key(name));
        self.answer = state.answer;
        self.device.complex_answer = crate::complex::Complex::from_real(state.answer);
        self.device.program_output = state.output.join("\n");
        self.device.program_waiting = false;
        match status {
            Ok(RunStatus::WaitingForInput { prompt, variable }) => {
                self.device.program_waiting = true;
                self.device
                    .program_output
                    .push_str(&format!("\n{prompt} ({variable})"));
                self.device.input = EditTarget::ProgramInput;
                self.device.cursor = None;
            }
            Ok(RunStatus::Paused) => self
                .device
                .program_output
                .push_str("\nPaused. Press Enter to continue."),
            Ok(RunStatus::Finished) => self.device.program_output.push_str("\nFinished."),
            Ok(RunStatus::Ready) => {}
            Err(error) => {
                self.device.program_output.push_str(&format!("\n{error}"));
                self.device.program_execution = None;
            }
        }
    }

    fn evaluate_list_expression(&mut self, text: &str) -> Result<bool, String> {
        let (source, target) = text
            .split_once('\u{2192}')
            .map_or((text, None), |(source, target)| {
                (source, Some(target.trim()))
            });
        let source = source.trim();
        let target_list = target.and_then(list_index);
        let is_list = list_index(source).is_some()
            || source.starts_with('{')
            || source.eq_ignore_ascii_case("ans") && self.device.list_answer.is_some();
        if !is_list && target_list.is_none() {
            return Ok(false);
        }
        if target.is_some() && target_list.is_none() {
            return Err("Store a list in L1 through L6.".into());
        }
        let values = self.device_list_values(source)?;
        if let Some(index) = target_list {
            self.device.lists[index] = list_source(&values, self.settings.access.decimal_comma);
            if self.device.selected_list == index {
                self.device.data = self.device.lists[index].clone();
            }
        }
        self.result = format!(
            "{{{}}}",
            values
                .iter()
                .map(|value| math::format(*value, self.settings.precision))
                .collect::<Vec<_>>()
                .join("; ")
        );
        self.device.list_answer = Some(values);
        self.device.display_complex = false;
        self.finish_device_calculation();
        Ok(true)
    }

    fn device_list_values(&self, source: &str) -> Result<Vec<f64>, String> {
        if let Some(index) = list_index(source) {
            return parse_list_values(
                &self.device.lists[index],
                self.settings.access.decimal_comma,
            );
        }
        if source.eq_ignore_ascii_case("ans") {
            return self
                .device
                .list_answer
                .clone()
                .ok_or_else(|| "Ans does not contain a list.".into());
        }
        let source = source
            .strip_prefix('{')
            .and_then(|source| source.strip_suffix('}'))
            .ok_or_else(|| "Enter a list such as {1; 2; 3}, or a list name L1-L6.".to_owned())?;
        // The home expression has already been normalized to dot decimals.
        parse_list_values(source, false)
    }

    fn expand_device_functions(&self, text: &str) -> Result<String, String> {
        let mut output = text.to_owned();
        // Evaluate the innermost device call first. Bound the passes and output so
        // pasted input cannot expand recursively or grow without limit.
        for _ in 0..64 {
            let Some((start, end, name, argument)) = innermost_device_call(&output)? else {
                return Ok(output);
            };
            let value = match name.as_str() {
                "u" | "v" | "w" => {
                    let argument = resolve_variables(&argument, &self.device.variables)?;
                    let n = math::calculate(&argument, self.answer, self.settings.degrees)?;
                    self.sequence_value(
                        match name.as_str() {
                            "u" => 0,
                            "v" => 1,
                            _ => 2,
                        },
                        n,
                    )?
                }
                _ => {
                    let values = self.device_list_values(argument.trim())?;
                    list_operation(&name, &values)?
                }
            };
            output.replace_range(start..end, &format!("({value:.16e})"));
            if output.len() > 8192 {
                return Err("The expanded expression is too long.".into());
            }
        }
        Err("Use at most 64 list or sequence calls per expression.".into())
    }

    fn sequence_value(&self, index: usize, n: f64) -> Result<f64, String> {
        if !n.is_finite() || n.fract() != 0.0 || n.abs() > 1_000_000.0 {
            return Err("Sequence n must be an integer from -1,000,000 to 1,000,000.".into());
        }
        let source = math::normalize_input(
            &self.device.sequences[index.min(2)],
            self.settings.access.decimal_comma,
        )?;
        if source.len() > 1024 || innermost_device_call(&source)?.is_some() {
            return Err("Use a closed formula in n, without u(), v(), w(), or list calls.".into());
        }
        let source = replace_identifier(&source, "n", "x");
        let source = resolve_variables(&source, &self.device.variables)?;
        math::parse(&close_parentheses(&source)?)?.eval(n, 0.0, self.answer, self.settings.degrees)
    }

    fn evaluate_sequence(&mut self) {
        let result = math::normalize_input(
            &self.device.sequence_index,
            self.settings.access.decimal_comma,
        )
        .and_then(|source| resolve_variables(&source, &self.device.variables))
        .and_then(|source| math::calculate(&source, self.answer, self.settings.degrees))
        .and_then(|n| {
            self.sequence_value(self.device.selected_sequence, n)
                .map(|value| (n, value))
        });
        self.device.sequence_result = match result {
            Ok((n, value)) => format!(
                "{}({}) = {}",
                ["u", "v", "w"][self.device.selected_sequence],
                math::format(n, 0),
                math::format(value, self.settings.precision)
            ),
            Err(error) => error,
        };
    }

    fn transfer_packet(&self) -> TransferPacket {
        let mut programs = self.device.programs.clone();
        if valid_program_name(&self.device.program_name) {
            programs.insert(
                self.device.program_name.clone(),
                self.device.program.clone(),
            );
        }
        TransferPacket {
            version: 1,
            variables: self.device.variables.clone(),
            complex_variables: self.device.complex_variables.clone(),
            lists: self.device.lists.clone(),
            sequences: self.device.sequences.clone(),
            functions: (0..10)
                .map(|index| self.graph_function(index).to_owned())
                .collect(),
            programs,
        }
    }

    fn export_device_transfer(&mut self) {
        let mut packet = self.transfer_packet();
        for source in &mut packet.lists {
            if source.trim().is_empty() {
                continue;
            }
            match parse_list_values(source, self.settings.access.decimal_comma) {
                Ok(values) => *source = list_source(&values, false),
                Err(error) => {
                    self.device.information = error;
                    return;
                }
            }
        }
        for source in &mut packet.sequences {
            match math::normalize_input(source, self.settings.access.decimal_comma) {
                Ok(value) => *source = value,
                Err(error) => {
                    self.device.information = error;
                    return;
                }
            }
        }
        if let Err(error) = validate_transfer(&packet) {
            self.device.information = format!("Cannot export: {error}");
            return;
        }
        match serde_json::to_string_pretty(&packet) {
            Ok(text) if text.len() <= MAX_TRANSFER_BYTES => {
                self.device.transfer = text;
                self.device.information =
                    "Export ready. Copy the text or select it manually.".into();
            }
            Ok(_) => self.device.information = "The export exceeds 160 KiB.".into(),
            Err(error) => self.device.information = format!("Export failed: {error}"),
        }
    }

    fn apply_transfer_packet(&mut self, packet: TransferPacket) {
        self.device.variables = packet.variables;
        self.device.complex_variables = packet.complex_variables;
        self.device.lists = packet.lists;
        self.device.data = self.device.lists[self.device.selected_list].clone();
        self.device.sequences = packet.sequences;
        for (index, source) in packet.functions.into_iter().enumerate().take(10) {
            *self.graph_function_mut(index) = source;
        }
        self.device.programs = packet.programs;
        if let Some((name, source)) = self.device.programs.first_key_value() {
            self.device.program_name = name.clone();
            self.device.program = source.clone();
        }
        self.device.program_execution = None;
        self.device.program_waiting = false;
    }

    fn import_device_transfer(&mut self) {
        if self.device.transfer.len() > MAX_TRANSFER_BYTES {
            self.device.information = "Import is limited to 160 KiB.".into();
            return;
        }
        let result = serde_json::from_str::<TransferPacket>(&self.device.transfer)
            .map_err(|error| format!("Invalid Equal transfer text: {error}"))
            .and_then(|packet| {
                validate_transfer(&packet)?;
                Ok(packet)
            });
        match result {
            Ok(mut packet) => {
                for source in &mut packet.lists {
                    if source.trim().is_empty() {
                        continue;
                    }
                    if let Ok(values) = parse_list_values(source, false) {
                        *source = list_source(&values, self.settings.access.decimal_comma);
                    }
                }
                self.device.transfer_undo = Some(self.transfer_packet());
                self.apply_transfer_packet(packet);
                self.device.information = "Imported variables, six lists, sequences, graph functions, and saved programs. Undo import is available.".into();
            }
            Err(error) => self.device.information = error,
        }
    }
}

fn device_input_id(target: EditTarget) -> egui::Id {
    if target == EditTarget::Home {
        egui::Id::new("expression")
    } else {
        egui::Id::new(("device-input", target))
    }
}

fn editor_limit(target: EditTarget) -> usize {
    match target {
        EditTarget::Transfer => MAX_TRANSFER_BYTES,
        EditTarget::Data | EditTarget::List(_) => MAX_LIST_BYTES,
        EditTarget::Program => 8000,
        EditTarget::ProgramName => 16,
        _ => 1024,
    }
}

fn character_offset(text: &str, position: usize) -> usize {
    text.char_indices()
        .nth(position)
        .map_or(text.len(), |(index, _)| index)
}

fn adjacent_index(index: usize, count: usize, forward: bool) -> usize {
    if forward {
        (index + 1) % count
    } else {
        (index + count - 1) % count
    }
}

fn graph_edit_target(index: usize) -> EditTarget {
    match index {
        0 => EditTarget::FirstFunction,
        1 => EditTarget::SecondFunction,
        _ => EditTarget::ExtraFunction(index.min(9) as u8),
    }
}

fn vertical_cursor(text: &str, position: usize, forward: bool) -> usize {
    let characters: Vec<char> = text.chars().collect();
    let position = position.min(characters.len());
    let line_start = characters[..position]
        .iter()
        .rposition(|character| *character == '\n')
        .map_or(0, |index| index + 1);
    let column = position - line_start;
    if forward {
        let Some(end) = characters[line_start..]
            .iter()
            .position(|character| *character == '\n')
            .map(|index| index + line_start)
        else {
            return characters.len();
        };
        let next_start = end + 1;
        let next_end = characters[next_start..]
            .iter()
            .position(|character| *character == '\n')
            .map_or(characters.len(), |index| index + next_start);
        (next_start + column).min(next_end)
    } else {
        if line_start == 0 {
            return 0;
        }
        let previous_end = line_start - 1;
        let previous_start = characters[..previous_end]
            .iter()
            .rposition(|character| *character == '\n')
            .map_or(0, |index| index + 1);
        (previous_start + column).min(previous_end)
    }
}

fn screen_for_target(target: EditTarget) -> DeviceScreen {
    match target {
        EditTarget::Home => DeviceScreen::Home,
        EditTarget::FirstFunction | EditTarget::SecondFunction | EditTarget::ExtraFunction(_) => {
            DeviceScreen::Functions
        }
        EditTarget::WindowX | EditTarget::WindowY | EditTarget::WindowRange => DeviceScreen::Window,
        EditTarget::TableStart | EditTarget::TableStep => DeviceScreen::TableSettings,
        EditTarget::Data => DeviceScreen::Statistics,
        EditTarget::Program | EditTarget::ProgramName | EditTarget::ProgramInput => {
            DeviceScreen::Programs
        }
        EditTarget::MatrixA | EditTarget::MatrixB => DeviceScreen::Matrices,
        EditTarget::List(_) => DeviceScreen::Lists,
        EditTarget::Sequence(_) | EditTarget::SequenceIndex => DeviceScreen::Sequences,
        EditTarget::Transfer => DeviceScreen::Link,
    }
}

fn split_store(text: &str) -> Result<(&str, Option<char>), String> {
    if let Some((expression, target)) = text.split_once('\u{2192}') {
        let mut characters = target.trim().chars();
        let name = characters
            .next()
            .filter(char::is_ascii_uppercase)
            .ok_or_else(|| "Choose one variable A-Z after STO.".to_owned())?;
        if characters.next().is_some() {
            return Err("Choose one variable A-Z after STO.".into());
        }
        Ok((expression, Some(name)))
    } else {
        Ok((text, None))
    }
}

fn close_parentheses(text: &str) -> Result<String, String> {
    let mut depth = 0_usize;
    for character in text.chars() {
        match character {
            '(' => {
                depth += 1;
                if depth > 48 {
                    return Err("Use at most 48 nested parentheses.".into());
                }
            }
            ')' => {
                if depth == 0 {
                    return Err("There is a closing parenthesis without an opening one.".into());
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    let mut result = text.to_owned();
    result.extend(std::iter::repeat_n(')', depth));
    Ok(result)
}

fn has_identifier(text: &str, name: &str) -> bool {
    text.split(|character: char| !character.is_ascii_alphabetic() && character != '_')
        .any(|token| token == name)
}

fn replace_identifier(text: &str, name: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut position = 0;
    while position < text.len() {
        let start = position;
        let character = text[position..].chars().next().unwrap_or(' ');
        if character.is_ascii_alphabetic() {
            position += character.len_utf8();
            while position < text.len() && text.as_bytes()[position].is_ascii_alphabetic() {
                position += 1;
            }
            let token = &text[start..position];
            result.push_str(if token == name { replacement } else { token });
        } else {
            result.push(character);
            position += character.len_utf8();
        }
    }
    result
}

fn list_index(source: &str) -> Option<usize> {
    let source = source.trim().as_bytes();
    if source.len() == 2 && source[0] == b'L' && (b'1'..=b'6').contains(&source[1]) {
        Some(usize::from(source[1] - b'1'))
    } else {
        None
    }
}

fn split_list_items(text: &str) -> Result<Vec<&str>, String> {
    let mut items = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Err("Unbalanced parentheses in list.".into());
                }
                depth -= 1;
            }
            _ if depth == 0
                && (character == ',' || character == ';' || character.is_whitespace()) =>
            {
                if !text[start..index].trim().is_empty() {
                    items.push(text[start..index].trim());
                }
                start = index + character.len_utf8();
            }
            _ => {}
        }
        if depth > 48 {
            return Err("Too many nested parentheses in list.".into());
        }
    }
    if depth != 0 {
        return Err("Close the parentheses in the list.".into());
    }
    if !text[start..].trim().is_empty() {
        items.push(text[start..].trim());
    }
    Ok(items)
}

fn parse_list_values(text: &str, decimal_comma: bool) -> Result<Vec<f64>, String> {
    if text.len() > MAX_LIST_BYTES {
        return Err(format!("A list is limited to {MAX_LIST_BYTES} bytes."));
    }
    // Split the list before normalizing each scalar. A thousand full-precision
    // values may exceed the scalar parser's size limit while remaining bounded.
    let separators = text.replace('\u{ff0c}', ",").replace('\u{ff1b}', ";");
    let source = if decimal_comma {
        separators.replace(',', ".")
    } else {
        separators
    };
    let items = split_list_items(&source)?;
    if items.is_empty() || items.len() > 1000 {
        return Err("Enter 1-1,000 finite values, separated by ; or spaces.".into());
    }
    items
        .iter()
        .map(|item| {
            math::normalize_input(item, false)
                .and_then(|item| math::calculate(&item, 0.0, true))
                .map_err(|error| format!("List value {item}: {error}"))
        })
        .collect()
}

fn list_source(values: &[f64], decimal_comma: bool) -> String {
    values
        .iter()
        .map(|value| {
            let value = format!("{value:e}");
            if decimal_comma {
                value.replace('.', ",")
            } else {
                value
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn list_operation(name: &str, values: &[f64]) -> Result<f64, String> {
    if values.is_empty() {
        return Err("The selected list is empty.".into());
    }
    let count = values.len() as f64;
    let sum = values.iter().sum::<f64>();
    let mean = sum / count;
    let squared = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>();
    let value = match name {
        "sum" => sum,
        "mean" => mean,
        "dim" => count,
        "min" => values.iter().copied().fold(f64::INFINITY, f64::min),
        "max" => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        "stdDev" | "stddev" => {
            if values.len() < 2 {
                return Err("Sample standard deviation needs at least two values.".into());
            }
            (squared / (count - 1.0)).sqrt()
        }
        "populationStdDev" => (squared / count).sqrt(),
        _ => return Err("Unknown list operation.".into()),
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err("The list result exceeds the numeric range.".into())
    }
}

fn list_report(text: &str, decimal_comma: bool) -> Result<String, String> {
    let values = parse_list_values(text, decimal_comma)?;
    let sample = list_operation("stdDev", &values)
        .map(|value| math::format(value, 6))
        .unwrap_or_else(|_| "needs two values".into());
    Ok(format!(
        "n={}  sum={}\nMean={}\nPopulation standard deviation={}\nSample standard deviation={sample}",
        values.len(),
        math::format(list_operation("sum", &values)?, 6),
        math::format(list_operation("mean", &values)?, 6),
        math::format(list_operation("populationStdDev", &values)?, 6)
    ))
}

fn innermost_device_call(text: &str) -> Result<Option<(usize, usize, String, String)>, String> {
    let mut found = None;
    let mut position = 0;
    while position < text.len() {
        let character = text[position..].chars().next().unwrap_or(' ');
        if !character.is_ascii_alphabetic() {
            position += character.len_utf8();
            continue;
        }
        let start = position;
        while position < text.len() && text.as_bytes()[position].is_ascii_alphabetic() {
            position += 1;
        }
        let name = &text[start..position];
        let open = position + text[position..].len() - text[position..].trim_start().len();
        if text.as_bytes().get(open) != Some(&b'(') {
            continue;
        }
        let device_call = matches!(
            name,
            "u" | "v" | "w" | "sum" | "mean" | "stdDev" | "stddev" | "populationStdDev" | "dim"
        ) || matches!(name, "min" | "max")
            && (text[open + 1..].trim_start().starts_with('L')
                || text[open + 1..].trim_start().starts_with('{'));
        if !device_call {
            continue;
        }
        let mut depth = 1_usize;
        let mut end = None;
        for (offset, character) in text[open + 1..].char_indices() {
            if character == '(' {
                depth += 1;
            }
            if character == ')' {
                depth -= 1;
                if depth == 0 {
                    end = Some(open + 1 + offset);
                    break;
                }
            }
            if depth > 48 {
                return Err("Too many nested device functions.".into());
            }
        }
        let close = end.ok_or_else(|| "Close the list or sequence function with ).".to_owned())?;
        found = Some((
            start,
            close + 1,
            name.to_owned(),
            text[open + 1..close].to_owned(),
        ));
    }
    Ok(found)
}

fn valid_program_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 16
        && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn validate_transfer(packet: &TransferPacket) -> Result<(), String> {
    if packet.version != 1 || packet.functions.len() != 10 {
        return Err("Expected Equal transfer version 1 with ten graph functions.".into());
    }
    if packet.variables.len() > 26
        || packet.complex_variables.len() > 26
        || packet
            .variables
            .iter()
            .any(|(name, value)| !name.is_ascii_uppercase() || !value.is_finite())
        || packet.complex_variables.iter().any(|(name, value)| {
            !name.is_ascii_uppercase() || !value.re.is_finite() || !value.im.is_finite()
        })
    {
        return Err("Variables must use A-Z and finite values.".into());
    }
    for list in &packet.lists {
        if !list.trim().is_empty() {
            parse_list_values(list, false)?;
        }
    }
    for source in &packet.functions {
        if source.len() > 1024 {
            return Err("A graph function exceeds 1,024 bytes.".into());
        }
        if !source.trim().is_empty() {
            math::parse(source)?;
        }
    }
    for source in &packet.sequences {
        if source.len() > 1024 || innermost_device_call(source)?.is_some() {
            return Err("Sequence formulas must be bounded closed formulas in n.".into());
        }
        math::parse(&resolve_variables(
            &replace_identifier(source, "n", "x"),
            &packet.variables,
        )?)?;
    }
    if packet.programs.len() > 32 {
        return Err("Import at most 32 programs at a time.".into());
    }
    for (name, source) in &packet.programs {
        if !valid_program_name(name) {
            return Err("Program names must start with a letter and contain at most 16 letters, digits, or underscores.".into());
        }
        crate::programs::Program::parse(source)?;
    }
    Ok(())
}

fn comparison_parts(text: &str) -> Option<(&str, &str, &str)> {
    let mut depth = 0_usize;
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => {
                for operation in ["!=", "<=", ">=", "=", "<", ">"] {
                    if text[index..].starts_with(operation) {
                        return Some((&text[..index], &text[index + operation.len()..], operation));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn fraction_answer(value: f64) -> Option<String> {
    if !value.is_finite() || value.abs() > 1e9 {
        return None;
    }
    for denominator in 1..=10_000 {
        let numerator = value * f64::from(denominator);
        if (numerator - numerator.round()).abs() < 1e-9 {
            return Some(if denominator == 1 {
                format!("{:.0}", numerator)
            } else {
                format!("{:.0}/{denominator}", numerator.round())
            });
        }
    }
    None
}

fn format_dms(value: f64, precision: usize) -> Result<String, String> {
    if !value.is_finite() || value.abs() > 1_000_000_000.0 {
        return Err(
            "DMS display needs a finite angle within plus or minus one billion degrees.".into(),
        );
    }
    let precision = precision.min(6);
    let scale = 10.0_f64.powi(precision as i32);
    let mut degrees = value.abs().floor();
    let remainder = (value.abs() - degrees) * 3600.0;
    let mut minutes = (remainder / 60.0).floor();
    let mut seconds = ((remainder - minutes * 60.0) * scale).round() / scale;
    if seconds >= 60.0 {
        seconds = 0.0;
        minutes += 1.0;
    }
    if minutes >= 60.0 {
        minutes = 0.0;
        degrees += 1.0;
    }
    Ok(format!(
        "{}{} degrees, {} minutes, {} seconds",
        if value.is_sign_negative() { "-" } else { "" },
        math::format(degrees, 0),
        math::format(minutes, 0),
        math::format(seconds, precision)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn returning_home_from_an_editor_restores_the_calculation_input() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let graph = app.graph_a.clone();
        for screen in [
            DeviceScreen::Functions,
            DeviceScreen::Window,
            DeviceScreen::Statistics,
        ] {
            app.set_device_screen(screen);
            app.set_device_screen(DeviceScreen::Home);
            app.press_device_key(&ctx, "7");
            assert_eq!(app.expression, "7");
            assert_eq!(app.graph_a, graph);
            app.press_device_key(&ctx, "clear");
        }
    }
    #[test]
    fn stores_and_recalls_named_variables_without_changing_function_names() {
        let mut device = DeviceState::default();
        assert_eq!(device.calculate("5→A", 0.0, true), Ok(5.0));
        assert_eq!(device.calculate("A*2+sin(30)", 0.0, true), Ok(10.5));
        assert!(device.calculate("B+1", 0.0, true).is_err());
        assert!(device.calculate("1→AB", 0.0, true).is_err());
        assert!(!device.variables.contains_key(&'B'));
    }
    #[test]
    fn fractions_statistics_and_handheld_parentheses_work() {
        assert_eq!(fraction_answer(0.75).as_deref(), Some("3/4"));
        assert!(fraction_answer(f64::INFINITY).is_none());
        assert!(list_report("1, 2, 3", false).unwrap().contains("Mean=2"));
        assert!(list_report("NaN", false).is_err());
        assert_eq!(
            DeviceState::default().calculate("sqrt(9", 0.0, true),
            Ok(3.0)
        );
    }
    #[test]
    fn graph_keys_stay_on_calculator_and_second_graph_opens_table() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let ctx = egui::Context::default();
        for (key, screen) in [
            ("y=", DeviceScreen::Functions),
            ("window", DeviceScreen::Window),
            ("zoom", DeviceScreen::Zoom),
            ("trace", DeviceScreen::Trace),
            ("graph", DeviceScreen::Graph),
            ("mode", DeviceScreen::Mode),
        ] {
            app.press_device_key(&ctx, key);
            assert_eq!(app.device.screen, screen);
            assert!(app.tab == Tab::Calculator);
        }
        app.press_device_key(&ctx, "2nd");
        app.press_device_key(&ctx, "graph");
        assert_eq!(app.device.screen, DeviceScreen::Table);
        assert!(!app.second);
    }
    #[test]
    fn memory_sign_and_cursor_delete_are_actual_key_actions() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let ctx = egui::Context::default();
        for key in ["9", "M+", "clear", "4", "M-", "clear", "MRC"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.expression, "5");
        app.press_device_key(&ctx, "MRC");
        assert_eq!(app.expression, "0");
        for key in ["clear", "1", "2", "3", "cursor left", "delete"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.expression, "13");
    }

    #[test]
    fn physical_keys_edit_function_and_data_fields_instead_of_home() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let ctx = egui::Context::default();
        app.expression = "123".into();
        for key in ["y=", "clear", "x,t", "x²", "+", "2"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.graph_a, "x^2+2");
        assert_eq!(app.expression, "123");
        assert_eq!(app.device.screen, DeviceScreen::Functions);
        for key in ["enter", "clear", "3", "x,t", "delete", "+", "1"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.graph_b, "3+1");
        for key in ["stat", "clear", "1", ",", "2", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.data, "1,2");
        assert!(app.device.stats.contains("Mean=1.5"));
        assert_eq!(app.expression, "123");
    }

    #[test]
    fn elementary_on_clear_and_root_preserve_answer_memory() {
        let mut app = EqualApp::with_settings(
            Settings {
                level: 0,
                ..Settings::default()
            },
            None,
        );
        let ctx = egui::Context::default();
        for key in ["9", "√", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.result, "3");
        app.device.fraction = true;
        app.press_device_key(&ctx, "on");
        assert_eq!(app.device_result_text(), "0");
        assert_eq!(app.answer, 3.0);
        app.press_device_key(&ctx, "Ans");
        app.press_device_key(&ctx, "enter");
        assert_eq!(app.result, "3");
    }

    #[test]
    fn school_store_picker_and_physical_menu_navigation_work() {
        let mut app = EqualApp::with_settings(
            Settings {
                level: 1,
                ..Settings::default()
            },
            None,
        );
        let ctx = egui::Context::default();
        for key in ["5", "sto", "enter", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.variables.get(&'A'), Some(&5.0));
        for key in ["mode", "cursor right", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert!(!app.settings.degrees);
        let range = app.range;
        for key in ["zoom", "cursor end", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert!(app.range > range);
        assert_eq!(app.device.screen, DeviceScreen::Graph);
    }

    #[test]
    fn window_keypad_edits_apply_and_secondary_tests_are_true_tests() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let ctx = egui::Context::default();
        for key in [
            "window", "clear", "3", "enter", "clear", "4", "enter", "clear", "5", "enter",
        ] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.center, [3.0, 4.0]);
        assert_eq!(app.range, 5.0);
        assert_eq!(app.device.screen, DeviceScreen::Graph);
        for key in ["clear", "2", "2nd", "math", "enter", "2", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.result, "1");
        for key in ["2nd", "."] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.screen, DeviceScreen::Home);
        assert!(app.device.complex_mode);
        app.press_device_key(&ctx, "enter");
        assert!(!app.error, "{}", app.result);
        assert_eq!(app.device.complex_answer.im, 1.0);
    }

    #[test]
    fn editing_selection_delete_and_undo_preserve_the_active_target() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        for key in ["1", "2", "3", "4"] {
            app.press_device_key(&ctx, key);
        }
        app.device.selection = Some((3, 1));
        app.press_device_key(&ctx, "9");
        assert_eq!(app.expression, "194");
        assert_eq!(app.device.cursor, Some(2));
        app.press_device_key(&ctx, "undo");
        assert_eq!(app.expression, "1234");
        app.press_device_key(&ctx, "redo");
        assert_eq!(app.expression, "194");
        app.device.selection = Some((0, 2));
        app.press_device_key(&ctx, "clear");
        assert_eq!(app.expression, "4");
        app.press_device_key(&ctx, "y=");
        app.press_device_key(&ctx, "clear");
        app.press_device_key(&ctx, "8");
        app.press_device_key(&ctx, "undo");
        assert_eq!(app.graph_a, "");
        assert_eq!(app.device.screen, DeviceScreen::Functions);
        assert_eq!(app.expression, "4");
    }

    #[test]
    fn exponent_entry_alpha_lock_and_all_letter_storage_work() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        for key in ["1", "2nd", ",", "3", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.answer, 1000.0);
        for name in 'A'..='Z' {
            app.expression = format!("{}\u{2192}{name}", u32::from(name) - u32::from('A') + 1);
            app.evaluate();
            assert!(!app.error, "{}", app.result);
        }
        assert_eq!(app.device.variables.len(), 26);
        app.expression = "A+Z".into();
        app.evaluate();
        assert_eq!(app.answer, 27.0);
        app.press_device_key(&ctx, "clear");
        for key in ["2nd", "alpha", "math", "apps", "alpha", "7"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.expression, "AB7");
        assert!(!app.device.alpha);
    }

    #[test]
    fn complex_answers_store_recall_and_survive_clear_without_losing_imaginary_part() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.expression = "1+2i\u{2192}Z".into();
        app.evaluate();
        assert!(!app.error, "{}", app.result);
        assert_eq!(
            app.device.complex_variables[&'Z'],
            crate::complex::Complex { re: 1.0, im: 2.0 }
        );
        app.device.complex_mode = false;
        app.press_device_key(&ctx, "clear");
        for key in ["Ans", "^", "2", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert!(!app.error, "{}", app.result);
        assert!((app.device.complex_answer.re + 3.0).abs() < 1e-10);
        assert!((app.device.complex_answer.im - 4.0).abs() < 1e-10);
        app.expression = "Z/(1+2i)".into();
        app.evaluate();
        assert!(!app.error, "{}", app.result);
        assert!((app.device.complex_answer.re - 1.0).abs() < 1e-10);
        assert!(app.device.complex_answer.im.abs() < 1e-10);
        app.expression = "Z<1".into();
        app.evaluate();
        assert!(app.error);
    }

    #[test]
    fn six_lists_have_distinct_keys_storage_and_correct_aggregates() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.expression = "{1;2;3}\u{2192}L6".into();
        app.evaluate();
        assert!(!app.error, "{}", app.result);
        app.press_device_key(&ctx, "clear");
        for key in ["2nd", "6", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.list_answer, Some(vec![1.0, 2.0, 3.0]));
        app.expression = "ans\u{2192}L2".into();
        app.evaluate();
        assert_eq!(app.device.lists[1], app.device.lists[5]);
        for (expression, expected) in [
            ("sum(L6)", 6.0),
            ("mean(L2)", 2.0),
            ("stdDev(L6)", 1.0),
            ("populationStdDev(L6)", (2.0_f64 / 3.0).sqrt()),
        ] {
            app.expression = expression.into();
            app.evaluate();
            assert!(!app.error, "{expression}: {}", app.result);
            assert!((app.answer - expected).abs() < 1e-10);
        }
        app.press_device_key(&ctx, "lists");
        for index in 1..=6 {
            app.press_device_key(&ctx, "2nd");
            app.press_device_key(&ctx, &index.to_string());
            assert_eq!(app.device.selected_list, index - 1);
            assert_eq!(app.device.input, EditTarget::List((index - 1) as u8));
        }
    }

    #[test]
    fn sequence_formulas_evaluate_in_n_and_enforce_integer_bounds() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.device.sequences = ["n^2+1".into(), "2*n".into(), "n+7".into()];
        app.expression = "u(3)+v(4)+w(2)".into();
        app.evaluate();
        assert!(!app.error, "{}", app.result);
        assert_eq!(app.answer, 27.0);
        for source in ["u(1.5)", "v(1000001)"] {
            app.expression = source.into();
            app.evaluate();
            assert!(app.error);
        }
        app.device.sequences[0] = "u(n-1)+1".into();
        app.expression = "u(5)".into();
        app.evaluate();
        assert!(app.error);
        assert!(app.result.contains("closed formula"));
    }

    #[test]
    fn ten_function_keys_and_draw_calc_menus_stay_inside_the_calculator() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.press_device_key(&ctx, "y=");
        for index in 0..10 {
            app.press_device_key(&ctx, "clear");
            app.press_device_key(&ctx, &(index + 1).to_string());
            assert_eq!(app.graph_function(index), (index + 1).to_string());
            app.press_device_key(&ctx, "enter");
        }
        assert_eq!(app.device.input, EditTarget::FirstFunction);
        for (key, screen) in [
            ("trace", DeviceScreen::GraphCalculate),
            ("prgm", DeviceScreen::GraphDraw),
            ("x,t", DeviceScreen::Link),
        ] {
            app.press_device_key(&ctx, "2nd");
            app.press_device_key(&ctx, key);
            assert_eq!(app.device.screen, screen);
            assert_eq!(app.tab, Tab::Calculator);
        }
    }

    #[test]
    fn calculator_program_input_pause_and_restart_have_real_state() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.device.program =
            "Input A\nFor(B,1,3)\nA+B\u{2192}A\nEnd\nDisp A\nPause\nDisp A*2".into();
        app.press_device_key(&ctx, "prgm");
        app.press_device_key(&ctx, "enter");
        assert!(app.device.program_waiting);
        app.device.program_input = "4".into();
        app.press_device_key(&ctx, "enter");
        assert!(!app.device.program_waiting);
        assert_eq!(app.device.variables.get(&'A'), Some(&10.0));
        assert!(app.device.program_output.contains("Paused"));
        app.press_device_key(&ctx, "enter");
        assert!(app.device.program_output.contains("20"));
        assert!(app.device.program_output.contains("Finished"));
    }

    #[test]
    fn transfer_import_is_atomic_bounded_and_reversible() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.device.variables.insert('A', 7.0);
        *app.graph_function_mut(9) = "x^3".into();
        app.export_device_transfer();
        assert!(
            !app.device.transfer.is_empty(),
            "{}",
            app.device.information
        );
        let export = app.device.transfer.clone();
        app.device.variables.insert('A', 99.0);
        app.device.transfer = "{\"version\":1}".into();
        app.import_device_transfer();
        assert_eq!(app.device.variables[&'A'], 99.0);
        app.device.transfer = export;
        app.import_device_transfer();
        assert_eq!(app.device.variables[&'A'], 7.0);
        assert_eq!(app.graph_function(9), "x^3");
        let previous = app.device.transfer_undo.take().unwrap();
        app.apply_transfer_packet(previous);
        assert_eq!(app.device.variables[&'A'], 99.0);
        let mut invalid = app.transfer_packet();
        invalid.variables.insert('z', 4.0);
        assert!(validate_transfer(&invalid).is_err());
    }

    #[test]
    fn decimal_comma_input_and_lists_keep_values_unambiguous() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.settings.access.decimal_comma = true;
        app.expression = "1,5+2,25".into();
        app.evaluate();
        assert_eq!(app.answer, 3.75);
        app.device.lists[0] = "1,5; 2,5; 3,5".into();
        app.expression = "mean(L1)".into();
        app.evaluate();
        assert!(!app.error, "{}", app.result);
        assert_eq!(app.answer, 2.5);
    }

    #[test]
    fn full_precision_thousand_value_lists_round_trip_within_their_own_bound() {
        let values: Vec<f64> = (0..1000)
            .map(|index| 1.2345678901234567 + f64::from(index) / 1000.0)
            .collect();
        let source = list_source(&values, false);
        assert!(source.len() > 8192);
        assert!(source.len() <= MAX_LIST_BYTES);
        assert_eq!(parse_list_values(&source, false).unwrap(), values);
        let localized = list_source(&values, true);
        assert_eq!(parse_list_values(&localized, true).unwrap(), values);
        assert!(parse_list_values(&list_source(&[1.0; 1001], false), false).is_err());
    }

    #[test]
    fn shifted_dpad_never_types_control_labels_and_device_calls_close_at_enter() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        for key in ["1", "2", "2nd", "cursor left", "9"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.expression, "19");
        assert!(!app.second);
        app.expression = "mean(L1".into();
        app.evaluate();
        assert!(!app.error, "{}", app.result);
        assert!((app.answer - 31.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn physical_up_down_reverses_fields_and_moves_multiline_carets() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.press_device_key(&ctx, "y=");
        app.press_device_key(&ctx, "previous calculation");
        assert_eq!(app.device.input, EditTarget::ExtraFunction(9));
        app.press_device_key(&ctx, "cursor end");
        assert_eq!(app.device.input, EditTarget::FirstFunction);
        for key in ["cursor end", "cursor end", "previous calculation"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.input, EditTarget::SecondFunction);
        app.press_device_key(&ctx, "window");
        app.press_device_key(&ctx, "previous calculation");
        assert_eq!(app.device.input, EditTarget::WindowRange);
        app.press_device_key(&ctx, "cursor end");
        assert_eq!(app.device.input, EditTarget::WindowX);
        app.press_device_key(&ctx, "prgm");
        app.device.program = "abc\n12\nwxyz".into();
        app.device.cursor = Some(2);
        for (key, position) in [
            ("cursor end", 6),
            ("cursor end", 9),
            ("previous calculation", 6),
            ("previous calculation", 2),
        ] {
            app.press_device_key(&ctx, key);
            assert_eq!(app.device.cursor, Some(position));
            assert_eq!(app.device.input, EditTarget::Program);
        }
        assert_eq!(
            vertical_cursor("\u{3b1}\u{3b2}\u{3b3}\n12\nxyz", 2, true),
            6
        );
        for index in 0..10 {
            *app.graph_function_mut(index) = String::new();
        }
        *app.graph_function_mut(0) = "x".into();
        *app.graph_function_mut(9) = "x^2".into();
        app.graph_access.selected = 0;
        app.press_device_key(&ctx, "trace");
        app.press_device_key(&ctx, "previous calculation");
        assert_eq!(app.graph_access.selected, 9);
        app.press_device_key(&ctx, "cursor end");
        assert_eq!(app.graph_access.selected, 0);
    }

    #[test]
    fn direct_expression_actions_undo_once_and_program_store_keeps_its_target() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.expression = "7+2".into();
        for key in ["\u{b1}", "1/x"] {
            app.press_device_key(&ctx, key);
            assert_ne!(app.expression, "7+2");
            app.press_device_key(&ctx, "undo");
            assert_eq!(app.expression, "7+2");
        }
        app.expression = "2+3".into();
        app.evaluate();
        app.press_device_key(&ctx, "sto");
        assert_eq!(app.expression, "ans\u{2192}");
        app.press_device_key(&ctx, "undo");
        assert_eq!(app.expression, "2+3");
        app.device.memory = 42.0;
        app.press_device_key(&ctx, "MRC");
        assert_eq!(app.expression, "42");
        app.press_device_key(&ctx, "undo");
        assert_eq!(app.expression, "2+3");
        app.expression = "99".into();
        for key in ["2nd", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.expression, "2+3");
        app.press_device_key(&ctx, "undo");
        assert_eq!(app.expression, "99");
        app.press_device_key(&ctx, "prgm");
        app.device.program = "7".into();
        app.press_device_key(&ctx, "sto");
        assert_eq!(app.device.program, "7\u{2192}");
        assert_eq!(app.expression, "99");
        app.press_device_key(&ctx, "undo");
        assert_eq!(app.device.program, "7");
        assert_eq!(app.device.screen, DeviceScreen::Programs);
        app.settings.level = 0;
        app.set_device_screen(DeviceScreen::Home);
        app.expression = "9".into();
        app.press_device_key(&ctx, "\u{221a}");
        assert_eq!(app.expression, "sqrt(9)");
        app.press_device_key(&ctx, "undo");
        assert_eq!(app.expression, "9");
    }

    #[test]
    fn insertion_menus_preserve_function_program_and_home_editing_targets() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.expression = "123".into();
        for key in ["y=", "clear", "math", "enter", "9"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.graph_a, "sqrt(9");
        assert_eq!(app.expression, "123");
        assert_eq!(app.device.screen, DeviceScreen::Functions);
        app.press_device_key(&ctx, "prgm");
        app.device.program = "Disp ".into();
        app.device.cursor = None;
        for key in ["vars", "cursor right", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.program, "Disp B");
        assert_eq!(app.expression, "123");
        app.set_device_screen(DeviceScreen::Home);
        app.device.cursor = Some(1);
        app.device.selection = Some((1, 2));
        for key in ["vars", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.expression, "1A3");
        app.expression = "2+3".into();
        app.evaluate();
        for key in ["vars", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.expression, "A");
    }

    #[test]
    fn angle_menu_converts_real_units_and_dms_without_changing_angle_mode() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.expression = "180".into();
        for key in ["2nd", "apps"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.screen, DeviceScreen::Angles);
        for key in ["enter", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert!(!app.error, "{}", app.result);
        assert!((app.answer - std::f64::consts::PI).abs() < 1e-12);
        assert!(app.settings.degrees);
        for key in ["2nd", "apps", "cursor right", "enter", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert!(!app.error, "{}", app.result);
        assert!((app.answer - 180.0).abs() < 1e-10);
        app.press_device_key(&ctx, "clear");
        for key in [
            "2nd",
            "apps",
            "cursor right",
            "cursor right",
            "enter",
            "-",
            "3",
            "0",
            ";",
            "1",
            "5",
            ";",
            "3",
            "0",
            ")",
            "enter",
        ] {
            app.press_device_key(&ctx, key);
        }
        assert!(!app.error, "{}", app.result);
        assert!((app.answer + 30.258333333333333).abs() < 1e-10);
        let expression = app.expression.clone();
        for key in [
            "2nd",
            "apps",
            "cursor right",
            "cursor right",
            "cursor right",
            "enter",
        ] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.screen, DeviceScreen::Angles);
        assert_eq!(app.expression, expression);
        assert_eq!(
            app.device.information,
            "-30 degrees, 15 minutes, 30 seconds"
        );
        assert!(app.settings.degrees);
        assert_eq!(
            format_dms(12.999999999999, 2).unwrap(),
            "13 degrees, 0 minutes, 0 seconds"
        );
        assert_eq!(
            format_dms(-0.5, 6).unwrap(),
            "-0 degrees, 30 minutes, 0 seconds"
        );
        assert!(format_dms(f64::INFINITY, 6).is_err());
    }

    #[test]
    fn angle_and_bracket_keys_preserve_the_exact_function_or_program_editor() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.expression = "123".into();
        app.set_device_screen(DeviceScreen::Functions);
        app.device.input = EditTarget::ExtraFunction(4);
        *app.graph_function_mut(4) = "12+34".into();
        app.device.cursor = Some(3);
        app.device.selection = Some((3, 5));
        for key in ["2nd", "apps", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.input, EditTarget::ExtraFunction(4));
        assert_eq!(app.device.screen, DeviceScreen::Functions);
        assert_eq!(app.graph_function(4), "12+rad(34)");
        assert_eq!(app.expression, "123");
        app.press_device_key(&ctx, "prgm");
        app.device.program = "Disp ".into();
        app.device.cursor = None;
        for key in ["2nd", "apps", "cursor right", "enter"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.program, "Disp deg(");
        assert_eq!(app.expression, "123");
        app.press_device_key(&ctx, "clear");
        for key in ["2nd", "\u{d7}", "1", "2nd", "\u{2212}"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.program, "[1]");
        assert_eq!(app.device.screen, DeviceScreen::Programs);
        assert_eq!(app.expression, "123");
        for key in ["2nd", "1/x"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.device.screen, DeviceScreen::Matrices);
    }

    #[test]
    fn fraction_conversion_uses_full_result_and_clear_preserves_ans() {
        let mut app = EqualApp::with_settings(
            Settings {
                level: 1,
                precision: 2,
                ..Settings::default()
            },
            None,
        );
        let ctx = egui::Context::default();
        for key in ["1", "fraction", "3", "enter", "convert"] {
            app.press_device_key(&ctx, key);
        }
        assert_eq!(app.result, "0.33");
        assert_eq!(app.device_result_text(), "1/3");
        app.press_device_key(&ctx, "clear");
        assert_eq!(app.device_result_text(), "0");
        assert_eq!(app.answer, 1.0 / 3.0);
    }

    #[test]
    fn physical_variable_menu_announces_and_scrolls_new_selection_once() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        app.set_device_screen(DeviceScreen::Variables);
        for _ in 0..25 {
            app.press_device_key(&ctx, "cursor end");
        }
        assert_eq!(app.device.menu_index, 25);
        assert!(app.device.menu_selection_changed);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(462.0, 940.0),
                )),
                ..Default::default()
            },
            |ctx| {
                app.theme(ctx);
                app.draw(ctx);
            },
        );
        assert!(!app.device.menu_selection_changed);
        assert!(
            output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Status
                    && node.live() == Some(egui::accesskit::Live::Polite)
                    && node.is_live_atomic()
                    && node.label() == Some("Selected: Z (not stored), 26 of 26"))
        );
        app.press_device_key(&ctx, "cursor end");
        assert!(
            !app.device.menu_selection_changed,
            "an unchanged selection must not override manual scrolling"
        );
    }

    #[test]
    fn lcd_stacks_expression_and_answer_inside_its_bounds_in_horizontal_parent() {
        let mut app = EqualApp::with_settings(Settings::default(), None);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        app.theme(&ctx);
        let lcd = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(220.0, 150.0));
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(462.0, 940.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| app.device_display(ui, lcd, 1.0));
                });
            },
        );
        let tree = output.platform_output.accesskit_update.unwrap();
        let bounds = |label| {
            tree.nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .unwrap_or_else(|| panic!("missing LCD control {label}"))
                .1
                .bounds()
                .unwrap()
        };
        let expression = bounds("Calculation expression");
        let answer = bounds("Answer: 0");
        assert!(
            answer.y0 >= expression.y1,
            "the result must appear below the expression"
        );
        for rect in [expression, answer] {
            assert!(rect.x0 >= f64::from(lcd.left()) && rect.x1 <= f64::from(lcd.right()));
            assert!(rect.y0 >= f64::from(lcd.top()) && rect.y1 <= f64::from(lcd.bottom()));
        }
    }
}
