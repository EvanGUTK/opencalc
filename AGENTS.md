# OpenCalc

OpenCalc is a native Rust/eframe/egui desktop app, not a Python project. The source repository root is this directory; upstream is https://github.com/EvanGUTK/opencalc.

## Confirmed product preferences

- Keep the calculator casing, LCD, key geometry and functional legends unchanged when editing branding. Calculator faces have no manufacturer/model lettering or logos; the app name is OpenCalc outside the face.
- Use plain descriptive copy, compact side-by-side layouts, and configurable accessibility. Preserve mathematical behavior and assistive input when adding features.
- After each verified milestone, commit and push the scoped branch without asking again. Use feature branches and reviewable PRs; the user reserves PR approval and merging.
- Do not add remote CI workflows. Run formatting, tests, Clippy and builds locally, then launch the native app for review.
- Never commit notebook files, preferences, logs, caches, executables, private data or optional TI photographs. Keep third-party font notices intact.

## Verification

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

On the original Windows workspace, the ignored `.cargo/config.toml` uses local vendored crates. Set `CARGO_HOME` to `.cargo-cache`, use `--offline`, and choose a separate temporary `--target-dir` if Windows locks an existing build. Public source omits the vendor override and uses normal registry dependencies.

The app has a native `--screenshot PATH --size WIDTHxHEIGHT` capture mode. Core UI behavior is also checked with native egui interaction tests. Neither screenshots nor synthetic tests establish actual screen-reader, Braille, switch or eye-tracker compatibility.
