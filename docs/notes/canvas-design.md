---
tags: [notebook, accessibility, canvas, architecture]
updated: 2026-11-08
---

# Notebook canvas

OpenCalc has a desktop vector notebook alongside its calculator work and plain notes. `src/canvas.rs` owns pages, editing, rendering, validation and SVG export. [[notebook-format-v2]] describes the containing notebook file; [[notebooks]] describes save/open behavior and [[inclusive-design]] records broader access limitations.

## Current scope

- Multiple named pages with blank, ruled or grid paper. A notebook always retains at least one page. Deleting another page requires confirmation and can be undone during the current session.
- Freehand ink, a whole-stroke eraser, typed text boxes, lines, rectangles and ellipses. Selection moves objects; coordinates can move and resize every object. Select and Delete remove text and shapes.
- Undo/redo, fit page, continuous canvas zoom from 25% to 400%, pointer panning and named pan buttons. Canvas zoom is separate from the app's interface zoom.
- Normalized page coordinates and optional pressure samples. The file does not depend on a particular display resolution or pointer device. Egui touch force is retained when supplied, with pressure 1 for ordinary mouse input.
- Save/open integration through the version 2 notebook envelope, including the selected page, zoom and pan. Runtime tools, selection, draft fields, pending coordinate points and undo history are excluded. Active pointer ink is included in the snapshot used for saving/recovery.
- UTF-8 SVG bytes for the current page or every page, plus clipboard actions. All-page SVG stacks pages vertically. Save copied XML as a `.svg` text file to share it or open it in a browser for printing. Text wrapping can differ from the editor; SVG is not an editable notebook backup.

This is a useful desktop notebook foundation. PDF/image import, image objects, OCR or handwriting recognition, searchable handwriting, automatic equation recognition, rich text, links, layers, simultaneous collaboration, cloud synchronization and native tablet/phone applications remain future work. No claim of full Goodnotes parity is made. Pen latency, palm rejection, pen buttons, device pressure calibration and actual touch interaction still need physical-device evaluation.

## Research and choices

Official Goodnotes documentation informed a small, explicit set of familiar notebook actions:

