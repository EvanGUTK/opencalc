use super::{Answer, Difficulty, Question, TopicId, Visual, WorkedStep, pick};
use crate::math;

const EXACT_TOLERANCE: f64 = 1.0e-8;
const THREE_DECIMAL_TOLERANCE: f64 = 0.0005;

fn text(value: f64) -> String {
    math::format(value, 8)
}

fn step(title: &str, calculation: impl Into<String>, explanation: &str) -> WorkedStep {
    WorkedStep {
        title: title.into(),
        calculation: calculation.into(),
        explanation: explanation.into(),
    }
}

fn number(
    prompt: String,
    value: f64,
    hint: &str,
    steps: Vec<WorkedStep>,
    check: String,
) -> Question {
    Question {
        prompt,
        answer: Answer::Number {
            value,
            tolerance: EXACT_TOLERANCE,
        },
        hint: hint.into(),
        steps,
        visual: Visual::None,
        calculator: None,
        check,
    }
}

fn values(
    prompt: String,
    results: Vec<f64>,
    ordered: bool,
    hint: &str,
    steps: Vec<WorkedStep>,
    check: String,
) -> Question {
    Question {
        prompt,
        answer: Answer::Values {
            values: results,
            tolerance: EXACT_TOLERANCE,
            ordered,
        },
        hint: hint.into(),
        steps,
        visual: Visual::None,
        calculator: None,
        check,
    }
}

pub(super) fn generate(topic: TopicId, difficulty: Difficulty, index: u32) -> Option<Question> {
    Some(match topic {
        TopicId::LinearEquations => linear(difficulty, index),
        TopicId::Quadratics => quadratic(difficulty, index),
        TopicId::Geometry => geometry(difficulty, index),
        TopicId::Coordinates => coordinates(difficulty, index),
        TopicId::Trigonometry => trigonometry(difficulty, index),
        TopicId::FunctionValues => functions(difficulty, index),
        TopicId::Derivatives => derivative(difficulty, index),
        TopicId::Integrals => integral(difficulty, index),
        TopicId::Limits => limit(difficulty, index),
        TopicId::Matrices => matrices(difficulty, index),
        TopicId::Determinants => determinant(difficulty, index),
        TopicId::LinearSystems => systems(difficulty, index),
        _ => return None,
    })
}

fn linear(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let x = pick(index, 11, if rank == 0 { 1 } else { -9 }, 9);
    let a = pick(index, 12, 2, 5 + rank);
    let b = pick(index, 13, 1, 8 + rank * 3);
    let (left, right, coefficient, constant) = match (difficulty, index % 3) {
        (Difficulty::Basic, 0) => (format!("{a}*x+{b}"), (a * x + b).to_string(), a, b),
        (Difficulty::Basic, 1) => (format!("x-{b}"), (x - b).to_string(), 1, -b),
        (Difficulty::Basic, _) => (format!("{a}*(x+{b})"), (a * (x + b)).to_string(), a, a * b),
        (Difficulty::Standard, 0) => {
            let c = a - 1;
            let d = (a - c) * x + b;
            (format!("{a}*x+{b}"), format!("{c}*x+({d})"), a - c, b - d)
        }
        (Difficulty::Standard, 1) => (
            format!("(x+{b})/{a}"),
            text(f64::from(x + b) / f64::from(a)),
            1,
            b,
        ),
        (Difficulty::Standard, _) => (format!("{a}*(x-{b})"), (a * (x - b)).to_string(), a, -a * b),
        (Difficulty::Challenge, 0) => {
            let c = a + 2;
            let d = (a - c) * x + a * b;
            (
                format!("{a}*(x+{b})"),
                format!("{c}*x+({d})"),
                a - c,
                a * b - d,
            )
        }
        (Difficulty::Challenge, 1) => (
            format!("({a}*x+{b})/3"),
            text(f64::from(a * x + b) / 3.0),
            a,
            b,
        ),
        (Difficulty::Challenge, _) => {
            let c = a - 1;
            let d = (a - c) * x - a * b;
            (
                format!("{a}*(x-{b})"),
                format!("{c}*x+({d})"),
                a - c,
                -a * b - d,
            )
        }
    };
    // Fractional right sides are written exactly to keep the equation unchanged.
    let right = if difficulty == Difficulty::Standard && index % 3 == 1 {
        format!("{}/{a}", x + b)
    } else if difficulty == Difficulty::Challenge && index % 3 == 1 {
        format!("{}/3", a * x + b)
    } else {
        right
    };
    let (isolation, result) = if (difficulty == Difficulty::Standard
        || difficulty == Difficulty::Challenge)
        && index % 3 == 1
    {
        let denominator = if difficulty == Difficulty::Standard {
            a
        } else {
            3
        };
        (
            format!("Multiply both sides by {denominator}; subtract {b}."),
            format!("x = {x}"),
        )
    } else if right.contains('x') {
        (
            format!("{coefficient}*x + ({constant}) = 0"),
            format!("x = -({constant})/({coefficient}) = {x}"),
        )
    } else {
        (
            format!("{coefficient}*x = ({right}) - ({constant})"),
            format!("x = {x}"),
        )
    };
    let mut question = number(
        format!("Solve {left} = {right}. Find x."),
        f64::from(x),
        "Do the same operation to both sides. Remove parentheses or fractions before collecting x terms.",
        vec![
            step(
                "Keep both sides balanced",
                format!("{left} = {right}"),
                "Expand parentheses or multiply by a denominator when needed.",
            ),
            step(
                "Isolate the variable term",
                isolation,
                "Move constant terms and variable terms using equal operations on both sides.",
            ),
            step(
                "Divide by the coefficient",
                result,
                "The value left for x must satisfy the original equation.",
            ),
        ],
        format!("Substitute x={x} into both sides of {left} = {right}; the two values agree."),
    );
    question.calculator = Some(format!("root(({left})-({right}),0)"));
    question
}

