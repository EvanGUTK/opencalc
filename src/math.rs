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

// Exact SI definitions represented as f64, in the SI units stated by NIST:
// https://www.nist.gov/pml/special-publication-330/sp-330-section-2
pub(crate) const AVOGADRO: f64 = 6.022_140_76e23; // mol^-1
pub(crate) const PLANCK: f64 = 6.626_070_15e-34; // J s
pub(crate) const BOLTZMANN: f64 = 1.380_649e-23; // J/K
pub(crate) const LIGHTSPEED: f64 = 299_792_458.0; // m/s
pub(crate) const GAS_CONSTANT: f64 = 8.314_462_618_153_24; // J/(mol K), N_A * k

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
            let literal: String = self.chars[start..self.pos].iter().collect();
            let value: f64 = literal.parse().map_err(|_| "Check the decimal number")?;
            if value == 0.0
                && literal.split(['e', 'E']).next().is_some_and(|mantissa| {
                    mantissa.chars().any(|digit| ('1'..='9').contains(&digit))
                })
            {
                return Err(
                    "This nonzero number is smaller than the representable numeric range.".into(),
                );
            }
            Expr::Number(value)
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
                "avogadro" => Expr::Number(AVOGADRO),
                "planck" => Expr::Number(PLANCK),
                "boltzmann" => Expr::Number(BOLTZMANN),
                "lightspeed" => Expr::Number(LIGHTSPEED),
                "gasconstant" => Expr::Number(GAS_CONSTANT),
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
                    '*' => {
                        let product = a * b;
                        if product == 0.0 && a != 0.0 && b != 0.0 {
                            return Err("This nonzero product is smaller than the representable numeric range.".into());
                        }
                        product
                    }
                    '/' => {
                        if b == 0.0 {
                            return Err("Cannot divide by zero".into());
                        }
                        let quotient = a / b;
                        if quotient == 0.0 && a != 0.0 {
                            return Err("This nonzero quotient is smaller than the representable numeric range.".into());
                        }
                        quotient
                    }
                    '^' => {
                        let power = a.powf(b);
                        if power == 0.0 && a != 0.0 {
                            return Err("This nonzero power is smaller than the representable numeric range.".into());
                        }
                        power
                    }
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
                    "exp" => {
                        let exponential = a.exp();
                        if exponential == 0.0 {
                            return Err("This nonzero exponential is smaller than the representable numeric range.".into());
                        }
                        exponential
                    }
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
                                    if !mean.is_finite() || !squared_differences.is_finite() {
                                        return Err("Statistical accumulation exceeded the finite numeric range; rescale the data.".into());
                                    }
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
                    "diff" => numerical_derivative(&args[0], arg(1)?, y, ans, degrees)?,
                    "integral" => numerical_integral(&args[0], arg(1)?, arg(2)?, y, ans, degrees)?,
                    "root" => numerical_root(&args[0], arg(1)?, y, ans, degrees)?,
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
fn numerical_derivative(
    expression: &Expr,
    at: f64,
    y: f64,
    ans: f64,
    degrees: bool,
) -> Result<f64, String> {
    let undefined = "A finite two-sided derivative could not be established here; the expression may be undefined, discontinuous or too steep for numerical differentiation.";
    let center = expression
        .eval(at, y, ans, degrees)
        .map_err(|_| undefined.to_string())?;
    let mut h = f64::EPSILON.cbrt() * at.abs().max(1.0);
    let mut previous = None;
    let mut reference_slope: f64 = 0.0;
    for _ in 0..48 {
        let left_at = at - h;
        let right_at = at + h;
        if !left_at.is_finite() || !right_at.is_finite() || left_at == at || right_at == at {
            break;
        }
        let left = expression
            .eval(left_at, y, ans, degrees)
            .map_err(|_| undefined.to_string())?;
        let right = expression
            .eval(right_at, y, ans, degrees)
            .map_err(|_| undefined.to_string())?;
        let left_slope = (center - left) / (at - left_at);
        let right_slope = (right - center) / (right_at - at);
        let slope = left_slope / 2.0 + right_slope / 2.0;
        reference_slope = reference_slope.max(left_slope.abs()).max(right_slope.abs());
        let tolerance = 1e-6 * reference_slope;
        if slope.is_finite()
            && (left_slope - right_slope).abs() <= tolerance
            && previous.is_some_and(|old: f64| (old - slope).abs() <= tolerance)
        {
            return Ok(slope);
        }
        previous = Some(slope);
        h /= 2.0;
    }
    Err(undefined.into())
}

