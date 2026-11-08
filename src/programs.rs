//! A bounded, local calculator-language subset. This never executes host code.
use crate::math::{self, Expr};
use std::collections::BTreeMap;

const MAX_SOURCE_BYTES: usize = 8_000;
const MAX_SOURCE_LINES: usize = 200;
const MAX_STATEMENTS: usize = 512;
const MAX_BLOCK_DEPTH: usize = 16;
const MAX_STEPS: usize = 10_000;
const MAX_LOOP_ITERATIONS: usize = 1_000;
const MAX_OUTPUT_LINES: usize = 256;
const MAX_OUTPUT_BYTES: usize = 16_000;
const MAX_EXPRESSION_NODES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RunStatus {
    Ready,
    WaitingForInput { prompt: String, variable: char },
    Paused,
    Finished,
}

#[derive(Clone, Debug)]
pub(crate) struct ProgramState {
    pub variables: BTreeMap<char, f64>,
    pub output: Vec<String>,
    pub answer: f64,
    pub status: RunStatus,
    pc: usize,
    steps: usize,
    loops: BTreeMap<usize, ForFrame>,
    visits: BTreeMap<usize, usize>,
    source: Option<String>,
    failure: Option<String>,
}

impl ProgramState {
    pub fn new(variables: BTreeMap<char, f64>) -> Self {
        Self {
            variables,
            output: Vec::new(),
            answer: 0.0,
            status: RunStatus::Ready,
            pc: 0,
            steps: 0,
            loops: BTreeMap::new(),
            visits: BTreeMap::new(),
            source: None,
            failure: None,
        }
    }
}

#[derive(Clone, Debug)]
struct ForFrame {
    variable: char,
    limit: f64,
    step: f64,
    iterations: usize,
}

#[derive(Clone, Debug)]
enum DisplayPart {
    Text(String),
    Number(String),
}

#[derive(Clone, Debug)]
enum Operation {
    Calculate {
        expression: String,
        target: Option<char>,
    },
    Display(Vec<DisplayPart>),
    Input {
        prompt: String,
        variable: char,
    },
    Pause(Vec<DisplayPart>),
    ClearOutput,
    Stop,
    IfFalse {
        condition: String,
        target: usize,
    },
    Jump(usize),
    ForStart {
        variable: char,
        first: String,
        last: String,
        step: String,
        exit: usize,
    },
    ForNext {
        start: usize,
    },
    While {
        condition: String,
        exit: usize,
    },
    Repeat {
        condition: String,
        start: usize,
    },
    Label,
}

#[derive(Clone, Debug)]
struct Instruction {
    line: usize,
    operation: Operation,
}

#[derive(Clone, Debug)]
pub(crate) struct Program {
    source: String,
    instructions: Vec<Instruction>,
}

#[derive(Clone)]
struct SourceLine {
    number: usize,
    text: String,
}

struct Compiler {
    lines: Vec<SourceLine>,
    cursor: usize,
    instructions: Vec<Instruction>,
    labels: BTreeMap<String, usize>,
    jumps: Vec<(usize, String)>,
}