fn quadratic(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let divisor = if difficulty == Difficulty::Challenge {
        2.0
    } else {
        1.0
    };
    let first = f64::from(pick(index, 21, if rank == 0 { 1 } else { -8 }, 8)) / divisor;
    let mut second = f64::from(pick(index, 22, if rank == 0 { 1 } else { -8 }, 8)) / divisor;
    if first == second {
        second += 1.0 / divisor;
    }
    let a = if difficulty == Difficulty::Challenge {
        4.0
    } else if difficulty == Difficulty::Standard {
        f64::from(pick(index, 23, 1, 3))
    } else {
        1.0
    };
    let rhs = if difficulty == Difficulty::Challenge {
        f64::from(pick(index, 24, -9, 9))
    } else {
        0.0
    };
    let b = -a * (first + second);
    let c = a * first * second;
    let left = if difficulty == Difficulty::Basic && index.is_multiple_of(3) {
        format!("(x-({}))*(x-({}))", text(first), text(second))
    } else {
        format!("{}*x^2+({})*x+({})", text(a), text(b), text(c + rhs))
    };
    values(
        format!(
            "Find all real roots of {left} = {}. Enter both roots separated by a comma, in either order.",
            text(rhs)
        ),
        vec![first, second],
        false,
        "Write the equation with zero on one side. A product is zero when at least one factor is zero.",
        vec![
            step(
                "Move everything to one side",
                format!("{}*x^2+({})*x+({}) = 0", text(a), text(b), text(c)),
                "Subtract the right-hand side from both sides first.",
            ),
            step(
                "Factor the quadratic",
                format!(
                    "{}*(x-({}))*(x-({})) = 0",
                    text(a),
                    text(first),
                    text(second)
                ),
                "Expanding these factors reproduces the coefficients.",
            ),
            step(
                "Use the zero-product rule",
                format!("x = {} or x = {}", text(first), text(second)),
                "Each root makes one factor zero.",
            ),
        ],
        format!(
            "Substitute each root into the original equation. Both produce the right-hand value {}.",
            text(rhs)
        ),
    )
}

fn geometry(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let a = f64::from(pick(index, 31, 2, 8 + rank * 3));
    let b = f64::from(pick(index, 32, 2, 9 + rank * 3));
    if difficulty == Difficulty::Basic {
        let (prompt, expression, answer, formula) = match index % 3 {
            0 => (
                format!(
                    "A rectangle is {} cm long and {} cm wide. Find its area in square centimeters.",
                    text(a),
                    text(b)
                ),
                format!("{a}*{b}"),
                a * b,
                "Area = length * width",
            ),
            1 => (
                format!(
                    "A triangle has base {} cm and perpendicular height {} cm. Find its area in square centimeters.",
                    text(a),
                    text(b)
                ),
                format!("{a}*{b}/2"),
                a * b / 2.0,
                "Area = base * height / 2",
            ),
            _ => (
                format!(
                    "A rectangle is {} cm long and {} cm wide. Find its perimeter in centimeters.",
                    text(a),
                    text(b)
                ),
                format!("2*({a}+{b})"),
                2.0 * (a + b),
                "Perimeter = 2 * (length + width)",
            ),
        };
        let mut q = number(
            prompt,
            answer,
            "Area measures the inside; perimeter measures the boundary. Choose the matching formula.",
            vec![
                step(
                    "Choose the formula",
                    formula,
                    "Check whether the question asks for area or perimeter.",
                ),
                step(
                    "Substitute and calculate",
                    format!("{expression} = {}", text(answer)),
                    "Area uses square units; perimeter uses length units.",
                ),
            ],
            format!(
                "Check the units and count the rectangle's equal sides or half of a base-height rectangle. Result: {}.",
                text(answer)
            ),
        );
        q.calculator = Some(expression);
        if index % 3 == 1 {
            q.visual = Visual::Triangle {
                labels: [
                    format!("base = {} cm", text(a)),
                    format!("height = {} cm", text(b)),
                    String::new(),
                ],
            };
        }
        return q;
    }
    if difficulty == Difficulty::Standard {
        match index % 3 {
            0 => {
                let triple = [[3.0_f64, 4.0, 5.0], [5.0, 12.0, 13.0], [8.0, 15.0, 17.0]]
                    [(index as usize / 3) % 3];
                let scale = f64::from(pick(index, 33, 1, 3));
                let (a, b, c) = (triple[0] * scale, triple[1] * scale, triple[2] * scale);
                let mut q = number(
                    format!(
                        "A right triangle has perpendicular sides {} cm and {} cm. Find the hypotenuse in centimeters.",
                        text(a),
                        text(b)
                    ),
                    c,
                    "The hypotenuse is opposite the right angle. Use a^2+b^2=c^2.",
                    vec![
                        step(
                            "Apply Pythagoras",
                            format!("c^2 = {a}^2+{b}^2"),
                            "The squared hypotenuse equals the sum of the squared legs.",
                        ),
                        step(
                            "Take the positive square root",
                            format!("c = sqrt({}) = {}", text(a * a + b * b), text(c)),
                            "Lengths are positive.",
                        ),
                    ],
                    format!(
                        "{}^2+{}^2 = {}^2 = {}.",
                        text(a),
                        text(b),
                        text(c),
                        text(c * c)
                    ),
                );
                q.visual = Visual::Triangle {
                    labels: [
                        format!("a={}", text(a)),
                        format!("b={}", text(b)),
                        "c=?".into(),
                    ],
                };
                q.calculator = Some(format!("sqrt({a}^2+{b}^2)"));
                return q;
            }
            1 => {
                let mut q = number(
                    format!(
                        "A circle has radius {} cm. Find its circumference in centimeters. You may use pi in your answer.",
                        text(a)
                    ),
                    2.0 * std::f64::consts::PI * a,
                    "Circumference is 2*pi times the radius.",
                    vec![
                        step(
                            "Use the radius formula",
                            "C = 2*pi*r",
                            "The radius is half the diameter.",
                        ),
                        step(
                            "Substitute",
                            format!(
                                "C = 2*pi*{} = {}",
                                text(a),
                                text(2.0 * std::f64::consts::PI * a)
                            ),
                            "Keep pi in the expression to avoid early rounding.",
                        ),
                    ],
                    format!(
                        "The diameter is {} cm, and circumference divided by diameter is pi.",
                        text(2.0 * a)
                    ),
                );
                q.calculator = Some(format!("2*pi*{a}"));
                return q;
            }
            _ => {
                let answer = (a + b) * 3.0 / 2.0;
                let mut q = number(
                    format!(
                        "A trapezoid has parallel sides {} cm and {} cm and perpendicular height 3 cm. Find its area in square centimeters.",
                        text(a),
                        text(b)
                    ),
                    answer,
                    "Average the two parallel side lengths, then multiply by height.",
                    vec![
                        step(
                            "Average the parallel sides",
                            format!("({a}+{b})/2 = {}", text((a + b) / 2.0)),
                            "The mean base length represents an equal-area rectangle.",
                        ),
                        step(
                            "Multiply by height",
                            format!("{}*3 = {}", text((a + b) / 2.0), text(answer)),
                            "Height is perpendicular to the parallel sides.",
                        ),
                    ],
                    format!(
                        "Two equal trapezoids form a parallelogram with base {} and height 3, so half its area is {}.",
                        text(a + b),
                        text(answer)
                    ),
                );
                q.calculator = Some(format!("({a}+{b})*3/2"));
                return q;
            }
        }
    }
    let (prompt, expression, answer, formula) = match index % 3 {
        0 => {
            let width = a + 4.0;
            let height = b + 4.0;
            (
                format!(
                    "A {} by {} cm rectangle has a right triangle of base {} cm and height {} cm removed. Find the remaining area in square centimeters.",
                    text(width),
                    text(height),
                    text(a),
                    text(b)
                ),
                format!("{width}*{height}-{a}*{b}/2"),
                width * height - a * b / 2.0,
                "Remaining area = rectangle area - triangle area",
            )
        }
        1 => (
            format!(
                "A circle's circumference is {}*pi cm. Find its radius in centimeters.",
                text(2.0 * a)
            ),
            format!("({}*pi)/(2*pi)", text(2.0 * a)),
            a,
            "Radius = circumference / (2*pi)",
        ),
        _ => {
            let area = a * b;
            (
                format!(
                    "A triangle has area {} square centimeters and base {} cm. Find its perpendicular height in centimeters.",
                    text(area),
                    text(a)
                ),
                format!("2*{area}/{a}"),
                2.0 * b,
                "Height = 2*area / base",
            )
        }
    };
    let mut q = number(
        prompt,
        answer,
        "Write the relevant area or circumference equation and isolate the quantity requested.",
        vec![
            step(
                "Rearrange the formula",
                formula,
                "Use the quantity named in the question, not a different measurement.",
            ),
            step(
                "Calculate",
                format!("{expression} = {}", text(answer)),
                "Keep exact values until the final step.",
            ),
        ],
        format!(
            "Substitute {} into the original geometry formula to reproduce the supplied measurement.",
            text(answer)
        ),
    );
    q.calculator = Some(expression);
    q
}

