mod elementary;
mod secondary;
mod view;
mod visuals;

use crate::math;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Difficulty {
    Basic,
    Standard,
    Challenge,
}
impl Difficulty {
    pub const ALL: [Self; 3] = [Self::Basic, Self::Standard, Self::Challenge];
    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Standard => "Standard",
            Self::Challenge => "Challenge",
        }
    }
    pub fn rank(self) -> u32 {
        match self {
            Self::Basic => 0,
            Self::Standard => 1,
            Self::Challenge => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TopicId {
    Addition,
    Subtraction,
    Counting,
    PlaceValue,
    NumberLines,
    MissingNumber,
    Multiplication,
    Division,
    Fractions,
    Decimals,
    Percentages,
    Ratios,
    OrderOfOperations,
    LinearEquations,
    Quadratics,
    Geometry,
    Coordinates,
    Trigonometry,
    FunctionValues,
    Derivatives,
    Integrals,
    Limits,
    Matrices,
    Determinants,
    LinearSystems,
}

pub(crate) struct Topic {
    pub id: TopicId,
    pub level: usize,
    pub title: &'static str,
    pub description: &'static str,
}
pub(crate) const TOPICS: &[Topic] = &[
    Topic {
        id: TopicId::Addition,
        level: 0,
        title: "Addition",
        description: "Add objects, numbers, and story quantities.",
    },
    Topic {
        id: TopicId::Subtraction,
        level: 0,
        title: "Subtraction",
        description: "Take away and find differences.",
    },
    Topic {
        id: TopicId::Counting,
        level: 0,
        title: "Counting and comparing",
        description: "Count groups and compare whole numbers.",
    },
    Topic {
        id: TopicId::PlaceValue,
        level: 0,
        title: "Place value",
        description: "Read hundreds, tens, and ones.",
    },
    Topic {
        id: TopicId::NumberLines,
        level: 0,
        title: "Number lines",
        description: "Move forward and backward along a number line.",
    },
    Topic {
        id: TopicId::MissingNumber,
        level: 0,
        title: "Missing numbers",
        description: "Find the unknown part of a calculation.",
    },
    Topic {
        id: TopicId::Multiplication,
        level: 1,
        title: "Multiplication",
        description: "Use equal groups and products.",
    },
    Topic {
        id: TopicId::Division,
        level: 1,
        title: "Division",
        description: "Share equally and undo multiplication.",
    },
    Topic {
        id: TopicId::Fractions,
        level: 1,
        title: "Fractions",
        description: "Compare and calculate with fractions.",
    },
    Topic {
        id: TopicId::Decimals,
        level: 1,
        title: "Decimals",
        description: "Use decimal place value and arithmetic.",
    },
    Topic {
        id: TopicId::Percentages,
        level: 1,
        title: "Percentages",
        description: "Find percentage amounts and changes.",
    },
    Topic {
        id: TopicId::Ratios,
        level: 1,
        title: "Ratios and proportions",
        description: "Scale ratios and solve proportional problems.",
    },
    Topic {
        id: TopicId::OrderOfOperations,
        level: 1,
        title: "Order of operations",
        description: "Apply parentheses, powers, and arithmetic in order.",
    },
    Topic {
        id: TopicId::LinearEquations,
        level: 2,
        title: "Linear equations",
        description: "Balance equations and isolate an unknown.",
    },
    Topic {
        id: TopicId::Quadratics,
        level: 2,
        title: "Quadratics",
        description: "Factor quadratics and find real roots.",
    },
    Topic {
        id: TopicId::Geometry,
        level: 2,
        title: "Geometry",
        description: "Calculate lengths, areas, and triangle relationships.",
    },
    Topic {
        id: TopicId::Coordinates,
        level: 2,
        title: "Coordinate geometry",
        description: "Find slopes, midpoints, and distances.",
    },
    Topic {
        id: TopicId::Trigonometry,
        level: 2,
        title: "Trigonometry",
        description: "Use trigonometric ratios and explicitly stated angles.",
    },
    Topic {
        id: TopicId::FunctionValues,
        level: 2,
        title: "Functions",
        description: "Evaluate functions and interpret changes.",
    },
    Topic {
        id: TopicId::Derivatives,
        level: 3,
        title: "Derivatives",
        description: "Find rates of change and tangent slopes.",
    },
    Topic {
        id: TopicId::Integrals,
        level: 3,
        title: "Definite integrals",
        description: "Use antiderivatives and evaluate bounds.",
    },
    Topic {
        id: TopicId::Limits,
        level: 3,
        title: "Limits",
        description: "Evaluate limits using substitution or simplification.",
    },
    Topic {
        id: TopicId::Matrices,
        level: 3,
        title: "Matrix operations",
        description: "Add, scale, and multiply matrix entries.",
    },
    Topic {
        id: TopicId::Determinants,
        level: 3,
        title: "Determinants",
        description: "Calculate determinants and identify singular matrices.",
    },
    Topic {
        id: TopicId::LinearSystems,
        level: 3,
        title: "Linear systems",
        description: "Solve simultaneous equations by elimination.",
    },
];

#[derive(Clone, Debug)]
pub(crate) enum Answer {
    Number {
        value: f64,
        tolerance: f64,
    },
    Values {
        values: Vec<f64>,
        tolerance: f64,
        ordered: bool,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct WorkedStep {
    pub title: String,
    pub calculation: String,
    pub explanation: String,
}

#[derive(Clone, Debug)]
pub(crate) enum Visual {
    None,
    NumberLine {
        start: i32,
        change: i32,
        min: i32,
        max: i32,
    },
    Groups {
        groups: u32,
        per_group: u32,
    },
    FractionBars {
        parts: Vec<(i32, u32)>,
    },
    PlaceValue {
        number: u32,
    },
    Triangle {
        labels: [String; 3],
    },
    CoordinatePlot {
        points: Vec<(f64, f64)>,
        line: bool,
    },
    Matrix {
        left: Vec<Vec<f64>>,
        right: Option<Vec<Vec<f64>>>,
        operator: String,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct Question {
    pub prompt: String,
    pub answer: Answer,
    pub hint: String,
    pub steps: Vec<WorkedStep>,
    pub visual: Visual,
    pub calculator: Option<String>,
    pub check: String,
}

pub(super) fn pick(index: u32, salt: u32, min: i32, max: i32) -> i32 {
    const SALT_MULTIPLIER: u32 = 0x9e37_79b9;
    const FIRST_MIX: u32 = 0x7feb_352d;
    const SECOND_MIX: u32 = 0x846c_a68b;
    // Mix before reduction so small operand ranges do not share the seed's short cycles.
    let mut seed = index.wrapping_add(salt.wrapping_mul(SALT_MULTIPLIER));
    seed ^= seed >> 16;
    seed = seed.wrapping_mul(FIRST_MIX);
    seed ^= seed >> 15;
    seed = seed.wrapping_mul(SECOND_MIX);
    seed ^= seed >> 16;
    min + (seed % (max - min + 1) as u32) as i32
}

pub(crate) fn topics(level: usize) -> impl Iterator<Item = &'static Topic> {
    TOPICS
        .iter()
        .filter(move |topic| topic.level == level.min(3))
}

impl Answer {
    pub fn display(&self) -> String {
        match self {
            Self::Number { value, .. } => math::format(*value, 8),
            Self::Values { values, .. } => values
                .iter()
                .map(|value| math::format(*value, 8))
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    pub fn check(&self, input: &str) -> Result<bool, String> {
        let actual = parse_answers(input)?;
        let (mut expected, tolerance, ordered) = match self {
            Self::Number { value, tolerance } => (vec![*value], *tolerance, true),
            Self::Values {
                values,
                tolerance,
                ordered,
            } => (values.clone(), *tolerance, *ordered),
        };
        if expected.is_empty()
            || expected.iter().any(|value| !value.is_finite())
            || !tolerance.is_finite()
            || tolerance < 0.0
        {
            return Err("This question's answer is unavailable. Choose another question.".into());
        }
        if actual.len() != expected.len() {
            return Err(if expected.len() == 1 {
                "Enter one value. Fractions such as 3/4 are accepted.".into()
            } else {
                format!("Enter {} values separated by commas.", expected.len())
            });
        }
        let mut actual = actual;
        if !ordered {
            expected.sort_by(f64::total_cmp);
            actual.sort_by(f64::total_cmp);
        }
        Ok(actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| (*actual - expected).abs() <= tolerance.max(1e-10)))
    }
}

fn parse_answers(input: &str) -> Result<Vec<f64>, String> {
    if input.len() > 512 || input.trim().is_empty() {
        return Err("Enter an answer of at most 512 characters.".into());
    }
    let mut depth = 0_i32;
    let mut start = 0;
    let mut parts = Vec::new();
    for (index, character) in input.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' | ';' if depth == 0 => {
                parts.push(&input[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
        if !(0..=48).contains(&depth) {
            return Err("Check the parentheses in your answer.".into());
        }
    }
    if depth != 0 {
        return Err("Close the parentheses in your answer.".into());
    }
    parts.push(&input[start..]);
    if parts.len() > 12 {
        return Err("Enter at most 12 answer values.".into());
    }
    parts
        .into_iter()
        .map(|part| {
            let expression = math::parse(part.trim())?;
            fn has_variable(expression: &math::Expr) -> bool {
                match expression {
                    math::Expr::Var(_) => true,
                    math::Expr::Unary(_, argument) => has_variable(argument),
                    math::Expr::Binary(_, first, second) => {
                        has_variable(first) || has_variable(second)
                    }
                    math::Expr::Call(_, arguments) => arguments.iter().any(has_variable),
                    math::Expr::Number(_) => false,
                }
            }
            if has_variable(&expression) {
                return Err("Enter a numeric answer; substitute any variables first.".into());
            }
            expression.eval(0.0, 0.0, 0.0, false).and_then(|value| {
                if value.is_finite() {
                    Ok(value)
                } else {
                    Err("Enter a finite number.".into())
                }
            })
        })
        .collect()
}

impl Question {
    pub fn problem_text(&self) -> String {
        let mut text = self.prompt.clone();
        if let Visual::Matrix { left, right, .. } = &self.visual {
            let shown = |matrix: &[Vec<f64>]| {
                matrix
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|value| math::format(*value, 8))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            text.push_str(&format!("\nA = [{}]", shown(left)));
            if let Some(right) = right {
                text.push_str(&format!("\nB = [{}]", shown(right)));
            }
        }
        text
    }

    pub fn solution_text(&self) -> String {
        let mut text = format!("{}\n\n", self.problem_text());
        for (index, step) in self.steps.iter().enumerate() {
            text.push_str(&format!(
                "{}. {}\n{}\n{}\n\n",
                index + 1,
                step.title,
                step.calculation,
                step.explanation
            ));
        }
        text.push_str(&format!(
            "Answer: {}\nCheck: {}",
            self.answer.display(),
            self.check
        ));
        text
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FeedbackKind {
    #[default]
    None,
    Correct,
    Incorrect,
    Invalid,
    Revealed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Progress {
    pub attempted: u32,
    pub correct: u32,
}

#[derive(Default)]
pub(crate) struct Session {
    pub response: String,
    pub feedback: String,
    pub feedback_kind: FeedbackKind,
    pub hint_shown: bool,
    pub show_steps: bool,
    pub revealed_steps: usize,
    pub show_visual: bool,
    pub progress: Progress,
    pub index: u32,
    counted: bool,
}

pub(crate) struct Practice {
    level: usize,
    selected: [TopicId; 4],
    difficulties: [Difficulty; 4],
    sessions: BTreeMap<(TopicId, Difficulty), Session>,
}

impl Default for Practice {
    fn default() -> Self {
        let mut practice = Self {
            level: 0,
            selected: [
                TopicId::Addition,
                TopicId::Multiplication,
                TopicId::LinearEquations,
                TopicId::Derivatives,
            ],
            difficulties: [Difficulty::Basic; 4],
            sessions: BTreeMap::new(),
        };
        practice.ensure_session();
        practice
    }
}

impl Practice {
    fn key(&self) -> (TopicId, Difficulty) {
        (self.selected[self.level], self.difficulties[self.level])
    }
    fn ensure_session(&mut self) {
        self.sessions.entry(self.key()).or_insert_with(|| Session {
            show_visual: true,
            ..Session::default()
        });
    }
    pub fn set_level(&mut self, level: usize) {
        self.level = level.min(3);
        self.ensure_session();
    }
    pub fn selected_topic(&self) -> TopicId {
        self.selected[self.level]
    }
    pub fn select_topic(&mut self, id: TopicId) {
        if topics(self.level).any(|topic| topic.id == id) {
            self.selected[self.level] = id;
            self.ensure_session();
        }
    }
    pub fn difficulty(&self) -> Difficulty {
        self.difficulties[self.level]
    }
    pub fn select_difficulty(&mut self, difficulty: Difficulty) {
        self.difficulties[self.level] = difficulty;
        self.ensure_session();
    }
    pub fn current_session(&self) -> &Session {
        self.sessions
            .get(&self.key())
            .expect("Selected practice session is initialized")
    }
    pub fn current_session_mut(&mut self) -> &mut Session {
        self.ensure_session();
        self.sessions
            .get_mut(&self.key())
            .expect("Selected practice session is initialized")
    }
    pub fn current_question(&self) -> Question {
        let (topic, difficulty) = self.key();
        let index = self.current_session().index;
        let question = if self.level < 2 {
            elementary::generate(topic, difficulty, index)
        } else {
            secondary::generate(topic, difficulty, index)
        };
        question.expect("Every listed practice topic has a question generator")
    }
    pub fn check_answer(&mut self) {
        self.check_answer_with_locale(false);
    }
    pub fn check_answer_with_locale(&mut self, decimal_comma: bool) {
        let question = self.current_question();
        let result = math::normalize_input(&self.current_session().response, decimal_comma)
            .and_then(|normalized| question.answer.check(&normalized))
            .map_err(|error| {
                if decimal_comma {
                    error.replace("separated by commas", "separated by semicolons")
                } else {
                    error
                }
            });
        let session = self.current_session_mut();
        match result {
            Ok(correct) => {
                if !session.counted {
                    session.progress.attempted = session.progress.attempted.saturating_add(1);
                    if correct {
                        session.progress.correct = session.progress.correct.saturating_add(1);
                    }
                    session.counted = true;
                }
                session.feedback_kind = if correct {
                    FeedbackKind::Correct
                } else {
                    FeedbackKind::Incorrect
                };
                session.feedback = if correct {
                    "Correct.".into()
                } else {
                    format!("Not correct yet. {}", question.hint)
                };
            }
            Err(error) => {
                session.feedback_kind = FeedbackKind::Invalid;
                session.feedback = error;
            }
        }
    }
    pub fn show_hint(&mut self) {
        self.current_session_mut().hint_shown = true;
    }
    pub fn reveal_answer(&mut self) {
        let answer = self.current_question().answer.display();
        let session = self.current_session_mut();
        if !session.counted {
            session.progress.attempted = session.progress.attempted.saturating_add(1);
            session.counted = true;
        }
        session.feedback = format!("Answer: {answer}.");
        session.feedback_kind = FeedbackKind::Revealed;
        session.show_steps = true;
        session.revealed_steps = session.revealed_steps.max(1);
    }
    pub fn reveal_answer_with_pacing(&mut self, one_step: bool) {
        let step_count = self.current_question().steps.len();
        self.reveal_answer();
        if !one_step {
            self.current_session_mut().revealed_steps = step_count;
        }
    }
    pub fn next_question(&mut self) {
        let session = self.current_session_mut();
        *session = Session {
            index: session.index.wrapping_add(1),
            progress: session.progress,
            show_visual: session.show_visual,
            ..Session::default()
        };
    }
    pub fn reset_current_progress(&mut self) {
        let session = self.current_session_mut();
        *session = Session {
            index: session.index.wrapping_add(1),
            show_visual: session.show_visual,
            ..Session::default()
        };
    }
    pub fn total_progress_for_level(&self) -> Progress {
        self.sessions
            .iter()
            .filter(|((id, _), _)| topics(self.level).any(|topic| topic.id == *id))
            .fold(Progress::default(), |mut total, (_, session)| {
                total.attempted = total.attempted.saturating_add(session.progress.attempted);
                total.correct = total.correct.saturating_add(session.progress.correct);
                total
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locale_answers_accept_fractions_and_semicolon_lists_without_editing_the_response() {
        let fraction = Answer::Number {
            value: 0.75,
            tolerance: 1e-8,
        };
        for input in ["0,75", "3/4", "٣/٤"] {
            let normalized = math::normalize_input(input, true).unwrap();
            assert_eq!(fraction.check(&normalized), Ok(true), "{input}");
        }
        let values = Answer::Values {
            values: vec![1.5, 2.5],
            tolerance: 1e-8,
            ordered: false,
        };
        for input in ["2,5;1,5", "min(1,5;2,5);5/2"] {
            let normalized = math::normalize_input(input, true).unwrap();
            assert_eq!(values.check(&normalized), Ok(true), "{input}");
        }
        let mut practice = Practice::default();
        let answer = practice.current_question().answer.display();
        let original = format!("  {answer},0  ");
        practice.current_session_mut().response = original.clone();
        practice.check_answer_with_locale(true);
        assert_eq!(
            practice.current_session().feedback_kind,
            FeedbackKind::Correct
        );
        assert_eq!(practice.current_session().response, original);
        assert_eq!(practice.current_session().progress.correct, 1);
    }

    #[test]
    fn explanation_pacing_applies_to_new_questions_without_resetting_visible_steps() {
        let mut practice = Practice::default();
        practice.reveal_answer_with_pacing(true);
        assert_eq!(practice.current_session().revealed_steps, 1);
        let count = practice.current_question().steps.len();
        practice.current_session_mut().revealed_steps = count;
        practice.reveal_answer_with_pacing(true);
        assert_eq!(practice.current_session().revealed_steps, count);
        assert_eq!(practice.current_session().progress.attempted, 1);
        practice.next_question();
        practice.reveal_answer_with_pacing(false);
        assert_eq!(
            practice.current_session().revealed_steps,
            practice.current_question().steps.len()
        );
        assert_eq!(practice.current_session().progress.attempted, 2);
        assert_eq!(practice.current_session().progress.correct, 0);
    }

    #[test]
    fn numeric_answers_accept_equivalent_fractions_and_constant_expressions() {
        let fraction = Answer::Number {
            value: 0.75,
            tolerance: 1e-8,
        };
        for input in ["3/4", "6/8", "0.75", "75%"] {
            assert_eq!(fraction.check(input), Ok(true), "{input}");
        }
        for input in ["", "x", "ans", "1/0", "sqrt(-1)", "NaN", "3,4"] {
            assert!(fraction.check(input).is_err(), "{input}");
        }
        assert_eq!(fraction.check("1/4"), Ok(false));
        let roots = Answer::Values {
            values: vec![-2.0, 3.0],
            tolerance: 1e-8,
            ordered: false,
        };
        assert_eq!(roots.check("3, -2"), Ok(true));
        assert_eq!(roots.check("min(1,3)-3; sqrt(9)"), Ok(true));
        assert_eq!(roots.check("3, 3"), Ok(false));
    }

    #[test]
    fn every_topic_and_difficulty_produces_checkable_varied_local_problems() {
        for topic in TOPICS {
            for difficulty in Difficulty::ALL {
                let mut prompts = std::collections::BTreeSet::new();
                for index in (0..24).chain([u32::MAX]) {
                    let question = if topic.level < 2 {
                        elementary::generate(topic.id, difficulty, index)
                    } else {
                        secondary::generate(topic.id, difficulty, index)
                    }
                    .expect("listed topic generator");
                    assert_eq!(
                        question.answer.check(&question.answer.display()),
                        Ok(true),
                        "{:?}/{difficulty:?}/{index}",
                        topic.id
                    );
                    assert!(
                        question.steps.len() >= 2
                            && !question.hint.is_empty()
                            && !question.check.is_empty(),
                        "{:?}/{difficulty:?}/{index}",
                        topic.id
                    );
                    if let (Some(expression), Answer::Number { value, .. }) =
                        (&question.calculator, &question.answer)
                    {
                        let calculated =
                            math::calculate(expression, 0.0, false).unwrap_or_else(|error| {
                                panic!(
                                    "{:?}/{difficulty:?}/{index}: {expression}: {error}",
                                    topic.id
                                )
                            });
                        assert!(
                            (calculated - value).abs() <= 1e-5 * value.abs().max(1.0),
                            "{:?}/{difficulty:?}/{index}: {expression} produced {calculated}, expected {value}",
                            topic.id
                        );
                    }
                    prompts.insert(question.prompt);
                }
                assert!(
                    prompts.len() >= 3,
                    "{:?}/{difficulty:?} must offer varied questions",
                    topic.id
                );
            }
        }
    }

    #[test]
    fn first_attempts_count_once_and_revealing_does_not_create_correct_answers() {
        let mut practice = Practice::default();
        let answer = practice.current_question().answer.display();
        practice.current_session_mut().response = "not a number".into();
        practice.check_answer();
        assert_eq!(practice.current_session().progress, Progress::default());
        practice.current_session_mut().response = answer;
        practice.check_answer();
        practice.check_answer();
        assert_eq!(
            practice.current_session().progress,
            Progress {
                attempted: 1,
                correct: 1
            }
        );
        practice.next_question();
        practice.reveal_answer();
        practice.current_session_mut().response = practice.current_question().answer.display();
        practice.check_answer();
        assert_eq!(
            practice.current_session().progress,
            Progress {
                attempted: 2,
                correct: 1
            }
        );
    }

    #[test]
    fn correcting_a_valid_wrong_answer_does_not_rewrite_the_first_attempt() {
        let mut practice = Practice::default();
        let answer = practice.current_question().answer.display();
        practice.current_session_mut().response = format!("({answer})+1");
        practice.check_answer();
        assert_eq!(
            practice.current_session().feedback_kind,
            FeedbackKind::Incorrect
        );
        practice.current_session_mut().response = answer;
        practice.check_answer();
        assert_eq!(
            practice.current_session().feedback_kind,
            FeedbackKind::Correct
        );
        assert_eq!(
            practice.current_session().progress,
            Progress {
                attempted: 1,
                correct: 0
            }
        );
    }

    #[test]
    fn matrix_givens_survive_text_copy_and_tutor_handoff() {
        let mut practice = Practice::default();
        practice.set_level(3);
        practice.select_topic(TopicId::Matrices);
        let question = practice.current_question();
        let text = question.problem_text();
        assert!(text.contains("A = [") && text.contains("B = ["));
        assert!(question.solution_text().contains(&text));
    }

    #[test]
    fn switching_topics_difficulty_and_grades_preserves_separate_sessions() {
        let mut practice = Practice::default();
        practice.current_session_mut().response = practice.current_question().answer.display();
        practice.check_answer();
        practice.select_topic(TopicId::Subtraction);
        assert_eq!(practice.current_session().progress, Progress::default());
        practice.select_topic(TopicId::Addition);
        practice.check_answer();
        assert_eq!(practice.current_session().progress.attempted, 1);
        practice.select_difficulty(Difficulty::Standard);
        assert!(practice.current_session().response.is_empty());
        practice.set_level(2);
        assert_eq!(practice.selected_topic(), TopicId::LinearEquations);
        practice.select_topic(TopicId::Addition);
        assert_eq!(practice.selected_topic(), TopicId::LinearEquations);
        practice.set_level(0);
        practice.select_difficulty(Difficulty::Basic);
        assert_eq!(
            practice.current_session().progress,
            Progress {
                attempted: 1,
                correct: 1
            }
        );
        practice.reset_current_progress();
        assert_eq!(practice.current_session().progress, Progress::default());
    }
}
