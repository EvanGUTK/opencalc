use crate::*;

const MAX_ITEMS: usize = 1000;
const INTEGER_LIMIT: f64 = 1_000_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Statistics,
    Regression,
    Combination,
    Permutation,
    Binomial,
    BinomialCdf,
    Normal,
    ZScore,
    Simplify,
    AddFractions,
    MultiplyFractions,
    Gcd,
    Lcm,
    Factors,
    Quadratic,
    Linear,
    Arithmetic,
    Geometric,
    Length,
    Mass,
    Temperature,
    Angle,
    PercentChange,
    Percentage,
    Compound,
    Savings,
    Loan,
    Base,
    Distance,
    RightTriangle,
}
struct Tool {
    kind: Kind,
    name: &'static str,
    group: &'static str,
    help: &'static str,
    fields: &'static [(&'static str, &'static str)],
}
const TOOLS: &[Tool] = &[
    Tool {
        kind: Kind::Statistics,
        name: "Data summary",
        group: "Statistics",
        help: "Paste numbers separated by commas, spaces, semicolons, or new lines. Quartiles use the median of each half, excluding the overall median for odd samples.",
        fields: &[("Data", "2, 4, 4, 4, 5, 5, 7, 9")],
    },
    Tool {
        kind: Kind::Regression,
        name: "Linear regression",
        group: "Statistics",
        help: "Fit y = slope*x + intercept with least squares. The lists must have equal lengths and at least two pairs.",
        fields: &[("X values", "1, 2, 3, 4, 5"), ("Y values", "2, 4, 5, 4, 5")],
    },
    Tool {
        kind: Kind::Combination,
        name: "Combinations (nCr)",
        group: "Probability",
        help: "Choose r items from n when order does not matter: n! / (r! (n-r)!). Whole numbers, 0 <= r <= n <= 170.",
        fields: &[("n: available items", "10"), ("r: chosen items", "3")],
    },
    Tool {
        kind: Kind::Permutation,
        name: "Permutations (nPr)",
        group: "Probability",
        help: "Choose and arrange r items from n when order matters: n! / (n-r)!. Whole numbers, 0 <= r <= n <= 170.",
        fields: &[("n: available items", "10"), ("r: arranged items", "3")],
    },
    Tool {
        kind: Kind::Binomial,
        name: "Binomial probability",
        group: "Probability",
        help: "Probability of exactly k successes in n independent trials, each with the same success probability p. Enter p from 0 to 1; n <= 170.",
        fields: &[
            ("n: trials", "10"),
            ("p: success probability", "0.5"),
            ("k: successes", "3"),
        ],
    },
    Tool {
        kind: Kind::BinomialCdf,
        name: "Cumulative binomial",
        group: "Probability",
        help: "Probability of at most k successes: sum the exact probabilities from 0 through k. Independent trials, fixed p, n <= 170.",
        fields: &[
            ("n: trials", "10"),
            ("p: success probability", "0.5"),
            ("k: maximum successes", "3"),
        ],
    },
    Tool {
        kind: Kind::Normal,
        name: "Normal probability",
        group: "Probability",
        help: "Approximate area between two bounds under a normal curve. Standard deviation must be positive. Numerical approximation, absolute error about 0.000001.",
        fields: &[
            ("Lower bound", "-1"),
            ("Upper bound", "1"),
            ("Mean", "0"),
            ("Standard deviation", "1"),
        ],
    },
    Tool {
        kind: Kind::ZScore,
        name: "Z-score",
        group: "Statistics",
        help: "How many standard deviations a value is from the mean: z = (value - mean) / standard deviation.",
        fields: &[
            ("Value", "85"),
            ("Mean", "70"),
            ("Standard deviation", "10"),
        ],
    },
    Tool {
        kind: Kind::Simplify,
        name: "Simplify a fraction",
        group: "Numbers",
        help: "Reduce numerator and denominator by their greatest common divisor. Negative denominators are normalized. Integers up to one billion in magnitude.",
        fields: &[("Numerator", "18"), ("Denominator", "24")],
    },
    Tool {
        kind: Kind::AddFractions,
        name: "Add fractions",
        group: "Numbers",
        help: "a/b + c/d = (a*d + b*c) / (b*d), then reduce. Use a negative numerator to subtract.",
        fields: &[
            ("First numerator", "1"),
            ("First denominator", "3"),
            ("Second numerator", "1"),
            ("Second denominator", "6"),
        ],
    },
    Tool {
        kind: Kind::MultiplyFractions,
        name: "Multiply fractions",
        group: "Numbers",
        help: "Multiply the numerators and denominators, then reduce to lowest terms.",
        fields: &[
            ("First numerator", "2"),
            ("First denominator", "3"),
            ("Second numerator", "3"),
            ("Second denominator", "4"),
        ],
    },
    Tool {
        kind: Kind::Gcd,
        name: "Greatest common divisor",
        group: "Numbers",
        help: "The largest positive whole number dividing both inputs. Uses Euclid's remainder algorithm.",
        fields: &[("First integer", "48"), ("Second integer", "18")],
    },
    Tool {
        kind: Kind::Lcm,
        name: "Least common multiple",
        group: "Numbers",
        help: "The smallest nonnegative common multiple: |a*b| / gcd(a,b). If either input is zero, the LCM is zero.",
        fields: &[("First integer", "12"), ("Second integer", "18")],
    },
    Tool {
        kind: Kind::Factors,
        name: "Prime factorization",
        group: "Numbers",
        help: "Break a whole number from 2 through one billion into prime factors using trial division.",
        fields: &[("Integer to factor", "360")],
    },
    Tool {
        kind: Kind::Quadratic,
        name: "Quadratic solver",
        group: "Algebra",
        help: "Solve a*x^2 + b*x + c = 0 over the real numbers. Uses a cancellation-resistant quadratic formula. A zero a becomes a linear equation.",
        fields: &[("a", "1"), ("b", "-5"), ("c", "6")],
    },
    Tool {
        kind: Kind::Linear,
        name: "Two-equation solver",
        group: "Algebra",
        help: "Solve a*x + b*y = c and d*x + e*y = f using the determinant. A zero determinant means there is no unique solution.",
        fields: &[
            ("a", "2"),
            ("b", "1"),
            ("c", "5"),
            ("d", "1"),
            ("e", "-1"),
            ("f", "1"),
        ],
    },
    Tool {
        kind: Kind::Arithmetic,
        name: "Arithmetic sequence",
        group: "Algebra",
        help: "Find term n and the sum of the first n terms: a_n = a_1 + (n-1)*d; S_n = n*(a_1+a_n)/2.",
        fields: &[
            ("First term", "3"),
            ("Common difference", "2"),
            ("n: term number", "10"),
        ],
    },
    Tool {
        kind: Kind::Geometric,
        name: "Geometric sequence",
        group: "Algebra",
        help: "Find term n and the sum of the first n terms: a_n = a_1*r^(n-1). A ratio of 1 is handled directly.",
        fields: &[
            ("First term", "2"),
            ("Common ratio", "3"),
            ("n: term number", "5"),
        ],
    },
    Tool {
        kind: Kind::Length,
        name: "Length converter",
        group: "Conversions",
        help: "Convert a value in meters into kilometers, centimeters, millimeters, inches, feet, yards, and miles. One inch is exactly 0.0254 meters.",
        fields: &[("Meters", "1")],
    },
    Tool {
        kind: Kind::Mass,
        name: "Mass converter",
        group: "Conversions",
        help: "Convert kilograms into grams, milligrams, pounds, and ounces. One international pound is exactly 0.45359237 kilograms.",
        fields: &[("Kilograms", "1")],
    },
    Tool {
        kind: Kind::Temperature,
        name: "Temperature converter",
        group: "Conversions",
        help: "Convert Celsius into Fahrenheit and kelvin: F = 9*C/5 + 32; K = C + 273.15. Values below absolute zero are rejected.",
        fields: &[("Celsius", "20")],
    },
    Tool {
        kind: Kind::Angle,
        name: "Angle converter",
        group: "Conversions",
        help: "Convert degrees into radians, turns, and gradians. Independent of the calculator's angle setting.",
        fields: &[("Degrees", "180")],
    },
    Tool {
        kind: Kind::PercentChange,
        name: "Percentage change",
        group: "Everyday math",
        help: "Change relative to the magnitude of the original: 100*(new-old)/|old|. The original value cannot be zero.",
        fields: &[("Original value", "80"), ("New value", "100")],
    },
    Tool {
        kind: Kind::Percentage,
        name: "Percentage of a value",
        group: "Everyday math",
        help: "Find a percentage of a value, then see the value after adding or removing that amount. Useful for tax and discount exercises.",
        fields: &[("Value", "80"), ("Percentage", "15")],
    },
    Tool {
        kind: Kind::Compound,
        name: "Compound interest",
        group: "Finance",
        help: "Educational estimate: A = P*(1+r/m)^(m*t). Annual rate is a percentage; compounding periods per year must be a positive whole number. No fees or taxes.",
        fields: &[
            ("Starting balance", "1000"),
            ("Annual rate (%)", "5"),
            ("Years", "10"),
            ("Compounds per year", "12"),
        ],
    },
    Tool {
        kind: Kind::Savings,
        name: "Savings with deposits",
        group: "Finance",
        help: "Educational estimate with monthly compounding and a deposit at the END of each month. Years must correspond to whole months. No fees or taxes.",
        fields: &[
            ("Starting balance", "1000"),
            ("Monthly deposit", "100"),
            ("Annual rate (%)", "5"),
            ("Years", "10"),
        ],
    },
    Tool {
        kind: Kind::Loan,
        name: "Loan payment",
        group: "Finance",
        help: "Educational fixed-rate estimate with monthly payments and compounding. Shows payment, total repaid, and interest; excludes fees, insurance, and taxes. Whole months only.",
        fields: &[
            ("Amount borrowed", "10000"),
            ("Annual rate (%)", "6"),
            ("Years", "3"),
        ],
    },
    Tool {
        kind: Kind::Base,
        name: "Number bases",
        group: "Conversions",
        help: "Convert a nonnegative whole decimal integer (up to one billion) into binary, octal, and hexadecimal.",
        fields: &[("Decimal integer", "255")],
    },
    Tool {
        kind: Kind::Distance,
        name: "Coordinate geometry",
        group: "Geometry",
        help: "Find the distance and midpoint between two points using the distance formula and coordinate averages.",
        fields: &[("x1", "0"), ("y1", "0"), ("x2", "3"), ("y2", "4")],
    },
    Tool {
        kind: Kind::RightTriangle,
        name: "Right triangle",
        group: "Geometry",
        help: "Enter two positive perpendicular sides. Find the hypotenuse, area, perimeter, and both acute angles in degrees.",
        fields: &[("Perpendicular side a", "3"), ("Perpendicular side b", "4")],
    },
];