impl Program {
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.len() > MAX_SOURCE_BYTES || source.lines().count() > MAX_SOURCE_LINES {
            return Err(format!(
                "Use at most {MAX_SOURCE_LINES} lines and {MAX_SOURCE_BYTES} bytes."
            ));
        }
        let mut lines = Vec::new();
        for (index, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            for statement in
                split_outside(line, ':').map_err(|error| line_error(index + 1, error))?
            {
                if !statement.trim().is_empty() {
                    lines.push(SourceLine {
                        number: index + 1,
                        text: statement.trim().into(),
                    });
                }
            }
        }
        if lines.len() > MAX_STATEMENTS {
            return Err(format!("Use at most {MAX_STATEMENTS} statements."));
        }
        let mut compiler = Compiler {
            lines,
            cursor: 0,
            instructions: Vec::new(),
            labels: BTreeMap::new(),
            jumps: Vec::new(),
        };
        compiler.block(0, false)?;
        for (index, name) in &compiler.jumps {
            let target = compiler.labels.get(name).copied().ok_or_else(|| {
                line_error(
                    compiler.instructions[*index].line,
                    format!("Unknown label {name}."),
                )
            })?;
            compiler.instructions[*index].operation = Operation::Jump(target);
        }
        Ok(Self {
            source: source.into(),
            instructions: compiler.instructions,
        })
    }

    /// Variables and output retain completed statements if a later statement fails.
    /// Failed runs cannot continue: start with a new state to retry edited code.
    pub fn run(&self, state: &mut ProgramState, degrees: bool) -> Result<RunStatus, String> {
        if let Some(error) = &state.failure {
            return Err(error.clone());
        }
        if let Some(source) = &state.source {
            if source != &self.source {
                return Err("The program changed. Start a new run.".into());
            }
        } else {
            if !state.answer.is_finite()
                || state
                    .variables
                    .iter()
                    .any(|(name, value)| !name.is_ascii_uppercase() || !value.is_finite())
            {
                return Err("Programs require finite real-number variables A–Z.".into());
            }
            state.source = Some(self.source.clone());
        }
        if matches!(
            state.status,
            RunStatus::Finished | RunStatus::WaitingForInput { .. }
        ) {
            return Ok(state.status.clone());
        }
        state.status = RunStatus::Ready;
        while state.pc < self.instructions.len() {
            let instruction = &self.instructions[state.pc];
            if state.steps >= MAX_STEPS {
                return self.fail(
                    state,
                    instruction.line,
                    format!("Stopped after {MAX_STEPS} operations. Check loop conditions."),
                );
            }
            state.steps += 1;
            if let Err(error) = self.execute(instruction, state, degrees) {
                return self.fail(state, instruction.line, error);
            }
            if state.status != RunStatus::Ready {
                return Ok(state.status.clone());
            }
        }
        state.status = RunStatus::Finished;
        Ok(RunStatus::Finished)
    }

    pub fn provide_input(
        &self,
        state: &mut ProgramState,
        text: &str,
        degrees: bool,
    ) -> Result<(), String> {
        if state.failure.is_some() {
            return Err("This run stopped with an error. Start a new run.".into());
        }
        if state.source.as_deref() != Some(self.source.as_str()) {
            return Err("Start this program before providing input.".into());
        }
        let RunStatus::WaitingForInput { variable, .. } = state.status else {
            return Err("This program is not waiting for input.".into());
        };
        let value = numeric(text, state, degrees)
            .map_err(|error| format!("Input for {variable}: {error}"))?;
        state.variables.insert(variable, value);
        state.answer = value;
        state.pc += 1;
        state.status = RunStatus::Ready;
        Ok(())
    }

    fn fail(
        &self,
        state: &mut ProgramState,
        line: usize,
        error: String,
    ) -> Result<RunStatus, String> {
        let error = line_error(line, error);
        state.failure = Some(error.clone());
        Err(error)
    }

    fn execute(
        &self,
        instruction: &Instruction,
        state: &mut ProgramState,
        degrees: bool,
    ) -> Result<(), String> {
        let pc = state.pc;
        match &instruction.operation {
            Operation::Calculate { expression, target } => {
                let value = numeric(expression, state, degrees)?;
                state.answer = value;
                if let Some(name) = target {
                    state.variables.insert(*name, value);
                }
                state.pc += 1;
            }
            Operation::Display(parts) => {
                display(parts, state, degrees)?;
                state.pc += 1;
            }
            Operation::Input { prompt, variable } => {
                state.status = RunStatus::WaitingForInput {
                    prompt: prompt.clone(),
                    variable: *variable,
                }
            }
            Operation::Pause(parts) => {
                display(parts, state, degrees)?;
                state.pc += 1;
                state.status = RunStatus::Paused;
            }
            Operation::ClearOutput => {
                state.output.clear();
                state.pc += 1;
            }
            Operation::Stop => {
                state.pc = self.instructions.len();
                state.status = RunStatus::Finished;
            }
            Operation::IfFalse { condition, target } => {
                state.pc = if condition_value(condition, state, degrees, 0)? {
                    pc + 1
                } else {
                    *target
                };
            }
            Operation::Jump(target) => state.pc = *target,
            Operation::ForStart {
                variable,
                first,
                last,
                step,
                exit,
            } => {
                let first = numeric(first, state, degrees)?;
                let limit = numeric(last, state, degrees)?;
                let step = numeric(step, state, degrees)?;
                if step == 0.0 {
                    return Err("For step cannot be zero.".into());
                }
                state.variables.insert(*variable, first);
                if beyond(first, limit, step) {
                    state.loops.remove(&pc);
                    state.pc = *exit;
                } else {
                    state.loops.insert(
                        pc,
                        ForFrame {
                            variable: *variable,
                            limit,
                            step,
                            iterations: 1,
                        },
                    );
                    state.pc += 1;
                }
            }
            Operation::ForNext { start } => {
                let frame = state
                    .loops
                    .get(start)
                    .cloned()
                    .ok_or("For loop was entered without its For statement.")?;
                let current = *state
                    .variables
                    .get(&frame.variable)
                    .ok_or("For variable is unavailable.")?;
                let next = current + frame.step;
                if !next.is_finite() || next == current {
                    return Err("For step does not advance to a finite value.".into());
                }
                state.variables.insert(frame.variable, next);
                if beyond(next, frame.limit, frame.step) {
                    state.loops.remove(start);
                    state.pc += 1;
                } else {
                    if frame.iterations >= MAX_LOOP_ITERATIONS {
                        return Err(format!(
                            "Stopped after {MAX_LOOP_ITERATIONS} For iterations."
                        ));
                    }
                    if let Some(frame) = state.loops.get_mut(start) {
                        frame.iterations += 1;
                    }
                    state.pc = start + 1;
                }
            }
            Operation::While { condition, exit } => {
                if !condition_value(condition, state, degrees, 0)? {
                    state.visits.remove(&pc);
                    state.pc = *exit;
                } else {
                    visit(state, pc)?;
                    state.pc += 1;
                }
            }
            Operation::Repeat { condition, start } => {
                if condition_value(condition, state, degrees, 0)? {
                    state.visits.remove(&pc);
                    state.pc += 1;
                } else {
                    visit(state, pc)?;
                    state.pc = *start;
                }
            }
            Operation::Label => state.pc += 1,
        }
        Ok(())
    }
}

