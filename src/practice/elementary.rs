use super::{Answer, Difficulty, Question, TopicId, Visual, WorkedStep, pick};
use crate::math;

fn shown(value: impl Into<f64>) -> String {
    math::format(value.into(), 8)
}

fn step(title: &str, calculation: String, explanation: &str) -> WorkedStep {
    WorkedStep {
        title: title.into(),
        calculation,
        explanation: explanation.into(),
    }
}

fn question(
    prompt: String,
    value: f64,
    hint: &str,
    steps: Vec<WorkedStep>,
    visual: Visual,
    calculator: Option<String>,
    check: String,
) -> Question {
    Question {
        prompt,
        answer: Answer::Number {
            value,
            tolerance: 1e-8,
        },
        hint: hint.into(),
        steps,
        visual,
        calculator,
        check,
    }
}

fn line(start: i32, change: i32) -> Visual {
    let end = start + change;
    Visual::NumberLine {
        start,
        change,
        min: start.min(end).saturating_sub(2).max(0),
        max: start.max(end) + 2,
    }
}

fn small_groups(groups: i32, per_group: i32) -> Visual {
    if groups <= 6 && per_group <= 12 {
        Visual::Groups {
            groups: groups as u32,
            per_group: per_group as u32,
        }
    } else {
        Visual::None
    }
}

pub(super) fn generate(topic: TopicId, difficulty: Difficulty, index: u32) -> Option<Question> {
    let rank = difficulty.rank() as usize;
    Some(match topic {
        TopicId::Addition => addition(rank, index),
        TopicId::Subtraction => subtraction(rank, index),
        TopicId::Counting => counting(rank, index),
        TopicId::PlaceValue => place_value(rank, index),
        TopicId::NumberLines => number_lines(rank, index),
        TopicId::MissingNumber => missing_number(rank, index),
        TopicId::Multiplication => multiplication(rank, index),
        TopicId::Division => division(rank, index),
        TopicId::Fractions => fractions(rank, index),
        TopicId::Decimals => decimals(rank, index),
        TopicId::Percentages => percentages(rank, index),
        TopicId::Ratios => ratios(rank, index),
        TopicId::OrderOfOperations => operations(rank, index),
        _ => return None,
    })
}

fn addition(rank: usize, index: u32) -> Question {
    let maximum = [9, 20, 45][rank];
    let a = pick(index, 1, 1, maximum);
    let b = pick(index, 2, 1, maximum);
    let c = pick(index, 3, 1, [3, 9, 10][rank]);
    let (prompt, total, expression, steps, visual) = match index % 3 {
        0 => (
            format!("{a} + {b} = ?"),
            a + b,
            format!("{a}+{b}"),
            vec![
                step(
                    "Start with the first amount",
                    format!("Start at {a}."),
                    "Addition combines quantities.",
                ),
                step(
                    "Count forward",
                    format!("{a} + {b} = {}", a + b),
                    "Move right once for each item being added.",
                ),
            ],
            line(a, b),
        ),
        1 => (
            format!(
                "There are {a} books on one shelf and {b} on another. How many books altogether?"
            ),
            a + b,
            format!("{a}+{b}"),
            vec![
                step(
                    "Choose the operation",
                    format!("{a} + {b}"),
                    "Altogether asks us to combine both amounts.",
                ),
                step(
                    "Add the quantities",
                    format!("{a} + {b} = {}", a + b),
                    "The result counts every book once.",
                ),
            ],
            line(a, b),
        ),
        _ => (
            format!("Add all three amounts: {a} + {b} + {c}."),
            a + b + c,
            format!("{a}+{b}+{c}"),
            vec![
                step(
                    "Combine two amounts",
                    format!("{a} + {b} = {}", a + b),
                    "Work with two numbers at a time.",
                ),
                step(
                    "Include the third amount",
                    format!("{} + {c} = {}", a + b, a + b + c),
                    "The third quantity must also be counted.",
                ),
            ],
            Visual::None,
        ),
    };
    question(
        prompt,
        f64::from(total),
        "Combine the amounts. Counting forward or making a ten can help.",
        steps,
        visual,
        Some(expression),
        format!("Undo the additions: {total} - {a} = {}.", total - a),
    )
}

fn subtraction(rank: usize, index: u32) -> Question {
    let answer = pick(index, 4, 0, [9, 25, 60][rank]);
    let removed = pick(index, 5, 1, [9, 20, 35][rank]);
    let extra = pick(index, 6, 1, [3, 8, 15][rank]);
    let (prompt, initial, expression, steps, visual) = match index % 3 {
        0 => {
            let initial = answer + removed;
            (
                format!("{initial} - {removed} = ?"),
                initial,
                format!("{initial}-{removed}"),
                vec![
                    step(
                        "Start with the whole",
                        format!("Whole: {initial}."),
                        "The first number is the amount before anything is removed.",
                    ),
                    step(
                        "Take away",
                        format!("{initial} - {removed} = {answer}"),
                        "Move left on the number line to subtract.",
                    ),
                ],
                line(initial, -removed),
            )
        }
        1 => {
            let initial = answer + removed;
            (
                format!("A box has {initial} pencils. {removed} are used. How many remain?"),
                initial,
                format!("{initial}-{removed}"),
                vec![
                    step(
                        "Represent the story",
                        format!("{initial} - {removed}"),
                        "Remain means the original amount minus the amount used.",
                    ),
                    step(
                        "Find what remains",
                        format!("{initial} - {removed} = {answer}"),
                        "Do not add the used pencils back into the remaining group.",
                    ),
                ],
                line(initial, -removed),
            )
        }
        _ => {
            let initial = answer + removed + extra;
            (
                format!(
                    "Start with {initial}. Take away {removed}, then take away {extra}. What remains?"
                ),
                initial,
                format!("{initial}-{removed}-{extra}"),
                vec![
                    step(
                        "Take away the first group",
                        format!("{initial} - {removed} = {}", answer + extra),
                        "Process the first change before the second.",
                    ),
                    step(
                        "Take away the second group",
                        format!("{} - {extra} = {answer}", answer + extra),
                        "Both groups have now been removed.",
                    ),
                ],
                Visual::None,
            )
        }
    };
    question(
        prompt,
        f64::from(answer),
        "Subtract the amounts taken away. Add them back to check.",
        steps,
        visual,
        Some(expression),
        format!(
            "The remaining {answer} plus the {} removed gives the original {initial}.",
            initial - answer
        ),
    )
}

