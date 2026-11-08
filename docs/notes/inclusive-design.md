---
tags: [accessibility, inclusive-design, ui-ux, verification]
updated: 2026-11-07
---

# Inclusive design acceptance matrix

OpenCalc should let people combine controls for their tasks and preferences without providing a diagnosis, age, gender, or other demographic profile. A preset is a starting point, not an assessment of a person. The design covers overlapping barriers rather than claiming that a list of features accommodates every disability. [WAI describes a diversity of abilities and barriers rather than an exhaustive disability checklist.](https://www.w3.org/WAI/people-use-web/abilities-barriers/)

This note defines acceptance work for step 3. A declared setting, a passing unit test, or a screenshot is not evidence that a learner can use their actual assistive technology. The step-3 build and interaction evidence belong to the parent integration report; this reviewer does not mark untried tasks as passing.

## Source review and capability boundaries

Source reviewed on 2026-11-07 includes the alternate large-key calculator, adjustable targets and spacing, word labels, left-handed operator placement, optional dwell and row/key scanning, Light/Warm/Dark palettes, independent text size and spacing, a reading guide, reduced motion, needs-based presets, speech settings and visible announcements. Quiet stops existing speech and graph playback as well as disabling future optional audio. Persisted sizing values have finite-value checks before layout use. These are source observations; this review did not run the candidate build or its tests.

Scanning and dwell apply to the calculator keypad. Menus, Settings, other workspaces and operating-system controls still need ordinary keyboard/pointer navigation or an external access tool. A switch mapped only to Space is not evidence of whole-app single-switch access. Source review also cannot establish speech pronunciation, Braille output, eye-tracker compatibility, preferred reading presentation or whether a scan interval is comfortable for a particular person.

## Functional coverage

| Barrier or task | Configurable support and required behavior | Engineering evidence to collect | Learner or device evidence still needed |
| --- | --- | --- | --- |
| Blind or nonvisual calculation | Named controls, visible focus, expression/result/error semantics, optional spoken keys/focus, readable math text and graph coordinates | Check native accessibility name/role/value and status updates; complete calculations, corrections, menu selection, and errors using keyboard input | NVDA and Narrator task runs; refreshable Braille display behavior, pronunciation and verbosity preferences |
| Low vision and light sensitivity | Large-key view, independent type/spacing controls, zoom, Light/Warm/Dark palettes, stronger contrast | Measure actual targets and text contrast in every palette; reach Settings, display, all keys and errors at narrow widths and magnification | Windows scaling and magnifier use; individual comfort and reading feedback |
| Color-vision differences | Curve names, patterns and text tables carry meaning independently of color | Distinguish all nonempty curve slots in grayscale/high contrast; preserve selected state and errors without color | Learners compare overlapping curves and dense diagrams with their preferred settings |
| Hearing loss or audio-muted use | Every speech, scan and graph-audio action has a usable visible state; sound remains optional | Calculate, trace, correct an error and stop playback with speech disabled and volume zero | Deaf/hard-of-hearing users assess the clarity and timing of visual feedback |
| Limited fine motor control, fatigue, touch or alternative pointers | Large targets, adjustable gaps, left-handed layout, no required dragging, optional dwell | Target bounds, keyboard equivalents, hover timing, leave-to-rearm and no unintended repeats | Touch, trackball, head/eye-controlled pointer and fatigue testing on actual setups |
| Tremor or involuntary repeated activation | Configurable activation guard and reversible editing | One deliberate activation gives one action; a guard rejection has visible feedback; undo restores work | Choose usable delay with the person; ensure rapid intentional arithmetic is not obstructed |
| Single-switch calculation | Explicitly started keypad row/key scan, adjustable interval, visible selection, one Space action per selection, pause/restart | No startup activation, no duplicate action from focused buttons, no actions behind a popup/modal, and no automatic resume after interruption; Start and Resume are deliberately reachable | A real switch mapped to Space; timing, reachability and endurance, including the external navigation needed outside the keypad |
| Speech impairment or inability to use a microphone | Keyboard and pointer cover tasks; keypad dwell/scanning offer additional input methods; OS voice control is optional | No task requires speech input or a microphone permission; labels support unambiguous control names | Windows Voice Access or the user's chosen voice tool, where relevant |
| Dyslexia, dyscalculia or reading difficulty | User-chosen type/spacing, word labels, reading guide, glossary, concrete math models and spoken/text alternatives | Guide never obscures text or consumes clicks; essential values remain present when optional aids are hidden | Learner feedback on recognition, symbol meaning and comprehension; no claim that a font treats a condition |
| Attention, memory, executive-function or anxiety barriers | Stable navigation, optional focus mode/help text/scores, one-step explanations, undo and clear recovery | Settings and a way back remain discoverable; hidden scores do not alter scoring or remove feedback; no unsolicited timers or task timeouts | People assess cognitive load, language, distractions and recovery from mistakes |
| Vestibular or photosensitivity needs | Reduced motion, no flashing cues, controllable scan/playback | Animation is disabled when selected; scan uses a stable outline/text state and always has Pause | Comfort testing; outline changes and sounds may still need individual adjustment |
| Different number conventions or writing systems | Supported non-Latin decimal digits, decimal-comma input, explicit function/multiple-answer separators | Test calculator, practice, lists and program input boundaries consistently; reject ambiguous formats instead of guessing | Readers of supported writing systems check examples and error wording; digit support does not establish translated or RTL interface support |
| Children, older adults, low literacy, small screens or limited resources | Grade-appropriate terms, plain labels, scalable targets, predictable layout, offline core features and no required account | Test each grade and task in a side-by-side window; preserve work when applying presets | Age-appropriate usability sessions, unfamiliar-computer users, and target low-resource devices |

Native targets in the alternate calculator are intended to be adjustable above the small handheld face's targets. Measure the effective clickable bounds, not only the printed keycap. [W3C target-size guidance explains why spacing and equivalent larger controls matter.](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) Web criteria inform this native application's engineering checks; they do not establish native accessibility conformance.

## Combination tasks

1. **Blind plus motor access:** start automatic scanning, enter `12+8`, calculate, correct an entry, open Settings, and return using the configured access method. Record any extra navigation keys or external software required outside the keypad. Repeat with a focused keypad button and with a menu open. There must be one action per selection, an understandable selected-row/key state, no scan action behind the overlay, and a deliberate way to resume after returning. Scan narration must not immediately cancel the calculated answer.
2. **Low vision plus muted audio:** use a dark/high-contrast palette and magnification in a narrow window; distinguish curves, obtain coordinates and find a division-by-zero error with all audio off. Every required value and control must remain reachable by keyboard or scrolling.
3. **Reading difficulty plus anxiety:** choose reading support, hide scores and reveal a solution one step at a time. Essential problem data, answer feedback and the path back must remain visible; hidden score presentation must not change the score rules.
4. **Left-handed pointer plus small window:** mirror the accessible keypad, enlarge targets and calculate with a 360-pixel-wide window. Visual order, keyboard order, scan order and the labels of the selected keys must agree.
5. **Locale plus multiple answers:** test non-Latin digits, a decimal-comma fraction, a multiargument function and unordered quadratic roots using the documented separators. Check the same forms in calculator and practice without overwriting the learner's original answer text.
6. **Dwell plus modal interruption:** hover until just before activation, open an overlay or move into a menu, then close it. No delayed activation may occur; a fresh deliberate dwell is required.
7. **Quiet plus graph exploration:** leave speech and graph sound disabled, inspect the graph summary/table and stop any previously started playback. Quiet must not remove the visible results or make stopping sound depend on hearing it.

## Interaction invariants

- Opening Settings, changing a preset or hiding navigation preserves the expression, selected grade, angle mode and current work unless the person explicitly changes that setting.
- A modal or menu suspends scan/dwell and blocks underlying calculator shortcuts. Closing an overlay restores a meaningful focus target and does not clear the expression.
- Zoom and typography controls do not move while their value is being edited; magnification applies on an explicit commit. Test intermediate percentages, not only minimum and maximum.
- Keyboard help lists only wired shortcuts. Voice-control names describe actions and distinguish repeated controls.
- Status announcements do not steal focus. Their names are explicitly exposed rather than relying on an egui label's visual text. [W3C status-message guidance explains the intended noninterrupting notification behavior.](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html)
- Reading guides and decorative layers do not cover text, intercept pointer input or create accessibility-tree noise.
- Optional speech, narration rate and graph audio have reachable Stop controls and a visible error state when the local voice/audio service is unavailable.
- Core feedback remains available when help, scores, illustrations or sound are disabled. Matrix operands, units and other essential givens are never treated as optional help.
- Labels remain literal and short; grouping and progressive disclosure make the growing option set manageable. [WAI cognitive guidance supports clear steps, familiar organization, predictable controls and recoverable mistakes.](https://www.w3.org/WAI/WCAG2/supplemental/)

## Evidence and release wording

Use three separate outcomes: **source reviewed**, **tested on the candidate**, and **not tested with the learner/device**. Record actual app/OS/assistive-tool versions in the in-app checklist without learner names. A need may be addressed by several controls, and a control may help several people; neither mapping is a guarantee.

Do not claim complete TI firmware compatibility, support for every switch/eye tracker/Braille display, full language translation/RTL layout, or universal disability accommodation without evidence for that specific capability. Keep remaining hardware, platform, translation and learner evaluations explicit.

Related: [[accessibility]], [[practice-learning]].
