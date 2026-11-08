---
tags: [notebooks, persistence, accessibility, verification]
updated: 2026-11-07
---

# Local OpenCalc notebooks

Step 4 adds explicit Save/Open/New/Save as actions for local `.opencalc.json` documents. The compact toolbar has Notebook; workspace users can choose More → Notebook. Ctrl+S, Ctrl+Shift+S, Ctrl+O and Ctrl+N work while text editors have focus. `--notebook PATH` opens a document at startup.

## Saved work

Version 1 contains a title, notes, math level/angle/precision/decimal notation, current calculation and history, full real/complex/list answer memory, scalar variables, six lists, closed-form sequences, saved programs plus the current draft, ten graph functions, graph view/table/trace state and drawings, matrices and a local lesson with its reveal state. Incomplete drafts can be saved without evaluating or executing them.

Practice is opt-in. When included, topic/difficulty selections, deterministic question indices, current answers, first-attempt counts and reveal state are restored without merging or inflating scores. When omitted, opening a notebook leaves the current session's practice work alone.

AI conversations, model settings, attachments, pending jobs, running program execution, undo caches, private file paths and accessibility preferences are excluded. Saved decimal notation is part of the math document; other display, speech, pointer and switch settings remain on the computer. Applying a document clears old tutor requests and playback state so they cannot contaminate restored work.

## File behavior

Documents use the `opencalc-notebook` format tag, schema version 1 and a 2 MiB maximum. Reading consumes at most the limit plus one byte before rejection. Every section, numeric value, selection and text bound is validated before any live work is replaced. Unsupported versions, unknown fields and malformed files produce a visible error and keep current work.

Saves write to an exclusively created temporary file in the destination directory, sync it, close it and rename it over the chosen notebook. The previous destination is not deleted first. Failure cleanup targets only the temporary file created by that operation. Windows replacement and locked-destination preservation have local filesystem regressions.

Native Windows file dialogs and disk work run on background threads. Suggested paths are passed as JSON data to a fixed dialog script. Typed-path controls are available without a native picker and require the notebook extension. Saving to a typed path explicitly replaces that notebook if it exists.

Unsaved work is protected before New, Open and app close. Save-first cancellation or failure keeps the existing document. New changes made after a save snapshot require another decision before continuing. File operations and confirmation dialogs block background calculator activation and pause scanning. Notebook windows scroll at narrow widths; actual assistive-device usability still needs evaluation.

The full file preserves resume state. Dirty comparison ignores ordinary focus, selection and navigation, so moving around the app does not count as editing the work. Notebooks and notebook temporary files are ignored by Git; there is no automatic cloud upload or automatic document saving.

## Verification

The integrated candidate passes 209 tests, formatting, Clippy on all targets with warnings denied, and a native Windows build. Native captures cover the unbranded handheld and notebook panel at 800×900 and 360×800 with 125% zoom. Tests cover document roundtrip, invalid nested-section atomicity, privacy exclusions, optional progress, score integrity, real filesystem replacement/failure preservation, asynchronous UI save/open, cancellation, changes during save, keyboard actions and narrow/high-zoom controls. Live file-dialog interaction and physical assistive-device behavior are not established by these synthetic tests.

Related: [[calculator-accessibility]], [[practice-learning]], [[inclusive-design]].