fn counting(rank: usize, index: u32) -> Question {
    match index % 3 {
        0 => {
            let groups = pick(index, 7, 1, [3, 5, 6][rank]);
            let per_group = pick(index, 8, 1, [4, 8, 10][rank]);
            let total = groups * per_group;
            question(
                format!(
                    "Count {groups} groups with {per_group} objects in each. How many objects are there?"
                ),
                f64::from(total),
                "Count every object once, or add the size of each group.",
                vec![
                    step(
                        "Count one group",
                        format!("Each group contains {per_group}."),
                        "Equal groups contain the same number of objects.",
                    ),
                    step(
                        "Count every group",
                        format!("{groups} groups of {per_group} = {total}"),
                        "Skip-count by the group size until every group has been included.",
                    ),
                ],
                small_groups(groups, per_group),
                Some(format!("{groups}*{per_group}")),
                format!("Sharing {total} into {groups} equal groups gives {per_group} in each."),
            )
        }
        1 => {
            let low = pick(index, 9, 1, [10, 30, 80][rank]);
            let difference = pick(index, 10, 1, [5, 15, 40][rank]);
            let high = low + difference;
            let (first, second) = if index.is_multiple_of(2) {
                (low, high)
            } else {
                (high, low)
            };
            question(
                format!("Which number is greater: {first} or {second}? Enter the greater number."),
                f64::from(high),
                "Compare the numbers from their highest place value.",
                vec![
                    step(
                        "Compare the amounts",
                        format!("{high} > {low}"),
                        "The greater number counts a larger quantity.",
                    ),
                    step(
                        "Choose the larger value",
                        format!("The greater number is {high}."),
                        "A greater number is farther right on a number line.",
                    ),
                ],
                Visual::None,
                None,
                format!("{high} - {low} = {difference}, a positive difference."),
            )
        }
        _ => {
            let low = pick(index, 11, 1, [10, 30, 80][rank]);
            let difference = pick(index, 12, 1, [5, 15, 40][rank]);
            let high = low + difference;
            question(
                format!(
                    "One group has {high} objects and another has {low}. How many more are in the first group?"
                ),
                f64::from(difference),
                "Match the objects in pairs. Count what is left in the larger group.",
                vec![
                    step(
                        "Compare by subtraction",
                        format!("{high} - {low}"),
                        "How many more asks for a difference, not the combined total.",
                    ),
                    step(
                        "Find the unmatched amount",
                        format!("{high} - {low} = {difference}"),
                        "This is the extra quantity in the first group.",
                    ),
                ],
                line(low, difference),
                Some(format!("{high}-{low}")),
                format!("{low} + {difference} = {high}."),
            )
        }
    }
}

fn place_value(rank: usize, index: u32) -> Question {
    let number = pick(
        index,
        13,
        if rank == 0 { 10 } else { 100 },
        if rank == 0 { 99 } else { 999 },
    );
    let hundreds = number / 100;
    let tens = number / 10 % 10;
    let ones = number % 10;
    let (prompt, answer, calculation, why) = match (rank, index % 3) {
        (0, 0) => (
            format!("What is the tens digit in {number}?"),
            tens,
            format!("Tens digit: {tens}."),
            "The tens digit is one place to the left of the ones.",
        ),
        (0, 1) => (
            format!("What is the ones digit in {number}?"),
            ones,
            format!("Ones digit: {ones}."),
            "The ones digit is the last digit of a whole number.",
        ),
        (0, _) => (
            format!("What is the value of the tens digit in {number}?"),
            tens * 10,
            format!("{tens} tens = {}.", tens * 10),
            "A digit's value depends on its position.",
        ),
        (1, 0) => (
            format!("What is the hundreds digit in {number}?"),
            hundreds,
            format!("Hundreds digit: {hundreds}."),
            "The hundreds digit is two places to the left of the ones.",
        ),
        (1, 1) => (
            format!("What is the value of the tens digit in {number}?"),
            tens * 10,
            format!("{tens} tens = {}.", tens * 10),
            "Each ten contains ten ones.",
        ),
        (1, _) => (
            format!("What is the ones digit in {number}?"),
            ones,
            format!("Ones digit: {ones}."),
            "Find the units that are not part of a complete ten.",
        ),
        (_, 0) => (
            format!("How many complete groups of ten are in {number}?"),
            number / 10,
            format!("{number} = {} tens + {ones} ones.", number / 10),
            "A hundred contains ten groups of ten.",
        ),
        (_, 1) => (
            format!("What is the value of the hundreds digit in {number}?"),
            hundreds * 100,
            format!("{hundreds} hundreds = {}.", hundreds * 100),
            "Multiply the hundreds digit by one hundred.",
        ),
        (_, _) => (
            format!(
                "In {number}, add the value of the hundreds digit and the ones digit. What is the sum?"
            ),
            hundreds * 100 + ones,
            format!("{} + {ones} = {}.", hundreds * 100, hundreds * 100 + ones),
            "Leave out the tens part because the question asks for hundreds and ones only.",
        ),
    };
    question(
        prompt,
        f64::from(answer),
        "Separate the number into hundreds, tens, and ones.",
        vec![
            step(
                "Expand the number",
                format!("{number} = {} + {} + {ones}", hundreds * 100, tens * 10),
                "Each part records the value of a position.",
            ),
            step("Read the requested place", calculation, why),
        ],
        Visual::PlaceValue {
            number: number as u32,
        },
        None,
        format!("{hundreds}*100 + {tens}*10 + {ones} = {number}."),
    )
}

