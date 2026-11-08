---
tags: [windows, release, packaging]
updated: 2026-11-08
---

# Windows release packaging

OpenCalc's Windows beta has two distributions: a per-user installer and a portable ZIP. The installer requests no administrator rights, offers a destination beneath `%LOCALAPPDATA%\Programs\OpenCalc`, and adds current-user Start menu/uninstall entries. It does not register file associations. The portable ZIP includes `opencalc.portable`; normal app runs use the adjacent `data` directory. Installed runs use `%LOCALAPPDATA%\OpenCalc`. Explicit `--data-dir PATH` overrides both. Uninstall removes only the generated owned-file list, then empty directories; personal notebook files and profile/recovery data remain.

This is an unsigned beta. Artifact hashes identify downloaded bytes; they do not establish publisher identity. Windows reputation prompts and actual fresh-machine installation remain distinct from the local isolated smoke checks described below. No security setting or OS prompt is bypassed by these scripts.

## Tool provenance

[NSIS 3.13 is the current stable release](https://sourceforge.net/projects/nsis/files/NSIS%203/3.13/). Its download endpoints returned HTML in this environment, and those responses were rejected by their digest mismatch before extraction or execution. The verified local fallback is the complete portable NSIS 3.12 ZIP, downloaded from a MacPorts mirror and checked against the [official NSIS 3.12 SHA256](https://sourceforge.net/projects/nsis/files/NSIS%203/3.12/nsis-3.12.zip/download):

```text
56581f90db321581c5381193d796fffcf2d24b2f8fed2160a6c6a3baa67f2c4f
```

The compiler remains under ignored `target/packaging-tools/`; no global package is installed. The installer uses zlib compression and carries the complete distribution's `COPYING` notice in `assets/licenses/NSIS-COPYING.txt`. [NSIS's license appendix](https://nsis.sourceforge.io/Docs/AppendixI.html) describes its component terms.

## Dependency notices

The bundle indexes all 345 registry-package versions in the current lockfile. It includes 619 unchanged upstream license/notice files for 344 packages, including build and other-platform notice texts as a redistribution superset. The sole omission is Apple-only `dispatch 0.2.0`, whose archive and exact upstream commit contain no standalone license notice. The index explicitly records its exclusion from the Windows binary; `cargo tree --offline --target x86_64-pc-windows-msvc -i dispatch@0.2.0` returned “nothing to print” on 2026-11-08.

`packaging/license-supplements.json` pins 31 immutable upstream notice files with SHA256 digests for 51 crate archives that omit their notice texts. URLs use the published crates' exact `.cargo_vcs_info.json` commit IDs. Fetching only downloads text, enforces the pins, and never runs downloaded content. Vendored source remains unchanged. Four embedded font license texts and OpenCalc's MIT/third-party notices are also included.

```powershell
python scripts/fetch-license-supplements.py --manifest packaging/license-supplements.json --destination target/license-inputs
python scripts/collect-dependency-licenses.py --lock Cargo.lock --vendor vendor --supplements target/license-inputs/license-supplements.json --output target/dependency-licenses
python scripts/test-packaging.py
```

Use a new destination for each collection. The collector fails before staging if a locked package lacks an accounted notice, an upstream input digest changes, or a path escapes its reviewed source root. Packaging copies only explicitly indexed notice paths plus the index and its README; incidental files in the source directory are excluded.

## Build and package

Root integration owns release builds and sign-off. To avoid an additional Visual C++ runtime prerequisite, build the Windows MSVC executable with static CRT and inspect its PE import table before publishing. Static CRT does not bundle the operating system's DLLs. No compiler runs inside the packaging script.

```powershell
$env:CARGO_ENCODED_RUSTFLAGS = '-C' + [char]31 + 'target-feature=+crt-static'
cargo build --release --locked --target x86_64-pc-windows-msvc
scripts/package-windows.ps1 -Executable target/x86_64-pc-windows-msvc/release/opencalc.exe -Version 0.2.0-beta.1 -Makensis target/packaging-tools/nsis-3.12-extracted/nsis-3.12/makensis.exe -DependencyLicenseDirectory target/dependency-licenses -OutputDirectory target/releases
python scripts/inspect-pe-imports.py target/x86_64-pc-windows-msvc/release/opencalc.exe --require-static-crt
```

The exact version must match `Cargo.toml`; the executable must be an x86_64 PE image, and the notice index must match `Cargo.lock`. Package artifacts contain only the executable, README, app/third-party notices, embedded-font/compiler notices, dependency notices, and a hashed file manifest. Optional manufacturer photos, preferences, notebooks, logs, credentials, build caches, and source configuration are excluded. ZIP entry names and timestamps are stable for identical inputs. NSIS setup bytes also depend on the compiler and input files; do not claim bit-for-bit reproducible native builds without independently rebuilding them.

## Isolated install/uninstall check

```powershell
scripts/smoke-install-windows.ps1 -Installer target/releases/OpenCalc-0.2.0-beta.1-windows-x86_64-setup.exe -PortableArchive target/releases/OpenCalc-0.2.0-beta.1-windows-x86_64-portable.zip -EvidenceDirectory target/install-smoke
```

The script creates a unique evidence directory, runs `/S /TESTPROFILE /D=...`, verifies every listed payload hash, and runs the installed and portable executables' `--self-test`. A direct `System.Diagnostics.Process` retains the native handle for very fast GUI exits; asynchronous stdout/stderr copies capture both streams into separate evidence files without a pipe deadlock. Both JSON reports must have 138 passed checks, zero failures, 138 rows, and the expected application version; the evidence records each actual executable's SHA256. The script then launches both screenshot modes and uninstalls. These checks exercise production calculation code in the packaged executable; they do not establish native accessibility usability.

`/TESTPROFILE` never writes registry entries, Start menu shortcuts, or file associations. The check locks `opencalc.exe` during uninstall and requires a nonzero result with ownership/uninstaller metadata preserved for retry. It then repairs the installation, adds a personal notebook and adjacent data sentinel, and requires both to survive successful uninstall. Evidence remains under the specified directory. There is no recursive directory removal in the installer, uninstaller, or smoke script.

On 2026-11-08 the synthetic-payload packaging smoke passed silent installation, closed-file-list hashing, locked-file uninstall failure/retry, successful uninstall, and both personal-file preservation checks. A deliberately unindexed source sentinel did not enter either artifact. This synthetic run intentionally skipped app launch/self-test and is not a publishable package. Seven Python notice/input/PE inspection regression checks also passed.

The final static-CRT executable, SHA256 `c6ae02b32359dfa488c96d64ecd1d099dffc4e725c462a75cfd09dbe291cbbc8`, returned 138 passed/zero failed checks in both the real installed and portable distributions. Their 630/631 indexed payload files verified, both created native captures, and locked-file uninstall failure/retry plus personal-file preservation passed. Its x86_64 PE normal/delay import tables contain no separate Visual C++ redistributable requirement. See [[release-readiness]]. These checks do not replace a clean Windows VM, signed publisher validation, or observed student and assistive-device trials.

Related: [[maintainer-release-check]], [[publication-review]], [[dependency-review]], [[notebooks]].
