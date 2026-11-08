use crate::math::{self, Expr};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    f64::consts::{FRAC_PI_2, LN_2, PI},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }
    pub fn is_real(self) -> bool {
        self.im == 0.0
    }
    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
    pub fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
    pub fn sub(self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }
    pub fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
    fn checked_product(self, other: Self) -> Result<Self, String> {
        let product = self.mul(other).checked()?;
        if product.re == 0.0
            && product.im == 0.0
            && (self.re != 0.0 || self.im != 0.0)
            && (other.re != 0.0 || other.im != 0.0)
        {
            return Err(
                "This nonzero product is smaller than the representable complex-number range."
                    .into(),
            );
        }
        Ok(product)
    }
    pub fn div(self, other: Self) -> Result<Self, String> {
        let scale = other.re.abs().max(other.im.abs());
        if scale == 0.0 {
            return Err("Cannot divide by zero".into());
        }
        let real = other.re / scale;
        let imaginary = other.im / scale;
        let denominator = real * real + imaginary * imaginary;
        let real_ratio = self.re / scale;
        let imaginary_ratio = self.im / scale;
        let result = if real_ratio.is_finite() && imaginary_ratio.is_finite() {
            Self {
                re: real_ratio * (real / denominator) + imaginary_ratio * (imaginary / denominator),
                im: imaginary_ratio * (real / denominator) - real_ratio * (imaginary / denominator),
            }
        } else {
            // Dividing by the normalized denominator first avoids intermediate
            // overflow when a small divisor rotates a large, finite answer.
            Self {
                re: ((self.re / denominator) * real + (self.im / denominator) * imaginary) / scale,
                im: ((self.im / denominator) * real - (self.re / denominator) * imaginary) / scale,
            }
        };
        let result = result.checked()?;
        if result.re == 0.0 && result.im == 0.0 && (self.re != 0.0 || self.im != 0.0) {
            return Err(
                "This nonzero quotient is smaller than the representable complex-number range."
                    .into(),
            );
        }
        Ok(result)
    }
    fn scale(self, value: f64) -> Self {
        Self {
            re: self.re * value,
            im: self.im * value,
        }
    }
    fn checked(mut self) -> Result<Self, String> {
        // Entered real values use the upper side of the principal branch cut.
        // Arithmetic signed zeros should not turn sqrt(-1) into -i.
        if self.im == 0.0 {
            self.im = 0.0;
        }
        if self.re == 0.0 {
            self.re = 0.0;
        }
        if self.is_finite() {
            Ok(self)
        } else {
            Err("Result is outside the finite complex-number range".into())
        }
    }
    fn logarithm(self) -> Result<Self, String> {
        let scale = self.re.abs().max(self.im.abs());
        if scale == 0.0 {
            return Err("The logarithm of zero is undefined".into());
        }
        let normalized = (self.re / scale).hypot(self.im / scale);
        Self {
            re: scale.ln() + normalized.ln(),
            im: self.im.atan2(self.re),
        }
        .checked()
    }
    fn exponential(self) -> Result<Self, String> {
        let magnitude = self.re.exp();
        if magnitude == 0.0 {
            return Err(
                "This nonzero exponential is smaller than the representable complex-number range."
                    .into(),
            );
        }
        let result = if magnitude.is_finite() {
            Self {
                re: magnitude * self.im.cos(),
                im: magnitude * self.im.sin(),
            }
        } else {
            // A magnitude just above f64::MAX can still have finite components.
            let half = (self.re - LN_2).exp();
            Self {
                re: half * (2.0 * self.im.cos()),
                im: half * (2.0 * self.im.sin()),
            }
        };
        result.checked()
    }
    fn square_root(self) -> Result<Self, String> {
        let scale = self.re.abs().max(self.im.abs());
        if scale == 0.0 {
            return Ok(Self {
                re: 0.0,
                im: self.im,
            });
        }
        let norm = (self.re / scale).hypot(self.im / scale);
        let result = if self.re >= 0.0 {
            let real = scale.sqrt() * ((norm + self.re / scale) / 2.0).sqrt();
            Self {
                re: real,
                im: self.im / (2.0 * real),
            }
        } else {
            let imaginary = scale.sqrt() * ((norm - self.re / scale) / 2.0).sqrt();
            Self {
                re: self.im.abs() / (2.0 * imaginary),
                im: imaginary.copysign(self.im),
            }
        };
        result.checked()
    }
    pub fn pow(self, exponent: Self) -> Result<Self, String> {
        const MAX_EXACT_INTEGER: f64 = 9_007_199_254_740_991.0;
        if exponent.is_real()
            && exponent.re.fract() == 0.0
            && exponent.re.abs() <= MAX_EXACT_INTEGER
        {
            let integer = exponent.re as i64;
            let mut power = integer.unsigned_abs();
            let mut factor = if integer < 0 {
                Self::from_real(1.0).div(self)?
            } else {
                self
            };
            let mut result = Self::from_real(1.0);
            while power != 0 {
                if power & 1 == 1 {
                    result = result.checked_product(factor)?;
                }
                power >>= 1;
                if power != 0 {
                    factor = factor.checked_product(factor)?;
                }
            }
            return Ok(result);
        }
        if self.re == 0.0 && self.im == 0.0 {
            return if exponent.is_real() && exponent.re > 0.0 {
                Ok(Self::default())
            } else {
                Err("This power of zero is undefined".into())
            };
        }
        self.logarithm()?.checked_product(exponent)?.exponential()
    }
    fn sine(self) -> Self {
        Self {
            re: self.re.sin() * self.im.cosh(),
            im: self.re.cos() * self.im.sinh(),
        }
    }
    fn cosine(self) -> Self {
        Self {
            re: self.re.cos() * self.im.cosh(),
            im: -self.re.sin() * self.im.sinh(),
        }
    }
    fn tangent(self) -> Result<Self, String> {
        if self.im == 0.0 && self.re.cos().abs() < 1e-12 {
            return Err("Tangent is undefined at this angle".into());
        }
        // Beyond this bound, the omitted real component is below double precision.
        const SATURATED_IMAGINARY: f64 = 20.0;
        if self.im.abs() > SATURATED_IMAGINARY {
            return Ok(Self {
                re: 0.0,
                im: self.im.signum(),
            });
        }
        let cosine = self.re.cos();
        let hyperbolic = self.im.sinh();
        let denominator = cosine * cosine + hyperbolic * hyperbolic;
        Self {
            re: self.re.sin() * cosine / denominator,
            im: hyperbolic * self.im.cosh() / denominator,
        }
        .checked()
    }
    fn arcsine(self) -> Result<Self, String> {
        const LARGE_ARGUMENT: f64 = 1.0e150;
        let (real, imaginary) = if self.re.abs().max(self.im.abs()) > LARGE_ARGUMENT {
            (self.re.atan2(self.im.abs()), self.logarithm()?.re + LN_2)
        } else {
            let first = (self.re + 1.0).hypot(self.im);
            let second = (self.re - 1.0).hypot(self.im);
            let alpha = (first / 2.0 + second / 2.0).max(1.0);
            // x/alpha avoids subtracting nearly equal distances for large x/y.
            ((self.re / alpha).clamp(-1.0, 1.0).asin(), alpha.acosh())
        };
        Self {
            re: real,
            im: imaginary.copysign(self.im),
        }
        .checked()
    }
    fn arctangent(self) -> Result<Self, String> {
        let first = Self {
            re: 1.0 - self.im,
            im: self.re,
        }
        .logarithm()?;
        let second = Self {
            re: 1.0 + self.im,
            im: -self.re,
        }
        .logarithm()?;
        let difference = first.sub(second);
        Self {
            re: difference.im / 2.0,
            im: -difference.re / 2.0,
        }
        .checked()
    }
    pub fn format(self, precision: usize) -> String {
        if self.im == 0.0 {
            return math::format(self.re, precision);
        }
        let coefficient = if self.im.abs() == 1.0 {
            String::new()
        } else {
            math::format(self.im.abs(), precision)
        };
        if self.re == 0.0 {
            format!("{}{coefficient}i", if self.im < 0.0 { "-" } else { "" })
        } else {
            format!(
                "{} {} {coefficient}i",
                math::format(self.re, precision),
                if self.im < 0.0 { "-" } else { "+" }
            )
        }
    }
}

