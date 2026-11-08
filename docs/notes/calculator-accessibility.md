---
tags: [calculator, accessibility, ui-ux, verification]
updated: 2026-11-07
---

# Calculator workflows and expanded accessibility

Step 3 adds configurable support for overlapping visual, hearing, motor, reading, attention and number-convention barriers. Preferences do not ask for a diagnosis or demographic identity. Presets are starting settings, not a judgment about a person's needs. Related acceptance tasks and external guidance are in [[inclusive-design]].

## Calculator behavior

- Native editor changes and printed-key transforms support undo/redo, selection, caret movement and forward/backward deletion. Math and variable menus insert into the originating editor rather than discarding its work.
- A–Z variables and Ans preserve complex values in complex mode. Real mode gives explicit errors for unsupported imaginary results. Principal complex branches are used; numerical calculus remains real-valued.
- Scientific notation, implicit multiplication and supported Unicode mathematical symbols/digits are normalized at input boundaries. Decimal-comma mode uses semicolons between arguments and list values. This is numeric input support, not translation or RTL layout.
- L1–L6 store up to 1,000 finite real values each. Statistics and list aggregates use the selected list. Closed sequence formulas u/v/w evaluate integer n within their documented bounds; recursive sequences are not implemented.
- Y1–Y10 share the calculator graph, larger graph, trace, table and CSV. Numerical value/zero/intersection/extrema/derivative/integral tools report errors for invalid bounds and detected domain gaps. Sampling and numerical methods can miss features and are not symbolic proofs. MATLAB's existing sample export covers Y1/Y2.
- Coordinate-based points, lines and circles appear on both graph views, with clear/remove/undo controls. Descriptions and sample tables remain available without audio. Optional graph pitch playback encodes sampled vertical values; undefined samples are silent.
- PRGM parses a bounded TI-BASIC-style subset locally: assignments, Disp, Input, Pause, If/Then/Else, For, While, Repeat, Lbl/Goto and Stop. Input and Pause resume without repeating previous effects. It has source, execution, nesting and output limits; it does not execute host commands or implement the full TI language.
- OpenCalc text transfer exports/imports versioned JSON for variables, lists, sequences, graph functions and programs. Import validates the entire packet before replacing data; Undo last import restores previous data. It does not communicate with TI hardware.

## Access controls

The alternate Large keys layout uses native controls with adjustable 40–80 point targets and gaps, word labels, left-handed operators and grid keyboard navigation. The physical reference layout remains selectable. No app task requires a microphone or online account.

Optional dwell is a deliberate 600–4,000 ms hover, with leave-before-rearm behavior. A configurable repeat guard covers keypad activation. Automatic scanning uses a paused-by-default row/key selection and Space; opening a popup or dialog stops scanning, with deliberate restart required. Dwell and scanning apply to the keypad only. Whole-app single-switch access has not been implemented.

Display settings include 90–400% interface zoom, 90–180% independent body text, 100–200% line spacing, Light/Warm/Dark, stronger contrast, reading-friendly text, a noninteractive reading guide, and reduced motion. Numeric sizing changes are staged until Set/Apply/Enter to avoid moving an editor during input. Larger content scrolls; the app does not force the window full screen.

Speech settings use installed Windows voices, rate/volume, spoken keys/focus and visible captions. Ctrl+Shift+Space stops speech and graph playback. Quiet also stops existing playback. Speech text/configuration is passed as data, not interpolated shell commands. Audio tests check serialization and generated waveform properties; they do not establish pronunciation or comfortable sound.

Learning settings offer optional help/scores, focus mode and one-step explanations. Essential givens, input conventions and error feedback remain available. The 50-entry glossary has symbol/topic search. The assistive checklist has 18 tasks and keeps untried items marked Not tested.

## Verification boundaries

Native egui interaction tests cover editing, real/complex storage, lists, program pause/input, ten-curve graph paths, numerical tools, local transfer, focus, clipboard events, modal input shielding, dwell/scan safeguards and settings reflow. The parent integration report records the actual final test result and screenshot sizes.

Hardware screen readers, refreshable Braille, physical switches, eye/head pointers, Windows magnification/scaling and disabled-learner usability are still unverified. The keypad scanner does not navigate every app screen. Full TI firmware parity and full language translation remain separate work. Notebook persistence is described in [[notebooks]]. See [[accessibility]] and [[practice-learning]].