| Official reference | OpenCalc choice |
| --- | --- |
| [Text tool](https://support.goodnotes.com/hc/en-us/articles/7353727615375-Type-text-with-the-Text-Tool) | Typed text is a movable, resizable object with editable content. |
| [Eraser tool](https://support.goodnotes.com/hc/en-us/articles/7353718249231-Erase-handwriting-and-page-content-with-the-Eraser-tool) | Eraser removes whole ink strokes. Other object types use selection and deletion. |
| [Shapes and diagrams](https://support.goodnotes.com/hc/en-us/articles/13682939148943-Draw-shapes-and-build-diagrams-in-Goodnotes) | Explicit line, rectangle and ellipse tools, with separate typed labels. |
| [Page templates](https://support.goodnotes.com/hc/en-us/articles/7353743758991-Change-a-page-template-in-Goodnotes) | Paper background is a per-page choice that preserves existing objects. |
| [Android, Windows and web introduction](https://support.goodnotes.com/hc/en-us/articles/7378598719759-Get-Started-With-Goodnotes-for-Android-Windows-and-Web) | Future device work should test each platform and input setup rather than infer mobile readiness from the desktop model. |

These references describe Goodnotes; they are not evidence that OpenCalc has Goodnotes' device support or features. OpenCalc uses an independent vector model and existing egui/serde dependencies.

## Access contract

The visible `Coordinates and elements` disclosure exposes named native text fields for X, Y, width and height in page percentages, text, an author-written description, stroke width and font size. `Add text`, `Add shape`, `Apply object changes`, `Delete selected object`, `Add ink point` and `Finish ink stroke` provide alternatives to dragging or freehand drawing. Lines use X/Y as their start and signed width/height as offsets to their end: zero on one axis makes an exact horizontal or vertical line, and negative offsets point left or up. Text, rectangles and ellipses retain positive box dimensions. Pending coordinate points are explicitly identified as unsaved until finished. Invalid values produce a visible polite status message and preserve the previous object.

The Elements list exposes each object's type, text where present, coordinates, ink point count and description. Selecting an entry loads it into the coordinate fields. Object descriptions are optional and written by the author. OpenCalc does not infer the mathematical or visual meaning of ink. A blind reader cannot recover an undescribed diagram from a point count or an SVG title.

Native buttons and inputs are keyboard reachable. Canvas content shortcuts are Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z or Ctrl/Cmd+Y, and Delete for the selected object. Text fields retain their native editing behavior. Page controls, tools, zoom, pan, element selection, copying and deletion have named controls; all essential edits have a route without dragging. Below 360 logical points, `Page tools` starts collapsed and exposes the same controls when opened by keyboard or pointer. A page/tool summary stays visible, and the fitted canvas uses the remaining viewport height so editing controls do not occupy the whole initial screen. Wider columns retain their full toolbar. Wrapped controls and a clipped canvas support narrow, magnified layouts inside the containing page's vertical scroll area. Confirmation keeps Cancel above its scrollable body. Themes apply to paper, marks, focus and selection; high contrast does not use mark color as the only object label.

Native egui event/AccessKit tests exercise the software contract. They do not establish compatibility with a real screen reader, Braille display, switch, voice access system, eye tracker or motor/vision/hearing condition. Actual users and actual assistive equipment remain necessary acceptance evidence. The application switch scanner currently covers the calculator keypad, not a promise of full notebook scanning.

## Data and resource boundaries

The DTOs are `CanvasNotebook`, `CanvasPage`, `CanvasElement`, `InkPoint`, `Paper` and `InkColor`. Serde rejects unknown structure fields. Load validation is deterministic and completes before the notebook controller mutates app state. Coordinates and pressures must be finite, coordinates/pressure are normalized to 0–1, and nonempty object bounds must stay inside the page. Ink width is 0.5–24 page points; text font size is 8–96 page points. Page geometry is 720 × 1000 logical points.

| Limit | Bound |
| --- | --- |
| Pages | 1–64 |
| Objects | 512 per page; 4,096 per notebook |
| Ink samples | 4,096 per stroke; 100,000 per notebook |
| Text | 16 KiB per object; 1 MiB per notebook |
| Author descriptions | 4,096 bytes per object; 512 KiB per notebook |
| Page titles | 100 Unicode characters; one line |
| Undo | At most 32 snapshots and 16 MiB estimated payload |
| SVG output | At most 16 MiB |

The notebook envelope also caps the full file at 16 MiB. Edits validate aggregate limits and roll back on failure. Pointer sampling enforces the point budget while a stroke is in progress. No filesystem paths, credentials, images, imported SVG, external fonts or external URLs are part of this canvas format.

SVG is generated from validated geometry and known color values. Text, descriptions and page titles are XML escaped. XML 1.0 disallowed controls and U+FFFE/U+FFFF are rejected in notebook text and replaced defensively in the XML encoder. Clip IDs are generated from integer indices, with internal references only. Export neither evaluates nor imports arbitrary XML. Typed text retains its full content in the notebook; SVG uses a bounded line layout and clips to the text box.

## Verification

The module includes save/decode round trips for every object type, invalid geometry/pressure/unknown-field and aggregate limit checks, rollback and undo bounds, actual native pointer ink/shape/eraser events, native text and coordinate editing, horizontal/vertical/reversed line creation without dragging, page deletion confirmation/undo, in-progress stroke snapshots and adversarial SVG text. The SVG test can emit a fixture between `CANVAS-SVG-XML-BEGIN` / `CANVAS-SVG-XML-END` under `--nocapture` for an independent XML parser check.

All ten canvas module tests pass in the 296-test integration gate, along with formatting and Clippy with warnings denied. The adversarial emitted SVG was independently parsed as XML with no scripts or external resources. Native captures at 800×900 and 360×800 with 125% zoom were reviewed, including the fitted page and collapsed narrow toolbar. See [[release-readiness]] for final executable evidence. Physical-device and assistive-hardware acceptance remains unverified.
