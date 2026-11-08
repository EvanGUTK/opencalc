---
tags: [learning, practice, curriculum, accessibility]
updated: 2026-11-07
---

# Grade-specific practice

Step 2 replaces the four single-template exercises with 25 topics, three selectable difficulty levels, varied question forms, hints, worked steps, and checks. The Practice tab is available at every grade level. Generated solutions can be read, copied, or opened in Learn without a model or network service.

## Topic coverage

- K–3: addition, subtraction, counting/comparing, place value, number lines, missing numbers.
- 4–8: multiplication, division, fractions, decimals, percentages, ratios/proportions, order of operations.
- 9–12: linear equations, quadratics, geometry, coordinate geometry, trigonometry, functions.
- Advanced: derivatives, definite integrals, limits, matrix operations, determinants, linear systems.

Questions use bounded deterministic integer mixing for repeatable variation without adding a random-number dependency. Difficulty changes the task complexity and operand ranges. Equation and calculus answers are derived from generated mathematical constructions; sampling tests independently check equations, roots, matrix operations, and calculator expressions.

## Answer and score conventions

- Equivalent fractions, decimals, and constant numeric expressions are accepted. Variables and nonfinite values are rejected. Answer expressions use radians; trig prompts explicitly state their own units.
- Questions requesting several roots accept comma-separated values in either order. Commas inside a function call are preserved when splitting answer values.
- Rounding is permitted only to the tolerance stated for that question. A fraction such as `1/3` avoids a rounded repeating decimal.
- Each topic/difficulty has its own question index, response, reveal state, and first-attempt score. Switching away and back preserves that session.
- Invalid text does not count. The first valid submitted answer counts once. Later correction provides feedback without rewriting the first-attempt score.
- Revealing, copying, or opening the full explanation counts an unanswered question as an attempt without a correct answer. Repeating these actions cannot add attempts.
- Reset affects the selected topic/difficulty and advances to a fresh question. No practice data is written to preferences; choose optional practice inclusion in a local [[notebooks|notebook]] to save it across sessions.

## Visual and accessibility behavior

Number lines, object groups, fraction bars, place-value blocks, right triangles, coordinate points, and matrices have visible text equivalents and native accessibility labels. Rendering is bounded and fits narrow content areas. Required matrix givens stay visible even when optional diagrams are hidden, and are included in copied/spoken questions and Learn handoffs. Triangle side labels remain separate from angle quantities.

Feedback uses polite atomic status announcements, native keyboard-focusable controls, and visible text. The view retains high contrast, reading-friendly fonts, and interface zoom. Actual assistive-technology and learner evaluation remains open; see [[accessibility]].

## Verification

All 85 tests pass, along with Clippy for all targets with warnings denied, formatting, and the native build. Tests cover every topic/difficulty combination, sampled question variety and numeric bounds, independent calculation checks, answer formats, score invariants, matrix-givens handoffs, keyboard/pointer interactions, accessible feedback, and narrow layouts. Runtime captures cover all four grade bands, worked steps, visual aids, and a narrow magnified high-contrast view.

The reviewed step-2 executable is `target/curriculum/opencalc.exe`. Start with `--practice --size 800x900` to open the practice workspace beside homework.

Related: [[accessibility]], [[feature-research]], [[notebooks]]. Guidebook parity and real assistive-device evaluation remain open.
