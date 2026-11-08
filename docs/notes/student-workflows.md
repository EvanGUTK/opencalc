---
tags: [testing, students, curriculum, native-ui]
updated: 2026-11-08
---

# Student workflow checks

`src/acceptance_tests.rs` drives the native egui application with pointer events,
keyboard events, and AccessKit actions against the widgets the app renders. It
uses the normal application input router, drawing, notebook polling and window
handling. These are synthetic native UI tests, not Windows desktop automation or
observations of students using the app.

| Case | Workflow and expected evidence |
|---|---|
| S01 | K–3 physical keypad at 462 pixels wide: addition, division, memory, divide-by-zero correction; one history entry per calculation. |
| S02 | Middle school fraction entry, fraction display conversion, operation order and parentheses. |
| S03 | Independent teacher calculator fixtures entered into the native expression editor and submitted with Enter, including stored variables, Ans, errors, scientific magnitudes and complex numbers. |
| S04 | Physical calculator Mode menu switches degrees and radians; sine answers follow the selected convention. |
| S05 | Y1 entered on the calculator, embedded graph opened, physical arrows trace a point, and table settings produce the expected coordinates. |
| S06 | Independent graph and matrix fixtures entered into the labeled native workspaces, using the actual matrix operation selector and Calculate button. |
| S07 | Local algebra worked solution, invalid practice input, independently added practice operands, and protection against repeated checks or revealed answers inflating scores. |
| S08 | Force, Avogadro particle count, photon energy and RLC science forms: tool selection, native field entry, actual Calculate action, result values, units and visible result text. |
| S09 | Notebook typed-path save and reopen, unsaved-work Cancel, and rejection of a damaged file while preserving current calculations and notes. All files are synthetic and isolated in a temporary directory. |
| S10 | Two notebook pages containing typed text, a rectangle and coordinate ink; Undo/Redo and exact object preservation through native save/open. |
| S11 | Opt-in recovery checkpoint in an isolated profile, a fresh-session recovery prompt, and a native Restore choice that returns unsaved work. |

The individual fixture IDs appear as `NATIVE-E2E PASS <id>` in test output. Run
`cargo test --locked acceptance_tests -- --nocapture` to reproduce these cases.
Compilation and execution results belong in the release validation evidence;
this note describes the test design and does not claim a successful run.

`src/curriculum_checks.rs` also compiles the independent fixture into the release
binary. `opencalc.exe --self-test` reports case IDs, expected and actual values,
tolerances, reference sources and application version as JSON, returning failure
for an unsupported case or mismatched result. That command exercises production
calculation APIs. It is distinct from the native UI workflows above, and can be
run on an extracted portable or installed binary. Invalid-input fixtures check
that an error is returned; semantic error classes describe the fixture and do
not require matching particular error wording.

The current independent fixture contains 138 cases: 70 calculator, 7 graph, 14
matrix and 47 physics/chemistry cases. Of those, 29 require invalid inputs to be
rejected. S03 and S06 reuse each calculator/graph/matrix case through native
widgets, while S08 checks four representative science forms and the packaged
checker evaluates every science case.

Representative school and university questions cannot establish complete
curriculum coverage, mathematical certification, or usability for every person.
Actual screen readers, refreshable Braille, switch devices, eye tracking, touch,
stylus hardware and student/teacher observations require separate sessions.

Related notes: [[teacher-validation]], [[accessibility-release-check]], [[notebook-format-v2]].