fn number_lines(rank: usize, index: u32) -> Question {
    let start = pick(index, 14, [2, 5, 20][rank], [10, 25, 70][rank]);
    let distance = pick(index, 15, 1, [5, 10, 20][rank]);
    match index % 3 {
        0 => {
            let end = start + distance;
            question(
                format!(
                    "Start at {start} on a number line. Move {distance} to the right. Where do you land?"
                ),
                f64::from(end),
                "Moving right increases the number.",
                vec![
                    step(
                        "Choose a direction",
                        format!("Right means +{distance}."),
                        "Numbers increase from left to right.",
                    ),
                    step(
                        "Find the endpoint",
                        format!("{start} + {distance} = {end}"),
                        "Add the distance to the starting number.",
                    ),
                ],
                line(start, distance),
                Some(format!("{start}+{distance}")),
                format!("{end} - {distance} = {start}, back at the start."),
            )
        }
        1 => {
            let distance = distance.min(start);
            let end = start - distance;
            question(
                format!(
                    "Start at {start} on a number line. Move {distance} to the left. Where do you land?"
                ),
                f64::from(end),
                "Moving left decreases the number.",
                vec![
                    step(
                        "Choose a direction",
                        format!("Left means -{distance}."),
                        "Count backward rather than forward.",
                    ),
                    step(
                        "Find the endpoint",
                        format!("{start} - {distance} = {end}"),
                        "Subtract the distance from the starting number.",
                    ),
                ],
                line(start, -distance),
                Some(format!("{start}-{distance}")),
                format!("{end} + {distance} = {start}."),
            )
        }
        _ => {
            let back = pick(index, 16, 1, distance);
            let end = start + distance - back;
            question(
                format!(
                    "Start at {start}. Move {distance} right, then {back} left. What number do you reach?"
                ),
                f64::from(end),
                "Record each move separately: right adds and left subtracts.",
                vec![
                    step(
                        "Move right",
                        format!("{start} + {distance} = {}", start + distance),
                        "The first move increases the number.",
                    ),
                    step(
                        "Move left",
                        format!("{} - {back} = {end}", start + distance),
                        "The second move starts from the new position.",
                    ),
                ],
                line(start, distance - back),
                Some(format!("{start}+{distance}-{back}")),
                format!(
                    "The net move is {} right; {start} + {} = {end}.",
                    distance - back,
                    distance - back
                ),
            )
        }
    }
}

fn missing_number(rank: usize, index: u32) -> Question {
    let a = pick(index, 17, 1, [8, 20, 50][rank]);
    let missing = pick(index, 18, 1, [8, 20, 50][rank]);
    let c = pick(index, 19, 1, [3, 9, 20][rank]);
    match index % 3 {
        0 => {
            let total = a + missing;
            question(
                format!("{a} + ? = {total}. Find the missing number."),
                f64::from(missing),
                "Undo addition with subtraction.",
                vec![
                    step(
                        "Identify the whole and known part",
                        format!("Whole {total}; known part {a}."),
                        "The unknown part is what must be added.",
                    ),
                    step(
                        "Subtract the known part",
                        format!("{total} - {a} = {missing}"),
                        "The whole minus one part gives the other part.",
                    ),
                ],
                Visual::None,
                Some(format!("{total}-{a}")),
                format!("Substitute: {a} + {missing} = {total}."),
            )
        }
        1 => {
            let whole = a + missing;
            question(
                format!("? - {a} = {missing}. Find the starting number."),
                f64::from(whole),
                "Add back the amount that was taken away.",
                vec![
                    step(
                        "Undo the subtraction",
                        format!("{missing} + {a}"),
                        "To find the starting whole, combine the remaining part and the removed part.",
                    ),
                    step(
                        "Find the starting amount",
                        format!("{missing} + {a} = {whole}"),
                        "This number goes in the empty position.",
                    ),
                ],
                Visual::None,
                Some(format!("{missing}+{a}")),
                format!("Substitute: {whole} - {a} = {missing}."),
            )
        }
        _ => {
            let total = a + missing + c;
            question(
                format!("{a} + ? + {c} = {total}. Find the missing number."),
                f64::from(missing),
                "Combine the two known parts, then subtract them from the whole.",
                vec![
                    step(
                        "Combine known parts",
                        format!("{a} + {c} = {}", a + c),
                        "Only the unknown part is missing from this subtotal.",
                    ),
                    step(
                        "Find the unknown part",
                        format!("{total} - {} = {missing}", a + c),
                        "Subtract all known parts, not just one.",
                    ),
                ],
                Visual::None,
                Some(format!("{total}-({a}+{c})")),
                format!("Substitute: {a} + {missing} + {c} = {total}."),
            )
        }
    }
}

fn multiplication(rank: usize, index: u32) -> Question {
    let a = pick(index, 20, [2, 3, 12][rank], [10, 12, 35][rank]);
    let b = pick(index, 21, [2, 12, 12][rank], [10, 35, 55][rank]);
    let product = a * b;
    match index % 3 {
        0 => question(
            format!("{a} * {b} = ?"),
            f64::from(product),
            "Break one factor into convenient parts, then add the partial products.",
            vec![
                step(
                    "Split one factor",
                    format!("{b} = {} + {}", b / 10 * 10, b % 10),
                    "Tens and ones can be multiplied separately.",
                ),
                step(
                    "Multiply both parts",
                    format!("{a}*{} + {a}*{} = {product}", b / 10 * 10, b % 10),
                    "Distributivity preserves the complete product.",
                ),
            ],
            small_groups(a, b),
            Some(format!("{a}*{b}")),
            format!("{product} / {a} = {b}."),
        ),
        1 => question(
            format!("Each of {a} boxes holds {b} cards. How many cards are in all the boxes?"),
            f64::from(product),
            "Equal groups can be combined by multiplication.",
            vec![
                step(
                    "Identify equal groups",
                    format!("{a} groups, {b} in each."),
                    "The quantity in one box is repeated for every box.",
                ),
                step(
                    "Multiply",
                    format!("{a} * {b} = {product}"),
                    "Count each box once.",
                ),
            ],
            small_groups(a, b),
            Some(format!("{a}*{b}")),
            format!("Sharing {product} cards among {a} boxes gives {b} per box."),
        ),
        _ => question(
            format!("{a} * ? = {product}. Find the missing factor."),
            f64::from(b),
            "Divide the product by the known factor.",
            vec![
                step(
                    "Use the inverse operation",
                    format!("{product} / {a}"),
                    "Division undoes multiplication.",
                ),
                step(
                    "Find the factor",
                    format!("{product} / {a} = {b}"),
                    "This factor completes the multiplication.",
                ),
            ],
            Visual::None,
            Some(format!("{product}/{a}")),
            format!("{a} * {b} = {product}."),
        ),
    }
}