fn beyond(value: f64, limit: f64, step: f64) -> bool {
    let tolerance = 1e-12 * limit.abs().max(1.0);
    if step > 0.0 {
        value > limit + tolerance
    } else {
        value < limit - tolerance
    }
}

fn visit(state: &mut ProgramState, pc: usize) -> Result<(), String> {
    let visits = state.visits.entry(pc).or_default();
    *visits += 1;
    if *visits > MAX_LOOP_ITERATIONS {
        Err(format!(
            "Stopped after {MAX_LOOP_ITERATIONS} loop iterations."
        ))
    } else {
        Ok(())
    }
}

impl Compiler {
    fn push(&mut self, line: usize, operation: Operation) -> Result<usize, String> {
        if self.instructions.len() >= MAX_STATEMENTS {
            return Err(line_error(
                line,
                format!("Use at most {MAX_STATEMENTS} compiled operations."),
            ));
        }
        let index = self.instructions.len();
        self.instructions.push(Instruction { line, operation });
        Ok(index)
    }

    fn block(&mut self, depth: usize, nested: bool) -> Result<(), String> {
        if depth > MAX_BLOCK_DEPTH {
            return Err("Use at most 16 nested control blocks.".into());
        }
        while self.cursor < self.lines.len() {
            let text = self.lines[self.cursor].text.to_ascii_uppercase();
            if text == "END" || text == "ELSE" {
                return if nested {
                    Ok(())
                } else {
                    Err(line_error(
                        self.lines[self.cursor].number,
                        format!("Unexpected {text}."),
                    ))
                };
            }
            self.statement(depth)?;
        }
        Ok(())
    }

    fn require_end(&mut self, line: usize) -> Result<(), String> {
        if self
            .lines
            .get(self.cursor)
            .is_some_and(|line| line.text.eq_ignore_ascii_case("End"))
        {
            self.cursor += 1;
            Ok(())
        } else {
            Err(line_error(line, "This block needs End."))
        }
    }

