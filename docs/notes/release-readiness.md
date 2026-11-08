---
tags: [release, testing, curriculum, accessibility]
updated: 2026-11-08
---

# OpenCalc release-readiness pass

Target: `0.2.0-beta.1`, Windows x64. The local qualification gates pass for an unsigned beta. This is not certification of every mathematical topic or disability setup.

## Work and evidence

| Area | Work in this pass | Current evidence status |
| --- | --- | --- |
| Student | Native keyboard/pointer homework workflows across four grade bands; representative college/graduate science | All 11 student workflows pass, including physical-key events, graph/matrix workspaces, lessons, science forms, save/reopen and simulated restart |
| Teacher-style check | Independent Decimal/Fraction/analytic expected results with primary SI/academic sources | All 138 checks pass in the optimized, installed and portable executables; raw actuals independently compared; second reviewer checked all 109 positive expectations and 29 rejection conditions |
| Access | Visual, nonvisual semantics, keyboard, motor guards, quiet feedback, reading/focus combinations | All 16 synthetic native scenarios pass; physical-device and learner checks remain unperformed |
| Returning user | v1 migration, canvas pages/ink/text/shapes, optional recovery and restart tests | Model, real filesystem, async UI and recovery regressions pass; corrupt input keeps existing work, recovered drafts cannot silently replace the original file |
| Maintainer | Current dependency advisory check, notices, source/private-data review, clean build | Local gates and clean exported-source check pass; exact license bundle and staged-publication scan reviewed; four maintenance advisories documented; CodeRabbit unavailable |
| Distribution | Portable Windows build and per-user installer, isolated install/uninstall, hashes and beta release | Published as an unsigned prerelease; downloaded artifacts match their checksums and pass the complete real-binary installer/portable smoke |

## Executed gates

- Rust: **296 passed, 0 failed, 0 ignored**; formatting passes; all-target Clippy passes with warnings denied.
- Packaging helpers: **7 Python tests passed**. No dependency was added or upgraded.
- Curriculum: **138 passed, 0 failed**: 70 calculator, 7 graph, 14 matrix and 47 science checks, including 29 expected rejections. All 32 science tools have at least one fixture. The science oracle generally checks the first reported quantity; secondary outputs and broader curricula need further review.
- Windows: optimized x64 MSVC build with static CRT. PE normal/delay imports contain no separate Visual C++ redistributable requirement.
- Isolated real installation: 630 indexed installed files and 631 portable files hashed; both packaged executables pass the 138 checks and create native captures. Locked-file uninstall returns failure while retaining retry metadata; retry removes owned files and preserves synthetic notebook/profile sentinels. Test mode writes no registry or Start menu entries.
- Native captures reviewed at 800×900 and 360×800 with 125% zoom. The narrow science browser and notebook tools collapse so inputs/the page remain visible. Tests also exercise combined 400% interface zoom and 180% body text.
- Public source exported from Git's index builds without local vendor overrides, preferences or optional product photos.

Final application executable SHA-256: `c6ae02b32359dfa488c96d64ecd1d099dffc4e725c462a75cfd09dbe291cbbc8`.

The committed [raw curriculum report](../validation/curriculum-0.2.0-beta.1.json) has SHA-256 `176e6a32f7a1c286b9d18a47ea076fa9ac3581f7855d4b3acf6f0ee12c0a7802`. Release assets supply their own SHA256SUMS and payload manifests; generated installer/log/profile artifacts are excluded from Git.

## Published artifacts

[OpenCalc 0.2.0-beta.1](https://github.com/EvanGUTK/opencalc/releases/tag/v0.2.0-beta.1) was published at 2026-11-08 06:46:30 UTC as a public prerelease. The tag and package manifest identify verified source commit `b615e395c90adca46d074bf2b6dcd06130201f30`. [PR #5](https://github.com/EvanGUTK/opencalc/pull/5) remains open for maintainer approval; it was not merged automatically.

| Download | SHA-256 |
| --- | --- |
| Windows x64 installer | `d30f3c70597bebfbf967685fdf3ec95ed38a2c2bd8c8a6398e156b458f642f96` |
| Windows x64 portable ZIP | `ff55d1bae16b9c4d7d18f5dfd988f26125b0eb84e5af74d387f28fcff712204a` |
| Curriculum results JSON | `176e6a32f7a1c286b9d18a47ea076fa9ac3581f7855d4b3acf6f0ee12c0a7802` |

Both distributions were downloaded from GitHub and reran the full isolated installation/payload/self-test/launch/uninstall checks successfully. An unauthenticated public portable download also matched the ZIP checksum. The downloaded release app was opened interactively and reported a responding native window with its portable data directory created. The [sanitized Windows verification record](../validation/windows-0.2.0-beta.1.json) contains no local evidence/profile paths.

## Reproduced baseline defects

An isolated Rust probe compiled the original `624014a` math module and executed these cases without changing application files:

- `diff(abs(x),0)` returned zero; the ordinary derivative is undefined at the cusp.
- Formatting `1e-7` with two decimal places returned `0`, hiding a nonzero science value.
- `diag(1e-15,1e-15)` returned a zero determinant and rejected inversion; the expected determinant is `1e-30` and the inverse diagonal is `1e15`.
- `root(1/x,1)` already rejected the input in that baseline; it is a regression/domain test, not an executed baseline failure.

The corrected cases pass. Independent source review also identified pole-related graph-analysis errors and a build-checkout preference path unsuitable for installed releases. Both were repaired and covered by the local gate. Native workflows exposed Enter submission, modal focus, repeated dwell and scanning continuity bugs; their regressions now pass.

One existing loopback transport test exposed a Windows scheduling race: its accepted socket retained nonblocking mode. The test explicitly restores blocking mode with its existing five-second timeout before reading; all request/response assertions remain. Microsoft's [accept documentation](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept) describes inherited socket properties. This change affects the test server only.

## Testing boundaries

The Computer Use native pipe is unavailable and the OpenMob hub did not answer device discovery. Installed Narrator was identified by metadata; no screen reader or operating-system accessibility setting was enabled. Native egui tests and the packaged `--self-test` calculation report are separate evidence types. Neither proves real NVDA/Narrator/Braille/switch/stylus/mobile usability or represents feedback from a student or teacher.

No private homework, credentials or personal notebook file is part of the fixtures or release. Packaging uses a checksum-verified portable compiler in the ignored build directory; no global tool installation or paid signing is authorized. A Windows beta can be unsigned, and that status must be stated honestly without bypassing security prompts.

Still outstanding: actual learner/teacher observation, Narrator/NVDA/Braille/switch/eye-tracker/stylus trials, clean Windows VM and interactive installer registration, other operating systems, PDF annotation/OCR, mobile companions and synchronization. Science forms are session-only; copy their reports into notebook notes or a text object to save them. No claim of universal disability compatibility, firmware parity or complete graduate-level mathematics is made.

Related: [[teacher-validation]], [[student-workflows]], [[accessibility-release-check]], [[canvas-design]], [[notebook-format-v2]], [[windows-release]], [[maintainer-release-check]].
