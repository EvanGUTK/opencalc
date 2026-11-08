---
tags: [accessibility, release, acceptance, verification]
updated: 2026-11-08
---

# Accessibility release acceptance

This record separates software behavior from usability with a person's actual setup. Synthetic native egui events exercise the application; they do not simulate a disability, a screen reader, a Braille display, an eye tracker or a physical switch. Preferences are combined by capability and task. No diagnosis or demographic identity is collected.

Related: [[calculator-accessibility]], [[accessibility]], [[notebooks]], [[inclusive-design]].

## Candidate evidence

`src/accessibility_acceptance.rs` adds behavioral acceptance through the production keyboard, drawing, overlay, notebook and AccessKit paths. Each frame uses explicit viewport size and time. Assertions check actual expressions/results, native roles/names/live status, focus, control bounds, clipboard output and persisted notebook work.

**All 16 native acceptance scenarios pass in the final 296-test gate.** Native screenshot review prompted compact Page tools and science-browser disclosures. The magnification cases exercise those disclosure steps; the 16th scenario checks visible default science inputs and keyboard browser selection at 360 × 800 with 125% zoom. Formatting, Clippy with warnings denied, the optimized build and native captures also pass; see [[release-readiness]]. These results establish the asserted software behavior, not physical assistive-technology compatibility.

| Capability/task | Concrete acceptance evidence | Boundary |
| --- | --- | --- |
| Magnification and overlapping display preferences | Calculate 7 + 5 with 100/200/400% interface zoom, up to 180% body text, high contrast, three themes, word labels, left-handed order, 80-point keys and 24-point gaps; verify named key bounds and answer 12. | Synthetic 480 × 900 viewport; actual Windows scaling/magnifier remains untested. |
| Nonvisual information | Native expression/key names, polite atomic result/error status, division-by-zero recovery and retained editor focus. | AccessKit tree evidence does not establish announcements or navigation in a particular reader. |
| Keyboard-only calculation | Tab to expression, type, calculate, open Settings by shortcut, cancel and resume the same editor. | Uses native egui key events; actual OS input methods remain separate. |
| Rapid repeated activation | An activation inside the configured guard is rejected visibly; the next deliberate activation remains usable. | Keypad guard only; not a claim about any particular tremor or motor condition. |
| Limited pointer movement | A stationary pointer requires the full dwell interval, activates once, and must leave before repeating. Large-target bounds are checked separately. | Synthetic pointer location/time; no eye/head tracker used. |
| Switch-style input | Scanning starts paused; Space starts and selects a row/key; opening help pauses and prevents later background activation. | Software Space-key workflow only. The scanner covers the keypad, not the whole application. |
| Muted use | Results/errors remain native text; copy produces clipboard text with speech disabled and volume zero. | No hearing assessment or audio-comfort claim. |
| Reading and number notation | Monospaced reading preference, enlarged text, increased paragraph line height, word alternatives and decimal-comma calculation preserve the number meaning. | No claim that one font or spacing choice works for every reader. |
| Attention and untimed learning | Focus mode, hidden scores/help and one-step preference do not advance a practice question, erase a draft, or change scoring after an hour of elapsed test time. | User pacing and learning effectiveness require feedback. |
| Vocabulary and visible feedback | Glossary actions name their term; copy returns the definition and an in-window polite live status while audio remains optional. | Definitions are English; broader language/localization support is separate. |
| Whole notebook task | Save through a local typed path, use keyboard Save, cancel keyboard New/Open without a live file picker, then reopen saved work and retain access preferences. | Native Windows file-picker interaction is not exercised. Test creates and removes only its owned temporary notebook. |
| Physics/chemistry | Native acceptance checks cover given/unit labels, a negative-mass error, correction through the keyboard, a 12 N result/copy report, and combined 400%/180% text control bounds. | Passed in candidate runs; actual reader and magnifier behavior remain untested. |
| Notebook pages/canvas | Native acceptance checks use Alt+7, Tab and Enter to create described text, undo/redo and copy SVG without a drawing gesture; combined 400%/180% preferences check tool and coordinate control bounds. | Passed in candidate runs. Freehand art has only user-supplied semantic descriptions. |

## Concrete corrections in this pass

- Glossary Read/Copy controls now name the term visibly, so several entries do not expose indistinguishable actions.
- Copying a glossary definition now exposes a polite atomic status inside the open glossary and updates optional visible captions; the feedback is no longer confined to the background workspace.
- Large-key captions use the actual key width minus padding when enlarged word labels wrap. A long label cannot enlarge the key past the narrow column; labels can use additional vertical space.
- Controller-driven expression focus no longer permanently prevents a later deliberate dwell. Actual keyboard editing still cancels the pending hover until the pointer leaves. Switch activation retains scanner focus so the next row/key choice can proceed.
- Dwell uses the current hover position and key/clip bounds. A PointerGone event cannot retain the old fired key through egui's retained interaction position; leaving rearms the next deliberate hold.
- Native single-line Enter handling in the calculator and science/math tools was corrected by the owning agents: Enter's focus-loss event now evaluates and keeps the intended editor usable.
- The full-app Settings/Escape workflow exposed a focus-return issue in the background-shielded path; the root owner corrected it. The strict native editor-focus assertion now passes.
- Canvas owners corrected field bounds and nested action rows that exceeded a 120-point column under combined magnification. The strict native control-bounds case now passes without weakening its limits.

Existing source tests also cover contrast values, input sanitization, staged sizing, narrow settings groups, paused scanning safeguards, modal input shielding, notebook validation and atomic replacement. Those tests must remain enabled. The final integration report owns their actual current pass count.

## Read-only tool discovery on 2026-11-08

- Windows Narrator executable is present (`10.0.26100.8972`). It was not launched or used.
- NVDA was not found at either standard Program Files path checked. The standard JAWS directory was not present. Portable or differently located installations are unknown.
- No running process named NVDA, Narrator, JAWS (`jfw`), ZoomText or Magnifier was found at the time of the metadata check.
- No assistive software was installed, no accessibility setting was enabled/disabled, and no reader, magnifier, hardware driver or device was configured.

## Real setup and user acceptance still required

1. Run representative calculations, error recovery, graph/table exploration, Settings, glossary, science tools and save/open/cancel using Windows Narrator and NVDA. Check spoken name/value/state, focus order, announcements and background shielding.
2. Test refreshable Braille math output, routing, cursor editing and results with actual reader/display combinations. No Braille compatibility is established here.
3. Test physical switches, switch interfaces, eye/head pointers and touch. Confirm deliberate start/pause, cancellation, target reachability and fatigue; whole-app single-switch navigation remains outside the keypad scanner.
4. Check 200–400% Windows display scaling and magnifier behavior, small side-by-side windows and combined maximum text/target preferences with real monitors.
5. Review pronunciation, voice choices, speech interruption, sonification volume and stop behavior with intended users. Visible status must remain sufficient with all audio disabled.
6. Ask learners with varied visual, hearing, motor, reading, numeracy, attention and cognitive needs to complete their own homework tasks. Include overlapping needs and differing ages, language familiarity and input methods; do not infer usability from demographic membership.
7. Test the real Windows Open/Save/Cancel dialogs, permissions, removable/network paths and recovery prompts through assistive technology. Synthetic cancellation is not live-dialog acceptance.

Release notes should describe configurable features and verified software behavior. Do not claim universal accessibility, medical effectiveness, full screen-reader/Braille compatibility or standards certification from this evidence.