fn coordinates(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let x1 = f64::from(pick(index, 41, if rank == 0 { 0 } else { -9 }, 9));
    let y1 = f64::from(pick(index, 42, if rank == 0 { 0 } else { -9 }, 9));
    let dx = f64::from(pick(index, 43, 1, 5 + rank));
    let dy = f64::from(pick(index, 44, 1, 7 + rank));
    let (x2, y2) = (x1 + dx, y1 + dy);
    let (prompt, value, expression, steps) = match index % 3 {
        0 => (
            format!(
                "Find the slope of the line through A=({}, {}) and B=({}, {}). Fractions are accepted.",
                text(x1),
                text(y1),
                text(x2),
                text(y2)
            ),
            dy / dx,
            format!("({y2}-({y1}))/({x2}-({x1}))"),
            vec![
                step(
                    "Find rise and run",
                    format!("rise={dy}, run={dx}"),
                    "Use the same point order in both subtractions.",
                ),
                step(
                    "Divide rise by run",
                    format!("m={dy}/{dx}={}", text(dy / dx)),
                    "The slope is change in y per unit change in x.",
                ),
            ],
        ),
        1 if difficulty == Difficulty::Challenge => {
            let slope = dy / dx;
            let intercept = y1 - slope * x1;
            (
                format!(
                    "A line has slope {}/{} and passes through ({}, {}). Find its y-intercept.",
                    text(dy),
                    text(dx),
                    text(x1),
                    text(y1)
                ),
                intercept,
                format!("{y1}-({dy}/{dx})*({x1})"),
                vec![
                    step(
                        "Use slope-intercept form",
                        "y=m*x+b",
                        "The y-intercept is the value of b.",
                    ),
                    step(
                        "Solve for b",
                        format!("b={y1}-({dy}/{dx})*({x1})={}", text(intercept)),
                        "Substitute the known point, then subtract m*x.",
                    ),
                ],
            )
        }
        1 => {
            let value = if difficulty == Difficulty::Basic {
                (x1 + x2) / 2.0
            } else {
                (y1 + y2) / 2.0
            };
            let axis = if difficulty == Difficulty::Basic {
                "x"
            } else {
                "y"
            };
            let (first, second) = if axis == "x" { (x1, x2) } else { (y1, y2) };
            (
                format!(
                    "A=({}, {}) and B=({}, {}). Find the {axis}-coordinate of their midpoint.",
                    text(x1),
                    text(y1),
                    text(x2),
                    text(y2)
                ),
                value,
                format!("({first}+({second}))/2"),
                vec![
                    step(
                        "Average matching coordinates",
                        format!("{axis}_mid=({first}+({second}))/2"),
                        "Average x coordinates with x, and y coordinates with y.",
                    ),
                    step(
                        "Calculate the midpoint coordinate",
                        format!("{axis}_mid={}", text(value)),
                        "The midpoint is equally far between the endpoint coordinates.",
                    ),
                ],
            )
        }
        _ => {
            let scale = f64::from(pick(index, 45, 1, 3 + rank));
            let (bx, by) = (x1 + 3.0 * scale, y1 + 4.0 * scale);
            let distance = 5.0 * scale;
            let mut q = number(
                format!(
                    "Find the distance between A=({}, {}) and B=({}, {}).",
                    text(x1),
                    text(y1),
                    text(bx),
                    text(by)
                ),
                distance,
                "Treat the horizontal and vertical changes as the legs of a right triangle.",
                vec![
                    step(
                        "Find the coordinate changes",
                        format!("dx={}, dy={}", text(3.0 * scale), text(4.0 * scale)),
                        "Subtract matching coordinates.",
                    ),
                    step(
                        "Use the distance formula",
                        format!(
                            "d=sqrt({}^2+{}^2)={}",
                            text(3.0 * scale),
                            text(4.0 * scale),
                            text(distance)
                        ),
                        "This is Pythagoras applied to coordinate differences.",
                    ),
                ],
                format!(
                    "Squaring the distance gives {}, equal to dx^2+dy^2.",
                    text(distance * distance)
                ),
            );
            q.visual = Visual::CoordinatePlot {
                points: vec![(x1, y1), (bx, by)],
                line: true,
            };
            q.calculator = Some(format!(
                "sqrt(({}-({x1}))^2+({}-({y1}))^2)",
                text(bx),
                text(by)
            ));
            return q;
        }
    };
    let mut q = number(
        prompt,
        value,
        "Use matching x and y coordinates and keep fractions exact.",
        steps,
        format!(
            "The result is {}. Reverse the point order for slope, or check equal midpoint differences, to verify the same result.",
            text(value)
        ),
    );
    q.visual = Visual::CoordinatePlot {
        points: vec![(x1, y1), (x2, y2)],
        line: index.is_multiple_of(3),
    };
    if difficulty == Difficulty::Challenge && index % 3 == 1 {
        q.check = format!(
            "Substitute the known point into y=({dy}/{dx})*x+({}); it reproduces y={} when x={}.",
            text(value),
            text(y1),
            text(x1)
        );
    }
    q.calculator = Some(expression);
    q
}

