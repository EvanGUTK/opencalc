---
tags: [notebooks, persistence, canvas, verification]
updated: 2026-11-08
---

# Notebook format version 2

OpenCalc writes UTF-8 JSON documents with `format: "opencalc-notebook"` and `version: 2`. The existing calculations, history, variables, lists, programs, graphs, matrices, local lesson, optional practice data, title and notes remain in the document. Version 2 adds a required `canvas` section for vector pages. File paths, recovery-manager state, device pointers, pending jobs and AI conversations/attachments remain outside the document.

## Compatibility

The reader first obtains the version using a small typed header, then reads the matching typed schema. It does not construct a full JSON value tree to migrate the document.

- Version 1 uses an explicit legacy schema with its original fields. Its content is moved unchanged into version 2, with one blank canvas page and default canvas view. The new Pages workspace is not a valid legacy workspace.
- A successful load validates the entire migrated document before any existing work or page is replaced. Number notation follows the saved math; display, speech and motor-access preferences remain on the computer.
- Version 2 requires its canvas section. Missing canvas data is an error, not an instruction to erase pages. An unexpected canvas section in a version 1 document is rejected instead of discarded.
- Unknown fields, unsupported versions and invalid nested sections are rejected. Saving an opened version 1 document writes version 2. Older OpenCalc builds that only understand version 1 cannot read the new version.

JSON whitespace is not part of the format. Version 2 saves use compact JSON so indentation does not consume the ink-document file budget.

## Canvas data

The canvas stores ordered pages with a title, Blank/Ruled/Grid paper and ordered vector objects. Object kinds are ink strokes, lines, rectangles, ellipses and text. Positions and bounds use normalized page coordinates. Ink points include pressure, objects have a bounded style/color, and descriptions provide a textual representation alongside visual content.

The full document also stores the selected page, canvas zoom and canvas pan for reopening. Tool choice, object selection, undo history and active pointer gestures are runtime state and are not serialized.

| Bound | Current limit |
| --- | --- |
| Pages | 1–64 |
| Objects | 512 per page, 4,096 across all pages |
| Ink points | 1–4,096 per stroke, 100,000 across all pages |
| Text | 16 KiB per text object, 1 MiB in total |
| Descriptions | 4,096 bytes per object, 512 KiB in total |
| Page title | 100 characters |
| Coordinates / pressure | Finite, within 0–1 |
| Font size / stroke width | 8–96 / 0.5–24 |
| Canvas zoom / pan | 0.25–4 / each component within ±1 |

Rectangles/ellipses need positive finite bounds inside the page; a line needs distinct valid endpoints. The canvas DTO rejects unknown object kinds and fields. Images, file references and executable content are not part of this schema.

## File and memory bounds

The complete notebook, including math and pages, is limited to **16 MiB**. Existing per-field math/text limits and canvas aggregate limits still apply. These are independent limits: a document that combines several maximum-sized sections may exceed the total file budget and must be reduced before it can be saved.

Serialization writes through a capped buffer. A chunk that would exceed the limit is rejected before it is appended; geometric capacity growth is capped at the same limit. A failed encoding does not produce bytes for a replacement file. This avoids the earlier behavior of constructing an arbitrarily large output vector and only checking its length afterward. The reader also rejects oversized input before parsing; the file helper reads at most the limit plus one byte.

Dirty comparison retains pages, object order, titles, paper, ink/text/shape content and mathematical work. It ignores the selected page, canvas zoom/pan, selected workspace and ordinary editor navigation. Canvas pages are streamed by reference during comparison; ink points are not expanded into a temporary `serde_json::Value` tree. The smaller non-canvas projection is itself size-bounded before it is parsed for navigation normalization.

The total process memory is not limited to 16 MiB: it also holds input, live editor state, snapshots and a bounded non-canvas comparison projection. Snapshot capture still clones bounded work, and serialization remains linear in document size. UI scheduling/revision caching controls how often this cost is paid; no latency benchmark is claimed by the model changes.

## Verification boundary

Model regressions cover representative version 1 migration without loss of math/progress, strict version/canvas handling, version 2 ink/text/shape plus math roundtrip, invalid canvas rejection before mutation, canvas navigation versus content edits, private runtime exclusions, capped output growth/escaping and an aggregate 100,000-point document. The parent integration report records executed test/build results. Physical pen pressure, assistive-device interaction and large-document UI latency require separate runtime evaluation.

Related: [[notebooks]], [[inclusive-design]], [[calculator-accessibility]].