    fn statement(&mut self, depth: usize) -> Result<(), String> {
        if depth > MAX_BLOCK_DEPTH {
            return Err("Use at most 16 nested control blocks.".into());
        }
        let line = self
            .lines
            .get(self.cursor)
            .cloned()
            .ok_or("Expected a statement after If.")?;
        self.cursor += 1;
        let (command, rest) = command(&line.text);
        match command.as_str() {
            "DISP" => {
                self.push(
                    line.number,
                    Operation::Display(
                        display_parts(rest).map_err(|error| line_error(line.number, error))?,
                    ),
                )?;
            }
            "PAUSE" => {
                self.push(
                    line.number,
                    Operation::Pause(if rest.is_empty() {
                        Vec::new()
                    } else {
                        display_parts(rest).map_err(|error| line_error(line.number, error))?
                    }),
                )?;
            }
            "STOP" | "RETURN" if rest.is_empty() => {
                self.push(line.number, Operation::Stop)?;
            }
            "CLRHOME" if rest.is_empty() => {
                self.push(line.number, Operation::ClearOutput)?;
            }
            "INPUT" | "PROMPT" => {
                let parts =
                    split_outside(rest, ',').map_err(|error| line_error(line.number, error))?;
                let (prompt, name) = match parts.as_slice() {
                    [name] => (None, *name),
                    [prompt, name] if command == "INPUT" => (
                        Some(string_literal(prompt).ok_or_else(|| {
                            line_error(line.number, "Put the input prompt in quotes.")
                        })?),
                        *name,
                    ),
                    _ => {
                        return Err(line_error(
                            line.number,
                            "Use Input A or Input \"Prompt\",A.",
                        ));
                    }
                };
                let variable = variable(name).map_err(|error| line_error(line.number, error))?;
                self.push(
                    line.number,
                    Operation::Input {
                        prompt: prompt.unwrap_or_else(|| format!("Value for {variable}")),
                        variable,
                    },
                )?;
            }
            "IF" => {
                if rest.is_empty() {
                    return Err(line_error(line.number, "If needs a condition."));
                }
                let index = self.push(
                    line.number,
                    Operation::IfFalse {
                        condition: rest.into(),
                        target: 0,
                    },
                )?;
                if self
                    .lines
                    .get(self.cursor)
                    .is_some_and(|line| line.text.eq_ignore_ascii_case("Then"))
                {
                    self.cursor += 1;
                    self.block(depth + 1, true)?;
                    if self
                        .lines
                        .get(self.cursor)
                        .is_some_and(|line| line.text.eq_ignore_ascii_case("Else"))
                    {
                        let jump = self.push(line.number, Operation::Jump(0))?;
                        let target = self.instructions.len();
                        if let Operation::IfFalse {
                            target: existing, ..
                        } = &mut self.instructions[index].operation
                        {
                            *existing = target;
                        }
                        self.cursor += 1;
                        self.block(depth + 1, true)?;
                        self.require_end(line.number)?;
                        self.instructions[jump].operation =
                            Operation::Jump(self.instructions.len());
                    } else {
                        self.require_end(line.number)?;
                        let target = self.instructions.len();
                        if let Operation::IfFalse {
                            target: existing, ..
                        } = &mut self.instructions[index].operation
                        {
                            *existing = target;
                        }
                    }
                } else {
                    if self.lines.get(self.cursor).is_none_or(|line| {
                        matches!(
                            line.text.to_ascii_uppercase().as_str(),
                            "END" | "ELSE" | "THEN"
                        )
                    }) {
                        return Err(line_error(
                            line.number,
                            "If needs a following statement, or Then...End.",
                        ));
                    }
                    self.statement(depth + 1)?;
                    let target = self.instructions.len();
                    if let Operation::IfFalse {
                        target: existing, ..
                    } = &mut self.instructions[index].operation
                    {
                        *existing = target;
                    }
                }
            }
            "FOR" => {
                let arguments = rest
                    .strip_prefix('(')
                    .and_then(|value| value.strip_suffix(')'))
                    .unwrap_or(rest);
                let arguments = split_outside(arguments, ',')
                    .map_err(|error| line_error(line.number, error))?;
                if !(3..=4).contains(&arguments.len()) {
                    return Err(line_error(line.number, "Use For(I,start,end[,step])."));
                }
                let name =
                    variable(arguments[0]).map_err(|error| line_error(line.number, error))?;
                if arguments[1..]
                    .iter()
                    .any(|argument| argument.trim().is_empty())
                {
                    return Err(line_error(
                        line.number,
                        "For bounds and step need numeric expressions.",
                    ));
                }
                let start = self.push(
                    line.number,
                    Operation::ForStart {
                        variable: name,
                        first: arguments[1].into(),
                        last: arguments[2].into(),
                        step: arguments.get(3).map_or("1", |value| *value).into(),
                        exit: 0,
                    },
                )?;
                self.block(depth + 1, true)?;
                self.require_end(line.number)?;
                self.push(line.number, Operation::ForNext { start })?;
                let exit = self.instructions.len();
                if let Operation::ForStart { exit: existing, .. } =
                    &mut self.instructions[start].operation
                {
                    *existing = exit;
                }
            }
            "WHILE" | "REPEAT" => {
                if rest.is_empty() {
                    return Err(line_error(line.number, "The loop needs a condition."));
                }
                if command == "WHILE" {
                    let start = self.push(
                        line.number,
                        Operation::While {
                            condition: rest.into(),
                            exit: 0,
                        },
                    )?;
                    self.block(depth + 1, true)?;
                    self.require_end(line.number)?;
                    self.push(line.number, Operation::Jump(start))?;
                    let exit = self.instructions.len();
                    if let Operation::While { exit: existing, .. } =
                        &mut self.instructions[start].operation
                    {
                        *existing = exit;
                    }
                } else {
                    let start = self.instructions.len();
                    self.block(depth + 1, true)?;
                    self.require_end(line.number)?;
                    self.push(
                        line.number,
                        Operation::Repeat {
                            condition: rest.into(),
                            start,
                        },
                    )?;
                }
            }
            "LBL" => {
                let name = label(rest).map_err(|error| line_error(line.number, error))?;
                let index = self.push(line.number, Operation::Label)?;
                if self.labels.insert(name.clone(), index).is_some() {
                    return Err(line_error(
                        line.number,
                        format!("Label {name} is already defined."),
                    ));
                }
            }
            "GOTO" => {
                let name = label(rest).map_err(|error| line_error(line.number, error))?;
                let index = self.push(line.number, Operation::Jump(0))?;
                self.jumps.push((index, name));
            }
            "THEN" | "ELSE" | "END" => {
                return Err(line_error(line.number, format!("Unexpected {command}.")));
            }
            _ => {
                let text = line.text.replace("->", "\u{2192}");
                let (expression, target) =
                    if let Some((expression, target)) = text.split_once('\u{2192}') {
                        (
                            expression.trim().to_owned(),
                            Some(variable(target).map_err(|error| line_error(line.number, error))?),
                        )
                    } else {
                        (text.trim().to_owned(), None)
                    };
                if expression.is_empty() {
                    return Err(line_error(
                        line.number,
                        "Enter an expression before the storage arrow.",
                    ));
                }
                if command.len() > 1
                    && !rest.is_empty()
                    && !rest.starts_with(['(', '+', '-', '*', '/', '^', '!', '%'])
                    && !matches!(command.as_str(), "ANS" | "PI")
                {
                    return Err(line_error(
                        line.number,
                        format!("Unsupported command {command}."),
                    ));
                }
                self.push(line.number, Operation::Calculate { expression, target })?;
            }
        }
        Ok(())
    }
}

