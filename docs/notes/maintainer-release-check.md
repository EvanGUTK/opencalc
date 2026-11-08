---
tags: [release, security, maintainers, verification]
updated: 2026-11-08
---

# Maintainer release qualification

**Verdict: PASS for the frozen local Windows `0.2.0-beta.1` qualification candidate and the exact artifacts below.** The concrete source, privacy and packaging findings raised by this review were repaired. This verdict is scoped to the inspected software behavior, source and package evidence; it is not universal security, accessibility or curriculum certification.

The reviewer changed only this note, made no remote changes and ran no Cargo/build/install commands. Root executed the local gates and installer smoke. The reviewer independently inspected source, package bytes/manifests, emitted XML, numerical expectations, advisory records and the resulting local evidence.

## Candidate and executed gates

- Base: `624014ae70489e060a33ae53a5ad98d27ffb202a`, branch `feat/release-readiness`.
- Frozen product/packaging fingerprint: `9d3616ccc14b78f7beef57bb70a22d3fe3adc898c51a2d36fb84c44ac22ebc75`. This is SHA-256 over compact JSON containing sorted `(relative path, raw file SHA-256)` pairs for the 48 Cargo, Rust source, packaging/script, fixture and ignore files inspected. Documentation-only edits do not identify a different executable.
- Root completed **296 Rust tests**, formatting, Clippy on all targets with warnings denied, an optimized Windows x86_64 build with static CRT, **7 Python packaging/input tests**, and a clean Git-index source build without local vendor configuration or optional reference photos.
- Cargo application version, package version and actual self-test version are `0.2.0-beta.1`.

The qualification packages were built before the source commit and intentionally have an empty `source_commit` manifest field. They are local, unpublished qualification artifacts, not signed commit attestations. Root will repackage after committing the reviewed source, populate its actual source SHA and verify the published/downloaded assets separately. The hashes in this note identify qualification bytes only.

## Independently checked real artifacts

| Artifact | SHA-256 |
| --- | --- |
| Optimized `opencalc.exe` | `c6ae02b32359dfa488c96d64ecd1d099dffc4e725c462a75cfd09dbe291cbbc8` |
| Qualification portable ZIP | `dab8fa0ecd048776ec6aecb4747dde0ef84a50a7527599272d8cdcd958daa1a2` |
| Qualification installer | `b99d64b210083713763a42f66c0bc652d1f0cc9c03f1e59332f46c56e0cf7671` |
| Actual curriculum result JSON | `176e6a32f7a1c286b9d18a47ea076fa9ac3581f7855d4b3acf6f0ee12c0a7802` |

Files are under local ignored `target/qualification-final/`. Both archive/installer hashes match their `SHA256SUMS.txt`. The ZIP contains exactly **631 indexed files plus its manifest**, with every recorded size and SHA-256 checked. Its executable matches the frozen executable above. No preference, private notebook, recovery/profile data, cache/vendor/config, credential or optional TI-photo entry was present.

The packaged dependency index matches the current lockfile and contains 345 exact package/version rows and 619 matching notice hashes. Four font-license texts match upstream. Packaging copies an explicit file list, and the portable marker is included in the portable manifest.

PE normal and delay-import tables were independently inspected without executing the image. No separate Visual C++ redistributable DLL import was found. System DLLs and drivers remain operating-system requirements. The beta is unsigned; checksums identify bytes rather than proving publisher identity.

## Actual installed and portable smoke evidence

Root ran the real installer/portable smoke, not the earlier synthetic payload. Evidence is under `target/final-install-smoke/run-0ee489dee7bb4031a7c5466032ec4d9f/`; `results.json` SHA-256 is `560b6fdb436730e9a0e46bb8594e12ba75d0a79ca06f65215547c69e1baee2a2`.