fn trigonometry(difficulty: Difficulty, index: u32) -> Question {
    let angle = f64::from([30, 45, 60][(index as usize / 3) % 3]);
    let radians = angle.to_radians();
    let side = f64::from(pick(index, 51, 3, 15));
    let (prompt,value,hint,formula,explanation,visual)=match (difficulty,index%3) {
        (Difficulty::Basic,0)=>(format!("Find sin({} degrees). Round to 3 decimal places.",text(angle)),radians.sin(),"Sine is opposite divided by hypotenuse.",format!("sin({} degrees)={}",text(angle),text(radians.sin())),"The degree symbol fixes the angle unit; use Degrees mode if checking on a calculator.",Visual::None),
        (Difficulty::Basic,1)=>(format!("Find cos({} degrees). Round to 3 decimal places.",text(angle)),radians.cos(),"Cosine is adjacent divided by hypotenuse.",format!("cos({} degrees)={}",text(angle),text(radians.cos())),"Cosine is the horizontal coordinate on a unit circle for this angle.",Visual::None),
        (Difficulty::Basic,_)=>("A right triangle has opposite side 3 and hypotenuse 5 relative to angle theta. Find sin(theta). Round to 3 decimal places.".into(),0.6,"Sine uses opposite / hypotenuse.","sin(theta)=3/5=0.6".into(),"A ratio of lengths has no angle unit.",Visual::Triangle {labels:["adjacent=4".into(),"opposite=3".into(),"hypotenuse=5".into()]}),
        (Difficulty::Standard,0)=>(format!("A right triangle has hypotenuse {} cm and angle {} degrees. Find the opposite side in centimeters. Round to 3 decimal places.",text(side),text(angle)),side*radians.sin(),"Rearrange sin(theta)=opposite/hypotenuse.",format!("opposite={}*sin({} degrees)={}",text(side),text(angle),text(side*radians.sin())),"Multiply the hypotenuse by the sine of the angle.",Visual::Triangle {labels:["adjacent".into(),"opposite=?".into(),format!("hypotenuse={}",text(side))]}),
        (Difficulty::Standard,1)=>{let opposite=f64::from(pick(index,52,1,12)); let value=(opposite/side).atan().to_degrees(); (format!("A right triangle has opposite side {} cm and adjacent side {} cm. Find theta in degrees, with 0 < theta < 90 degrees. Round to 3 decimal places.",text(opposite),text(side)),value,"Tangent is opposite/adjacent; inverse tangent finds the angle.",format!("theta=atan({}/{})={} degrees",text(opposite),text(side),text(value)),"Inverse tangent returns the acute angle for two positive legs.",Visual::Triangle {labels:[format!("adjacent={}",text(side)),format!("opposite={}",text(opposite)),"hypotenuse".into()]})},
        (Difficulty::Standard,_)=>{let denominator=[6,4,3][(index as usize/3)%3]; let value=(std::f64::consts::PI/f64::from(denominator)).cos(); (format!("Find cos(pi/{denominator}) with the angle in radians. Round to 3 decimal places."),value,"Convert pi radians to 180 degrees, or use Radians mode.",format!("cos(pi/{denominator})={}",text(value)),"The explicit radian unit overrides any unrelated calculator setting.",Visual::None)},
        (Difficulty::Challenge,0)=>{let (numerator,denominator)=[(5,6),(7,4),(4,3)][(index as usize/3)%3]; let value=(f64::from(numerator)*std::f64::consts::PI/f64::from(denominator)).sin(); (format!("Find sin({numerator}*pi/{denominator}) with the angle in radians. Round to 3 decimal places."),value,"Use the reference angle and the sign of sine in the correct quadrant.",format!("sin({numerator}*pi/{denominator})={}",text(value)),"Sine is positive above and negative below the horizontal axis on the unit circle.",Visual::None)},
        (Difficulty::Challenge,1)=>(format!("A point is {} meters horizontally from a tower. The angle of elevation is {} degrees. Find the tower height in meters, ignoring observer height. Round to 3 decimal places.",text(side),text(angle)),side*radians.tan(),"The height is opposite and the horizontal distance is adjacent.",format!("height={}*tan({} degrees)={}",text(side),text(angle),text(side*radians.tan())),"Tangent compares opposite height to adjacent horizontal distance.",Visual::Triangle {labels:[format!("distance={}",text(side)),"height=?".into(),"hypotenuse".into()]}),
        (Difficulty::Challenge,_)=>{let ratio=f64::from(pick(index,53,1,9))/10.0; let value=ratio.asin().to_degrees(); (format!("Solve sin(theta)={} for 0 < theta < 90 degrees. Give theta in degrees, rounded to 3 decimal places.",text(ratio)),value,"The stated interval selects one acute inverse-sine answer.",format!("theta=asin({})={} degrees",text(ratio),text(value)),"Sine increases from 0 to 1 on the specified interval, so this solution is unique.",Visual::None)},
    };
    let mut q = number(
        prompt,
        value,
        hint,
        vec![
            step("Choose the trig relationship", formula, explanation),
            step(
                "Round only at the end",
                format!("Answer = {value:.3}"),
                "Keep the full intermediate value, then round to the nearest thousandth.",
            ),
        ],
        format!(
            "Substituting the unrounded value into the stated ratio reproduces the given angle or side. Accepted rounding is within 0.0005 of {}.",
            text(value)
        ),
    );
    q.answer = Answer::Number {
        value,
        tolerance: THREE_DECIMAL_TOLERANCE,
    };
    q.visual = visual;
    q
}

