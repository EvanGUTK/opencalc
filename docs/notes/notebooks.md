---
tags: [notebooks, persistence, accessibility, verification]
updated: 2026-11-08
---

# Local OpenCalc notebooks

Step 4 adds explicit Save/Open/New/Save as actions for local `.opencalc.json` documents. The compact toolbar has Notebook; workspace users can choose More → Notebook. Ctrl+S, Ctrl+Shift+S, Ctrl+O and Ctrl+N work while text editors have focus. `--notebook PATH` opens a document at startup.

## Saved work

Version 2 contains a title, notes, math level/angle/precision/decimal notation, current calculation and history, full real/complex/list answer memory, scalar variables, six lists, closed-form sequences, saved programs plus the current draft, ten graph functions, graph view/table/trace state and drawings, matrices, a local lesson with its reveal state, and notebook pages. Incomplete drafts can be saved without evaluating or executing them. Strict version 1 documents migrate explicitly to one blank page.

Pages support ink, text, line/rectangle/ellipse objects, editable coordinates and text descriptions, blank/ruled/grid backgrounds, zoom/pan, undo/redo, and SVG export. Pointer drawing has an ordinary-control alternative through object creation and coordinate editing. Limits are 64 pages, 4,096 objects, 100,000 ink points, 1 MiB text and 512 KiB descriptions within the overall file limit. See [[canvas-design]] and [[notebook-format-v2]].

Practice is opt-in. When included, topic/difficulty selections, deterministic question indices, current answers, first-attempt counts and reveal state are restored without merging or inflating scores. When omitted, opening a notebook leaves the current session's practice work alone.

AI conversations, model settings, attachments, pending jobs, running program execution, undo caches, private file paths and accessibility preferences are excluded. Saved decimal notation is part of the math document; other display, speech, pointer and switch settings remain on the computer. Applying a document clears old tutor requests and playback state so they cannot contaminate restored work.

## File behavior

Documents use the `opencalc-notebook` format tag, schema version 2 and a 16 MiB maximum. Reading consumes at most the limit plus one byte before rejection. Serialization stops at the output limit. Every section, numeric value, selection and text bound is validated before any live work is replaced. Unsupported versions, unknown fields and malformed files produce a visible error and keep current work.

Saves write to an exclusively created temporary file in the destination directory, sync it, close it and rename it over the chosen notebook. The previous destination is not deleted first. Failure cleanup targets only the temporary file created by that operation. Windows replacement and locked-destination preservation have local filesystem regressions.

Native Windows file dialogs and disk work run on background threads. Suggested paths are passed as JSON data to a fixed dialog script. Typed-path controls are available without a native picker and require the notebook extension. Saving to a typed path explicitly replaces that notebook if it exists.

Unsaved work is protected before New, Open and app close. Save-first cancellation or failure keeps the existing document. New changes made after a save snapshot require another decision before continuing. File operations and confirmation dialogs block background calculator activation and pause scanning. Notebook windows scroll at narrow widths; actual assistive-device usability still needs evaluation.

The full file preserves resume state. Dirty comparison ignores ordinary focus, selection and navigation, including page selection, zoom and pan. Notebooks and notebook temporary files are ignored by Git. There is no cloud upload.

## Optional local recovery

Protect unsaved work defaults off. When enabled, a background actor coalesces bounded snapshots in the selected user-data folder, retaining a latest and previous revision. A five-second write throttle avoids writing every frame. Startup offers Restore, Keep current or Discard; unresolved copies pause new checkpoints. Restore deliberately clears the original Save target and leaves the document unsaved, so a recovered draft cannot silently overwrite a named file. A corrupt latest copy can fall back to the previous revision; unreadable copies expose Retry/Discard. Disabling clears recovery through the same ordered worker. Discarding before New/Open/Close waits for cleanup, and closing cannot queue a replacement checkpoint after discard. Recovery is supplementary; use Save for a named document.

## Verification

The earlier version 1 candidate passed 209 tests. The current release adds migration, canvas, recovery actor and synthetic native keyboard/file-workflow regressions; executed results are recorded in [[release-readiness]]. Checks cover document roundtrip, invalid nested-section atomicity, privacy exclusions, optional progress, score integrity, real filesystem replacement/failure preservation, asynchronous save/open, cancellation, changes during save, and narrow/high-zoom controls. Live file-dialog interaction and physical assistive-device behavior are not established by synthetic tests.

Related: [[calculator-accessibility]], [[practice-learning]], [[inclusive-design]].