fn numerical_root(
    expression: &Expr,
    mut at: f64,
    y: f64,
    ans: f64,
    degrees: bool,
) -> Result<f64, String> {
    let no_root = "No finite root converged. Try another starting value; a small function value alone does not establish a root.";
    let mut value = expression.eval(at, y, ans, degrees)?;
    let initial_residual = value.abs();
    for _ in 0..100 {
        if value == 0.0 {
            return Ok(at);
        }
        let derivative = numerical_derivative(expression, at, y, ans, degrees)?;
        if derivative == 0.0 {
            return Err(no_root.into());
        }
        let step = value / derivative;
        if !step.is_finite() {
            return Err(no_root.into());
        }
        if step.abs() <= 1e-10 * at.abs().max(1.0) && value.abs() <= 1e-10 * initial_residual {
            return Ok(at);
        }
        let mut factor = 1.0;
        let mut next = None;
        for _ in 0..24 {
            let candidate = at - factor * step;
            if candidate.is_finite()
                && candidate != at
                && let Ok(candidate_value) = expression.eval(candidate, y, ans, degrees)
                && candidate_value.abs() < value.abs()
            {
                next = Some((candidate, candidate_value));
                break;
            }
            factor /= 2.0;
        }
        let Some((candidate, candidate_value)) = next else {
            return Err(no_root.into());
        };
        at = candidate;
        value = candidate_value;
    }
    Err(no_root.into())
}

struct Integrator<'a> {
    expression: &'a Expr,
    y: f64,
    ans: f64,
    degrees: bool,
    evaluations: usize,
}

#[derive(Clone, Copy)]
struct SimpsonPanel {
    lo: f64,
    hi: f64,
    left: f64,
    middle: f64,
    right: f64,
    estimate: f64,
}

impl Integrator<'_> {
    fn sample(&mut self, at: f64) -> Result<f64, String> {
        const EVALUATION_LIMIT: usize = 32_768;
        if self.evaluations >= EVALUATION_LIMIT {
            return Err("Numerical integration did not converge within its evaluation limit; singular or rapidly varying integrands are unsupported.".into());
        }
        self.evaluations += 1;
        self.expression.eval(at, self.y, self.ans, self.degrees).map_err(|_| "Numerical integration requires finite values throughout the interval, including both endpoints. Improper integrals are unsupported.".into())
    }
    fn refine(&mut self, panel: SimpsonPanel, tolerance: f64, depth: usize) -> Result<f64, String> {
        let middle_at = panel.lo / 2.0 + panel.hi / 2.0;
        let left_at = panel.lo / 2.0 + middle_at / 2.0;
        let right_at = middle_at / 2.0 + panel.hi / 2.0;
        if left_at == panel.lo || right_at == panel.hi {
            return Err("The integration interval is too small to resolve numerically.".into());
        }
        let left_middle = self.sample(left_at)?;
        let right_middle = self.sample(right_at)?;
        let left_estimate =
            simpson_estimate(middle_at - panel.lo, panel.left, left_middle, panel.middle);
        let right_estimate = simpson_estimate(
            panel.hi - middle_at,
            panel.middle,
            right_middle,
            panel.right,
        );
        let refined = left_estimate + right_estimate;
        let difference = refined - panel.estimate;
        // Independent Gauss-Legendre nodes prevent equal-spaced Simpson samples
        // from mistaking an ordinary oscillating function for a constant.
        let half_width = (panel.hi - panel.lo) / 2.0;
        let gauss_offset = half_width * (3.0_f64 / 5.0).sqrt();
        let gauss_left = self.sample(middle_at - gauss_offset)?;
        let gauss_right = self.sample(middle_at + gauss_offset)?;
        let gauss = (panel.hi - panel.lo)
            * (gauss_left * (5.0 / 18.0) + panel.middle * (4.0 / 9.0) + gauss_right * (5.0 / 18.0));
        let scale = [
            panel.left,
            panel.middle,
            panel.right,
            left_middle,
            right_middle,
            gauss_left,
            gauss_right,
        ]
        .into_iter()
        .map(f64::abs)
        .fold(0.0, f64::max)
            * (panel.hi - panel.lo).abs();
        if !scale.is_finite()
            || !refined.is_finite()
            || !panel.estimate.is_finite()
            || !gauss.is_finite()
        {
            return Err("Numerical integration exceeded the finite numeric range; rescale the integrand or interval.".into());
        }
        let roundoff = 64.0 * f64::EPSILON * scale;
        let tolerance = if depth == 0 { 1e-10 * scale } else { tolerance };
        let error = (difference.abs() / 15.0).max((gauss - refined).abs());
        if error <= tolerance.max(roundoff) {
            return Ok(refined + difference / 15.0);
        }
        if depth >= 60 {
            return Err("Numerical integration did not converge; check for a discontinuity, singularity or rapidly varying integrand.".into());
        }
        let left = self.refine(
            SimpsonPanel {
                lo: panel.lo,
                hi: middle_at,
                left: panel.left,
                middle: left_middle,
                right: panel.middle,
                estimate: left_estimate,
            },
            tolerance / 2.0,
            depth + 1,
        )?;
        let right = self.refine(
            SimpsonPanel {
                lo: middle_at,
                hi: panel.hi,
                left: panel.middle,
                middle: right_middle,
                right: panel.right,
                estimate: right_estimate,
            },
            tolerance / 2.0,
            depth + 1,
        )?;
        Ok(left + right)
    }
}