fn functions(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let a = f64::from(pick(index, 61, 1, 5));
    let b = f64::from(pick(index, 62, -5, 5));
    let x = f64::from(pick(index, 63, if rank == 0 { 0 } else { -4 }, 5));
    let power = rank + 1;
    let evaluate = |at: f64| a * at.powi(power) + b;
    let expression = format!("{a}*x^{power}+({b})");
    let mut q = match index % 3 {
        0 => number(
            format!("For f(x)={expression}, find f({}).", text(x)),
            evaluate(x),
            "Replace every x by the supplied input, then follow order of operations.",
            vec![
                step(
                    "Substitute the input",
                    format!("f({})={a}*({x})^{power}+({b})", text(x)),
                    "Parentheses keep a negative input together before exponentiation.",
                ),
                step(
                    "Evaluate",
                    format!("f({})={}", text(x), text(evaluate(x))),
                    "Do powers, multiplication, then addition.",
                ),
            ],
            format!(
                "The point ({}, {}) is on the function.",
                text(x),
                text(evaluate(x))
            ),
        ),
        1 => {
            let h = f64::from(pick(index, 64, 1, 4));
            let change = evaluate(x + h) - evaluate(x);
            number(
                format!(
                    "For f(x)={expression}, find the change f({})-f({}).",
                    text(x + h),
                    text(x)
                ),
                change,
                "Calculate both output values, then subtract final minus initial.",
                vec![
                    step(
                        "Evaluate both endpoints",
                        format!(
                            "f({})={}, f({})={}",
                            text(x + h),
                            text(evaluate(x + h)),
                            text(x),
                            text(evaluate(x))
                        ),
                        "A change in output is not the same as the final output alone.",
                    ),
                    step(
                        "Subtract",
                        format!(
                            "{}-({})={}",
                            text(evaluate(x + h)),
                            text(evaluate(x)),
                            text(change)
                        ),
                        "Use the subtraction order requested.",
                    ),
                ],
                format!(
                    "Adding the change {} to the initial output {} gives the final output {}.",
                    text(change),
                    text(evaluate(x)),
                    text(evaluate(x + h))
                ),
            )
        }
        _ => {
            let c = f64::from(pick(index, 65, 1, 4));
            let d = f64::from(pick(index, 66, -3, 5));
            let inside = c * x + d;
            let value = evaluate(inside);
            number(
                format!(
                    "Let f(x)={expression} and g(x)={c}*x+({d}). Find f(g({})).",
                    text(x)
                ),
                value,
                "Composition means evaluate the inside function first.",
                vec![
                    step(
                        "Evaluate the inside function",
                        format!("g({})={c}*({x})+({d})={}", text(x), text(inside)),
                        "The output of g becomes the input of f.",
                    ),
                    step(
                        "Evaluate the outside function",
                        format!(
                            "f({})={a}*({inside})^{power}+({b})={}",
                            text(inside),
                            text(value)
                        ),
                        "Composition is not multiplication of f and g.",
                    ),
                ],
                format!(
                    "Substituting g(x) into f gives {a}*({c}*x+({d}))^{power}+({b}); at x={} it gives {}.",
                    text(x),
                    text(value)
                ),
            )
        }
    };
    q.calculator = Some(match index % 3 {
        0 => format!("{a}*({x})^{power}+({b})"),
        1 => {
            let h = f64::from(pick(index, 64, 1, 4));
            format!(
                "({a}*({})^{power}+({b}))-({a}*({x})^{power}+({b}))",
                text(x + h)
            )
        }
        _ => {
            let c = f64::from(pick(index, 65, 1, 4));
            let d = f64::from(pick(index, 66, -3, 5));
            format!("{a}*({c}*({x})+({d}))^{power}+({b})")
        }
    });
    q
}

fn derivative(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let a = f64::from(pick(index, 71, 1, 5));
    let b = f64::from(pick(index, 72, -6, 6));
    let at = f64::from(pick(index, 73, if rank == 0 { 1 } else { -3 }, 4));
    let power = pick(index, 74, 2 + rank, 3 + rank);
    let (expression, value, rule) = if difficulty == Difficulty::Challenge && index % 3 == 2 {
        let c = f64::from(pick(index, 75, -5, 5));
        (
            format!("(x+({a}))*(x+({c}))"),
            2.0 * at + a + c,
            format!("f'(x)=2*x+({})", text(a + c)),
        )
    } else if index % 3 == 1 {
        (
            format!("{a}*x^{power}+({b})*x"),
            a * f64::from(power) * at.powi(power - 1) + b,
            format!("f'(x)={}*x^{}+({b})", text(a * f64::from(power)), power - 1),
        )
    } else {
        (
            format!("{a}*x^{power}+({b})"),
            a * f64::from(power) * at.powi(power - 1),
            format!("f'(x)={}*x^{}", text(a * f64::from(power)), power - 1),
        )
    };
    let prompt = if index % 3 == 2 {
        format!(
            "For f(x)={expression}, find the slope of the tangent at x={}.",
            text(at)
        )
    } else {
        format!(
            "Find the derivative of f(x)={expression} at x={}.",
            text(at)
        )
    };
    let mut q = number(
        prompt,
        value,
        "Use the power rule: d(x^n)/dx=n*x^(n-1). Constants have derivative zero.",
        vec![
            step(
                "Differentiate symbolically",
                rule,
                "Differentiate each term; expand a product first if that is simpler.",
            ),
            step(
                "Substitute the point",
                format!("f'({})={}", text(at), text(value)),
                "The derivative's value at the point is the tangent slope.",
            ),
        ],
        format!(
            "A symmetric difference [f({}+h)-f({}-h)]/(2*h), with small nonzero h, approaches {}.",
            text(at),
            text(at),
            text(value)
        ),
    );
    q.calculator = Some(format!("diff({expression},{at})"));
    q
}

fn integral(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let a = f64::from(pick(index, 81, 1, 5));
    let b = if rank == 0 {
        0.0
    } else {
        f64::from(pick(index, 82, -4, 4))
    };
    let c = if index % 3 == 1 {
        f64::from(pick(index, 83, 1, 5))
    } else {
        0.0
    };
    let power = if index % 3 == 2 && difficulty == Difficulty::Basic {
        0
    } else {
        1 + rank
    };
    let mut lo = if rank == 0 {
        0.0
    } else {
        f64::from(pick(index, 84, -3, 1))
    };
    let mut hi = f64::from(pick(index, 85, 2, 5));
    if difficulty == Difficulty::Challenge && index % 3 == 2 {
        std::mem::swap(&mut lo, &mut hi);
    }
    let primitive = |x: f64| a * x.powi(power + 1) / f64::from(power + 1) + b * x * x / 2.0 + c * x;
    let value = primitive(hi) - primitive(lo);
    let expression = format!("{a}*x^{power}+({b})*x+({c})");
    let mut q = number(
        format!(
            "Find the signed definite integral of {expression} from x={} to x={}. Fractions are accepted.",
            text(lo),
            text(hi)
        ),
        value,
        "Find an antiderivative, then subtract its lower-bound value from its upper-bound value.",
        vec![
            step(
                "Integrate each term",
                format!(
                    "F(x)=({a}/{})*x^{}+({b}/2)*x^2+({c})*x",
                    power + 1,
                    power + 1
                ),
                "Increase a power by one and divide by the new power.",
            ),
            step(
                "Evaluate both bounds",
                format!(
                    "F({})={}, F({})={}",
                    text(hi),
                    text(primitive(hi)),
                    text(lo),
                    text(primitive(lo))
                ),
                "These are antiderivative values, not values of the original integrand.",
            ),
            step(
                "Subtract upper minus lower",
                format!(
                    "{}-({})={}",
                    text(primitive(hi)),
                    text(primitive(lo)),
                    text(value)
                ),
                "A signed integral can be negative; reversing the bounds reverses its sign.",
            ),
        ],
        format!(
            "Differentiating F returns the original integrand. Reversing the bounds gives {}.",
            text(-value)
        ),
    );
    q.calculator = Some(format!("integral({expression},{lo},{hi})"));
    q
}