fn substitute_variables(text: &str, variables: &BTreeMap<char, Complex>) -> Result<String, String> {
    let characters: Vec<char> = text.chars().collect();
    let mut result = String::new();
    let mut index = 0;
    while index < characters.len() {
        if characters[index].is_ascii_alphabetic() {
            let start = index;
            while index < characters.len() && characters[index].is_ascii_alphabetic() {
                index += 1;
            }
            let token: String = characters[start..index].iter().collect();
            let scientific_exponent = token == "E"
                && start > 0
                && (characters[start - 1].is_ascii_digit() || characters[start - 1] == '.')
                && characters.get(index).is_some_and(|character| {
                    character.is_ascii_digit() || matches!(character, '+' | '-')
                });
            if token.len() == 1 && characters[start].is_ascii_uppercase() && !scientific_exponent {
                let name = characters[start];
                let value = variables
                    .get(&name)
                    .ok_or_else(|| format!("Variable {name} has no value. Store a value first."))?;
                if !value.is_finite() {
                    return Err(format!(
                        "Variable {name} is outside the finite numeric range"
                    ));
                }
                if value.is_real() {
                    result.push_str(&format!("({:.17e})", value.re));
                } else if value.re == 0.0 {
                    result.push_str(&format!("(({:.17e})*i)", value.im));
                } else {
                    result.push_str(&format!("(({:.17e})+({:.17e})*i)", value.re, value.im));
                }
            } else {
                result.push_str(&token);
            }
        } else {
            result.push(characters[index]);
            index += 1;
        }
    }
    Ok(result)
}