pub struct Toolkit {
    selected: usize,
    search: String,
    group: &'static str,
    inputs: [String; 6],
    report: Option<Result<Report, String>>,
    pub(crate) science: crate::science::ScienceWorkbench,
    pub(crate) science_open: bool,
}
impl Default for Toolkit {
    fn default() -> Self {
        let mut this = Self {
            selected: 0,
            search: String::new(),
            group: "Statistics",
            inputs: Default::default(),
            report: None,
            science: Default::default(),
            science_open: false,
        };
        this.select(0);
        this
    }
}
impl Toolkit {
    fn select(&mut self, index: usize) {
        self.selected = index;
        self.inputs = Default::default();
        for (i, (_, value)) in TOOLS[index].fields.iter().enumerate() {
            self.inputs[i] = (*value).into();
        }
        self.report = None;
    }
    pub fn statistics(&mut self) {
        self.science_open = false;
        self.select(0);
        self.group = "Statistics";
        self.search.clear();
    }
    fn run(&mut self, precision: usize) {
        self.report = Some(calculate(
            TOOLS[self.selected].kind,
            &self.inputs,
            precision,
        ));
    }
}

#[derive(Debug)]
struct Report {
    rows: Vec<(String, String)>,
    method: String,
}
impl Report {
    fn new(method: &str) -> Self {
        Self {
            rows: Vec::new(),
            method: method.into(),
        }
    }
    fn value(&mut self, name: &str, value: f64, precision: usize) -> Result<(), String> {
        if !value.is_finite() {
            return Err(
                "The result exceeds the supported numeric range. Try smaller values.".into(),
            );
        }
        self.rows
            .push((name.into(), math::format(value, precision)));
        Ok(())
    }
    fn text(&self) -> String {
        format!(
            "{}\n\n{}",
            self.rows
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n"),
            self.method
        )
    }
}
fn number(inputs: &[String; 6], i: usize) -> Result<f64, String> {
    let value = inputs[i].trim().parse::<f64>().map_err(|_| {
        format!(
            "Field {} needs a number. Use a decimal point, without units.",
            i + 1
        )
    })?;
    if !value.is_finite() {
        return Err(format!("Field {} needs a finite number.", i + 1));
    }
    Ok(value)
}
fn whole(inputs: &[String; 6], i: usize) -> Result<i64, String> {
    let v = number(inputs, i)?;
    if v.fract() != 0.0 || v.abs() > INTEGER_LIMIT {
        return Err(format!(
            "Field {} needs a whole number up to one billion in magnitude.",
            i + 1
        ));
    }
    Ok(v as i64)
}
fn list(s: &str) -> Result<Vec<f64>, String> {
    let mut result = Vec::new();
    for token in s
        .split(|c: char| c.is_whitespace() || ",;".contains(c))
        .filter(|s| !s.is_empty())
    {
        if result.len() >= MAX_ITEMS {
            return Err("Use at most 1,000 values per list.".into());
        }
        let v = token
            .parse::<f64>()
            .map_err(|_| format!("'{token}' is not a number."))?;
        if !v.is_finite() {
            return Err("Every data value must be finite.".into());
        }
        result.push(v);
    }
    if result.is_empty() {
        return Err("Enter at least one data value.".into());
    }
    Ok(result)
}
fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
fn fraction(mut a: i128, mut b: i128) -> Result<String, String> {
    if b == 0 {
        return Err("A fraction's denominator cannot be zero.".into());
    }
    let d = gcd(a, b);
    a /= d;
    b /= d;
    if b < 0 {
        a = -a;
        b = -b;
    }
    Ok(format!("{a}/{b}"))
}
fn choose(n: u32, r: u32) -> f64 {
    let r = r.min(n - r);
    (0..r).fold(1.0, |v, i| v * (n - i) as f64 / (i + 1) as f64)
}
fn binomial(n: u32, p: f64, k: u32) -> f64 {
    choose(n, k) * p.powi(k as i32) * (1.0 - p).powi((n - k) as i32)
}
fn normal_cdf(z: f64) -> f64 {
    // Abramowitz and Stegun 26.2.17, absolute error < 7.5e-8 per CDF.
    let t = 1.0 / (1.0 + 0.2316419 * z.abs());
    let tail = (-0.5 * z * z).exp() / std::f64::consts::TAU.sqrt()
        * t
        * (0.319381530
            + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
    if z < 0.0 { tail } else { 1.0 - tail }
}
fn median(v: &[f64]) -> f64 {
    let n = v.len();
    if n.is_multiple_of(2) {
        v[n / 2 - 1] / 2.0 + v[n / 2] / 2.0
    } else {
        v[n / 2]
    }
}
fn nonnegative(v: f64, label: &str) -> Result<(), String> {
    if v < 0.0 {
        Err(format!("{label} must be zero or positive."))
    } else {
        Ok(())
    }
}
fn months(years: f64) -> Result<f64, String> {
    let n = years * 12.0;
    if !(1.0..=12000.0).contains(&n) || (n - n.round()).abs() > 1e-7 {
        return Err("Use a duration from 1 to 12,000 whole months (years = months / 12).".into());
    }
    Ok(n.round())
}
fn calculate(kind: Kind, inputs: &[String; 6], precision: usize) -> Result<Report, String> {
    let tool = TOOLS
        .iter()
        .find(|t| t.kind == kind)
        .expect("registered tool");
    let mut report = Report::new(tool.help);
    let p = precision;
    let mut out = |name: &str, v: f64| report.value(name, v, p);
    match kind {
        Kind::Statistics => {
            let mut v = list(&inputs[0])?;
            v.sort_by(f64::total_cmp);
            let n = v.len();
            let sum: f64 = v.iter().sum();
            let mean = sum / n as f64;
            let ss: f64 = v.iter().map(|x| (x - mean).powi(2)).sum();
            let (q1, q3) = if n == 1 {
                (v[0], v[0])
            } else {
                (median(&v[..n / 2]), median(&v[n.div_ceil(2)..]))
            };
            for (name, value) in [
                ("Count", n as f64),
                ("Sum", sum),
                ("Mean", mean),
                ("Median", median(&v)),
                ("Minimum", v[0]),
                ("Maximum", v[n - 1]),
                ("Range", v[n - 1] - v[0]),
                ("Q1", q1),
                ("Q3", q3),
                ("Interquartile range", q3 - q1),
                ("Population variance", ss / n as f64),
                ("Population standard deviation", (ss / n as f64).sqrt()),
            ] {
                out(name, value)?;
            }
            if n > 1 {
                out("Sample variance", ss / (n - 1) as f64)?;
                out("Sample standard deviation", (ss / (n - 1) as f64).sqrt())?;
            }
            let mut counts: Vec<(f64, usize)> = Vec::new();
            for x in &v {
                if let Some(last) = counts.last_mut()
                    && last.0 == *x
                {
                    last.1 += 1;
                } else {
                    counts.push((*x, 1));
                }
            }
            let max = counts.iter().map(|(_, n)| *n).max().unwrap_or(1);
            let modes = if max == 1 {
                "No repeated value".into()
            } else {
                counts
                    .iter()
                    .filter(|(_, n)| *n == max)
                    .map(|(v, _)| math::format(*v, p))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            report.rows.push(("Mode(s)".into(), modes));
            let low = q1 - 1.5 * (q3 - q1);
            let high = q3 + 1.5 * (q3 - q1);
            let outside = v
                .iter()
                .filter(|x| **x < low || **x > high)
                .map(|x| math::format(*x, p))
                .collect::<Vec<_>>();
            report.rows.push((
                "Outliers (1.5 x IQR)".into(),
                if outside.is_empty() {
                    "None".into()
                } else {
                    outside.join(", ")
                },
            ));
            report.rows.push((
                "Sorted data".into(),
                v.iter()
                    .map(|x| math::format(*x, p))
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
        Kind::Regression => {
            let x = list(&inputs[0])?;
            let y = list(&inputs[1])?;
            if x.len() != y.len() || x.len() < 2 {
                return Err("Enter equally sized X and Y lists with at least two pairs.".into());
            }
            let n = x.len() as f64;
            let mx = x.iter().sum::<f64>() / n;
            let my = y.iter().sum::<f64>() / n;
            let xx = x.iter().map(|v| (v - mx).powi(2)).sum::<f64>();
            let yy = y.iter().map(|v| (v - my).powi(2)).sum::<f64>();
            let xy = x
                .iter()
                .zip(&y)
                .map(|(a, b)| (a - mx) * (b - my))
                .sum::<f64>();
            if xx == 0.0 {
                return Err("X values must vary to fit a line.".into());
            }
            out("Slope", xy / xx)?;
            out("Intercept", my - xy / xx * mx)?;
            if yy > 0.0 {
                out(
                    "Correlation r",
                    (xy / xx.sqrt() / yy.sqrt()).clamp(-1.0, 1.0),
                )?;
                out("R squared", (xy / xx / yy * xy).clamp(0.0, 1.0))?;
            }
            out("Data pairs", n)?;
        }
        Kind::Combination | Kind::Permutation => {
            let n = whole(inputs, 0)?;
            let r = whole(inputs, 1)?;
            if !(0..=170).contains(&n) || !(0..=n).contains(&r) {
                return Err("Use whole numbers with 0 <= r <= n <= 170.".into());
            }
            let v = if kind == Kind::Combination {
                choose(n as u32, r as u32)
            } else {
                (0..r).fold(1.0, |v, i| v * (n - i) as f64)
            };
            out("Ways", v)?;
        }
        Kind::Binomial | Kind::BinomialCdf => {
            let n = whole(inputs, 0)?;
            let prob = number(inputs, 1)?;
            let k = whole(inputs, 2)?;
            if !(0..=170).contains(&n) || !(0..=n).contains(&k) || !(0.0..=1.0).contains(&prob) {
                return Err("Use 0 <= k <= n <= 170 and a probability p from 0 to 1.".into());
            }
            let v = if kind == Kind::Binomial {
                binomial(n as u32, prob, k as u32)
            } else {
                (0..=k)
                    .map(|i| binomial(n as u32, prob, i as u32))
                    .sum::<f64>()
                    .clamp(0.0, 1.0)
            };
            out(
                if kind == Kind::Binomial {
                    "P(X = k)"
                } else {
                    "P(X <= k)"
                },
                v,
            )?;
            out("Percentage", v * 100.0)?;
        }
        Kind::Normal => {
            let a = number(inputs, 0)?;
            let b = number(inputs, 1)?;
            let mean = number(inputs, 2)?;
            let sd = number(inputs, 3)?;
            if sd <= 0.0 || a > b {
                return Err("Standard deviation must be positive and the lower bound must not exceed the upper bound.".into());
            }
            let v = (normal_cdf((b - mean) / sd) - normal_cdf((a - mean) / sd)).clamp(0.0, 1.0);
            out("Probability between bounds", v)?;
            out("Percentage", v * 100.0)?;
        }
        Kind::ZScore => {
            let sd = number(inputs, 2)?;
            if sd <= 0.0 {
                return Err("Standard deviation must be positive.".into());
            }
            out("Z-score", (number(inputs, 0)? - number(inputs, 1)?) / sd)?;
        }
        Kind::Simplify | Kind::AddFractions | Kind::MultiplyFractions => {
            let a = whole(inputs, 0)? as i128;
            let b = whole(inputs, 1)? as i128;
            let (num, den) = if kind == Kind::Simplify {
                (a, b)
            } else {
                let c = whole(inputs, 2)? as i128;
                let d = whole(inputs, 3)? as i128;
                if b == 0 || d == 0 {
                    return Err("A fraction's denominator cannot be zero.".into());
                }
                (
                    if kind == Kind::AddFractions {
                        a * d + b * c
                    } else {
                        a * c
                    },
                    b * d,
                )
            };
            let simplified = fraction(num, den)?;
            out("Decimal", num as f64 / den as f64)?;
            report
                .rows
                .insert(0, ("Reduced fraction".into(), simplified));
        }
        Kind::Gcd | Kind::Lcm => {
            let a = whole(inputs, 0)? as i128;
            let b = whole(inputs, 1)? as i128;
            let d = gcd(a, b);
            if kind == Kind::Gcd && d == 0 {
                return Err("gcd(0, 0) has no greatest positive divisor.".into());
            }
            report.rows.push((
                "Exact integer result".into(),
                if kind == Kind::Gcd {
                    d
                } else if d == 0 {
                    0
                } else {
                    (a / d * b).abs()
                }
                .to_string(),
            ));
        }
        Kind::Factors => {
            let mut n = whole(inputs, 0)?;
            if n < 2 {
                return Err("Enter a whole number from 2 through one billion.".into());
            }
            let mut factors = Vec::new();
            let mut d = 2;
            while d * d <= n {
                while n % d == 0 {
                    factors.push(d.to_string());
                    n /= d;
                }
                d += if d == 2 { 1 } else { 2 };
            }
            if n > 1 {
                factors.push(n.to_string());
            }
            report
                .rows
                .push(("Prime factors".into(), factors.join(" x ")));
        }
        Kind::Quadratic => {
            let a = number(inputs, 0)?;
            let b = number(inputs, 1)?;
            let c = number(inputs, 2)?;
            if a == 0.0 {
                if b == 0.0 {
                    return Err(if c == 0.0 {
                        "Every real x is a solution."
                    } else {
                        "This equation has no solution."
                    }
                    .into());
                }
                out("Linear solution x", -c / b)?;
            } else {
                // Scale first so ordinary large coefficients do not overflow b*b.
                let scale = a.abs().max(b.abs()).max(c.abs());
                let (a, b, c) = (a / scale, b / scale, c / scale);
                let disc = b * b - 4.0 * a * c;
                if disc < 0.0 {
                    return Err("No real roots: the discriminant is negative. Complex roots are outside this tool.".into());
                }
                if disc == 0.0 {
                    out("Repeated root", -b / (2.0 * a))?;
                } else {
                    let q = -0.5 * (b + disc.sqrt().copysign(b));
                    out("Root 1", q / a)?;
                    out("Root 2", c / q)?;
                }
            }
        }
        Kind::Linear => {
            let a = number(inputs, 0)?;
            let b = number(inputs, 1)?;
            let c = number(inputs, 2)?;
            let d = number(inputs, 3)?;
            let e = number(inputs, 4)?;
            let f = number(inputs, 5)?;
            let det = a * e - b * d;
            if det == 0.0 {
                return Err("No unique solution: the lines are parallel or coincident.".into());
            }
            out("x", (c * e - b * f) / det)?;
            out("y", (a * f - c * d) / det)?;
        }
        Kind::Arithmetic | Kind::Geometric => {
            let a = number(inputs, 0)?;
            let d = number(inputs, 1)?;
            let n = whole(inputs, 2)?;
            if !(1..=10000).contains(&n) {
                return Err("The term number must be a whole number from 1 to 10,000.".into());
            }
            let (term, sum) = if kind == Kind::Arithmetic {
                let t = a + (n - 1) as f64 * d;
                (t, n as f64 * (a + t) / 2.0)
            } else {
                let t = a * d.powi((n - 1) as i32);
                (
                    t,
                    if d == 1.0 {
                        a * n as f64
                    } else {
                        a * (1.0 - d.powi(n as i32)) / (1.0 - d)
                    },
                )
            };
            out("Nth term", term)?;
            out("Sum of first n terms", sum)?;
        }
        Kind::Length => {
            let v = number(inputs, 0)?;
            for (name, unit) in [
                ("Meters", 1.0),
                ("Kilometers", 1000.0),
                ("Centimeters", 0.01),
                ("Millimeters", 0.001),
                ("Inches", 0.0254),
                ("Feet", 0.3048),
                ("Yards", 0.9144),
                ("Miles", 1609.344),
            ] {
                out(name, v / unit)?;
            }
        }
        Kind::Mass => {
            let v = number(inputs, 0)?;
            nonnegative(v, "Mass")?;
            for (name, unit) in [
                ("Kilograms", 1.0),
                ("Grams", 0.001),
                ("Milligrams", 0.000001),
                ("Pounds", 0.45359237),
                ("Ounces", 0.028349523125),
            ] {
                out(name, v / unit)?;
            }
        }
        Kind::Temperature => {
            let c = number(inputs, 0)?;
            if c < -273.15 {
                return Err("Temperature cannot be below absolute zero (-273.15 Celsius).".into());
            }
            out("Fahrenheit", c * 1.8 + 32.0)?;
            out("Kelvin", c + 273.15)?;
        }
        Kind::Angle => {
            let d = number(inputs, 0)?;
            out("Radians", d.to_radians())?;
            out("Turns", d / 360.0)?;
            out("Gradians", d / 0.9)?;
        }
        Kind::PercentChange => {
            let a = number(inputs, 0)?;
            let b = number(inputs, 1)?;
            if a == 0.0 {
                return Err(
                    "Percentage change is undefined when the original value is zero.".into(),
                );
            }
            out("Change", b - a)?;
            out("Percentage change", (b - a) / a.abs() * 100.0)?;
        }
        Kind::Percentage => {
            let a = number(inputs, 0)?;
            let r = number(inputs, 1)? / 100.0;
            out("Percentage amount", a * r)?;
            out("After increase", a * (1.0 + r))?;
            out("After decrease", a * (1.0 - r))?;
        }
        Kind::Compound => {
            let a = number(inputs, 0)?;
            let r = number(inputs, 1)? / 100.0;
            let years = number(inputs, 2)?;
            let m = whole(inputs, 3)?;
            nonnegative(a, "Starting balance")?;
            nonnegative(r, "Rate")?;
            nonnegative(years, "Years")?;
            if !(1..=365).contains(&m) {
                return Err("Use 1 through 365 compounding periods per year.".into());
            }
            let total = a * ((r / m as f64).ln_1p() * m as f64 * years).exp();
            out("Ending balance", total)?;
            out("Interest earned", total - a)?;
        }
        Kind::Savings => {
            let a = number(inputs, 0)?;
            let deposit = number(inputs, 1)?;
            let r = number(inputs, 2)? / 1200.0;
            let n = months(number(inputs, 3)?)?;
            nonnegative(a, "Starting balance")?;
            nonnegative(deposit, "Deposit")?;
            nonnegative(r, "Rate")?;
            let growth = (n * r.ln_1p()).exp_m1();
            let total = a * (1.0 + growth) + deposit * if r == 0.0 { n } else { growth / r };
            out("Ending balance", total)?;
            out("Total contributed", a + n * deposit)?;
            out("Interest earned", total - a - n * deposit)?;
        }
        Kind::Loan => {
            let a = number(inputs, 0)?;
            let r = number(inputs, 1)? / 1200.0;
            let n = months(number(inputs, 2)?)?;
            if a <= 0.0 {
                return Err("The amount borrowed must be positive.".into());
            }
            nonnegative(r, "Rate")?;
            let payment = if r == 0.0 {
                a / n
            } else {
                a * r / (-(-n * r.ln_1p()).exp_m1())
            };
            out("Monthly payment", payment)?;
            out("Total repaid", payment * n)?;
            out("Total interest", payment * n - a)?;
        }
        Kind::Base => {
            let n = whole(inputs, 0)?;
            if n < 0 {
                return Err("Enter a nonnegative whole decimal integer.".into());
            }
            report.rows.extend([
                ("Binary".into(), format!("{n:b}")),
                ("Octal".into(), format!("{n:o}")),
                ("Hexadecimal".into(), format!("{n:X}")),
            ]);
        }
        Kind::Distance => {
            let x1 = number(inputs, 0)?;
            let y1 = number(inputs, 1)?;
            let x2 = number(inputs, 2)?;
            let y2 = number(inputs, 3)?;
            out("Distance", (x2 - x1).hypot(y2 - y1))?;
            out("Midpoint x", x1 / 2.0 + x2 / 2.0)?;
            out("Midpoint y", y1 / 2.0 + y2 / 2.0)?;
        }
        Kind::RightTriangle => {
            let a = number(inputs, 0)?;
            let b = number(inputs, 1)?;
            if a <= 0.0 || b <= 0.0 {
                return Err("Both perpendicular sides must be positive.".into());
            }
            let c = a.hypot(b);
            out("Hypotenuse", c)?;
            out("Area", a / 2.0 * b)?;
            out("Perimeter", a + b + c)?;
            out("Angle opposite a (degrees)", a.atan2(b).to_degrees())?;
            out("Angle opposite b (degrees)", b.atan2(a).to_degrees())?;
        }
    }
    Ok(report)
}

impl EqualApp {
    pub fn toolkit_workspace(&mut self, ui: &mut egui::Ui) {
        if self.toolkit.science_open {
            self.science_workspace(ui);
            return;
        }
        let palette = crate::appearance::palette(ui.ctx());
        ui.horizontal_wrapped(|ui| {
            ui.heading("Math tools");
            ui.label(R::new("30 tools").color(palette.accent));
        });
        ui.label("Choose a tool and enter its inputs.");
        if ui.button("Physics & chemistry").clicked() {
            self.toolkit.science_open = true;
            return;
        }
        if ui.button("Practice").clicked() {
            self.tab = Tab::Practice;
            return;
        }
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label("Find a tool");
            ui.add(
                egui::TextEdit::singleline(&mut self.toolkit.search)
                    .hint_text("Search tools")
                    .desired_width(225.0)
                    .char_limit(100),
            );
            if ui.small_button("Clear search").clicked() {
                self.toolkit.search.clear();
                self.toolkit.group = "All";
            }
        });
        let mut category_changed = false;
        ui.horizontal_wrapped(|ui| {
            for group in [
                "All",
                "Statistics",
                "Probability",
                "Numbers",
                "Algebra",
                "Conversions",
                "Everyday math",
                "Finance",
                "Geometry",
            ] {
                let active = self.toolkit.group == group;
                category_changed |= ui
                    .selectable_value(
                        &mut self.toolkit.group,
                        group,
                        R::new(group).color(if active {
                            palette.selected_text
                        } else {
                            palette.ink
                        }),
                    )
                    .changed();
            }
        });
        if category_changed {
            self.toolkit.search.clear();
            if let Some(index) = TOOLS
                .iter()
                .position(|t| self.toolkit.group == "All" || t.group == self.toolkit.group)
            {
                self.toolkit.select(index);
            }
        }
        let query = self.toolkit.search.to_lowercase();
        let mut selected = None;
        ui.horizontal_wrapped(|ui| {
            for (index, tool) in TOOLS.iter().enumerate().filter(|(_, t)| {
                (!query.is_empty() || self.toolkit.group == "All" || t.group == self.toolkit.group)
                    && format!("{} {} {}", t.name, t.group, t.help)
                        .to_lowercase()
                        .contains(&query)
            }) {
                if ui
                    .selectable_label(
                        self.toolkit.selected == index,
                        R::new(tool.name).color(if self.toolkit.selected == index {
                            palette.selected_text
                        } else {
                            palette.ink
                        }),
                    )
                    .clicked()
                {
                    selected = Some(index);
                }
            }
        });
        if !TOOLS.iter().any(|t| {
            (!query.is_empty() || self.toolkit.group == "All" || t.group == self.toolkit.group)
                && format!("{} {} {}", t.name, t.group, t.help)
                    .to_lowercase()
                    .contains(&query)
        }) {
            ui.label("No tools match. Clear the search or choose another category.");
        }
        if let Some(index) = selected {
            self.toolkit.select(index);
        }
        ui.add_space(8.0);
        let tool = &TOOLS[self.toolkit.selected];
        crate::ui::card(ui, |ui| {
            ui.heading(tool.name);
            ui.label(crate::appearance::paragraph(ui.ctx(), tool.help));
            ui.add_space(8.0);
            for (i, (label, _)) in tool.fields.iter().enumerate() {
                ui.label(R::new(*label).strong());
                let data_list = matches!(tool.kind, Kind::Statistics | Kind::Regression);
                let input = if data_list {
                    egui::TextEdit::multiline(&mut self.toolkit.inputs[i]).desired_rows(2)
                } else {
                    egui::TextEdit::singleline(&mut self.toolkit.inputs[i])
                }
                .desired_width(ui.available_width().min(550.0))
                .char_limit(16000)
                .id_salt(("tool-input", i));
                let response = ui.add(input);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, *label)
                });
                if response.changed() {
                    self.toolkit.report = None;
                }
                if (response.has_focus() || response.lost_focus())
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    response.request_focus();
                    self.toolkit.run(self.settings.precision);
                }
            }
            ui.horizontal_wrapped(|ui| {
                if crate::ui::primary(ui, "Calculate tool").clicked() {
                    self.toolkit.run(self.settings.precision);
                }
                if ui.button("Reset example").clicked() {
                    self.toolkit.select(self.toolkit.selected);
                }
            });
        });
        ui.add_space(8.0);
        if let Some(result) = &self.toolkit.report {
            match result {
                Err(error) => {
                    ui.label(R::new(format!("Check your inputs: {error}")).strong());
                }
                Ok(report) => {
                    let text = format!("{}\n{}", tool.name, report.text());
                    let mut read = false;
                    crate::ui::card(ui, |ui| {
                        ui.heading("Result");
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Copy result & method").clicked() {
                                ui.ctx().copy_text(text.clone());
                            }
                            if ui.button("Read tool result aloud").clicked() {
                                read = true;
                            }
                        });
                        for (label, value) in &report.rows {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(R::new(format!("{label}:")).strong());
                                ui.label(value);
                            });
                        }
                        ui.collapsing("Method", |ui| {
                            ui.label(crate::appearance::paragraph(ui.ctx(), &report.method));
                        });
                    });
                    if read {
                        self.speak(&text);
                    }
                }
            }
        }
        ui.collapsing("Research & conventions",|ui| {
            ui.label("Tools were selected around TI classroom workflows, NIST probability definitions, and accessible calculator interaction patterns. Results use your decimal-place setting; integer fractions and bases remain exact. These additions do not imply exam approval or assistive-technology certification.");
            for (label,url) in [("TI-84 Plus CE classroom features","https://education.ti.com/en/products/calculators/graphing-calculators/ti-84-plus-ce-python"),("NIST binomial distribution","https://www.itl.nist.gov/div898/handbook/eda/section3/eda366i.htm"),("NIST normal distribution","https://www.itl.nist.gov/div898/handbook/pmc/section5/pmc51.htm"),("Desmos accessibility","https://www.desmos.com/accessibility"),("CFPB amortization explanation","https://www.consumerfinance.gov/ask-cfpb/what-is-amortization-and-how-could-it-affect-my-auto-loan-en-771/")] {ui.hyperlink_to(label,url);}
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dark_toolkit_cards_and_accent_text_follow_the_palette() {
        let ctx = egui::Context::default();
        let mut app = EqualApp::with_settings(Settings::default(), None);
        app.settings.access.theme = crate::accessibility::ColorTheme::Dark;
        app.theme(&ctx);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 1000.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.toolkit_workspace(ui));
            },
        );
        let palette = crate::appearance::palette(&ctx);
        assert!(output.shapes.iter().any(
            |shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == palette.paper)
        ));
        let title = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text() == "30 tools" => Some(text),
                _ => None,
            })
            .expect("toolkit section label");
        for section in &title.galley.job.sections {
            let color = title.override_text_color.unwrap_or_else(|| {
                if section.format.color == C::PLACEHOLDER {
                    title.fallback_color
                } else {
                    section.format.color
                }
            });
            assert_eq!(color, palette.accent);
        }
    }
    fn run(kind: Kind, values: &[&str]) -> Result<Report, String> {
        let mut inputs: [String; 6] = Default::default();
        for (i, s) in values.iter().enumerate() {
            inputs[i] = (*s).into();
        }
        calculate(kind, &inputs, 8)
    }
    fn value(report: &Report, label: &str) -> f64 {
        report
            .rows
            .iter()
            .find(|(l, _)| l == label)
            .unwrap()
            .1
            .parse()
            .unwrap()
    }
    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} != {b}");
    }
    #[test]
    fn every_tool_has_a_valid_working_example() {
        for tool in TOOLS {
            let values: Vec<_> = tool.fields.iter().map(|(_, v)| *v).collect();
            assert!(run(tool.kind, &values).is_ok(), "{:?}", tool.kind);
        }
    }
    #[test]
    fn stats_include_correct_sample_population_and_odd_quartiles() {
        let r = run(Kind::Statistics, &["2,4,4,4,5,5,7,9"]).unwrap();
        near(value(&r, "Mean"), 5.0);
        near(value(&r, "Population standard deviation"), 2.0);
        near(value(&r, "Sample variance"), 32.0 / 7.0);
        let r = run(Kind::Statistics, &["1 2 3 4 5"]).unwrap();
        near(value(&r, "Q1"), 1.5);
        near(value(&r, "Q3"), 4.5);
        let r = run(Kind::Statistics, &["7"]).unwrap();
        assert!(!r.rows.iter().any(|(l, _)| l == "Sample variance"));
    }
    #[test]
    fn regression_and_probability_match_known_values() {
        let r = run(Kind::Regression, &["1,2,3", "3,5,7"]).unwrap();
        near(value(&r, "Slope"), 2.0);
        near(value(&r, "Intercept"), 1.0);
        near(value(&r, "Correlation r"), 1.0);
        near(
            value(&run(Kind::Combination, &["10", "3"]).unwrap(), "Ways"),
            120.0,
        );
        near(
            value(&run(Kind::Permutation, &["10", "3"]).unwrap(), "Ways"),
            720.0,
        );
        near(
            value(
                &run(Kind::Binomial, &["10", "0.5", "3"]).unwrap(),
                "P(X = k)",
            ),
            0.1171875,
        );
        near(
            value(
                &run(Kind::BinomialCdf, &["10", "0.5", "3"]).unwrap(),
                "P(X <= k)",
            ),
            0.171875,
        );
        near(
            value(
                &run(Kind::Normal, &["-1", "1", "0", "1"]).unwrap(),
                "Probability between bounds",
            ),
            0.68268949,
        );
    }
    #[test]
    fn fractions_and_integer_operations_are_exact() {
        assert_eq!(
            run(Kind::Simplify, &["18", "-24"]).unwrap().rows[0].1,
            "-3/4"
        );
        assert_eq!(
            run(Kind::AddFractions, &["1", "3", "1", "6"]).unwrap().rows[0].1,
            "1/2"
        );
        assert_eq!(
            run(Kind::MultiplyFractions, &["2", "3", "3", "4"])
                .unwrap()
                .rows[0]
                .1,
            "1/2"
        );
        assert_eq!(
            run(Kind::Lcm, &["999999999", "1000000000"]).unwrap().rows[0].1,
            "999999999000000000"
        );
        assert_eq!(
            run(Kind::Factors, &["360"]).unwrap().rows[0].1,
            "2 x 2 x 2 x 3 x 3 x 5"
        );
    }
    #[test]
    fn algebra_and_geometry_solve_known_problems() {
        let r = run(Kind::Quadratic, &["1", "-5", "6"]).unwrap();
        near(value(&r, "Root 1"), 3.0);
        near(value(&r, "Root 2"), 2.0);
        let r = run(Kind::Linear, &["2", "1", "5", "1", "-1", "1"]).unwrap();
        near(value(&r, "x"), 2.0);
        near(value(&r, "y"), 1.0);
        near(
            value(
                &run(Kind::Geometric, &["2", "1", "5"]).unwrap(),
                "Sum of first n terms",
            ),
            10.0,
        );
        near(
            value(
                &run(Kind::RightTriangle, &["3", "4"]).unwrap(),
                "Hypotenuse",
            ),
            5.0,
        );
    }
    #[test]
    fn conversions_and_zero_rate_finance() {
        near(
            value(&run(Kind::Temperature, &["-273.15"]).unwrap(), "Kelvin"),
            0.0,
        );
        near(
            value(&run(Kind::Length, &["0.0254"]).unwrap(), "Inches"),
            1.0,
        );
        near(
            value(&run(Kind::Angle, &["180"]).unwrap(), "Radians"),
            std::f64::consts::PI,
        );
        near(
            value(
                &run(Kind::Loan, &["1200", "0", "1"]).unwrap(),
                "Monthly payment",
            ),
            100.0,
        );
        near(
            value(
                &run(Kind::Savings, &["1000", "100", "0", "1"]).unwrap(),
                "Ending balance",
            ),
            2200.0,
        );
        near(
            value(
                &run(Kind::Compound, &["1000", "5", "2", "1"]).unwrap(),
                "Ending balance",
            ),
            1102.5,
        );
        near(
            value(
                &run(Kind::Loan, &["10000", "6", "3"]).unwrap(),
                "Monthly payment",
            ),
            304.21937452,
        );
        assert_eq!(run(Kind::Base, &["255"]).unwrap().rows[0].1, "11111111");
    }
    #[test]
    fn invalid_and_degenerate_inputs_are_rejected() {
        for (kind, values) in [
            (Kind::Statistics, vec![""]),
            (Kind::Statistics, vec!["NaN"]),
            (Kind::Statistics, vec!["1,typo"]),
            (Kind::Regression, vec!["1,1", "2,3"]),
            (Kind::Combination, vec!["3", "4"]),
            (Kind::Binomial, vec!["3", "1.2", "2"]),
            (Kind::Normal, vec!["2", "1", "0", "1"]),
            (Kind::ZScore, vec!["1", "0", "0"]),
            (Kind::Simplify, vec!["1", "0"]),
            (Kind::Gcd, vec!["0", "0"]),
            (Kind::Factors, vec!["1"]),
            (Kind::Quadratic, vec!["1", "0", "1"]),
            (Kind::Linear, vec!["1", "1", "1", "2", "2", "3"]),
            (Kind::Temperature, vec!["-274"]),
            (Kind::PercentChange, vec!["0", "10"]),
            (Kind::Loan, vec!["10", "-1", "1"]),
            (Kind::Loan, vec!["10", "1", "0"]),
            (Kind::Base, vec!["2.5"]),
            (Kind::RightTriangle, vec!["-3", "4"]),
        ] {
            assert!(run(kind, &values).is_err(), "{kind:?}");
        }
    }
    #[test]
    fn bounded_inputs_and_overflow_return_errors() {
        let huge = vec!["1"; MAX_ITEMS + 1].join(",");
        assert!(run(Kind::Statistics, &[&huge]).is_err());
        assert!(run(Kind::Compound, &["1e308", "100", "1000", "12"]).is_err());
        assert!(run(Kind::Loan, &["100", "1", "0.1"]).is_err());
    }
    #[test]
    fn binomial_extremes_and_stable_quadratic() {
        near(
            value(
                &run(Kind::Binomial, &["170", "1", "170"]).unwrap(),
                "P(X = k)",
            ),
            1.0,
        );
        near(
            value(
                &run(Kind::BinomialCdf, &["170", "0", "170"]).unwrap(),
                "P(X <= k)",
            ),
            1.0,
        );
        let r = run(Kind::Quadratic, &["1e200", "-5e200", "6e200"]).unwrap();
        near(value(&r, "Root 1"), 3.0);
        near(value(&r, "Root 2"), 2.0);
    }
}