fn command(text: &str) -> (String, &str) {
    let end = text
        .char_indices()
        .find_map(|(index, character)| (!character.is_ascii_alphabetic()).then_some(index))
        .unwrap_or(text.len());
    (text[..end].to_ascii_uppercase(), text[end..].trim())
}

fn variable(text: &str) -> Result<char, String> {
    let text = text.trim();
    let mut chars = text.chars();
    let character = chars.next().ok_or("Choose a variable A–Z.")?;
    if chars.next().is_some() || !character.is_ascii_alphabetic() {
        return Err("Choose one variable A–Z.".into());
    }
    Ok(character.to_ascii_uppercase())
}

fn label(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty()
        || text.len() > 8
        || !text
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        Err("Use a label of 1–8 letters or digits.".into())
    } else {
        Ok(text.to_ascii_uppercase())
    }
}

fn split_outside(text: &str, separator: char) -> Result<Vec<&str>, String> {
    let mut result = Vec::new();
    let mut depth = 0_i32;
    let mut quoted = false;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        match character {
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            value if value == separator && depth == 0 && !quoted => {
                result.push(&text[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
        if !(0..=48).contains(&depth) {
            return Err("Check the parentheses.".into());
        }
    }
    if quoted {
        return Err("Close the quoted text.".into());
    }
    if depth != 0 {
        return Err("Close the parentheses.".into());
    }
    result.push(&text[start..]);
    Ok(result)
}

fn string_literal(text: &str) -> Option<String> {
    let text = text.trim();
    let text = text.strip_prefix('"')?.strip_suffix('"')?;
    let unescaped = text.replace("\"\"", "");
    if unescaped.contains('"') {
        None
    } else {
        Some(text.replace("\"\"", "\""))
    }
}

fn display_parts(text: &str) -> Result<Vec<DisplayPart>, String> {
    if text.trim().is_empty() {
        return Err("Disp needs quoted text or a numeric expression.".into());
    }
    split_outside(text, ',')?
        .into_iter()
        .map(|part| {
            if let Some(text) = string_literal(part) {
                Ok(DisplayPart::Text(text))
            } else if part.trim().is_empty() || part.contains('"') {
                Err("Use quoted text or a numeric expression after Disp.".into())
            } else {
                Ok(DisplayPart::Number(part.trim().into()))
            }
        })
        .collect()
}

fn display(parts: &[DisplayPart], state: &mut ProgramState, degrees: bool) -> Result<(), String> {
    for part in parts {
        let text = match part {
            DisplayPart::Text(text) => text.clone(),
            DisplayPart::Number(expression) => {
                let value = numeric(expression, state, degrees)?;
                state.answer = value;
                math::format(value, 8)
            }
        };
        let bytes = state.output.iter().map(String::len).sum::<usize>();
        if state.output.len() >= MAX_OUTPUT_LINES || bytes + text.len() > MAX_OUTPUT_BYTES {
            return Err("Program output is full. Use ClrHome or a shorter loop.".into());
        }
        state.output.push(text);
    }
    Ok(())
}

fn numeric(text: &str, state: &ProgramState, degrees: bool) -> Result<f64, String> {
    if text.len() > 1024 {
        return Err("Use numeric expressions of at most 1024 characters.".into());
    }
    let mut expanded = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index].is_ascii_digit() || chars[index] == '.' {
            let start = index;
            while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
                index += 1;
            }
            if index < chars.len()
                && matches!(chars[index], 'e' | 'E')
                && chars.get(index + 1).is_some_and(|character| {
                    character.is_ascii_digit() || matches!(character, '+' | '-')
                })
            {
                index += 1;
                if index < chars.len() && matches!(chars[index], '+' | '-') {
                    index += 1;
                }
                while index < chars.len() && chars[index].is_ascii_digit() {
                    index += 1;
                }
            }
            expanded.extend(&chars[start..index]);
        } else if chars[index].is_ascii_alphabetic() {
            let start = index;
            while index < chars.len() && chars[index].is_ascii_alphabetic() {
                index += 1;
            }
            let token: String = chars[start..index].iter().collect();
            if token.eq_ignore_ascii_case("ans") {
                expanded.push_str(&format!("({:.16e})", state.answer));
            } else if token.len() == 1
                && (state
                    .variables
                    .contains_key(&chars[start].to_ascii_uppercase())
                    || token != "e" && token != "i")
            {
                let name = chars[start].to_ascii_uppercase();
                let value = state
                    .variables
                    .get(&name)
                    .ok_or_else(|| format!("Variable {name} has no value."))?;
                if !value.is_finite() {
                    return Err(format!("Variable {name} is not finite."));
                }
                expanded.push_str(&format!("({value:.16e})"));
            } else {
                expanded.push_str(&token);
            }
        } else {
            expanded.push(chars[index]);
            index += 1;
        }
    }
    let expression = math::parse(&expanded)?;
    fn validate(expression: &Expr, nodes: &mut usize) -> Result<(), String> {
        *nodes += 1;
        if *nodes > MAX_EXPRESSION_NODES {
            return Err("Use a simpler numeric expression.".into());
        }
        match expression {
            Expr::Var(_) => return Err("Programs use real numbers and assigned variables A–Z.".into()),
            Expr::Call(name, _) if matches!(name.as_str(), "integral" | "diff" | "root") => return Err("Program expressions do not run numerical calculus; use the calculator or graph tools.".into()),
            Expr::Unary(_, value) => validate(value, nodes)?,
            Expr::Binary(_, first, second) => { validate(first, nodes)?; validate(second, nodes)?; }
            Expr::Call(_, arguments) => { for argument in arguments { validate(argument, nodes)?; } }
            Expr::Number(_) => {}
        }
        Ok(())
    }
    validate(&expression, &mut 0)?;
    let result = expression.eval(0.0, 0.0, 0.0, degrees)?;
    if result.is_finite() {
        Ok(result)
    } else {
        Err("The expression did not produce a finite real number.".into())
    }
}

