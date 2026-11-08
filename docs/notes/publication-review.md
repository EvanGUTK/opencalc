---
tags: [release, security, verification]
updated: 2026-11-07
---

# Initial public source review

The initial Equal source snapshot uses the app directory as its repository root. The original code and original artwork use MIT. Third-party fonts and optional product references are separately described in THIRD_PARTY_NOTICES.md.

## Local evidence

- 171 tests passed, with no skipped tests.
- Formatting, Clippy on all targets with warnings denied, and a native Windows build passed.
- Native screenshots checked the large-key calculator, Light/Dark settings, live complex result, graph workspace, on-calculator lists/programs and 400% compact navigation. Settings uses viewport height while keeping Close available.
- Native egui interaction tests cover physical-key workflows, math domains, editing/clipboard events, overlay input shielding/focus return, dwell/scanning guards, locale input and narrow settings bounds.
- The updated native app was launched and its window was responding. Real NVDA, Narrator, Braille, switch, eye-tracker, live speech pronunciation and disabled-learner testing remain outstanding.

## Publication boundary

Only source, lockfile, project documentation, original banner/app screenshots and required font-license texts are intended for publication. Build products, vendored dependencies, dependency caches, the local Cargo vendor override, logs, preferences and optional TI photos stay local. Incidental reference-fetch scripts are excluded. No credentials are required for the core app.

The staged source is checked for high-confidence secret patterns without printing matching values. Such a scan cannot prove the absence of all sensitive data or application defects. Dependency advisory evidence is recorded in [[dependency-review]]: four informational maintenance advisories require follow-up, and no other matches were returned for the exact locked versions.

A clean copy exported from the Git index passed `cargo check --locked` without the excluded vendor configuration, vendored sources or optional TI image files. Public build instructions use normal Cargo registry dependencies; offline builds require a populated dependency cache.

No remote CI workflow, model weights, TI firmware, proprietary font file or executable is included in the source publication. Full TI compatibility and universal accessibility are not claimed. See [[calculator-accessibility]] and [[inclusive-design]].
