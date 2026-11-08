---
tags: [accessibility, calculator, requirements]
updated: 2026-11-07
---

# Accessibility and compact calculator milestone

Requirements confirmed on 2026-11-07:

- Students must be able to keep the calculator beside online homework in a narrow desktop window.
- Use plain descriptive interface copy. Remove promotional slogans and oversized introductory sections.
- K–3 and 4–8 need distinct physical calculator layouts; 9–12 and Advanced share the TI-84-style face.
- Calculator graphs belong on the calculator display. A separate larger graph workspace is optional.
- Every printed key represents an expected working calculator function, including secondary functions. Button wiring alone is not evidence of behavioral compatibility.
- Work through accessibility, grade-specific learning, calculator workflows, and saved notebooks in that order.

## This milestone

The workspace has a compact header and wrapping navigation. Settings use a sidebar only when space permits; otherwise a scrolling modal retains access without resizing the app. Picture-guide cards reflow by width. Keyboard focus, contrast, font selection, magnification, graph tracing, and speech controls are addressed on the handheld and workspace surfaces.

The calculator now has live display screens and a shared device controller, rather than a fixed reference graph and shortcuts that immediately leave the calculator. The elementary and middle-school layouts are drawn controls, not photo overlays.

Graph accessibility includes keyboard trace, coordinates as an AccessKit live status, non-dragging pan/zoom controls, and editable table coordinates. Selected angle mode is consistent in graphs, tables, tracing, and MATLAB samples.

Local speech uses math narration, can be stopped, and reports errors visibly. Screen-reader semantics and synthetic keyboard/pointer interaction tests are useful engineering evidence; they do not establish actual assistive-technology compatibility.

## Verification still requiring a learner's setup

- Complete representative calculations with NVDA and Windows Narrator, checking expression editing, result/error announcements, every key, focus order, and Settings close/focus restoration.
- Explore both graph curves with keyboard and screen reader, including undefined coordinates.
- Check Windows display scaling and the app's 200–400% zoom with a magnifier.
- Try switch input and gather feedback from learners with dyslexia, low vision, color blindness, and hearing impairment.
- Compare all primary and secondary key sequences against the relevant TI guidebooks before claiming firmware parity. Step 3 adds complex arithmetic, a bounded program-language subset and OpenCalc JSON transfer; full TI firmware, language and hardware-transfer compatibility remains unverified.

The existing in-app assistive-technology checklist records only tasks actually tried. Do not mark untried tasks as passing.

## Local verification on 2026-11-07

- 60 tests pass from a fresh isolated target; Clippy with all targets and warnings denied, formatting, and the native build pass.
- Rendered captures checked at 360×720, 480×800, and 640×840, including both school layouts, live LCD graph/trace, high contrast, readable fonts, and 200% magnification.
- Tests cover actual pointer sequences for editing Y1 with physical keys and drawing on the LCD, plus navigation overlap, key magnification, LCD expression/result geometry, native status announcements, fraction precision, memory, storage, and menu navigation.
- eframe 0.33 loads saved window geometry even when `persist_window` is false; apply the initial launch size once after stored zoom settles, then leave resizing to the user.
- The step-1 executable is `target/accessibility/opencalc.exe`; the newer step-2 build is `target/curriculum/opencalc.exe` (see [[practice-learning]]).
- Zoom follow-up: reproduced typed 125% jumping to 200% because the slider displayed percentages but parsed scale factors. Replaced it with presets, 5% buttons, and a staged integer percentage input; intermediate and invalid inputs have native interaction regressions.

## Remaining milestones

2. Added 25 topics and three difficulty levels per topic; see [[practice-learning]].
3. Calculator workflows and expanded access controls are implemented in [[calculator-accessibility]]; guidebook parity and real assistive-technology acceptance remain separate checks.
4. Save and reopen local calculation/graph/lesson notebooks and optional progress.

Related: [[feature-research]]. Reflow is informed by [W3C's reflow guidance](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html); this is a native application and no WCAG conformance claim is made.