fn simpson_estimate(width: f64, left: f64, middle: f64, right: f64) -> f64 {
    width * (left / 6.0 + middle * (2.0 / 3.0) + right / 6.0)
}

fn numerical_integral(
    expression: &Expr,
    lo: f64,
    hi: f64,
    y: f64,
    ans: f64,
    degrees: bool,
) -> Result<f64, String> {
    if !(hi - lo).is_finite() {
        return Err("Use a finite, numerically resolvable integration interval.".into());
    }
    let mut integrator = Integrator {
        expression,
        y,
        ans,
        degrees,
        evaluations: 0,
    };
    let left = integrator.sample(lo)?;
    if lo == hi {
        return Ok(0.0);
    }
    let middle = integrator.sample(lo / 2.0 + hi / 2.0)?;
    let right = integrator.sample(hi)?;
    integrator.refine(
        SimpsonPanel {
            lo,
            hi,
            left,
            middle,
            right,
            estimate: simpson_estimate(hi - lo, left, middle, right),
        },
        0.0,
        0,
    )
}

pub fn calculate(s: &str, ans: f64, degrees: bool) -> Result<f64, String> {
    parse(s)?.eval(0.0, 0.0, ans, degrees)
}
pub fn format(v: f64, precision: usize) -> String {
    // More decimal places cannot recover information beyond f64 precision.
    let precision = precision.min(16);
    if v != 0.0 && (v.abs() >= 1e12 || v.abs() < 1e-8) {
        return scientific_format(v, precision);
    }
    let s = format!("{v:.precision$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        &s
    };
    if v != 0.0 && (s == "0" || s == "-0") {
        // Rounding a concentration or a small coefficient must not erase it.
        scientific_format(v, precision)
    } else if s.is_empty() || s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}
fn scientific_format(value: f64, precision: usize) -> String {
    let scientific = format!("{value:.precision$e}");
    if let Some((mantissa, exponent)) = scientific.split_once('e') {
        let mantissa = if mantissa.contains('.') {
            mantissa.trim_end_matches('0').trim_end_matches('.')
        } else {
            mantissa
        };
        format!("{mantissa}e{exponent}")
    } else {
        scientific
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
    let valid = |matrix: &Matrix| {
        !matrix.is_empty()
            && !matrix[0].is_empty()
            && matrix.len() <= 12
            && matrix[0].len() <= 12
            && matrix.iter().all(|row| {
                row.len() == matrix[0].len() && row.iter().all(|value| value.is_finite())
            })
    };
    if !valid(a) {
        return Err(
            "Use a rectangular matrix of finite values, with at most 12 rows and columns.".into(),
        );
    }
    if matches!(op, "A + B" | "A × B") && !valid(b) {
        return Err("Matrix B must contain rectangular rows of finite values.".into());
    }
    let (n, m) = (a.len(), a[0].len());
    let result: Result<Matrix, String> = match op {
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
            let inverse_requested = op == "Inverse A";
            let mut scales: Vec<f64> = a
                .iter()
                .map(|row| row.iter().map(|value| value.abs()).fold(0.0, f64::max))
                .collect();
            let mut inv = vec![vec![0.0; n]; n];
            for (i, row) in inv.iter_mut().enumerate() {
                row[i] = 1.0;
            }
            let mut det = ScaledProduct::new();
            for k in 0..n {
                let p = (k..n)
                    .max_by(|&i, &j| {
                        let ratio = |row: usize| {
                            if scales[row] == 0.0 {
                                0.0
                            } else {
                                work[row][k].abs() / scales[row]
                            }
                        };
                        ratio(i).total_cmp(&ratio(j))
                    })
                    .unwrap();
                if work[p][k] == 0.0 {
                    return if op == "Determinant A" {
                        Ok(vec![vec![0.0]])
                    } else {
                        Err("This matrix is singular; it has no inverse".into())
                    };
                }
                if p != k {
                    work.swap(p, k);
                    inv.swap(p, k);
                    scales.swap(p, k);
                    det.negative = !det.negative;
                }
                let pivot = work[k][k];
                det.multiply(pivot);
                for j in 0..n {
                    work[k][j] /= pivot;
                    if inverse_requested {
                        inv[k][j] /= pivot;
                    }
                }
                for i in 0..n {
                    if i == k {
                        continue;
                    }
                    let f = work[i][k];
                    for j in 0..n {
                        work[i][j] -= f * work[k][j];
                        if inverse_requested {
                            inv[i][j] -= f * inv[k][j];
                        }
                    }
                }
                if work
                    .iter()
                    .chain(inv.iter())
                    .flatten()
                    .any(|value| !value.is_finite())
                {
                    return Err(
                        "Matrix elimination exceeded the finite numeric range; rescale the matrix."
                            .into(),
                    );
                }
            }
            if op == "Determinant A" {
                Ok(vec![vec![det.value()?]])
            } else {
                Ok(inv)
            }
        }
        _ => Err("Choose a matrix operation".into()),
    };
    let result = result?;
    if result.iter().flatten().any(|value| !value.is_finite()) {
        Err("The matrix result is outside the finite numeric range.".into())
    } else {
        Ok(result)
    }
}