fn division(rank: usize, index: u32) -> Question {
    let divisor = pick(index, 22, [2, 3, 11][rank], [10, 12, 25][rank]);
    let quotient = pick(index, 23, [2, 5, 10][rank], [10, 20, 30][rank]);
    let dividend = divisor * quotient;
    match index % 3 {
        0 => question(
            format!("{dividend} / {divisor} = ?"),
            f64::from(quotient),
            "Ask which number times the divisor makes the dividend.",
            vec![
                step(
                    "Use a multiplication fact",
                    format!("{divisor} * ? = {dividend}"),
                    "Division and multiplication describe the same equal groups.",
                ),
                step(
                    "Find the quotient",
                    format!("{dividend} / {divisor} = {quotient}"),
                    "The quotient is the size of one equal group.",
                ),
            ],
            Visual::None,
            Some(format!("{dividend}/{divisor}")),
            format!("{quotient} * {divisor} = {dividend}."),
        ),
        1 => question(
            format!(
                "Share {dividend} stickers equally among {divisor} people. How many stickers does each person get?"
            ),
            f64::from(quotient),
            "The total is divided into the stated number of equal groups.",
            vec![
                step(
                    "Represent the sharing",
                    format!("{dividend} / {divisor}"),
                    "Each person must receive the same number.",
                ),
                step(
                    "Find one share",
                    format!("{dividend} / {divisor} = {quotient}"),
                    "The generated problem has no remainder.",
                ),
            ],
            Visual::None,
            Some(format!("{dividend}/{divisor}")),
            format!("{divisor} people receiving {quotient} each use all {dividend} stickers."),
        ),
        _ => question(
            format!("{dividend} / ? = {quotient}. Find the missing divisor."),
            f64::from(divisor),
            "The missing divisor times the quotient must equal the dividend.",
            vec![
                step(
                    "Rewrite as multiplication",
                    format!("? * {quotient} = {dividend}"),
                    "The size of each group times the number of groups is the total.",
                ),
                step(
                    "Find the missing divisor",
                    format!("{dividend} / {quotient} = {divisor}"),
                    "Divide by the known quotient to recover the divisor.",
                ),
            ],
            Visual::None,
            Some(format!("{dividend}/{quotient}")),
            format!("{dividend} / {divisor} = {quotient}."),
        ),
    }
}

