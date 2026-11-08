---
tags: [security, dependencies, verification]
updated: 2026-11-07
---

# Dependency advisory review

Checked at **2026-11-08 02:10 UTC** (2026-11-07 locally) for the initial public source snapshot. This is an advisory-database check, not a security certification or a substitute for ongoing updates.

The reviewed `Cargo.lock` SHA-256 is `63b587c8909e375cc68de85358878fdab4e37d4984e39d77eb3f060c01c0b079`. A changed lockfile requires a new check.

## Method and result

Python's standard-library `tomllib` read all 345 crates.io package/version pairs from the lockfile. A read-only [OSV batch query](https://google.github.io/osv.dev/post-v1-querybatch/) sent only those public package names and exact versions to `https://api.osv.dev/v1/querybatch`. It returned 345 result rows with no pagination tokens. Individual matching records were retrieved through OSV and checked against the primary RustSec advisory pages.

**Four packages matched informational maintenance advisories. No other advisory matches were returned.** All four records classify the condition as `unmaintained`; none supplies a CVSS severity score. This result does not mean the program has no vulnerabilities.

| Locked package | Primary advisory | Classification and follow-up |
| --- | --- | --- |
| `ttf-parser 0.25.1` | [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) | Unmaintained font parser; the advisory lists no patched version. Track the GUI/font dependency's migration to a maintained parser. |
| `zbus_names 4.3.5` | [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html) | Standalone crate retired; its functionality moved into zbus 6. Track the upstream AccessKit/D-Bus migration. |
| `zvariant 5.15.1` | [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html) | Standalone crate retired; D-Bus functionality moved into zbus 6. Track the upstream migration rather than forcing an incompatible transitive replacement. |
| `zvariant_derive 5.15.1` | [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html) | Standalone derive crate retired with the zvariant migration. |

## Application context

Lockfile dependency paths and the corresponding vendored package manifests were inspected without changing them:

- Font path: `eframe → egui → epaint → ab_glyph → owned_ttf_parser → ttf-parser`. This is a runtime text-rendering dependency, including on Windows. Equal accepts no font-file uploads; it normally uses egui's packaged fonts and the installed Windows Arial font. This observation does not establish the parser's safety with arbitrary font data. The [upstream discussion](https://github.com/harfbuzz/ttf-parser/issues/217) includes an algorithmic-complexity report, so this warning deserves follow-up even though the indexed advisory is informational.
- D-Bus paths: `eframe → egui-winit → accesskit_winit → accesskit_unix`, then `zbus`/`accesskit_atspi_common` and the three retired crates. `accesskit_winit` declares `accesskit_unix` only for Linux and selected BSD targets. These are not part of that dependency's Windows target path. Their presence in a cross-platform lockfile still matters for future Linux/BSD builds. Upstream documents the consolidation in [zbus issue 2043](https://github.com/z-galaxy/zbus/issues/2043).

No dependencies were upgraded, removed or suppressed by this review. Major GUI/accessibility-stack migrations should be tested as a separate change. No advisory ignore list was added.

## Limits

This check covers the exact registry versions recorded in this lockfile and the database state at the stated time. It does not cover undisclosed defects, unindexed advisories, operating-system components, drivers, MATLAB, Ollama/model weights, malicious local configuration, or application logic. No exploit reproduction, platform build or Cargo command was run by this reviewer. The parent integration records tests and native build results separately.

Related: [[calculator-accessibility]], [[inclusive-design]].

The application package was renamed to `opencalc`; all 345 registry package/version pairs are unchanged from this review.