struct ScaledProduct {
    mantissa: f64,
    exponent: i32,
    negative: bool,
}
impl ScaledProduct {
    fn new() -> Self {
        Self {
            mantissa: 1.0,
            exponent: 0,
            negative: false,
        }
    }
    fn multiply(&mut self, value: f64) {
        self.negative ^= value.is_sign_negative();
        let mut bits = value.abs().to_bits();
        let mut adjustment = 0;
        if bits >> 52 == 0 {
            bits = (value.abs() * 4_503_599_627_370_496.0).to_bits();
            adjustment = -52;
        }
        self.exponent += (bits >> 52) as i32 - 1023 + adjustment;
        self.mantissa *= f64::from_bits((bits & ((1_u64 << 52) - 1)) | (1023_u64 << 52));
        if self.mantissa >= 2.0 {
            self.mantissa /= 2.0;
            self.exponent += 1;
        }
    }
    fn value(self) -> Result<f64, String> {
        let mut value = self.mantissa;
        let mut exponent = self.exponent;
        while exponent > 512 {
            value *= 2.0_f64.powi(512);
            exponent -= 512;
        }
        while exponent < -512 {
            value *= 2.0_f64.powi(-512);
            exponent += 512;
        }
        value *= 2.0_f64.powi(exponent);
        if !value.is_finite() || value == 0.0 {
            return Err(
                "The nonzero determinant is outside the representable numeric range.".into(),
            );
        }
        Ok(if self.negative { -value } else { value })
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
    fn named_si_constants_support_chemistry_and_physics_without_letter_aliases() {
        let relative = |expression: &str, expected: f64| {
            let actual = calculate(expression, 0.0, false).unwrap();
            assert!(
                (actual / expected - 1.0).abs() <= 2e-14,
                "{expression}: {actual}"
            );
        };
        relative("avogadro", 6.022_140_76e23);
        relative("PLANCK", 6.626_070_15e-34);
        relative("boltzmann", 1.380_649e-23);
        relative("lightspeed", 299_792_458.0);
        relative("gasconstant", 8.314_462_618_153_24);
        relative("avogadro*boltzmann", 8.314_462_618_153_24);
        relative("planck*lightspeed/(500E-9)", 3.972_891_714_297_857e-19);
        relative("2avogadro", 1.204_428_152e24);
        near("-log(1E-7)", 7.0, false);
        relative("10^(-7.4)", 3.981_071_705_534_969e-8);
        for letter in ["N", "H", "K", "C", "R"] {
            assert!(parse(letter).is_err(), "{letter}");
        }
    }
    #[test]
    fn scientific_format_preserves_nonzero_magnitude_and_selected_precision() {
        assert_eq!(format(6.626_070_15e-34, 8), "6.62607015e-34");
        assert_eq!(format(6.022_140_76e23, 8), "6.02214076e23");
        assert_eq!(format(1e-7, 2), "1e-7");
        assert_eq!(format(-1e-7, 2), "-1e-7");
        assert_eq!(format(1e-8, 8), "0.00000001");
        assert_eq!(format(3.981_071_705_534_969e-8, 2), "3.98e-8");
        assert_eq!(format(100.0, 0), "100");
        assert_eq!(format(0.0, 0), "0");
        assert_eq!(format(-0.0, 8), "0");
        assert_eq!(format(0.125, 3), "0.125");
        assert_eq!(format(1.0 / 3.0, 6), "0.333333");
        assert_ne!(format(f64::from_bits(1), 8), "0");
    }
    #[test]
    fn nonzero_underflow_never_becomes_a_false_zero_or_root() {
        for expression in [
            "1E-999",
            "exp(-1000)",
            "1E-300*1E-100",
            "1E-300/1E100",
            "(1E-200)^2",
            "root(exp(x),-1000)",
        ] {
            assert!(calculate(expression, 0.0, false).is_err(), "{expression}");
        }
        assert_eq!(calculate("0E-999", 0.0, false).unwrap(), 0.0);
        assert!(calculate("5E-324", 0.0, false).unwrap() > 0.0);
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
    fn numerical_derivatives_require_a_defined_two_sided_limit() {
        near("diff(x^2,0)", 0.0, false);
        near("diff(x^3,0)", 0.0, false);
        near(
            "diff(sin(x),30)",
            30.0_f64.to_radians().cos() * PI / 180.0,
            true,
        );
        for expression in [
            "diff(abs(x),0)",
            "diff(1E-15*abs(x),0)",
            "diff(sqrt(x),0)",
            "diff(1/x,0)",
            "diff(floor(x),0)",
            "diff(sign(x),0)",
        ] {
            assert!(calculate(expression, 0.0, false).is_err(), "{expression}");
        }
    }
    #[test]
    fn numerical_integrals_converge_or_report_unsupported_domains() {
        near("integral(exp(-x^2),0,1)", 0.746_824_132_812_427_1, false);
        near("integral(sqrt(x),0,1)", 2.0 / 3.0, false);
        near("integral(x^2,3,0)", -9.0, false);
        near("integral(sin(x),0,2pi)", 0.0, false);
        near("integral(cos(8pi*x),0,1)", 0.0, false);
        near(
            "integral(sin(100x),0,1)",
            (1.0 - 100.0_f64.cos()) / 100.0,
            false,
        );
        let tiny = calculate("integral(planck*x,0,2)", 0.0, false).unwrap();
        assert!((tiny / 1.325_214_03e-33 - 1.0).abs() < 1e-12);
        for expression in [
            "integral(1/x,-1,1)",
            "integral(ln(x),0,1)",
            "integral(tan(x),0,pi)",
            "integral(1/(x-0.1234567),0,1)",
            "integral(1/(x-0.1250000001)^2,-1,1)",
        ] {
            assert!(calculate(expression, 0.0, false).is_err(), "{expression}");
        }
    }
    #[test]
    fn numerical_roots_require_step_and_residual_convergence() {
        near("root(ln(x),0.1)", 1.0, false);
        near("root(sqrt(x),1)", 0.0, false);
        near("root(1E-100*(x-2),1)", 2.0, false);
        near("root(x^2,1)", 0.0, false);
        for expression in [
            "root(1/x,1)",
            "root(1/(x-0.1),1)",
            "root(exp(x),0)",
            "root(x^2+1,0)",
            "root(1E-100,2)",
        ] {
            assert!(calculate(expression, 0.0, false).is_err(), "{expression}");
        }
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
    #[test]
    fn matrix_singularity_and_determinants_are_independent_of_unit_scale() {
        let small = matrix("1E-15 0;0 1E-15").unwrap();
        let determinant = matrix_op(&small, &small, "Determinant A").unwrap()[0][0];
        assert!((determinant / 1e-30 - 1.0).abs() < 1e-12);
        let inverse = matrix_op(&small, &small, "Inverse A").unwrap();
        assert!((inverse[0][0] / 1e15 - 1.0).abs() < 1e-12);
        assert!((inverse[1][1] / 1e15 - 1.0).abs() < 1e-12);
        let balanced = matrix("1E200 0 0;0 1E200 0;0 0 1E-200").unwrap();
        assert!(
            (matrix_op(&balanced, &balanced, "Determinant A").unwrap()[0][0] / 1e200 - 1.0).abs()
                < 1e-12
        );
        let subnormal = matrix("5E-324").unwrap();
        assert_eq!(
            matrix_op(&subnormal, &subnormal, "Determinant A").unwrap()[0][0],
            f64::from_bits(1)
        );
        assert!(matrix_op(&subnormal, &subnormal, "Inverse A").is_err());
        let unrepresentable = matrix("1E-200 0;0 1E-200").unwrap();
        assert!(matrix_op(&unrepresentable, &unrepresentable, "Determinant A").is_err());
        assert!(matrix_op(&vec![], &small, "Inverse A").is_err());
        assert!(matrix_op(&small, &vec![], "A + B").is_err());
    }
}
