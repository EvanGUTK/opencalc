use std::f64::consts::{E, PI};

#[derive(Clone, Debug)]
pub enum Expr {
    Number(f64),
    Var(char),
    Unary(char, Box<Expr>),
    Binary(char, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

const MAX_EXPRESSION_CHARS: usize = 1024;
const MAX_NORMALIZED_CHARS: usize = 8192;
const MAX_GROUPS: usize = 48;
const MAX_PARSE_DEPTH: usize = 128;

/// Converts supported decimal digit scripts and mathematical punctuation without
/// changing argument separators into guessed thousands separators.
pub(crate) fn normalize_input(text: &str, decimal_comma: bool) -> Result<String, String> {
    if text.chars().count() > MAX_NORMALIZED_CHARS {
        return Err("Use at most 8192 characters of mathematical input.".into());
    }
    let mapped: Vec<char> = text
        .chars()
        .map(|character| {
            if let Some(digit) = decimal_digit(character) {
                return char::from(b'0' + digit);
            }
            match character {
                '\u{2212}' | '\u{2013}' | '\u{2014}' | '\u{ff0d}' => '-',
                '\u{00d7}' | '\u{00b7}' | '\u{22c5}' | '\u{ff0a}' => '*',
                '\u{00f7}' | '\u{2044}' | '\u{2215}' | '\u{ff0f}' => '/',
                '\u{ff0b}' => '+',
                '\u{ff08}' => '(',
                '\u{ff09}' => ')',
                '\u{ff0e}' | '\u{066b}' => '.',
                '\u{ff0c}' => ',',
                '\u{ff1b}' => ';',
                '\u{ff05}' | '\u{066a}' => '%',
                _ => character,
            }
        })
        .collect();
    let mut normalized = String::new();
    let mut index = 0;
    while index < mapped.len() {
        let character = mapped[index];
        if let Some(first) = superscript(character)
            && (!matches!(first, '+' | '-')
                || mapped
                    .get(index + 1)
                    .is_some_and(|c| superscript(*c).is_some_and(|c| c.is_ascii_digit())))
        {
            normalized.push_str("^(");
            normalized.push(first);
            index += 1;
            while let Some(digit) = mapped
                .get(index)
                .and_then(|c| superscript(*c))
                .filter(char::is_ascii_digit)
            {
                normalized.push(digit);
                index += 1;
            }
            normalized.push(')');
            continue;
        }
        match character {
            '\u{03c0}' => normalized.push_str("pi"),
            '\u{212f}' => normalized.push('e'),
            '\u{221a}' => normalized.push_str("sqrt"),
            ';' => normalized.push(','),
            ',' if decimal_comma => {
                let previous = index.checked_sub(1).and_then(|i| mapped.get(i)).copied();
                let next = mapped.get(index + 1).copied();
                let previous_digit = previous.is_some_and(|c| c.is_ascii_digit());
                let next_digit = next.is_some_and(|c| c.is_ascii_digit());
                let starts_decimal = next_digit
                    && previous.is_none_or(|c| "(;+-*/^".contains(c) || c.is_whitespace());
                if (previous_digit && next_digit) || starts_decimal {
                    normalized.push('.');
                } else {
                    return Err(
                        "In decimal-comma mode use ; between function arguments or answer values."
                            .into(),
                    );
                }
            }
            '\u{066c}' => {
                if !valid_group_separator(&mapped, index) {
                    return Err(
                        "A thousands separator must be followed by a group of three digits.".into(),
                    );
                }
            }
            ' ' | '\u{00a0}' | '\u{2009}' | '\u{202f}' if valid_group_separator(&mapped, index) => {
            }
            _ => normalized.push(character),
        }
        index += 1;
    }
    Ok(normalized)
}

fn valid_group_separator(characters: &[char], index: usize) -> bool {
    if index == 0 || !characters[index - 1].is_ascii_digit() {
        return false;
    }
    characters[index + 1..]
        .iter()
        .take_while(|c| c.is_ascii_digit())
        .count()
        == 3
}

fn decimal_digit(character: char) -> Option<u8> {
    // Unicode decimal digit blocks, each containing the ten values zero through nine.
    const ZERO_POINTS: [u32; 23] = [
        0x0030, 0x0660, 0x06f0, 0x07c0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66,
        0x0ce6, 0x0d66, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0,
        0xff10,
    ];
    let point = u32::from(character);
    ZERO_POINTS
        .iter()
        .find_map(|zero| (point >= *zero && point < *zero + 10).then(|| (point - zero) as u8))
}

fn superscript(character: char) -> Option<char> {
    match character {
        '\u{2070}' => Some('0'),
        '\u{00b9}' => Some('1'),
        '\u{00b2}' => Some('2'),
        '\u{00b3}' => Some('3'),
        '\u{2074}'..='\u{2079}' => char::from_u32(u32::from(character) - 0x2074 + u32::from('4')),
        '\u{207a}' => Some('+'),
        '\u{207b}' => Some('-'),
        _ => None,
    }
}

pub fn parse(text: &str) -> Result<Expr, String> {
    if text.chars().count() > MAX_EXPRESSION_CHARS
        || text.chars().filter(|c| *c == '(').count() > MAX_GROUPS
    {
        return Err("Please use a shorter expression (up to 1024 characters and 48 groups)".into());
    }
    let normalized = normalize_input(text, false)?;
    if normalized.chars().filter(|c| *c == '(').count() > MAX_GROUPS {
        return Err("Please use at most 48 groups in one expression".into());
    }
    let mut p = Parser {
        chars: normalized.chars().collect(),
        pos: 0,
        depth: 0,
    };
    let e = p.expression(0)?;
    p.space();
    if p.pos != p.chars.len() {
        return Err(format!(
            "Unexpected '{}' at position {}",
            p.chars[p.pos],
            p.pos + 1
        ));
    }
    Ok(e)
}
struct Parser {
    chars: Vec<char>,
    pos: usize,
    depth: usize,
}
impl Parser {
    fn space(&mut self) {
        while self.chars.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.space();
        self.chars.get(self.pos).copied()
    }
    fn expression(&mut self, min: u8) -> Result<Expr, String> {
        if self.depth >= MAX_PARSE_DEPTH {
            return Err(
                "Please simplify the nested signs, powers or groups in this expression".into(),
            );
        }
        self.depth += 1;
        let result = self.expression_inner(min);
        self.depth -= 1;
        result
    }
    fn expression_inner(&mut self, min: u8) -> Result<Expr, String> {
        self.space();
        let c = self.peek().ok_or("Enter an expression first")?;
        let mut lhs = if c == '-' || c == '+' {
            self.pos += 1;
            Expr::Unary(c, Box::new(self.expression(5)?))
        } else if c == '(' {
            self.pos += 1;
            let e = self.expression(0)?;
            if self.peek() != Some(')') {
                return Err("Close the parenthesis with )".into());
            }
            self.pos += 1;
            e
        } else if c.is_ascii_digit() || c == '.' {
            let start = self.pos;
            while self
                .chars
                .get(self.pos)
                .is_some_and(|c| c.is_ascii_digit() || *c == '.')
            {
                self.pos += 1;
            }
            if matches!(self.chars.get(self.pos), Some('e' | 'E'))
                && self
                    .chars
                    .get(self.pos + 1)
                    .is_some_and(|c| c.is_ascii_digit() || *c == '+' || *c == '-')
            {
                self.pos += 1;
                if matches!(self.chars.get(self.pos), Some('+' | '-')) {
                    self.pos += 1;
                }
                while self.chars.get(self.pos).is_some_and(|c| c.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
            Expr::Number(
                self.chars[start..self.pos]
                    .iter()
                    .collect::<String>()
                    .parse()
                    .map_err(|_| "Check the decimal number")?,
            )
        } else if c.is_ascii_alphabetic() {
            let start = self.pos;
            while self
                .chars
                .get(self.pos)
                .is_some_and(|c| c.is_ascii_alphabetic())
            {
                self.pos += 1;
            }
            let name: String = self.chars[start..self.pos]
                .iter()
                .collect::<String>()
                .to_lowercase();
            match name.as_str() {
                "pi" => Expr::Number(PI),
                "e" => Expr::Number(E),
                "x" => Expr::Var('x'),
                "y" => Expr::Var('y'),
                "ans" => Expr::Var('a'),
                "i" => Expr::Var('i'),
                _ => {
                    if self.peek() != Some('(') {
                        return Err(format!("Use {name}(...) for a function"));
                    }
                    self.pos += 1;
                    let mut args = vec![self.expression(0)?];
                    while self.peek() == Some(',') {
                        self.pos += 1;
                        args.push(self.expression(0)?);
                    }
                    if self.peek() != Some(')') {
                        return Err("Close the function with )".into());
                    }
                    self.pos += 1;
                    Expr::Call(name, args)
                }
            }
        } else {
            return Err(format!("Unexpected '{c}'"));
        };
        while let Some(op) = self.peek() {
            if op == '!' || op == '%' {
                if 9 < min {
                    break;
                }
                self.pos += 1;
                lhs = Expr::Unary(op, Box::new(lhs));
                continue;
            }
            let implicit =
                op == '(' || op.is_ascii_alphabetic() || op.is_ascii_digit() || op == '.';
            let (l, r) = match op {
                '+' | '-' => (1, 2),
                '*' | '/' => (3, 4),
                '^' => (7, 6),
                _ if implicit => (3, 4),
                _ => break,
            };
            if l < min {
                break;
            }
            if implicit {
                if (op.is_ascii_digit() || op == '.')
                    && self.chars[..self.pos]
                        .iter()
                        .rev()
                        .find(|c| !c.is_whitespace())
                        .is_some_and(|c| c.is_ascii_digit() || *c == '.')
                {
                    return Err(
                        "Use * between separate numbers, or remove the space within one number."
                            .into(),
                    );
                }
            } else {
                self.pos += 1;
            }
            lhs = Expr::Binary(
                if implicit { '*' } else { op },
                Box::new(lhs),
                Box::new(self.expression(r)?),
            );
        }
        Ok(lhs)
    }
}
impl Expr {
    pub fn eval(&self, x: f64, y: f64, ans: f64, degrees: bool) -> Result<f64, String> {
        let ev = |e: &Expr| e.eval(x, y, ans, degrees);
        let value = match self {
            Self::Number(v) => *v,
            Self::Var(c) => match c {
                'x' => x,
                'y' => y,
                'a' => ans,
                'i' => {
                    return Err(
                        "This expression needs Complex mode; i is not a real number.".into(),
                    );
                }
                _ => return Err("Unknown variable".into()),
            },
            Self::Unary(op, a) => {
                let a = ev(a)?;
                match op {
                    '-' => -a,
                    '+' => a,
                    '%' => a / 100.0,
                    '!' => {
                        if a < 0.0 || a.fract() != 0.0 || a > 170.0 {
                            return Err("Factorial needs a whole number from 0 to 170".into());
                        }
                        (1..=a as u32).map(f64::from).product()
                    }
                    _ => return Err("Unknown operator".into()),
                }
            }
            Self::Binary(op, a, b) => {
                let (a, b) = (ev(a)?, ev(b)?);
                match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => {
                        if b == 0.0 {
                            return Err("Cannot divide by zero".into());
                        }
                        a / b
                    }
                    '^' => a.powf(b),
                    _ => return Err("Unknown operator".into()),
                }
            }
            Self::Call(name, args) => {
                let arg = |i: usize| {
                    args.get(i)
                        .ok_or_else(|| format!("{name} needs more arguments"))
                        .and_then(ev)
                };
                let (minimum, maximum) = match name.as_str() {
                    "sum" | "prod" | "mean" | "median" | "variance" | "variancep" | "stddev"
                    | "stddevp" => (1, 1000),
                    "round" => (1, 2),
                    "diff" | "root" | "min" | "max" | "hypot" | "ncr" | "npr" | "logbase" => (2, 2),
                    "integral" | "dms" => (3, 3),
                    _ => (1, 1),
                };
                if !(minimum..=maximum).contains(&args.len()) {
                    return Err(if minimum == maximum {
                        format!("{name} needs {minimum} argument(s)")
                    } else {
                        format!("{name} needs {minimum} to {maximum} arguments")
                    });
                }
                // Calculus must evaluate its expression at the requested coordinates,
                // rather than at the caller's x (which may be outside its domain).
                let a = if matches!(name.as_str(), "diff" | "integral" | "root") {
                    0.0
                } else {
                    arg(0)?
                };
                let angle = if degrees { a.to_radians() } else { a };
                match name.as_str() {
                    "sin" => angle.sin(),
                    "cos" => angle.cos(),
                    "tan" => {
                        if angle.cos().abs() < 1e-12 {
                            return Err("Tangent is undefined at this angle".into());
                        }
                        angle.tan()
                    }
                    "asin" => {
                        if degrees {
                            a.asin().to_degrees()
                        } else {
                            a.asin()
                        }
                    }
                    "acos" => {
                        if degrees {
                            a.acos().to_degrees()
                        } else {
                            a.acos()
                        }
                    }
                    "atan" => {
                        if degrees {
                            a.atan().to_degrees()
                        } else {
                            a.atan()
                        }
                    }
                    "sinh" => a.sinh(),
                    "cosh" => a.cosh(),
                    "tanh" => a.tanh(),
                    "asinh" => a.asinh(),
                    "acosh" => a.acosh(),
                    "atanh" => a.atanh(),
                    "sqrt" => a.sqrt(),
                    "cbrt" => a.cbrt(),
                    "ln" => a.ln(),
                    "log" => a.log10(),
                    "exp" => a.exp(),
                    "abs" => a.abs(),
                    "floor" => a.floor(),
                    "ceil" => a.ceil(),
                    "trunc" => a.trunc(),
                    "round" => {
                        let digits = if args.len() == 2 { arg(1)? } else { 0.0 };
                        if !(-12.0..=12.0).contains(&digits) || digits.fract() != 0.0 {
                            return Err(
                                "round decimal places must be a whole number from -12 to 12".into(),
                            );
                        }
                        let scale = 10.0_f64.powi(digits as i32);
                        (a * scale).round() / scale
                    }
                    "sign" => {
                        if a == 0.0 {
                            0.0
                        } else {
                            a.signum()
                        }
                    }
                    "rad" => a.to_radians(),
                    "deg" => a.to_degrees(),
                    "dms" => {
                        let minutes = arg(1)?;
                        let seconds = arg(2)?;
                        if a.fract() != 0.0
                            || !(0.0..60.0).contains(&minutes)
                            || !(0.0..60.0).contains(&seconds)
                        {
                            return Err("dms needs integer degrees and minutes/seconds from 0 up to 60; put the sign on degrees.".into());
                        }
                        (a.abs() + minutes / 60.0 + seconds / 3600.0).copysign(a)
                    }
                    "re" | "conj" => a,
                    "im" => 0.0,
                    "norm" => a * a,
                    "arg" => {
                        if a == 0.0 {
                            return Err("The argument of zero is undefined".into());
                        }
                        let phase = 0.0_f64.atan2(a);
                        if degrees { phase.to_degrees() } else { phase }
                    }
                    "logbase" => {
                        let base = arg(1)?;
                        if a <= 0.0 || base <= 0.0 || base == 1.0 {
                            return Err(
                                "logbase needs a positive value and a positive base other than 1"
                                    .into(),
                            );
                        }
                        a.ln() / base.ln()
                    }
                    "ncr" | "npr" => {
                        let count = arg(1)?;
                        if a.fract() != 0.0
                            || count.fract() != 0.0
                            || !(0.0..=170.0).contains(&a)
                            || !(0.0..=a).contains(&count)
                        {
                            return Err("Use whole counts with 0 <= r <= n <= 170".into());
                        }
                        let n = a as u32;
                        let r = count as u32;
                        if name == "npr" {
                            ((n - r + 1)..=n).map(f64::from).product()
                        } else {
                            let r = r.min(n - r);
                            (1..=r).fold(1.0, |value, step| {
                                value * f64::from(n - r + step) / f64::from(step)
                            })
                        }
                    }
                    "sum" | "prod" | "mean" | "median" | "variance" | "variancep" | "stddev"
                    | "stddevp" => {
                        let mut values = args.iter().map(ev).collect::<Result<Vec<_>, _>>()?;
                        match name.as_str() {
                            "sum" => values.iter().sum(),
                            "prod" => values.iter().product(),
                            "median" => {
                                values.sort_by(f64::total_cmp);
                                let middle = values.len() / 2;
                                if values.len().is_multiple_of(2) {
                                    values[middle - 1] / 2.0 + values[middle] / 2.0
                                } else {
                                    values[middle]
                                }
                            }
                            _ => {
                                let mut mean = 0.0;
                                let mut squared_differences = 0.0;
                                for (index, value) in values.iter().enumerate() {
                                    let difference = value - mean;
                                    mean += difference / (index + 1) as f64;
                                    squared_differences += difference * (value - mean);
                                }
                                if name == "mean" {
                                    mean
                                } else {
                                    let sample = matches!(name.as_str(), "variance" | "stddev");
                                    if sample && values.len() < 2 {
                                        return Err("Sample variance and standard deviation need at least two values".into());
                                    }
                                    let divisor = values.len() - usize::from(sample);
                                    let variance = (squared_differences / divisor as f64).max(0.0);
                                    if matches!(name.as_str(), "stddev" | "stddevp") {
                                        variance.sqrt()
                                    } else {
                                        variance
                                    }
                                }
                            }
                        }
                    }
                    "min" => a.min(arg(1)?),
                    "max" => a.max(arg(1)?),
                    "hypot" => a.hypot(arg(1)?),
                    "diff" => {
                        let at = arg(1)?;
                        let h = 1e-5 * (1.0 + at.abs());
                        (args[0].eval(at + h, y, ans, degrees)?
                            - args[0].eval(at - h, y, ans, degrees)?)
                            / (2.0 * h)
                    }
                    "integral" => {
                        let lo = arg(1)?;
                        let hi = arg(2)?;
                        let h = (hi - lo) / 1000.0;
                        let mut sum = 0.0;
                        for i in 0..=1000 {
                            let w = if i == 0 || i == 1000 {
                                1.0
                            } else if i % 2 == 0 {
                                2.0
                            } else {
                                4.0
                            };
                            sum += w * args[0].eval(lo + i as f64 * h, y, ans, degrees)?;
                        }
                        sum * h / 3.0
                    }
                    "root" => {
                        let mut at = arg(1)?;
                        let mut converged = false;
                        for _ in 0..100 {
                            let f = args[0].eval(at, y, ans, degrees)?;
                            if f.abs() < 1e-10 {
                                converged = true;
                                break;
                            }
                            let h = 1e-5 * (1.0 + at.abs());
                            let d = (args[0].eval(at + h, y, ans, degrees)?
                                - args[0].eval(at - h, y, ans, degrees)?)
                                / (2.0 * h);
                            if d.abs() < 1e-12 {
                                break;
                            }
                            at -= f / d;
                        }
                        if !converged {
                            return Err("No root found. Try another starting value".into());
                        }
                        at
                    }
                    _ => return Err(format!("Unknown function: {name}")),
                }
            }
        };
        if value.is_finite() {
            Ok(value)
        } else {
            Err("Result is outside the real-number domain".into())
        }
    }
}
pub fn calculate(s: &str, ans: f64, degrees: bool) -> Result<f64, String> {
    parse(s)?.eval(0.0, 0.0, ans, degrees)
}
pub fn format(v: f64, precision: usize) -> String {
    if v != 0.0 && (v.abs() >= 1e12 || v.abs() < 1e-8) {
        return format!("{v:.precision$e}");
    }
    let s = format!("{v:.precision$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        &s
    };
    if s.is_empty() || s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}
pub type Matrix = Vec<Vec<f64>>;
pub fn matrix(s: &str) -> Result<Matrix, String> {
    let rows: Result<Matrix, _> = s
        .trim()
        .trim_matches(|c| c == '[' || c == ']')
        .split([';', '\n'])
        .filter(|r| !r.trim().is_empty())
        .map(|r| {
            r.split(|c: char| c == ',' || c.is_whitespace())
                .filter(|v| !v.is_empty())
                .map(|v| {
                    v.parse::<f64>().map_err(|_| {
                        "Use numbers separated by spaces; separate rows with ;".to_string()
                    })
                })
                .collect()
        })
        .collect();
    let m = rows?;
    if m.is_empty()
        || m[0].is_empty()
        || m.iter()
            .any(|r| r.len() != m[0].len() || r.iter().any(|v| !v.is_finite()))
    {
        return Err("Each matrix row must have the same number of finite values".into());
    }
    if m.len() > 12 || m[0].len() > 12 {
        return Err("Use at most 12 rows and 12 columns".into());
    }
    Ok(m)
}
pub fn matrix_op(a: &Matrix, b: &Matrix, op: &str) -> Result<Matrix, String> {
    let (n, m) = (a.len(), a[0].len());
    match op {
        "A + B" => {
            if n != b.len() || m != b[0].len() {
                return Err("Addition needs matrices of the same size".into());
            }
            Ok((0..n)
                .map(|i| (0..m).map(|j| a[i][j] + b[i][j]).collect())
                .collect())
        }
        "A × B" => {
            if m != b.len() {
                return Err("Columns of A must equal rows of B".into());
            }
            Ok((0..n)
                .map(|i| {
                    (0..b[0].len())
                        .map(|j| (0..m).map(|k| a[i][k] * b[k][j]).sum())
                        .collect()
                })
                .collect())
        }
        "Transpose A" => Ok((0..m).map(|j| (0..n).map(|i| a[i][j]).collect()).collect()),
        "Inverse A" | "Determinant A" => {
            if n != m {
                return Err("Use a square matrix".into());
            }
            let mut work = a.clone();
            let mut inv = vec![vec![0.0; n]; n];
            for (i, row) in inv.iter_mut().enumerate() {
                row[i] = 1.0;
            }
            let mut det = 1.0;
            for k in 0..n {
                let p = (k..n)
                    .max_by(|&i, &j| work[i][k].abs().total_cmp(&work[j][k].abs()))
                    .unwrap();
                if work[p][k].abs() < 1e-12 {
                    return if op == "Determinant A" {
                        Ok(vec![vec![0.0]])
                    } else {
                        Err("This matrix is singular; it has no inverse".into())
                    };
                }
                if p != k {
                    work.swap(p, k);
                    inv.swap(p, k);
                    det = -det;
                }
                let pivot = work[k][k];
                det *= pivot;
                for j in 0..n {
                    work[k][j] /= pivot;
                    inv[k][j] /= pivot;
                }
                for i in 0..n {
                    if i == k {
                        continue;
                    }
                    let f = work[i][k];
                    for j in 0..n {
                        work[i][j] -= f * work[k][j];
                        inv[i][j] -= f * inv[k][j];
                    }
                }
            }
            if op == "Determinant A" {
                Ok(vec![vec![det]])
            } else {
                Ok(inv)
            }
        }
        _ => Err("Choose a matrix operation".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn angle_conversions_and_dms_preserve_units_sign_and_domain() {
        for degrees in [false, true] {
            near("deg(pi)", 180.0, degrees);
            near("rad(180)", PI, degrees);
            near("dms(12,30,45)", 12.5125, degrees);
            near("dms(-12,30,45)", -12.5125, degrees);
            near("dms(-0,30,0)", -0.5, degrees);
            for input in [
                "dms(1,60,0)",
                "dms(1,0,60)",
                "dms(1,-1,0)",
                "dms(1.5,0,0)",
                "dms(1,2)",
            ] {
                assert!(calculate(input, 0.0, degrees).is_err(), "{input}");
            }
        }
    }
    fn near(s: &str, want: f64, deg: bool) {
        assert!((calculate(s, 0.0, deg).unwrap() - want).abs() < 1e-6, "{s}");
    }
    #[test]
    fn arithmetic() {
        near("3+4*2", 11.0, false);
        near("-2^2", -4.0, false);
        near("2^3^2", 512.0, false);
        near("(3+5)/2", 4.0, false);
        near("5!", 120.0, false);
        near("50%*80", 40.0, false);
        near("2e-3", 0.002, false);
    }
    #[test]
    fn scientific() {
        near("sin(30)", 0.5, true);
        near("cos(pi)", -1.0, false);
        near("asin(0.5)", 30.0, true);
        near("log(1000)", 3.0, false);
    }
    #[test]
    fn implicit_multiplication_and_power_precedence() {
        near("2pi", 2.0 * PI, false);
        near("2(3+4)", 14.0, false);
        near("(2)(3)", 6.0, false);
        near("2sin(pi/2)", 2.0, false);
        near("2^-3", 0.125, false);
        near("-2^-2", -0.25, false);
        near("(-2)^2", 4.0, false);
        near("3E-2+2E3", 2000.03, false);
        assert_eq!(
            parse("2x+3y").unwrap().eval(4.0, 5.0, 0.0, false).unwrap(),
            23.0
        );
        assert_eq!(calculate("2ans", 7.0, false).unwrap(), 14.0);
        assert!(calculate("2 3", 0.0, false).is_err());
    }
    #[test]
    fn input_accepts_decimal_digit_scripts_and_math_symbols() {
        for text in ["١٢+٣", "۱۲+۳", "१२+३", "１２＋３", "১২+৩", "๑๒+๓", "၁၂+၃"]
        {
            near(text, 15.0, false);
        }
        near("١٬٢٣٤٫٥−٢×٣", 1228.5, false);
        near("1\u{202f}234.5 ÷ 2", 617.25, false);
        near("2π", 2.0 * PI, false);
        near("√(9)+2³+2⁻²", 11.25, false);
        near("2\u{22c5}3", 6.0, false);
        near("50٪*8", 4.0, false);
        assert!(normalize_input("١٬٢٣", false).is_err());
    }
    #[test]
    fn decimal_comma_has_explicit_argument_and_list_separators() {
        let normalized = normalize_input("min(1,5;2,5);3,5", true).unwrap();
        assert_eq!(normalized, "min(1.5,2.5),3.5");
        let expression = normalize_input("sum(1,5;2,5;٣,٥)", true).unwrap();
        near(&expression, 7.5, false);
        near(&normalize_input(",5+1,25", true).unwrap(), 1.75, false);
        near(&normalize_input("sum(,5;,25)", true).unwrap(), 0.75, false);
        near("min(1.5,2.5)", 1.5, false);
        near("min(1.5;2.5)", 1.5, false);
        for text in ["min(1, 2)", "min(1.5, 2.5)", "1,2,3", "1.234,5", "1,234.5"] {
            let result = normalize_input(text, true)
                .and_then(|normalized| calculate(&normalized, 0.0, false));
            assert!(result.is_err(), "{text}");
        }
    }
    #[test]
    fn scientific_and_statistical_functions_have_defined_domains() {
        near("asinh(sinh(2))", 2.0, false);
        near("acosh(cosh(2))", 2.0, false);
        near("atanh(tanh(0.5))", 0.5, false);
        near("cbrt(-8)", -2.0, false);
        near("round(12.345,2)", 12.35, false);
        near("round(1234,-2)", 1200.0, false);
        near("trunc(-2.8)", -2.0, false);
        near("logbase(8,2)", 3.0, false);
        near("ncr(10,3)", 120.0, false);
        near("npr(10,3)", 720.0, false);
        near("ncr(0,0)+npr(0,0)", 2.0, false);
        near("sum(1,2,3)", 6.0, false);
        near("prod(2,3,4)", 24.0, false);
        near("mean(2,4,6)", 4.0, false);
        near("median(4,1,3,2)", 2.5, false);
        near("variance(1,2,3)", 1.0, false);
        near("variancep(1,2,3)", 2.0 / 3.0, false);
        near("stddev(1,2,3)", 1.0, false);
        near("stddevp(2)", 0.0, false);
        for text in [
            "i",
            "asinh(1,2)",
            "acosh(0)",
            "atanh(1)",
            "round(1,1.5)",
            "ncr(3,4)",
            "stddev(1)",
            "logbase(4,1)",
        ] {
            assert!(calculate(text, 0.0, false).is_err(), "{text}");
        }
    }
    #[test]
    fn nesting_and_zero_decimal_formatting_are_bounded() {
        assert!(parse(&format!("{}1", "-".repeat(500))).is_err());
        assert!(parse(&format!("{}1{}", "(".repeat(49), ")".repeat(49))).is_err());
        assert!(parse(&"2²+".repeat(50)).is_err());
        assert_eq!(format(100.0, 0), "100");
        assert_eq!(format(10.0, 8), "10");
        assert_eq!(format(-0.0, 0), "0");
    }
    #[test]
    fn calculus() {
        near("diff(ln(x),2)", 0.5, false);
        near("integral(1/x,1,2)", 2f64.ln(), false);
        near("root(ln(x),2)", 1.0, false);
        near("diff(x^3,2)", 12.0, false);
        near("integral(x^2,0,3)", 9.0, false);
        near("root(x^2-2,1)", 2f64.sqrt(), false);
    }
    #[test]
    fn errors() {
        for s in ["1/0", "sqrt(-1)", "2+", "sin(2,3)", "foo(2)", "(-1)!"] {
            assert!(calculate(s, 0.0, false).is_err(), "{s}");
        }
    }
    #[test]
    fn matrices() {
        let a = matrix("1 2;3 4").unwrap();
        assert_eq!(
            matrix_op(&a, &a, "A × B").unwrap(),
            vec![vec![7.0, 10.0], vec![15.0, 22.0]]
        );
        assert_eq!(
            matrix_op(&a, &a, "Determinant A").unwrap(),
            vec![vec![-2.0]]
        );
        let inv = matrix_op(&a, &a, "Inverse A").unwrap();
        let identity = matrix_op(&a, &inv, "A × B").unwrap();
        assert!((identity[0][0] - 1.0).abs() < 1e-8);
        assert!(matrix("1 2;3").is_err());
    }
}