fn condition_value(
    text: &str,
    state: &ProgramState,
    degrees: bool,
    depth: usize,
) -> Result<bool, String> {
    if depth > MAX_BLOCK_DEPTH {
        return Err("Use a simpler condition.".into());
    }
    let mut text = text.trim();
    while let Some(inner) = text
        .strip_prefix('(')
        .and_then(|text| text.strip_suffix(')'))
    {
        if balanced(inner) {
            text = inner.trim();
        } else {
            break;
        }
    }
    if let Some((first, second)) = logical_parts(text, "or") {
        return Ok(condition_value(first, state, degrees, depth + 1)?
            || condition_value(second, state, degrees, depth + 1)?);
    }
    if let Some((first, second)) = logical_parts(text, "and") {
        return Ok(condition_value(first, state, degrees, depth + 1)?
            && condition_value(second, state, degrees, depth + 1)?);
    }
    if text
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("not "))
    {
        return Ok(!condition_value(&text[4..], state, degrees, depth + 1)?);
    }
    if let Some((left, right, operation)) = comparison_parts(text) {
        let left = numeric(left, state, degrees)?;
        let right = numeric(right, state, degrees)?;
        return Ok(match operation {
            "=" | "==" => left == right,
            "!=" | "<>" => left != right,
            "<" => left < right,
            "<=" => left <= right,
            ">" => left > right,
            ">=" => left >= right,
            _ => unreachable!("comparison_parts emits known operators"),
        });
    }
    Ok(numeric(text, state, degrees)? != 0.0)
}