fn limit(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let a = f64::from(pick(index, 91, -4, 5));
    let b = f64::from(pick(index, 92, 1, 5));
    match index % 3 {
        0 => {
            let power = rank + 1;
            let value = a.powi(power) + b;
            let expression = format!("x^{power}+{b}");
            let mut q = number(
                format!(
                    "Find the limit of {expression} as x approaches {}.",
                    text(a)
                ),
                value,
                "Polynomials are continuous, so direct substitution gives this limit.",
                vec![
                    step(
                        "Check continuity",
                        format!("f(x)={expression}"),
                        "A polynomial has no domain break at this finite input.",
                    ),
                    step(
                        "Substitute",
                        format!("({a})^{power}+{b}={}", text(value)),
                        "Continuity makes the limiting value equal to the function value.",
                    ),
                ],
                format!(
                    "Inputs approaching {} from both sides produce outputs approaching {}.",
                    text(a),
                    text(value)
                ),
            );
            q.calculator = Some(format!("({a})^{power}+{b}"));
            q
        }
        1 => {
            let (numerator, factor, value) = match difficulty {
                Difficulty::Basic => (
                    format!("x^2-({})", text(a * a)),
                    format!("x+({a})"),
                    2.0 * a,
                ),
                Difficulty::Standard => (
                    format!("x^3-({})", text(a * a * a)),
                    format!("x^2+({a})*x+({})", text(a * a)),
                    3.0 * a * a,
                ),
                Difficulty::Challenge => (
                    format!("x^2+{b}*x-({})", text(a * a + b * a)),
                    format!("x+({})", text(a + b)),
                    2.0 * a + b,
                ),
            };
            number(
                format!(
                    "Find the limit of ({numerator})/(x-({a})) as x approaches {}.",
                    text(a)
                ),
                value,
                "Direct substitution gives 0/0, which is indeterminate. Factor and cancel only for x different from the approach value.",
                vec![
                    step(
                        "Factor the numerator",
                        format!("{numerator}=(x-({a}))*({factor})"),
                        "The common factor explains the removable discontinuity.",
                    ),
                    step(
                        "Cancel away from the missing point",
                        format!("({numerator})/(x-({a}))={factor}, for x != {}", text(a)),
                        "A limit depends on nearby inputs, not the undefined value at the point.",
                    ),
                    step(
                        "Evaluate the simplified limit",
                        format!("{factor} at x={} gives {}", text(a), text(value)),
                        "The remaining polynomial is continuous.",
                    ),
                ],
                format!(
                    "The simplified expression agrees with the original at nearby x != {}; both one-sided limits are {}.",
                    text(a),
                    text(value)
                ),
            )
        }
        _ => {
            let k = if difficulty == Difficulty::Basic {
                1.0
            } else {
                b
            };
            if difficulty == Difficulty::Challenge {
                number(
                    format!(
                        "Find lim as x approaches 0 of (1-cos({}*x))/x^2. Angles are in radians.",
                        text(k)
                    ),
                    k * k / 2.0,
                    "Use 1-cos(t)=2*sin(t/2)^2 and the radian limit sin(u)/u -> 1.",
                    vec![
                        step(
                            "Use the half-angle identity",
                            format!("(1-cos({k}*x))/x^2=2*(sin({k}*x/2)/x)^2"),
                            "The identity avoids substituting into an indeterminate 0/0 expression.",
                        ),
                        step(
                            "Use the standard radian limit",
                            format!("2*({k}/2)^2={}", text(k * k / 2.0)),
                            "As x approaches zero, sin(k*x/2)/x approaches k/2.",
                        ),
                    ],
                    format!(
                        "The expression is nonnegative and even near zero; both sides approach {}.",
                        text(k * k / 2.0)
                    ),
                )
            } else {
                number(
                    format!(
                        "Find lim as x approaches 0 of sin({}*x)/x. Angles are in radians.",
                        text(k)
                    ),
                    k,
                    "Rewrite as k times sin(k*x)/(k*x). The radian limit sin(u)/u is 1.",
                    vec![
                        step(
                            "Match the standard limit",
                            format!("sin({k}*x)/x={k}*[sin({k}*x)/({k}*x)]"),
                            "The numerator's angle and denominator must use the same scaled variable.",
                        ),
                        step(
                            "Take the limit",
                            format!("{k}*1={}", text(k)),
                            "The ratio tends to 1 only when the angle is measured in radians.",
                        ),
                    ],
                    format!(
                        "The same value {} is approached for small positive and negative x.",
                        text(k)
                    ),
                )
            }
        }
    }
}

fn matrix_data(
    index: u32,
    salt: u32,
    rows: usize,
    columns: usize,
    min: i32,
    max: i32,
) -> Vec<Vec<f64>> {
    (0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| {
                    f64::from(pick(
                        index,
                        salt + (row * columns + column) as u32,
                        min,
                        max,
                    ))
                })
                .collect()
        })
        .collect()
}

fn matrices(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as usize;
    let size = if rank == 2 { 3 } else { 2 };
    let min = if rank == 0 { 1 } else { -5 };
    let left = matrix_data(index, 101, size, size, min, 6);
    let right = matrix_data(index, 121, size, size, min, 6);
    let row = (index as usize / 3) % size;
    let column = (index as usize / 7) % size;
    let (operator, value, formula, why, second) = match index % 3 {
        0 => (
            "A+B".to_string(),
            left[row][column] + right[row][column],
            format!("{}+({})", text(left[row][column]), text(right[row][column])),
            "Add entries in the same row and column.",
            Some(right.clone()),
        ),
        1 => {
            let scalar = f64::from(pick(index, 151, if rank == 0 { 2 } else { -4 }, 5));
            (
                format!("{}*A", text(scalar)),
                scalar * left[row][column],
                format!("{}*({})", text(scalar), text(left[row][column])),
                "Scalar multiplication multiplies each entry by the same scalar.",
                None,
            )
        }
        _ => {
            let terms: Vec<String> = (0..size)
                .map(|k| format!("({})*({})", text(left[row][k]), text(right[k][column])))
                .collect();
            let value = (0..size).map(|k| left[row][k] * right[k][column]).sum();
            (
                "A*B".into(),
                value,
                terms.join("+"),
                "Matrix multiplication uses a row of A dotted with a column of B, not matching-entry multiplication.",
                Some(right.clone()),
            )
        }
    };
    let mut q = number(
        format!(
            "For the matrices shown, find row {}, column {} of {operator}. Rows and columns start at 1.",
            row + 1,
            column + 1
        ),
        value,
        "First identify the operation, then locate the requested row and column.",
        vec![
            step(
                "Locate the entries",
                format!("Requested entry: ({}, {})", row + 1, column + 1),
                why,
            ),
            step(
                "Calculate the entry",
                format!("{formula}={}", text(value)),
                "Only the requested entry is needed for this answer.",
            ),
        ],
        format!(
            "The operation produces a {size} by {size} matrix. The requested entry is {}.",
            text(value)
        ),
    );
    q.visual = Visual::Matrix {
        left,
        right: second,
        operator,
    };
    q.calculator = Some(formula);
    q
}

