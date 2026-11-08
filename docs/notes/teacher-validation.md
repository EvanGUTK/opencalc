---
tags: [opencalc, validation, curriculum, mathematics, science]
updated: 2026-11-08
---

# Independent teacher-style answer acceptance set

Status: **138 of 138 independent value/rejection cases match the final optimized Windows release executable; 0 differ**. Both debug and release raw actuals were independently re-compared against the frozen oracle. This is a scoped mathematical answer result, not a complete product-release verdict. Exact executable and report hashes identify the uncommitted feature-worktree candidates below; student UI, accessibility, recovery and packaging have their separate gates.

Related: [[architecture]], [[decisions]], [[todo]].

## Frozen packet

- Criteria: `teacher-oracle-v1-2026-11-08`, representing the user's K–graduate representative math/physics/chemistry request.
- Starting revision: `624014ae70489e060a33ae53a5ad98d27ffb202a`; branch observed `feat/release-readiness`.
- Tracking: [release-readiness issue #4](https://github.com/EvanGUTK/opencalc/issues/4).
- Author: `teacher_oracle` specialist. These original acceptance examples do not constitute credentialed teacher accreditation or full curriculum alignment.
- Owned artifacts: `tests/fixtures/curriculum.json` and this report. Product source was read only. This author ran no Cargo/build commands.
- Fixture SHA-256: `701f8c50195eef4cae33291a1d41ed26f5f183c94448220e0442006ad6bcd563`.
- Total: 138 cases, including 29 explicit rejection cases.

| Learning stage | Cases |
|---|---:|
| k3 | 10 |
| middle | 16 |
| high | 23 |
| college | 75 |
| graduate_representative | 14 |

| Production path for the downstream runner | Cases |
|---|---:|
| calculator | 70 |
| graph | 7 |
| matrix | 14 |
| science | 47 |

Learning-level indices are0=K–3,1=4–8,2=9–12,3=Advanced. `stage` distinguishes college and graduate-representative examples inside Advanced. Topics are representative, not exclusive to these ages.

## Independent method and acceptance

Expected answers were derived outside the app with Python standard-library `Fraction` and `Decimal` at 60 decimal-digit precision, direct polynomial/complex/matrix identities and Python `math` for pi and independent 80-step bisection of cos(x)-x. No OpenCalc evaluator, scientific tool or numerical calculus implementation was used to generate expected answers. Examples are original rather than copied textbook exercises.

A finite result passes when `abs(actual-expected) <= abs_tolerance + rel_tolerance*abs(expected)`. Complex components and matrix entries are checked separately; matrix shapes must match. Numerical calculus has explicitly looser tolerances than exact arithmetic. Tiny nonzero scientific values have zero absolute tolerance so a rounded zero cannot accidentally pass.

Science input strings follow `science::evaluate(id, inputs, decimal_comma)`. Fixtures compare numeric `ScienceReport.values[0]`: RLC impedance magnitude, relativistic total energy including rest energy, remaining radioactive amount, mixed strong-acid/base pH, and RC voltage. Other quantities in those multi-output reports are outside this packet. Every scientific case states its assumptions or derivation.

Error `code_class` describes the mathematical reason to reject, not exact text. A nonempty error is a minimum check; useful wording still needs review. Real-domain fixtures stay real. Complex fixtures explicitly request complex mode and principal branches. Graph fixtures test coordinates and angle conventions, not rendering, axes or discontinuity segmentation.

Artifact checks actually executed on 2026-11-08: valid JSON with `allow_nan=False`, unique stable IDs, resolved source references, level bounds, science input array types, nonnegative tolerances, independent rational identities, tiny determinant arithmetic and bisection consistency. These check the fixture packet, not the application.

## Scientific sources and assumptions

[BIPM defining constants](https://www.bipm.org/en/measurement-units/si-defining-constants) supply exact c, h, k_B and N_A values. This packet derives R=N_A*k_B=8.31446261815324 J/(mol K). Their decimal definitions are exact; binary arithmetic can still round.

[CIAAW's abridged table](https://ciaaw.org/abridged-atomic-weights.htm) supports conventional H 1.008, O 15.999 and Ca 40.078 used here. Molar masses follow those conventions rather than asserting an exact isotope-independent value for every natural sample. [NIST](https://www.nist.gov/how-do-you-measure-it/how-do-you-measure-strength-gravity) supplies the nominal measured G=6.67430e-11; it is not treated as exact experimentally.

Gas examples assume ideal gases, kelvin and consistent units; pressure input volume is cubic metres. pH assumes ideal activities, monoprotic strong acid/base, additive volumes and 25 ?C with teaching Kw=1e-14. Neutralization includes water autoionization, so equivalence yields pH 7. Arrhenius assumes unchanged activation energy and pre-exponential factor. Beer-Lambert assumes dilute solution and monochromatic light. The Boltzmann example assumes equal degeneracy and per-particle energy.

All external references are primary: BIPM, CIAAW, NIST, OpenStax, MIT course material or the University of Delhi Chemistry department. Direct formula URLs are stored in the fixture's `sources`. The IUPAC Beer-Lambert page returned HTTP 403; the readable Delhi teaching PDF supports that relation instead. The historical MIT lecture's distribution formula is used with current BIPM constants.

## Baseline findings and repair requirements

The root coordinator executed an isolated copy of starting-revision `math.rs` and supplied the results on 2026-11-08. This is **coordinator-provided baseline evidence**, not integrated candidate acceptance and not execution by this author. The author subsequently read the supplied `results.txt` artifact; its SHA-256 is `0acb802c72c605ed47600c5f27c3284c2b0cbf7f691f3092f18ae8890eff8d17`. It reports `diff(abs(x),0)=>Ok(0)`, formatting `1e-7` at precision 2 as `0`, tiny-diagonal determinant `0`, and inverse rejection. These disagree with the analytic expectations below. Root also confirmed the original `root(1/x,1)` already rejects; that case is a regression guard, not a baseline defect.

| Baseline reproduction | Independent expected behavior | Case or related guard | Candidate status |
|---|---|---|---|
| A=diag(1e-15,1e-15), determinant/inverse | det=1e-30; inverse=diag(1e15,1e15) | `matrix-small-determinant`, `matrix-small-inverse` | Matches debug and final release candidates |
| `diff(abs(x),0)` | Error: one-sided derivatives disagree | `error-derivative-corner` | Matches debug and final release candidates |
| Format 1e-7 at precision 2 | Visible nonzero scientific value | `high-tiny-result` protects a related nonzero value; exact precision 2 display is outside this fixture | Release guard displays `1e-24`; exact original precision setting is not certified by this packet |
| `root(1/x,1)` | Error: no finite zero | `error-root-asymptote` | Baseline, debug and final release reject |

The packet also covers an ordinary integral across a pole, singular matrices, malformed chemical formulas and physical domains. These are acceptance requirements, not assertions that every case was broken. Expectations must not be loosened merely to obtain a passing run.

## Execution evidence and limits

The root coordinator executed the actual native debug binary with `opencalc.exe --self-test` and redirected stdout using PowerShell `Set-Content -Encoding UTF8` to `target/curriculum-initial.json`. The coordinator reported binary exit code 0. The report contains all 138 unique fixture IDs, 138 passed and 0 failed; its application version is `0.2.0-beta.1`. This command exercises production calculator evaluation, graph-coordinate, matrix and science paths. It does not itself drive native widgets.

Evidence reviewed on 2026-11-08:

- Candidate: uncommitted `feat/release-readiness` worktree based on `624014ae70489e060a33ae53a5ad98d27ffb202a`; no claim of a committed candidate revision.
- Debug executable SHA-256, independently read and hashed by this author: `f5c1bae0594439758821f5f8585abf9f4edef6b7d112cbae3f93ff60876206a2`.
- Initial raw report SHA-256: `27fac0cdeb981a7b046f046e949b7ae7cd9a2a5a8cd3c1b17683bec6bb4a3cbe` (UTF-8 with BOM; decoded as `utf-8-sig`).
- Fixture SHA-256 unchanged: `701f8c50195eef4cae33291a1d41ed26f5f183c94448220e0442006ad6bcd563`.
- This author independently re-compared every raw actual result against the separately loaded frozen fixture with Python `Decimal` at 60-digit precision, ignoring the report's pass flags and embedded expected values: **138 matched, 0 differed**. Shape/type/error checks were repeated; every nonzero real calculator result also had a nonzero numeric display.
- Further oracle spot checks: exact rational multiplication confirms the proposed 3?3 inverse gives identity; Decimal arithmetic rechecks ideal-gas pressure and photon energy; analytic differentiation gives 88; independent bisection gives the cosine fixed point. None uses the app to derive an expectation.

Selected actual results make the practical tolerances visible:

| Case | Independent expected | Initial debug actual | Result |
|---|---|---|---|
| `high-tiny-result` | 1e-24 | value 1e-24; display `1e-24` | Match |
| `matrix-small-determinant` | 1e-30 | 1e-30 | Match |
| `matrix-small-inverse` | diagonal 1e15 | diagonal 999999999999999.9 | Within relative tolerance |
| `college-polynomial-derivative` | 88 | 88.00000001173393 | Within numerical derivative tolerance |
| `college-log-derivative` | 0.5 | 0.5 | Match |
| `college-sine-integral` | 2 | 1.9999999999999893 | Within numerical integration tolerance |
| `college-square-root-equation` | sqrt(2) ? 1.4142135623730951 | 1.4142135623746899 | Within numerical root tolerance |
| `college-trig-fixed-point` | 0.7390851332151607 | 0.7390851332151607 | Match |
| `physics-ideal-gas-pressure` | 103930.7827269155 Pa | 103930.7827269155 Pa | Match |
| `physics-photon-energy` | 3.972891714297857e-19 J | 3.972891714297857e-19 J | Match |
| `physics-debroglie` | 6.62607015e-10 m | 6.62607015e-10 m | Match |
| `physics-relativistic-total-energy` | 1.123443973421022e17 J | 1.123443973421022e17 J | Match |
| `chem-half-mole-particles` | 3.01107038e23 entities | 3.01107038e23 entities | Match |
| `chem-arrhenius-rate-ratio` | 6.1861220848555 approximately | 6.186122084855502 | Within tolerance |
| `chem-neutralization-equivalence` | pH 7 | pH 7 | Match |
| `error-derivative-corner` | Reject undefined two-sided derivative | Nonempty derivative-domain error | Match |
| `error-root-asymptote` | Reject nonexistent finite root | Nonempty convergence error | Match |
| `error-integral-pole` | Reject ordinary divergent integral | Nonempty finite-interval-domain error | Match |

### Final optimized release evidence

The coordinator ran the optimized Windows x86_64 MSVC executable built with static CRT using `opencalc.exe --self-test`, invoking the actual process with `Start-Process -Wait` and stdout redirection. The coordinator explicitly observed process `ExitCode` **0**. The raw report is published at [curriculum-0.2.0-beta.1.json](../validation/curriculum-0.2.0-beta.1.json), application version `0.2.0-beta.1`, with all 138 unique case IDs, 138 passed and 0 failed.

The teacher-oracle author then independently read and hashed the release executable, fixture and raw report, and repeated the full Python `Decimal` comparison of each raw actual against the frozen fixture, ignoring report pass flags and embedded expectations: **138 matched, 0 differed**. Error/type/shape checks and nonzero numeric-display guards were repeated. The first-value science quantities remain the agreed contract.

| Immutable release evidence | SHA-256 |
|---|---|
| Final optimized release executable | `c6ae02b32359dfa488c96d64ecd1d099dffc4e725c462a75cfd09dbe291cbbc8` |
| Raw release curriculum report | `176e6a32f7a1c286b9d18a47ea076fa9ac3581f7855d4b3acf6f0ee12c0a7802` |
| Frozen independent fixture | `701f8c50195eef4cae33291a1d41ed26f5f183c94448220e0442006ad6bcd563` |

The final executable was rebuilt after UI polish. Its new hash was independently read and checked, and its raw curriculum report retains SHA-256 `176e6a32f7a1c286b9d18a47ea076fa9ac3581f7855d4b3acf6f0ee12c0a7802`; the frozen fixture is unchanged.

The release results reproduce the selected debug actuals above: `1e-24` stays nonzero in the display, tiny determinant is `1e-30`, inverse diagonal is `999999999999999.9`, photon energy is `3.972891714297857e-19 J`, half a mole is `3.01107038e23` entities, and polynomial derivative is `88.00000001173393`. The undefined derivative, nonexistent finite root and divergent ordinary integral are rejected with nonempty explanatory messages.

Packaging smoke evidence also records separate installed and portable executable runs, each with 138 matching cases and 0 failures under the same final executable hash. This author read both raw self-test reports and confirmed that all 138 actual records in each exactly equal the already independently checked public release report. The smoke summary `results.json` has SHA-256 `560b6fdb436730e9a0e46bb8594e12ba75d0a79ca06f65215547c69e1baee2a2`. This records review of the packaging owner's execution evidence; this author did not personally run the installer.

This author ran no Cargo/build commands and did not independently execute the complete native UI suite. The coordinator separately reports 296 Rust tests passing and clean formatting/Clippy after addressing the earlier Enter-focus defect. Those are coordinator-reported integration results, not proof of assistive-hardware behavior or a release verdict from this mathematical packet. The source worktree still had base HEAD `624014ae70489e060a33ae53a5ad98d27ffb202a` when reviewed; executable hashes, rather than an invented new commit, identify the evaluated candidates. Any fixture or executable change requires fresh evidence.

These value checks do not establish every keypad key, lesson pedagogy, expression editing, native E2E use, screen-reader/Braille/switch hardware, notebook preservation, crash recovery or installer behavior. Those require their separate workflow and release evidence. Graduate-representative complex/statistical/relativistic examples do not establish complete graduate physics, symbolic algebra, PDEs, uncertainty propagation or all chemistry courses.

Rollback: remove these two new artifacts before integration; no app settings, notebooks or user files were changed.