fn balanced(text: &str) -> bool {
    let mut depth = 0_i32;
    for character in text.chars() {
        if character == '(' {
            depth += 1;
        } else if character == ')' {
            depth -= 1;
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

fn logical_parts<'a>(text: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0;
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        let end = index + keyword.len();
        if depth == 0
            && text
                .get(index..end)
                .is_some_and(|word| word.eq_ignore_ascii_case(keyword))
            && (index == 0
                || !text[..index]
                    .chars()
                    .next_back()
                    .is_some_and(|value| value.is_ascii_alphabetic()))
            && !text
                .get(end..)
                .and_then(|tail| tail.chars().next())
                .is_some_and(|value| value.is_ascii_alphabetic())
        {
            return Some((&text[..index], &text[end..]));
        }
    }
    None
}

fn comparison_parts(text: &str) -> Option<(&str, &str, &str)> {
    let mut depth = 0;
    for (index, character) in text.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            for operation in ["<=", ">=", "!=", "<>", "==", "=", "<", ">"] {
                if text[index..].starts_with(operation) {
                    return Some((
                        &text[..index],
                        &text[index + operation.len()..],
                        &text[index..index + operation.len()],
                    ));
                }
            }
        }
    }
    None
}

fn line_error(line: usize, error: impl AsRef<str>) -> String {
    format!("Line {line}: {}", error.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(source: &str) -> ProgramState {
        let program = Program::parse(source).unwrap();
        let mut state = ProgramState::new(BTreeMap::new());
        assert_eq!(program.run(&mut state, true), Ok(RunStatus::Finished));
        state
    }

    #[test]
    fn for_loops_sum_and_handle_negative_and_fractional_steps() {
        let state = run("0->A\nFor(I,1,5)\nA+I->A\nEnd\nDisp \"Sum\",A");
        assert_eq!(state.variables[&'A'], 15.0);
        assert_eq!(state.output, ["Sum", "15"]);
        assert_eq!(run("0->A\nFor(I,3,1,-1)\nA+I->A\nEnd").variables[&'A'], 6.0);
        assert_eq!(
            run("0->A\nFor(I,1,2,0.5)\nA+I->A\nEnd").variables[&'A'],
            4.5
        );
        assert_eq!(run("7->A\nFor(I,5,1)\n99->A\nEnd").variables[&'A'], 7.0);
        assert_eq!(run("0->a\nfor(i,1,3)\na+i->a\nend\ndisp a").output, ["6"]);
    }

    #[test]
    fn conditions_nested_blocks_and_single_statement_if_use_real_values() {
        let source = "{value}->A\nIf A>2 and A<10\nThen\nIf A=3\nThen\nDisp \"three\"\nElse\nDisp \"middle\"\nEnd\nElse\nDisp \"outside\"\nEnd\nIf A<0\nDisp \"negative\"";
        assert_eq!(run(&source.replace("{value}", "3")).output, ["three"]);
        assert_eq!(run(&source.replace("{value}", "5")).output, ["middle"]);
        assert_eq!(run(&source.replace("{value}", "0")).output, ["outside"]);
        assert_eq!(
            run(&source.replace("{value}", "-1")).output,
            ["outside", "negative"]
        );
        assert_eq!(run("If not (2=3) or 0\nDisp 12").output, ["12"]);
    }

    #[test]
    fn input_and_pause_resume_without_repeating_prior_effects() {
        let program =
            Program::parse("Disp \"Start\"\nInput \"Number\",A\nA*2->B\nPause B\nDisp \"Done\"")
                .unwrap();
        let mut state = ProgramState::new(BTreeMap::new());
        assert_eq!(
            program.run(&mut state, true),
            Ok(RunStatus::WaitingForInput {
                prompt: "Number".into(),
                variable: 'A'
            })
        );
        assert!(program.provide_input(&mut state, "NaN", true).is_err());
        assert_eq!(state.output, ["Start"]);
        assert!(!state.variables.contains_key(&'A'));
        program.provide_input(&mut state, "3/2", true).unwrap();
        assert_eq!(program.run(&mut state, true), Ok(RunStatus::Paused));
        assert_eq!(state.output, ["Start", "3"]);
        assert_eq!(program.run(&mut state, true), Ok(RunStatus::Finished));
        assert_eq!(state.output, ["Start", "3", "Done"]);
        assert_eq!(state.variables[&'B'], 3.0);
    }

    #[test]
    fn repeat_while_labels_and_stop_have_bounded_control_flow() {
        let state = run(
            "0->A\nWhile A<2\nA+1->A\nEnd\nRepeat A>=4\nA+1->A\nEnd\nLbl DONE\nDisp A\nStop\nDisp 999",
        );
        assert_eq!(state.output, ["4"]);
        assert_eq!(run("Goto DONE\nDisp 999\nLbl DONE\nDisp 8").output, ["8"]);
        let program = Program::parse("Lbl LOOP\nGoto LOOP").unwrap();
        let mut state = ProgramState::new(BTreeMap::new());
        let error = program.run(&mut state, true).unwrap_err();
        assert!(error.contains("operations"));
        let steps = state.steps;
        assert_eq!(program.run(&mut state, true), Err(error));
        assert_eq!(state.steps, steps);
    }

    #[test]
    fn malformed_programs_and_unsafe_or_expensive_expressions_fail_clearly() {
        for source in [
            "For(I,1,3)\nDisp I",
            "While 1\nDisp 1",
            "If 1\nThen\nDisp 1",
            "Else",
            "End",
            "Then",
            "For(I,1)\nEnd",
            "Input \"A\"",
            "Disp \"unclosed",
            "Goto NOPE",
            "Lbl A\nLbl A",
            "Python something",
        ] {
            assert!(Program::parse(source).is_err(), "{source}");
        }
        for source in [
            "For(I,1,3,0)\nEnd",
            "Disp 1/0",
            "Disp X",
            "Disp integral(x,0,1)",
            "While 1\nEnd",
            "0->A\nRepeat 0\nA+1->A\nEnd",
        ] {
            let mut state = ProgramState::new(BTreeMap::new());
            let error = Program::parse(source)
                .unwrap()
                .run(&mut state, true)
                .unwrap_err();
            assert!(error.contains("Line "), "{source}: {error}");
        }
        assert!(Program::parse(&"x".repeat(MAX_SOURCE_BYTES + 1)).is_err());
    }

    #[test]
    fn completed_statements_survive_failure_and_new_source_needs_new_state() {
        let program = Program::parse("5->A\nDisp A\n1/0->B").unwrap();
        let mut state = ProgramState::new(BTreeMap::new());
        assert!(program.run(&mut state, true).is_err());
        assert_eq!(state.variables[&'A'], 5.0);
        assert!(!state.variables.contains_key(&'B'));
        assert_eq!(state.output, ["5"]);
        let first = Program::parse("Pause").unwrap();
        let second = Program::parse("Disp 2").unwrap();
        let mut state = ProgramState::new(BTreeMap::new());
        assert_eq!(first.run(&mut state, true), Ok(RunStatus::Paused));
        assert!(
            second
                .run(&mut state, true)
                .unwrap_err()
                .contains("changed")
        );
    }

    #[test]
    fn string_colons_function_commas_and_scientific_notation_are_preserved() {
        let state = run(":Disp \"A:B\",min(3,2):1e-3->E\nDisp E\nDisp \"Say \"\"yes\"\"\"");
        assert_eq!(state.output, ["A:B", "2", "0.001", "Say \"yes\""]);
        assert_eq!(state.variables[&'E'], 0.001);
    }
}