fn determinant_value(matrix: &[Vec<f64>]) -> f64 {
    if matrix.len() == 2 {
        matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]
    } else {
        matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
            - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
            + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0])
    }
}

fn determinant(difficulty: Difficulty, index: u32) -> Question {
    let size = if difficulty == Difficulty::Challenge {
        3
    } else {
        2
    };
    let mut matrix = matrix_data(
        index,
        161,
        size,
        size,
        if difficulty == Difficulty::Basic {
            1
        } else {
            -5
        },
        6,
    );
    if index % 3 == 1 {
        for (row, values) in matrix.iter_mut().enumerate() {
            for value in values.iter_mut().take(row) {
                *value = 0.0;
            }
        }
    }
    if index % 3 == 2 {
        matrix[1] = matrix[0].clone();
    }
    let value = determinant_value(&matrix);
    let (formula, why) = if size == 2 {
        (
            format!(
                "({})*({})-({})*({})",
                text(matrix[0][0]),
                text(matrix[1][1]),
                text(matrix[0][1]),
                text(matrix[1][0])
            ),
            "For a 2 by 2 matrix, subtract the off-diagonal product from the main-diagonal product.",
        )
    } else {
        (
            format!(
                "({})*(({})*({})-({})*({}))-({})*(({})*({})-({})*({}))+({})*(({})*({})-({})*({}))",
                text(matrix[0][0]),
                text(matrix[1][1]),
                text(matrix[2][2]),
                text(matrix[1][2]),
                text(matrix[2][1]),
                text(matrix[0][1]),
                text(matrix[1][0]),
                text(matrix[2][2]),
                text(matrix[1][2]),
                text(matrix[2][0]),
                text(matrix[0][2]),
                text(matrix[1][0]),
                text(matrix[2][1]),
                text(matrix[1][1]),
                text(matrix[2][0])
            ),
            "Expand along the first row with cofactor signs plus, minus, plus.",
        )
    };
    let shortcut = if index % 3 == 1 {
        "This triangular matrix can also be checked by multiplying its diagonal entries."
    } else if index % 3 == 2 {
        "Two rows are equal, so the determinant must be zero."
    } else {
        "A zero determinant means the square matrix is singular."
    };
    let prompt = match index % 3 {
        0 => format!("Find the determinant of the {size} by {size} matrix A shown."),
        1 => format!("A is a triangular {size} by {size} matrix. Find its determinant."),
        _ => format!("The {size} by {size} matrix A has two equal rows. Find its determinant."),
    };
    let mut q = number(
        prompt,
        value,
        "Use ad-bc for a 2 by 2 matrix or cofactors for a 3 by 3 matrix. Look for a structural shortcut.",
        vec![
            step("Choose the determinant rule", formula.clone(), why),
            step("Evaluate", format!("det(A)={}", text(value)), shortcut),
        ],
        shortcut.into(),
    );
    q.visual = Visual::Matrix {
        left: matrix,
        right: None,
        operator: "det(A)".into(),
    };
    q.calculator = Some(formula);
    q
}

