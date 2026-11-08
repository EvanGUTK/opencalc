# Contributing to Equal

Small, focused contributions are welcome. Open an issue describing a reproducible problem or proposed behavior before a large change. For access issues, describe the task, relevant settings and assistive-tool versions; a diagnosis or personal identity is not needed. Do not include private schoolwork or personal information.

Use a feature branch and keep the calculation, keyboard and assistive behavior consistent. Add a regression for a behavior change. Do not weaken tests or describe an untried assistive setup as verified.

Run local checks before submitting a pull request:

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked
```

Run the native app and try the changed behavior at a narrow window width. There is no required remote CI workflow. Explain what changed, what you checked, and any remaining limitations in the pull request.

Contributions to original project code are under the [MIT license](LICENSE). Only contribute material you have the right to distribute. TI photos, firmware, ROMs, proprietary fonts and model weights must not be added. Preserve applicable third-party notices.