- Silent installation and all 630 installed indexed files were checked. All 631 portable indexed files were checked.
- Both actual executables' self-test reports contain **138 passed, 0 failed**, all 138 unique fixture IDs and the expected application version. Their recorded executable hashes match the frozen executable. These JSON reports were independently read and counted.
- Both installed and portable screenshot launches completed. The 800x900 captures were visually inspected and are byte-identical: SHA-256 `8ba3569e972003f09191b947ed9656b2ca3ea1571c38eb36bf9f130f3f8c5faa`.
- The locked-file uninstall test reported failure while retaining ownership/uninstaller metadata, then repair/retry succeeded. Successful uninstall removed owned payloads while preserving both synthetic personal-file sentinels. Independent remaining-filename inspection found only `keep.opencalc.json` and `data/keep.txt`; their contents were not needed for this audit.
- The smoke used isolated `/TESTPROFILE`, which skips registry, Start menu and file-association changes. Ordinary per-user registry/shortcut creation and a fresh-machine interactive installation were not exercised by this isolated mode.

## Input, persistence and privacy boundaries

Notebook reads use a limit-plus-one bounded reader. Version 2 requires canvas state; a strict version-1 structure migrates unchanged work to a blank canvas. Nested data is validated before live-state replacement. Encoding is capped before append; the 16 MiB file bound is not a total-process-memory claim. Saves use an exclusively created same-directory temporary, sync/close it and rename without first deleting the destination.

Recovery is opt-in and outside source repositories. It uses a coalescing worker, validated latest/previous generations, revision checks and a durable discard marker. New/Open/Close transitions wait for confirmed cleanup. Discard-close cannot queue the discarded document again; failed discovery has an explicit paused state with Retry/Discard routes. Restoring recovery clears any saved-file target and leaves work unsaved.

Canvas stores bounded generated vector objects, text and descriptions, with no image/SVG import or file-path payload. SVG uses generated numeric geometry, colors and IDs; untrusted strings are escaped and invalid XML characters rejected. The actual adversarial test output was independently parsed with Python ElementTree: 5,098 UTF-8 bytes, 69 elements, two unique IDs and two internal clip references, no script/event/external-resource nodes, and preserved literal script-like text, ampersands, Chinese text and chemical subscripts. Extracted SVG SHA-256: `7053751837d688045097c9c3de5d2abf8f4724172a2ef9313e6db86a2ba9f42e`.

Science adds no file import, interpreter, network request or new dependency. Each field is limited to 512 Unicode characters. Numeric expressions reject variables/Ans, outputs must be finite, and chemical nesting/count arithmetic is bounded and checked. Its workbench state is explicitly ephemeral.

The final staged scan inspected **51 changed paths / 48 text blobs**, with no forbidden path or high-confidence secret-pattern match. Its point-in-time staged-manifest SHA-256 is `2747c7837e293bd8666c4f2b3e3055f5e3f3ccf5d3e77ad02caa92794bb89f34`; subsequent documentation restaging changes that fingerprint. Private preference, credential and `.env` contents were not read. Current and legacy notebook temporary-file names, notebooks, preferences, logs, build/cache directories and optional TI photos are ignored. Scans reduce exposure risk but cannot prove absence of every secret or application defect.

## Findings repaired during review

| Finding | Verified repair |
| --- | --- |
| Discard-close could checkpoint the discarded dirty work again. | Closing-state guard and asynchronous cleanup regression; final local test gate passed. |
| Failed recovery discovery could look active while checkpoints remained held. | Explicit paused/needs-action state and Retry/Discard controls; no silent overwrite of unreadable drafts. |
| Notice-directory recursion could include unindexed incidental files. | Packaging copies indexed paths and explicit index/readme only; private sentinel excluded. |
| Portable marker was absent from the file manifest. | Separate variant manifest includes its marker and exact hashes. |
| Uninstall could orphan locked payloads while deleting retry metadata. | File-deletion checks preserve retry metadata and return failure; actual locked-file retry smoke passed. |
| Package/Cargo/Windows metadata could disagree. | Version guard plus numeric/readable Windows prerelease metadata. |
| Interrupted save temporaries were not covered by ignore rules. | Both `.opencalc-notebook-*.tmp` and legacy `.equal-notebook-*.tmp` probes are ignored. |

No unresolved blocker from this audit remains for the scoped local beta qualification.

## Fresh dependency and notice check

At **2026-11-08 06:26 UTC**, a read-only OSV batch query sent only all 345 public crates.io names and exact versions. It returned 345 rows without pagination. Only the same four informational maintenance records matched; no other matches were returned. Individual primary RustSec records were inspected. No dependency was upgraded or advisory suppressed.