fn systems(difficulty: Difficulty, index: u32) -> Question {
    let rank = difficulty.rank() as i32;
    let x = f64::from(pick(index, 181, if rank == 0 { 1 } else { -6 }, 8));
    let y = f64::from(pick(index, 182, if rank == 0 { 1 } else { -6 }, 8));
    if difficulty == Difficulty::Challenge && index.is_multiple_of(3) {
        let z = f64::from(pick(index, 183, -5, 7));
        let (r, s, t) = (x + y + z, 2.0 * x - y + z, x + 2.0 * y - z);
        return values(
            format!(
                "Solve the system:\nx+y+z={}\n2*x-y+z={}\nx+2*y-z={}\nEnter x, y, z in that order, separated by commas.",
                text(r),
                text(s),
                text(t)
            ),
            vec![x, y, z],
            true,
            "Use the first equation to express z, then substitute into the other two.",
            vec![
                step(
                    "Eliminate z",
                    format!("z={r}-x-y"),
                    "Substitute this expression into equations two and three.",
                ),
                step(
                    "Solve the resulting pair",
                    format!("x-2*y={} and 2*x+3*y={}", text(s - r), text(t + r)),
                    "Twice the first reduced equation subtracted from the second leaves 7*y.",
                ),
                step(
                    "Back-substitute",
                    format!("y={}, x={}, z={}", text(y), text(x), text(z)),
                    "Once y is known, use the reduced first equation for x and the original first equation for z.",
                ),
            ],
            format!(
                "Substitution produces the three right-hand values {}, {}, and {}.",
                text(r),
                text(s),
                text(t)
            ),
        );
    }
    let (a, b, c, mut d) = if difficulty == Difficulty::Basic {
        (1.0, 1.0, 1.0, -1.0)
    } else {
        (
            f64::from(pick(index, 184, 1, 6)),
            f64::from(pick(index, 185, -4, 5)),
            f64::from(pick(index, 186, 1, 6)),
            f64::from(pick(index, 187, -5, 5)),
        )
    };
    if (a * d - b * c).abs() < 0.5 {
        d += 1.0;
    }
    let (r, s) = (a * x + b * y, c * x + d * y);
    let det = a * d - b * c;
    let system = format!(
        "Solve the system:\n{a}*x+({b})*y={}\n{c}*x+({d})*y={}",
        text(r),
        text(s)
    );
    let steps = vec![
        step(
            "Eliminate y",
            format!("({a}*{d}-({b})*{c})*x=({r})*{d}-({b})*({s})"),
            "Multiply the first equation by the second y coefficient and the second by the first y coefficient, then subtract.",
        ),
        step(
            "Solve for x",
            format!("x=({})/({})={}", text(r * d - b * s), text(det), text(x)),
            "The nonzero coefficient ensures a unique solution.",
        ),
        step(
            "Back-substitute for y",
            format!("y={}", text(y)),
            "Use either original equation that has a nonzero coefficient of y.",
        ),
    ];
    let check = format!(
        "For x={}, y={}, the original equations evaluate to {}={} and {}={}.",
        text(x),
        text(y),
        text(a * x + b * y),
        text(r),
        text(c * x + d * y),
        text(s)
    );
    if index % 3 == 2 || difficulty != Difficulty::Basic {
        values(
            format!("{system}\nEnter x, y in that order, separated by a comma."),
            vec![x, y],
            true,
            "Eliminate one variable, solve for the other, then substitute back.",
            steps,
            check,
        )
    } else {
        let ask_y = index % 3 == 1;
        let mut q = number(
            format!("{system}\nFind {}.", if ask_y { "y" } else { "x" }),
            if ask_y { y } else { x },
            "Add or subtract the equations to eliminate a variable.",
            steps,
            check,
        );
        q.calculator = Some(if ask_y {
            format!("({a}*({s})-({r})*{c})/({det})")
        } else {
            format!("(({r})*{d}-({b})*({s}))/({det})")
        });
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const TOPICS: [TopicId; 12] = [
        TopicId::LinearEquations,
        TopicId::Quadratics,
        TopicId::Geometry,
        TopicId::Coordinates,
        TopicId::Trigonometry,
        TopicId::FunctionValues,
        TopicId::Derivatives,
        TopicId::Integrals,
        TopicId::Limits,
        TopicId::Matrices,
        TopicId::Determinants,
        TopicId::LinearSystems,
    ];

    fn numbers(answer: &Answer) -> Vec<f64> {
        match answer {
            Answer::Number { value, .. } => vec![*value],
            Answer::Values { values, .. } => values.clone(),
        }
    }

    #[test]
    fn every_secondary_topic_has_bounded_varied_worked_questions() {
        for topic in TOPICS {
            for difficulty in Difficulty::ALL {
                let mut prompts = BTreeSet::new();
                for index in (0..24).chain([u32::MAX]) {
                    let q = generate(topic, difficulty, index).expect("owned topic");
                    assert!(
                        numbers(&q.answer).iter().all(|v| v.is_finite()),
                        "{topic:?} {difficulty:?}"
                    );
                    assert!((2..=5).contains(&q.steps.len()));
                    assert!(!q.hint.is_empty() && !q.check.is_empty());
                    assert!(
                        q.steps
                            .iter()
                            .all(|s| !s.calculation.is_empty() && !s.explanation.is_empty())
                    );
                    prompts.insert(q.prompt);
                }
                assert!(prompts.len() >= 3, "{topic:?} must vary its tasks");
            }
        }
        assert!(generate(TopicId::Addition, Difficulty::Basic, 0).is_none());
    }

    #[test]
    fn scalar_calculator_cross_checks_the_independent_answers() {
        for topic in TOPICS {
            for difficulty in Difficulty::ALL {
                for index in 0..16 {
                    let q = generate(topic, difficulty, index).unwrap();
                    if let (Some(expression), Answer::Number { value, .. }) =
                        (&q.calculator, &q.answer)
                    {
                        let computed = math::calculate(expression, 0.0, false)
                            .unwrap_or_else(|error| panic!("{topic:?}: {expression}: {error}"));
                        let tolerance =
                            if matches!(topic, TopicId::Derivatives | TopicId::Integrals) {
                                1e-6 * (1.0 + value.abs())
                            } else {
                                1e-7
                            };
                        assert!(
                            (computed - value).abs() <= tolerance,
                            "{topic:?} {difficulty:?} {expression}: {computed} != {value}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn quadratic_answers_satisfy_the_generated_equation() {
        for difficulty in Difficulty::ALL {
            for index in 0..30 {
                let q = quadratic(difficulty, index);
                let equation = q
                    .prompt
                    .strip_prefix("Find all real roots of ")
                    .unwrap()
                    .split(". Enter")
                    .next()
                    .unwrap();
                let (left, right) = equation.split_once(" = ").unwrap();
                let expression = math::parse(left).unwrap();
                let rhs = right.parse::<f64>().unwrap();
                let roots = numbers(&q.answer);
                assert_eq!(roots.len(), 2);
                assert_ne!(roots[0], roots[1]);
                for root in roots {
                    assert!((expression.eval(root, 0.0, 0.0, false).unwrap() - rhs).abs() < 1e-8);
                }
            }
        }
    }

    #[test]
    fn linear_answers_balance_both_sides_and_systems_satisfy_every_equation() {
        for difficulty in Difficulty::ALL {
            for index in 0..24 {
                let q = linear(difficulty, index);
                let x = numbers(&q.answer)[0];
                let equation = q
                    .prompt
                    .strip_prefix("Solve ")
                    .unwrap()
                    .split(". Find")
                    .next()
                    .unwrap();
                let (left, right) = equation.split_once(" = ").unwrap();
                let evaluate = |s: &str| math::parse(s).unwrap().eval(x, 0.0, 0.0, false).unwrap();
                assert!((evaluate(left) - evaluate(right)).abs() < 1e-8);
                let q = systems(difficulty, index);
                if let Answer::Values { values, .. } = &q.answer {
                    for line in q.prompt.lines().filter(|line| line.contains('=')) {
                        let (left, right) = line.split_once('=').unwrap();
                        let left = if values.len() == 3 {
                            left.replace('z', &format!("({})", values[2]))
                        } else {
                            left.to_string()
                        };
                        let value = math::parse(&left)
                            .unwrap()
                            .eval(values[0], values[1], 0.0, false)
                            .unwrap();
                        assert!(
                            (value - right.parse::<f64>().unwrap()).abs() < 1e-8,
                            "{}",
                            q.prompt
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn determinants_match_gaussian_elimination_and_matrix_entries_match_full_operations() {
        for difficulty in Difficulty::ALL {
            for index in 0..24 {
                let q = determinant(difficulty, index);
                if let Visual::Matrix { left, .. } = &q.visual {
                    let result = math::matrix_op(left, left, "Determinant A").unwrap()[0][0];
                    assert!((result - numbers(&q.answer)[0]).abs() < 1e-8);
                }
                let q = matrices(difficulty, index);
                if let Visual::Matrix {
                    left,
                    right,
                    operator,
                } = &q.visual
                {
                    let size = left.len();
                    let row = (index as usize / 3) % size;
                    let column = (index as usize / 7) % size;
                    if let Some(right) = right {
                        let op = if operator == "A+B" { "A + B" } else { "A × B" };
                        let result = math::matrix_op(left, right, op).unwrap()[row][column];
                        assert!((result - numbers(&q.answer)[0]).abs() < 1e-8);
                    }
                }
            }
        }
    }

    #[test]
    fn trigonometry_states_units_and_rounding_without_using_global_mode() {
        for difficulty in Difficulty::ALL {
            for index in 0..18 {
                let q = trigonometry(difficulty, index);
                assert!(
                    q.prompt.contains("degrees")
                        || q.prompt.contains("radians")
                        || q.prompt.contains("opposite side")
                );
                assert!(q.prompt.contains("3 decimal"));
                assert!(q.calculator.is_none());
                if let Answer::Number { tolerance, .. } = q.answer {
                    assert_eq!(tolerance, THREE_DECIMAL_TOLERANCE);
                }
            }
        }
        let q = trigonometry(Difficulty::Basic, 0);
        assert!((numbers(&q.answer)[0] - 0.5).abs() < 1e-12);
        let q = trigonometry(Difficulty::Basic, 2);
        assert_eq!(numbers(&q.answer)[0], 0.6);
    }
}