pub(crate) fn evaluate(
    text: &str,
    ans: Complex,
    degrees: bool,
    variables: &BTreeMap<char, Complex>,
) -> Result<Complex, String> {
    let normalized = math::normalize_input(text, false)?;
    let substituted = substitute_variables(&normalized, variables)?;
    let expression = math::parse(&substituted)?;
    eval(&expression, ans, degrees)
}

fn needs_complex(expression: &Expr, ans: Complex) -> bool {
    match expression {
        Expr::Var('i') => true,
        Expr::Var('a') => !ans.is_real(),
        Expr::Unary(_, argument) => needs_complex(argument, ans),
        Expr::Binary(_, left, right) => needs_complex(left, ans) || needs_complex(right, ans),
        Expr::Call(_, arguments) => arguments
            .iter()
            .any(|argument| needs_complex(argument, ans)),
        _ => false,
    }
}

fn eval(expression: &Expr, ans: Complex, degrees: bool) -> Result<Complex, String> {
    let result = match expression {
        Expr::Number(value) => Complex::from_real(*value),
        Expr::Var('i') => Complex { re: 0.0, im: 1.0 },
        Expr::Var('a') => ans,
        Expr::Var(_) => return Err("Enter a stored uppercase variable or a numeric value.".into()),
        Expr::Unary(operation, argument) => {
            let argument = eval(argument, ans, degrees)?;
            match operation {
                '-' => argument.scale(-1.0),
                '+' => argument,
                '%' => argument.scale(0.01),
                '!' if argument.is_real() => Complex::from_real(
                    Expr::Unary('!', Box::new(Expr::Number(argument.re)))
                        .eval(0.0, 0.0, 0.0, degrees)?,
                ),
                '!' => return Err("Factorial requires a real whole number.".into()),
                _ => return Err("Unknown complex operator".into()),
            }
        }
        Expr::Binary(operation, left, right) => {
            let left = eval(left, ans, degrees)?;
            let right = eval(right, ans, degrees)?;
            match operation {
                '+' => left.add(right),
                '-' => left.sub(right),
                '*' => left.checked_product(right)?,
                '/' => left.div(right)?,
                '^' => left.pow(right)?,
                _ => return Err("Unknown complex operator".into()),
            }
        }
        Expr::Call(name, arguments) => {
            if matches!(name.as_str(), "diff" | "integral" | "root") {
                if needs_complex(expression, ans) {
                    return Err("Numerical derivatives, integrals and root finding currently require real-valued expressions.".into());
                }
                return expression
                    .eval(0.0, 0.0, ans.re, degrees)
                    .map(Complex::from_real);
            }
            let values = arguments
                .iter()
                .map(|argument| eval(argument, ans, degrees))
                .collect::<Result<Vec<_>, _>>()?;
            let argument = *values
                .first()
                .ok_or_else(|| format!("{name} needs an argument"))?;
            let unary = matches!(
                name.as_str(),
                "sin"
                    | "cos"
                    | "tan"
                    | "asin"
                    | "acos"
                    | "atan"
                    | "sinh"
                    | "cosh"
                    | "tanh"
                    | "asinh"
                    | "acosh"
                    | "atanh"
                    | "sqrt"
                    | "cbrt"
                    | "ln"
                    | "log"
                    | "exp"
                    | "abs"
                    | "arg"
                    | "re"
                    | "im"
                    | "conj"
                    | "norm"
            );
            if unary && values.len() != 1 {
                return Err(format!("{name} needs 1 argument"));
            }
            let angle = if degrees {
                argument.scale(PI / 180.0)
            } else {
                argument
            };
            let inverse_scale = if degrees { 180.0 / PI } else { 1.0 };
            match name.as_str() {
                "sin" => angle.sine(),
                "cos" => angle.cosine(),
                "tan" => angle.tangent()?,
                "asin" => argument.arcsine()?.scale(inverse_scale),
                "acos" => Complex::from_real(FRAC_PI_2)
                    .sub(argument.arcsine()?)
                    .scale(inverse_scale),
                "atan" => argument.arctangent()?.scale(inverse_scale),
                "sinh" => Complex {
                    re: argument.im,
                    im: -argument.re,
                }
                .sine()
                .mul(Complex { re: 0.0, im: 1.0 }),
                "cosh" => Complex {
                    re: -argument.im,
                    im: argument.re,
                }
                .cosine(),
                "tanh" => Complex {
                    re: -argument.im,
                    im: argument.re,
                }
                .tangent()?
                .mul(Complex { re: 0.0, im: -1.0 }),
                "asinh" => Complex {
                    re: -argument.im,
                    im: argument.re,
                }
                .arcsine()?
                .mul(Complex { re: 0.0, im: -1.0 }),
                "acosh" => argument
                    .add(Complex::from_real(1.0))
                    .square_root()?
                    .mul(argument.sub(Complex::from_real(1.0)).square_root()?)
                    .add(argument)
                    .logarithm()?,
                "atanh" => {
                    let first = argument.add(Complex::from_real(1.0)).logarithm()?;
                    let second = Complex {
                        re: 1.0 - argument.re,
                        im: -argument.im,
                    }
                    .logarithm()?;
                    first.sub(second).scale(0.5)
                }
                "sqrt" => argument.square_root()?,
                "cbrt" if argument.is_real() => Complex::from_real(argument.re.cbrt()),
                "cbrt" => argument.pow(Complex::from_real(1.0 / 3.0))?,
                "ln" => argument.logarithm()?,
                "log" => argument.logarithm()?.scale(1.0 / 10.0_f64.ln()),
                "exp" => argument.exponential()?,
                "abs" => Complex::from_real(argument.re.hypot(argument.im)),
                "arg" => {
                    if argument.re == 0.0 && argument.im == 0.0 {
                        return Err("The argument of zero is undefined".into());
                    }
                    Complex::from_real(argument.im.atan2(argument.re) * inverse_scale)
                }
                "re" => Complex::from_real(argument.re),
                "im" => Complex::from_real(argument.im),
                "conj" => Complex {
                    re: argument.re,
                    im: -argument.im,
                },
                "norm" => Complex::from_real(argument.re * argument.re + argument.im * argument.im),
                "logbase" => {
                    if values.len() != 2 {
                        return Err("logbase needs 2 arguments".into());
                    }
                    argument.logarithm()?.div(values[1].logarithm()?)?
                }
                _ => {
                    if values.iter().any(|value| !value.is_real()) {
                        return Err(format!("{name} requires real arguments."));
                    }
                    Expr::Call(
                        name.clone(),
                        values.iter().map(|value| Expr::Number(value.re)).collect(),
                    )
                    .eval(0.0, 0.0, 0.0, degrees)
                    .map(Complex::from_real)?
                }
            }
        }
    };
    result.checked()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(text: &str, expected: Complex) {
        let result = evaluate(text, Complex::default(), false, &BTreeMap::new()).unwrap();
        assert!(
            (result.re - expected.re).abs() < 1e-8 && (result.im - expected.im).abs() < 1e-8,
            "{text}: {result:?}"
        );
    }
    #[test]
    fn imaginary_arithmetic_and_principal_roots_are_consistent() {
        near("i^2", Complex::from_real(-1.0));
        near("sqrt(-1)", Complex { re: 0.0, im: 1.0 });
        near("(1+2i)*(3-4i)", Complex { re: 11.0, im: 2.0 });
        near("(1+2i)/(3-4i)", Complex { re: -0.2, im: 0.4 });
        near("i^i", Complex::from_real((-FRAC_PI_2).exp()));
        near("conj(3-4i)", Complex { re: 3.0, im: 4.0 });
        near("abs(3+4i)", Complex::from_real(5.0));
        near("exp(i*pi)", Complex::from_real(-1.0));
        near("cbrt(-8)", Complex::from_real(-2.0));
    }
    #[test]
    fn logarithms_and_inverse_functions_use_principal_branches() {
        near("ln(-1)", Complex { re: 0.0, im: PI });
        near(
            "asin(2)",
            Complex {
                re: FRAC_PI_2,
                im: 2.0_f64.acosh(),
            },
        );
        near(
            "acos(2)",
            Complex {
                re: 0.0,
                im: -2.0_f64.acosh(),
            },
        );
        near(
            "atan(2i)",
            Complex {
                re: FRAC_PI_2,
                im: 3.0_f64.ln() / 2.0,
            },
        );
        near(
            "asin(0.5+0.25i)",
            Complex {
                re: 0.5016088532755008,
                im: 0.28139605624529274,
            },
        );
        near("sinh(1)", Complex::from_real(1.0_f64.sinh()));
        near("cosh(1)", Complex::from_real(1.0_f64.cosh()));
        near("tanh(1)", Complex::from_real(1.0_f64.tanh()));
        near("asinh(2)", Complex::from_real(2.0_f64.asinh()));
        near(
            "acosh(-2)",
            Complex {
                re: 2.0_f64.acosh(),
                im: PI,
            },
        );
        near(
            "atanh(2)",
            Complex {
                re: 3.0_f64.ln() / 2.0,
                im: FRAC_PI_2,
            },
        );
    }
    #[test]
    fn variables_ans_and_exponents_remain_distinct() {
        let variables = BTreeMap::from([
            ('A', Complex { re: 2.0, im: 3.0 }),
            ('E', Complex::from_real(7.0)),
        ]);
        assert_eq!(
            evaluate("A+ans", Complex { re: 1.0, im: -3.0 }, false, &variables).unwrap(),
            Complex::from_real(3.0)
        );
        assert_eq!(
            evaluate("2E3+E", Complex::default(), false, &variables).unwrap(),
            Complex::from_real(2007.0)
        );
        assert!(evaluate("B", Complex::default(), false, &variables).is_err());
        let sine = evaluate("sin(30)", Complex::default(), true, &variables).unwrap();
        assert!((sine.re - 0.5).abs() < 1e-12 && sine.im == 0.0);
    }
    #[test]
    fn real_calculus_remains_available_in_complex_mode() {
        near("diff(x^3,2)", Complex::from_real(12.0));
        near("integral(x^2,0,3)", Complex::from_real(9.0));
        near("root(x^2-2,1)", Complex::from_real(2.0_f64.sqrt()));
        let variables = BTreeMap::from([('A', Complex::from_real(2.0))]);
        let derivative = evaluate("diff(A*x^2,3)", Complex::default(), false, &variables).unwrap();
        assert!((derivative.re - 12.0).abs() < 1e-8 && derivative.im == 0.0);
    }
    #[test]
    fn scientific_constants_remain_distinct_from_stored_letter_variables() {
        let variables = BTreeMap::from([
            ('P', Complex::from_real(7.0)),
            ('A', Complex::from_real(3.0)),
        ]);
        let result = evaluate("planck/P", Complex::default(), false, &variables).unwrap();
        assert!((result.re / (6.626_070_15e-34 / 7.0) - 1.0).abs() < 1e-14 && result.im == 0.0);
        assert_eq!(
            evaluate("A+2E3", Complex::default(), false, &variables).unwrap(),
            Complex::from_real(2003.0)
        );
        assert_eq!(
            Complex::from_real(6.626_070_15e-34).format(8),
            "6.62607015e-34"
        );
        assert_eq!(Complex { re: 0.0, im: 1e-7 }.format(2), "1e-7i");
        for expression in [
            "exp(-1000)",
            "(1E-200)^2",
            "1E-300*1E-100",
            "1E-300/1E100",
            "diff(abs(x),0)",
            "root(1/x,1)",
        ] {
            assert!(
                evaluate(expression, Complex::default(), false, &variables).is_err(),
                "{expression}"
            );
        }
    }
    #[test]
    fn bounded_errors_and_large_finite_division_are_explicit() {
        for text in [
            "1/0",
            "ln(0)",
            "atan(i)",
            "min(i,2)",
            "i!",
            "integral(i*x,0,1)",
            "exp(10000)",
        ] {
            assert!(
                evaluate(text, Complex::default(), false, &BTreeMap::new()).is_err(),
                "{text}"
            );
        }
        let large = Complex {
            re: 1e308,
            im: 1e308,
        };
        let result = large.div(large).unwrap();
        assert!((result.re - 1.0).abs() < 1e-12 && result.im.abs() < 1e-12);
        let root = large.square_root().unwrap();
        assert!(root.is_finite());
        let rotated = Complex::from_real(1.7e308)
            .div(Complex { re: 0.75, im: 0.75 })
            .unwrap();
        assert!((rotated.re / (1.7e308 / 1.5) - 1.0).abs() < 1e-12);
        assert!((rotated.im / (-1.7e308 / 1.5) - 1.0).abs() < 1e-12);
        let tiny = Complex::from_real(f64::from_bits(1))
            .div(Complex {
                re: f64::from_bits(1),
                im: f64::from_bits(1),
            })
            .unwrap();
        assert_eq!(tiny, Complex { re: 0.5, im: -0.5 });
        assert_eq!(Complex { re: 0.0, im: -1.0 }.format(8), "-i");
        assert_eq!(Complex { re: 3.0, im: 2.0 }.format(8), "3 + 2i");
    }
}