- Current lock SHA-256: `8e0f33a1c733eda6a6305d800f7bf944557a2fd5b4c7ce22a77931d9b7226ad1`.
- Sorted registry-pair list SHA-256: `20ffe8aa0cc1bd2b82eaaf32dfb2e68f7cf344805b5d297cd1dbc35a15b47311`.
- OSV response SHA-256: `2f0aa8dd2ce8851f4132932dad42014af6b1139f19394acb8d3959342084d608`.
- Packaged notice-index SHA-256: `286dc5fd335a714ab51821783a62fda3fc8baa2d83e7afa72c4434e7ff3ba0b3`.

| Locked dependency | Primary record | Ongoing maintainer concern |
| --- | --- | --- |
| ttf-parser 0.25.1 | [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) | Track the GUI/font parser migration. The [upstream discussion](https://github.com/harfbuzz/ttf-parser/issues/217) includes an algorithmic-complexity report. |
| zbus_names 4.3.5 | [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html) | Follow the zbus 6 consolidation, particularly for future Linux/BSD builds. |
| zvariant 5.15.1 | [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html) | Follow the same upstream migration. |
| zvariant_derive 5.15.1 | [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html) | Track the derive-macro consolidation. |

These records supply no CVSS score or patched version. Informational status is not a safety guarantee. Undisclosed defects, OS/driver components and external MATLAB/Ollama/model behavior are outside this query.

The notice bundle carries unchanged upstream texts with hashes and commit-pinned supplement provenance. Its sole empty-notice row is `dispatch 0.2.0`, explicitly excluded from this Windows binary; root's exact Windows dependency graph returned no path to it. No upstream notice was reconstructed. This explicit platform exception is recorded in the index rather than claiming every cross-platform package was redistributed.

## Independent teacher-fixture review

Frozen fixture SHA-256: `701f8c50195eef4cae33291a1d41ed26f5f183c94448220e0442006ad6bcd563`; criteria `teacher-oracle-v1-2026-11-08`.

All **109 positive expectations** were independently recomputed without OpenCalc code: 58 calculator cases, 5 graph values, 10 exact rational matrix cases and 36 science values. Independent arithmetic/complex functions, analytic calculus, bisection, Fraction matrices and direct scientific formulas agree within the declared tolerances; inverses also satisfy exact identity products. The reviewer corrected an initial Unicode dispatch error in its own matrix checker; the fixture was unchanged. The 29 error expectations were reviewed analytically; their actual rejection is established by the application's 138-case reports.

Constants and conventions were checked against [BIPM defining constants](https://www.bipm.org/en/measurement-units/si-defining-constants), [NIST CODATA 2022](https://physics.nist.gov/cuu/Constants/Table/allascii.txt), [CIAAW 2024](https://ciaaw.org/abridged-atomic-weights.htm), [DLMF principal logarithms](https://dlmf.nist.gov/4.2), [OpenStax RLC impedance](https://openstax.org/books/university-physics-volume-2/pages/15-3-rlc-series-circuits-with-ac) and [25-degree-C pH relations](https://openstax.org/books/chemistry-2e/pages/14-2-ph-and-poh). Exact SI values, measured G and classroom atomic-mass approximations are distinguished.

IDs are unique and tolerances finite/nonnegative. Tiny nonzero outputs and small-scale matrices use zero absolute tolerance so zero cannot falsely pass. These checks concern stated mathematical models, not experimental uncertainty or significant figures. Teacher science fixtures compare the first reported quantity; this is not independent coverage of every secondary output or every graduate topic.

## Limits of the pass

CodeRabbit 0.9.0 was attempted with `review --agent --uncommitted --include-untracked`, but authentication/provider setup prevented review. It produced no findings and is not a passed CodeRabbit review; credentials were not changed.

Physical screen-reader/Braille/switch/eye-tracker, pen/pressure/touch, learner/classroom, clean Windows VM and broader-platform evaluation remain unperformed. Native synthetic tests and numerical self-tests do not substitute for those checks. This unsigned beta qualification supports a clearly labeled prerelease with those limits; it does not claim universal accommodation, medical effectiveness, full TI firmware parity or complete graduate curriculum coverage.

Related: [[release-readiness]], [[teacher-validation]], [[windows-release]], [[dependency-review]], [[notebook-format-v2]], [[inclusive-design]].