fn reduced(numerator: i32, denominator: i32) -> String {
    let (mut a, mut b) = (numerator.abs(), denominator.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    let divisor = a.max(1);
    if denominator / divisor == 1 {
        (numerator / divisor).to_string()
    } else {
        format!("{}/{}", numerator / divisor, denominator / divisor)
    }
}

fn fractions(rank: usize, index: u32) -> Question {
    let d1 = pick(index, 24, 2, [8, 12, 16][rank]);
    let mut d2 = if rank == 0 {
        d1
    } else {
        pick(index, 25, 2, [8, 12, 16][rank])
    };
    if rank != 0 && d1 == d2 {
        d2 = d2 % 15 + 2;
    }
    let mut n1 = pick(index, 26, 1, d1 - 1);
    let mut n2 = pick(index, 27, 1, d2 - 1);
    let mut left_denominator = d1;
    if index % 3 == 1 && n1 * d2 < n2 * d1 {
        (n1, n2) = (n2, n1);
        left_denominator = d2;
        d2 = d1;
    }
    let d1 = left_denominator;
    let left = format!("{n1}/{d1}");
    let right = format!("{n2}/{d2}");
    let visual = Visual::FractionBars {
        parts: vec![(n1, d1 as u32), (n2, d2 as u32)],
    };
    let (operation, numerator, denominator, steps, hint, check) = match index % 3 {
        0 => (
            "+",
            n1 * d2 + n2 * d1,
            d1 * d2,
            vec![
                step(
                    "Use equal-size parts",
                    format!(
                        "{left} = {}/{}, {right} = {}/{}",
                        n1 * d2,
                        d1 * d2,
                        n2 * d1,
                        d1 * d2
                    ),
                    "Equivalent fractions let both numerators count the same-size pieces.",
                ),
                step(
                    "Add the numerators",
                    format!(
                        "({} + {})/{} = {}",
                        n1 * d2,
                        n2 * d1,
                        d1 * d2,
                        reduced(n1 * d2 + n2 * d1, d1 * d2)
                    ),
                    "Keep the common denominator; do not add the denominators.",
                ),
            ],
            "Give both fractions a common denominator, then add their numerators.",
            format!("Subtracting {right} from the result gives {left}."),
        ),
        1 => (
            "-",
            n1 * d2 - n2 * d1,
            d1 * d2,
            vec![
                step(
                    "Use a common denominator",
                    format!("{left} - {right} = ({} - {})/{}", n1 * d2, n2 * d1, d1 * d2),
                    "Only equal-size pieces can be subtracted directly.",
                ),
                step(
                    "Subtract the numerators",
                    format!(
                        "({} - {})/{} = {}",
                        n1 * d2,
                        n2 * d1,
                        d1 * d2,
                        reduced(n1 * d2 - n2 * d1, d1 * d2)
                    ),
                    "The denominator still describes the size of each piece.",
                ),
            ],
            "Rewrite both fractions using the same denominator, then subtract.",
            format!("Adding {right} back to the result gives {left}."),
        ),
        _ if rank == 2 => (
            "/",
            n1 * d2,
            d1 * n2,
            vec![
                step(
                    "Multiply by the reciprocal",
                    format!("({left}) / ({right}) = ({n1}/{d1}) * ({d2}/{n2})"),
                    "Dividing by a nonzero fraction means multiplying by its reciprocal.",
                ),
                step(
                    "Multiply across",
                    format!("({n1}*{d2})/({d1}*{n2}) = {}", reduced(n1 * d2, d1 * n2)),
                    "Multiply numerators and denominators separately.",
                ),
            ],
            "Keep the first fraction and multiply by the reciprocal of the second.",
            format!("Multiplying the result by {right} gives {left}."),
        ),
        _ => (
            "*",
            n1 * n2,
            d1 * d2,
            vec![
                step(
                    "Multiply numerators",
                    format!("{n1} * {n2} = {}", n1 * n2),
                    "The numerator counts the selected parts of parts.",
                ),
                step(
                    "Multiply denominators",
                    format!("({n1}*{n2})/({d1}*{d2}) = {}", reduced(n1 * n2, d1 * d2)),
                    "The denominator counts the complete set of equally sized parts.",
                ),
            ],
            "Multiply the numerators together and the denominators together.",
            format!("Dividing the result by {right} gives {left}."),
        ),
    };
    question(
        format!(
            "Calculate ({left}) {operation} ({right}). Enter a fraction or equivalent decimal."
        ),
        f64::from(numerator) / f64::from(denominator),
        hint,
        steps,
        visual,
        Some(format!("({left}){operation}({right})")),
        check,
    )
}

fn decimals(rank: usize, index: u32) -> Question {
    let scale = if rank == 0 { 10.0 } else { 100.0 };
    let a_units = pick(index, 28, 11, [99, 999, 4999][rank]);
    let b_units = pick(index, 29, 1, [89, 899, 3999][rank]);
    let a = f64::from(a_units) / scale;
    let b = f64::from(b_units) / scale;
    let (left, right) = if a >= b { (a, b) } else { (b, a) };
    match index % 3 {
        0 => question(
            format!("{} + {} = ?", shown(a), shown(b)),
            a + b,
            "Line up decimal points so equal place values are combined.",
            vec![
                step(
                    "Use whole-number units",
                    format!(
                        "{} = {a_units}/{}; {} = {b_units}/{}",
                        shown(a),
                        shown(scale),
                        shown(b),
                        shown(scale)
                    ),
                    "Tenths or hundredths make the place values explicit.",
                ),
                step(
                    "Add matching units",
                    format!("({a_units}+{b_units})/{} = {}", shown(scale), shown(a + b)),
                    "Convert the combined units back to a decimal.",
                ),
            ],
            Visual::None,
            Some(format!("{}+{}", shown(a), shown(b))),
            format!("{} - {} = {}.", shown(a + b), shown(b), shown(a)),
        ),
        1 => question(
            format!("{} - {} = ?", shown(left), shown(right)),
            left - right,
            "Align decimal points before subtracting. A trailing zero does not change a number.",
            vec![
                step(
                    "Align the place values",
                    format!("Subtract {} from {}.", shown(right), shown(left)),
                    "Hundredths must be subtracted from hundredths, and tenths from tenths.",
                ),
                step(
                    "Find the difference",
                    format!(
                        "{} - {} = {}",
                        shown(left),
                        shown(right),
                        shown(left - right)
                    ),
                    "Regroup when a place does not contain enough units.",
                ),
            ],
            Visual::None,
            Some(format!("{}-{}", shown(left), shown(right))),
            format!(
                "{} + {} = {}.",
                shown(left - right),
                shown(right),
                shown(left)
            ),
        ),
        _ if rank == 2 => {
            let count = pick(index, 30, 2, 8);
            let cost = a * f64::from(count);
            let paid = cost.ceil() + f64::from(pick(index, 31, 1, 20));
            question(
                format!(
                    "A notebook costs {}. Buy {count} notebooks and pay {}. How much change is due?",
                    shown(a),
                    shown(paid)
                ),
                paid - cost,
                "First find the total cost, then subtract it from the amount paid.",
                vec![
                    step(
                        "Find the total cost",
                        format!("{} * {count} = {}", shown(a), shown(cost)),
                        "Every notebook costs the same amount.",
                    ),
                    step(
                        "Find the change",
                        format!("{} - {} = {}", shown(paid), shown(cost), shown(paid - cost)),
                        "Change is the paid amount that was not spent.",
                    ),
                ],
                Visual::None,
                Some(format!("{}-{}*{count}", shown(paid), shown(a))),
                format!(
                    "Change {} plus cost {} equals the {} paid.",
                    shown(paid - cost),
                    shown(cost),
                    shown(paid)
                ),
            )
        }
        _ => question(
            format!("{} * {} = ?", shown(a), shown(b)),
            a * b,
            "Multiply as whole-number units, then restore both decimal scales.",
            vec![
                step(
                    "Multiply the unit counts",
                    format!("{a_units} * {b_units} = {}", a_units * b_units),
                    "Treating the decimals as fractions keeps track of their scale.",
                ),
                step(
                    "Restore the decimal scale",
                    format!(
                        "{}/{} = {}",
                        a_units * b_units,
                        shown(scale * scale),
                        shown(a * b)
                    ),
                    "Both input scales must be included in the denominator.",
                ),
            ],
            Visual::None,
            Some(format!("{}*{}", shown(a), shown(b))),
            format!("{} / {} = {}.", shown(a * b), shown(b), shown(a)),
        ),
    }
}

fn percentages(rank: usize, index: u32) -> Question {
    let choices: &[f64] = match rank {
        0 => &[10.0, 25.0, 50.0],
        1 => &[5.0, 15.0, 20.0, 35.0, 60.0, 75.0],
        _ => &[5.0, 12.5, 17.5, 22.5, 37.5, 65.0, 85.0],
    };
    let percentage = choices[pick(index, 32, 0, choices.len() as i32 - 1) as usize];
    let whole = f64::from(pick(index, 33, 5, [20, 50, 100][rank]) * 4);
    let part = whole * percentage / 100.0;
    match index % 3 {
        0 => question(
            format!("Find {}% of {}.", shown(percentage), shown(whole)),
            part,
            "Percent means per hundred. Divide the percentage by 100 before multiplying.",
            vec![
                step(
                    "Convert percent to a decimal",
                    format!(
                        "{}% = {}/100 = {}",
                        shown(percentage),
                        shown(percentage),
                        shown(percentage / 100.0)
                    ),
                    "A percentage represents a fraction of one whole.",
                ),
                step(
                    "Multiply by the whole",
                    format!(
                        "{} * {} = {}",
                        shown(whole),
                        shown(percentage / 100.0),
                        shown(part)
                    ),
                    "This gives the requested amount, not a percentage.",
                ),
            ],
            Visual::None,
            Some(format!("{}*{}/100", shown(whole), shown(percentage))),
            format!(
                "{}/{} * 100 = {}%.",
                shown(part),
                shown(whole),
                shown(percentage)
            ),
        ),
        1 if rank == 2 => {
            let discount = f64::from(pick(index, 34, 1, 8) * 5);
            let increased = whole * (1.0 + percentage / 100.0);
            let final_price = increased * (1.0 - discount / 100.0);
            question(
                format!(
                    "A price of {} increases by {}%, then is discounted by {}%. Find the final price without rounding.",
                    shown(whole),
                    shown(percentage),
                    shown(discount)
                ),
                final_price,
                "The second percentage acts on the increased price, not the original price.",
                vec![
                    step(
                        "Apply the increase",
                        format!(
                            "{} * (1 + {}/100) = {}",
                            shown(whole),
                            shown(percentage),
                            shown(increased)
                        ),
                        "An increase keeps the original whole and adds the percentage amount.",
                    ),
                    step(
                        "Apply the discount to the new price",
                        format!(
                            "{} * (1 - {}/100) = {}",
                            shown(increased),
                            shown(discount),
                            shown(final_price)
                        ),
                        "Successive percentage changes use successive starting amounts.",
                    ),
                ],
                Visual::None,
                Some(format!(
                    "{}*(1+{}/100)*(1-{}/100)",
                    shown(whole),
                    shown(percentage),
                    shown(discount)
                )),
                format!(
                    "Dividing {} by {} and then by {} recovers {}.",
                    shown(final_price),
                    shown(1.0 - discount / 100.0),
                    shown(1.0 + percentage / 100.0),
                    shown(whole)
                ),
            )
        }
        1 => question(
            format!(
                "An item costs {} and has a {}% discount. What is its sale price?",
                shown(whole),
                shown(percentage)
            ),
            whole - part,
            "Find the discount amount first, then subtract it from the price.",
            vec![
                step(
                    "Find the amount taken off",
                    format!(
                        "{} * {}/100 = {}",
                        shown(whole),
                        shown(percentage),
                        shown(part)
                    ),
                    "The discount percentage applies to the original price.",
                ),
                step(
                    "Find the sale price",
                    format!(
                        "{} - {} = {}",
                        shown(whole),
                        shown(part),
                        shown(whole - part)
                    ),
                    "The sale price is what remains after the discount.",
                ),
            ],
            Visual::None,
            Some(format!(
                "{}-{}*{}/100",
                shown(whole),
                shown(whole),
                shown(percentage)
            )),
            format!(
                "Sale price {} plus discount {} equals {}.",
                shown(whole - part),
                shown(part),
                shown(whole)
            ),
        ),
        _ => question(
            format!(
                "{} is what percent of {}? Enter the percentage number without a % sign.",
                shown(part),
                shown(whole)
            ),
            percentage,
            "Divide the part by the whole, then multiply by 100.",
            vec![
                step(
                    "Find the fraction of the whole",
                    format!(
                        "{}/{} = {}",
                        shown(part),
                        shown(whole),
                        shown(percentage / 100.0)
                    ),
                    "The denominator is the original whole.",
                ),
                step(
                    "Convert the fraction to percent",
                    format!(
                        "{} * 100 = {}%",
                        shown(percentage / 100.0),
                        shown(percentage)
                    ),
                    "Enter the percentage number requested by the prompt.",
                ),
            ],
            Visual::None,
            Some(format!("{}/{}*100", shown(part), shown(whole))),
            format!(
                "{}% of {} gives {}.",
                shown(percentage),
                shown(whole),
                shown(part)
            ),
        ),
    }
}

fn ratios(rank: usize, index: u32) -> Question {
    let a = pick(index, 35, 1, [5, 12, 20][rank]);
    let b = pick(index, 36, 1, [5, 12, 20][rank]);
    let multiplier = pick(index, 37, 2, [5, 9, 15][rank]);
    match index % 3 {
        0 => question(
            format!(
                "Red : blue is {a} : {b}. There are {} red counters. How many blue counters are there?",
                a * multiplier
            ),
            f64::from(b * multiplier),
            "Find the common scale factor before changing the second quantity.",
            vec![
                step(
                    "Find the multiplier",
                    format!("{} / {a} = {multiplier}", a * multiplier),
                    "Both sides of a ratio must be scaled by the same factor.",
                ),
                step(
                    "Scale the blue amount",
                    format!("{b} * {multiplier} = {}", b * multiplier),
                    "Use the same scale factor for blue counters.",
                ),
            ],
            Visual::None,
            Some(format!("{} / {a} * {b}", a * multiplier)),
            format!(
                "Cross-products agree: {} * {b} = {} * {a}.",
                a * multiplier,
                b * multiplier
            ),
        ),
        1 => {
            let total = a * multiplier;
            question(
                format!(
                    "{a} identical pens cost {total} currency units. What is the cost of one pen?"
                ),
                f64::from(multiplier),
                "A unit rate tells you the amount for one item.",
                vec![
                    step(
                        "Set up the unit rate",
                        format!("Cost per pen = {total}/{a}"),
                        "Divide the total cost by the number of identical items.",
                    ),
                    step(
                        "Calculate one item's cost",
                        format!("{total}/{a} = {multiplier}"),
                        "The unit rate lets us scale to any number of pens.",
                    ),
                ],
                Visual::None,
                Some(format!("{total}/{a}")),
                format!("{a} * {multiplier} = {total}, the stated total cost."),
            )
        }
        _ => {
            let total = (a + b) * multiplier;
            question(
                format!(
                    "Red : blue is {a} : {b}, with {total} counters altogether. How many are red?"
                ),
                f64::from(a * multiplier),
                "The whole ratio has the sum of both parts, not just the red part.",
                vec![
                    step(
                        "Count all ratio parts",
                        format!("{a} + {b} = {}", a + b),
                        "The total includes both red and blue counters.",
                    ),
                    step(
                        "Find one part",
                        format!("{total}/{} = {multiplier}", a + b),
                        "Divide the whole collection by all ratio parts.",
                    ),
                    step(
                        "Find the red quantity",
                        format!("{a} * {multiplier} = {}", a * multiplier),
                        "Red occupies the first number of ratio parts.",
                    ),
                ],
                Visual::None,
                Some(format!("{total}/({a}+{b})*{a}")),
                format!(
                    "Red {} + blue {} = {total}; their ratio remains {a}:{b}.",
                    a * multiplier,
                    b * multiplier
                ),
            )
        }
    }
}

fn operations(rank: usize, index: u32) -> Question {
    let a = pick(index, 38, 2, [9, 12, 20][rank]);
    let b = pick(index, 39, 2, [9, 12, 15][rank]);
    let c = pick(index, 40, 2, [5, 9, 12][rank]);
    let d = pick(index, 41, 1, c);
    let (expression, value, steps, hint) = match (rank, index % 3) {
        (0, 0) => (
            format!("{a}+{b}*{c}"),
            f64::from(a + b * c),
            vec![
                step(
                    "Multiply first",
                    format!("{b}*{c} = {}", b * c),
                    "Multiplication has priority over addition.",
                ),
                step(
                    "Then add",
                    format!("{a}+{} = {}", b * c, a + b * c),
                    "Do the remaining addition after the product is known.",
                ),
            ],
            "Multiplication comes before addition.",
        ),
        (0, 1) => (
            format!("({a}+{b})*{c}"),
            f64::from((a + b) * c),
            vec![
                step(
                    "Evaluate the parentheses",
                    format!("{a}+{b} = {}", a + b),
                    "Parentheses explicitly group this addition.",
                ),
                step(
                    "Multiply the grouped result",
                    format!("{}*{c} = {}", a + b, (a + b) * c),
                    "The entire sum is a factor.",
                ),
            ],
            "Work inside parentheses before multiplying.",
        ),
        (0, _) => (
            format!("{} / {b} + {c}", a * b),
            f64::from(a + c),
            vec![
                step(
                    "Divide first",
                    format!("{}/{b} = {a}", a * b),
                    "Division has priority over addition.",
                ),
                step(
                    "Add the remaining term",
                    format!("{a}+{c} = {}", a + c),
                    "Use the quotient in the final addition.",
                ),
            ],
            "Division comes before addition.",
        ),
        (1, 0) => (
            format!("{a}^2+{b}*{c}"),
            f64::from(a * a + b * c),
            vec![
                step(
                    "Evaluate the power",
                    format!("{a}^2 = {}", a * a),
                    "A square multiplies the base by itself.",
                ),
                step(
                    "Evaluate multiplication",
                    format!("{b}*{c} = {}", b * c),
                    "Complete multiplication before addition.",
                ),
                step(
                    "Add the results",
                    format!("{}+{} = {}", a * a, b * c, a * a + b * c),
                    "Combine the two computed terms.",
                ),
            ],
            "Do powers, then multiplication, then addition.",
        ),
        (1, 1) => (
            format!("({a}+{b})^2"),
            f64::from((a + b).pow(2)),
            vec![
                step(
                    "Find the grouped sum",
                    format!("{a}+{b} = {}", a + b),
                    "The parentheses identify the base of the power.",
                ),
                step(
                    "Square the entire sum",
                    format!("{}^2 = {}", a + b, (a + b).pow(2)),
                    "Do not square the two addends separately.",
                ),
            ],
            "The whole parenthesized sum is squared.",
        ),
        (1, _) => (
            format!("({a}*{b}+{})/{c}", c * c),
            f64::from(a * b + c * c) / f64::from(c),
            vec![
                step(
                    "Compute the product",
                    format!("{a}*{b} = {}", a * b),
                    "Work through the operations inside the parentheses.",
                ),
                step(
                    "Complete the numerator",
                    format!("{}+{} = {}", a * b, c * c, a * b + c * c),
                    "The complete parenthesized quantity is divided.",
                ),
                step(
                    "Divide the complete sum",
                    format!(
                        "{}/{c} = {}",
                        a * b + c * c,
                        shown(f64::from(a * b + c * c) / f64::from(c))
                    ),
                    "A fraction answer is valid if the division does not end in a short decimal.",
                ),
            ],
            "Complete the entire numerator before dividing.",
        ),
        (_, 0) => (
            format!("{a}+{b}*({c}-{d})"),
            f64::from(a + b * (c - d)),
            vec![
                step(
                    "Evaluate the parentheses",
                    format!("{c}-{d} = {}", c - d),
                    "The grouped subtraction comes first.",
                ),
                step(
                    "Multiply",
                    format!("{b}*{} = {}", c - d, b * (c - d)),
                    "Use the grouped result as one factor.",
                ),
                step(
                    "Add",
                    format!("{a}+{} = {}", b * (c - d), a + b * (c - d)),
                    "Addition is the last remaining operation.",
                ),
            ],
            "Use the order: parentheses, multiplication, addition.",
        ),
        (_, 1) => (
            format!("({a}+{b})^2/{c}"),
            f64::from((a + b).pow(2)) / f64::from(c),
            vec![
                step(
                    "Evaluate the grouped base",
                    format!("{a}+{b} = {}", a + b),
                    "The complete sum is the base of the square.",
                ),
                step(
                    "Square the base",
                    format!("{}^2 = {}", a + b, (a + b).pow(2)),
                    "Powers are evaluated before division.",
                ),
                step(
                    "Divide",
                    format!(
                        "{}/{c} = {}",
                        (a + b).pow(2),
                        shown(f64::from((a + b).pow(2)) / f64::from(c))
                    ),
                    "An exact fraction may be entered instead of a rounded decimal.",
                ),
            ],
            "Evaluate the parentheses, square that result, then divide.",
        ),
        (_, _) => (
            format!("{a}^2-({b}+{c})/{d}"),
            f64::from(a * a) - f64::from(b + c) / f64::from(d),
            vec![
                step(
                    "Evaluate the power and parentheses",
                    format!("{a}^2 = {}; {b}+{c} = {}", a * a, b + c),
                    "These two grouped quantities can be evaluated independently.",
                ),
                step(
                    "Divide the grouped sum",
                    format!("{}/{d} = {}", b + c, shown(f64::from(b + c) / f64::from(d))),
                    "Division still comes before the outer subtraction.",
                ),
                step(
                    "Subtract",
                    format!(
                        "{} - {} = {}",
                        a * a,
                        shown(f64::from(b + c) / f64::from(d)),
                        shown(f64::from(a * a) - f64::from(b + c) / f64::from(d))
                    ),
                    "Keep any exact fraction until the final calculation to avoid rounding drift.",
                ),
            ],
            "Compute the square and parentheses, divide, and only then subtract.",
        ),
    };
    question(
        format!("Evaluate {expression}. Enter a fraction or a decimal."), value, hint, steps,
        Visual::None, Some(expression),
        "Check each intermediate result and substitute it back into the original expression, preserving the original grouping.".into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const OWNED_TOPICS: [TopicId; 13] = [
        TopicId::Addition,
        TopicId::Subtraction,
        TopicId::Counting,
        TopicId::PlaceValue,
        TopicId::NumberLines,
        TopicId::MissingNumber,
        TopicId::Multiplication,
        TopicId::Division,
        TopicId::Fractions,
        TopicId::Decimals,
        TopicId::Percentages,
        TopicId::Ratios,
        TopicId::OrderOfOperations,
    ];

    fn value(question: &Question) -> f64 {
        match question.answer {
            Answer::Number { value, .. } => value,
            _ => panic!("Elementary questions have numeric answers"),
        }
    }

    #[test]
    fn every_elementary_topic_has_varied_bounded_explained_questions() {
        for topic in OWNED_TOPICS {
            for difficulty in Difficulty::ALL {
                let mut prompts = HashSet::new();
                let mut forms = HashSet::new();
                for index in (0..36).chain([u32::MAX]) {
                    let question = generate(topic, difficulty, index).expect("owned topic");
                    assert!(
                        value(&question).is_finite(),
                        "{topic:?} {difficulty:?} {index}"
                    );
                    assert!(
                        !question.prompt.is_empty()
                            && !question.hint.is_empty()
                            && !question.check.is_empty()
                    );
                    assert!((2..=4).contains(&question.steps.len()));
                    assert!(question.steps.iter().all(|step| !step.title.is_empty()
                        && !step.calculation.is_empty()
                        && !step.explanation.is_empty()));
                    if matches!(
                        topic,
                        TopicId::Addition
                            | TopicId::Subtraction
                            | TopicId::Counting
                            | TopicId::PlaceValue
                            | TopicId::NumberLines
                            | TopicId::MissingNumber
                    ) {
                        assert!(value(&question) >= 0.0, "K-3 values stay nonnegative");
                    }
                    forms.insert(
                        question
                            .prompt
                            .chars()
                            .map(|c| if c.is_ascii_digit() { '#' } else { c })
                            .collect::<String>(),
                    );
                    prompts.insert(question.prompt);
                }
                assert!(
                    prompts.len() >= 9,
                    "{topic:?} {difficulty:?}: too few varied examples"
                );
                assert!(
                    forms.len() >= 3,
                    "{topic:?} {difficulty:?}: need multiple task forms"
                );
            }
        }
        assert!(generate(TopicId::Derivatives, Difficulty::Basic, 0).is_none());
    }

    #[test]
    fn calculator_expressions_independently_agree_with_generated_answers() {
        for topic in OWNED_TOPICS {
            for difficulty in Difficulty::ALL {
                for index in (0..48).chain([u32::MAX]) {
                    let question = generate(topic, difficulty, index).unwrap();
                    if let Some(expression) = &question.calculator {
                        let calculated = math::calculate(expression, 0.0, true)
                            .unwrap_or_else(|error| panic!("{topic:?}: {expression}: {error}"));
                        assert!(
                            (calculated - value(&question)).abs() < 1e-7,
                            "{topic:?} {difficulty:?} {index}: {expression} returned {calculated}, expected {}",
                            value(&question)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn place_value_and_number_line_aids_match_the_requested_quantities() {
        for difficulty in Difficulty::ALL {
            for index in 0..36 {
                let question = generate(TopicId::PlaceValue, difficulty, index).unwrap();
                let Visual::PlaceValue { number } = question.visual else {
                    panic!("place-value aid")
                };
                let expected = if question.prompt.contains("add the value") {
                    number / 100 * 100 + number % 10
                } else if question.prompt.contains("tens digit")
                    && !question.prompt.contains("value of")
                {
                    number / 10 % 10
                } else if question.prompt.contains("hundreds digit")
                    && !question.prompt.contains("value of")
                {
                    number / 100
                } else if question.prompt.contains("ones digit")
                    && !question.prompt.contains("add the value")
                {
                    number % 10
                } else if question.prompt.contains("value of the tens") {
                    number / 10 % 10 * 10
                } else if question.prompt.contains("value of the hundreds") {
                    number / 100 * 100
                } else if question.prompt.contains("complete groups") {
                    number / 10
                } else {
                    number / 100 * 100 + number % 10
                };
                assert_eq!(value(&question), f64::from(expected));
                let question = generate(TopicId::NumberLines, difficulty, index).unwrap();
                let Visual::NumberLine {
                    start,
                    change,
                    min,
                    max,
                } = question.visual
                else {
                    panic!("number-line aid")
                };
                assert_eq!(value(&question), f64::from(start + change));
                assert!(
                    min <= start && min <= start + change && max >= start && max >= start + change
                );
            }
        }
    }

    #[test]
    fn fraction_answers_match_rational_arithmetic_and_operations_do_not_round_early() {
        for difficulty in Difficulty::ALL {
            for index in 0..36 {
                let question = generate(TopicId::Fractions, difficulty, index).unwrap();
                let Visual::FractionBars { parts } = &question.visual else {
                    panic!("fraction operands")
                };
                let left = f64::from(parts[0].0) / f64::from(parts[0].1);
                let right = f64::from(parts[1].0) / f64::from(parts[1].1);
                let expected = match index % 3 {
                    0 => left + right,
                    1 => left - right,
                    _ if difficulty == Difficulty::Challenge => left / right,
                    _ => left * right,
                };
                assert!((value(&question) - expected).abs() < 1e-12);
            }
        }
        assert_eq!(reduced(3, 9), "1/3");
        assert_eq!(reduced(0, 24), "0");
    }
}
