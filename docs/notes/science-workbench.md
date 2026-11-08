---
tags: [science, chemistry, physics, verification]
updated: 2026-11-08
---

# Physics and chemistry workbench

Open **Tools → Physics & chemistry**. The workbench has 19 physics models and 13 chemistry models. Each model supplies editable examples, named quantities with units, a method, assumptions and a text result. Enter in the stated units; there is no automatic dimensional algebra. The result can be copied with its method or read through the app's existing local speech support. Audio is optional and every result/error is visible text.

Below 360 logical points wide, the current model's inputs come first. **Choose science tool** opens the search, subject filters and complete model browser; selecting a model closes the browser and focuses its first input. **Model & input notes** contains the same assumptions and numeric conventions shown inline in wider windows.

Selected university calculations include ideal series-RLC phasors, relativistic energy/momentum, photon energy, de Broglie wavelength and Boltzmann population ratios. These are representative calculations used in advanced study, not complete graduate coverage, symbolic field equations, a quantum simulator, a reaction balancer or an unrestricted chemistry solver. The calculator, graphing, matrices and calculus workspaces remain separate. See [[student-workflows]] and [[release-readiness]].

## Numerical and scientific conventions

- Numeric fields accept constant expressions and scientific notation. Variables and `Ans` are rejected to prevent silently substituting calculator state. Trigonometric functions inside these expressions use radians. The **Angle (degrees)** field in Work explicitly uses degrees.
- Decimal-comma input follows the accessibility preference; use semicolons to separate function arguments in that mode. Formula text is case-sensitive and unaffected by decimal preferences.
- Values use `f64`. Finite givens and finite reported values are required. Domain errors name the relevant quantity. The displayed precision is a decimal-place preference, not experimental significant figures. Uncertainty is not propagated.
- Exponential decay and transmittance may underflow to zero for extremely small results; those methods say so. Ratios that overflow produce an error. Extreme intermediate magnitudes can exceed the supported range even when an algebraic rearrangement could remain finite.
- The pH tools use ideal concentration approximations and `Kw = 10^-14` at 25 °C. The strong acid/base model is monoprotic/monobasic, assumes additive volumes, and includes water autoionization near equivalence. It does not solve weak acids, buffers, activities or arbitrary mixtures. pH is not clamped to 0–14.
- The ideal-gas tool uses kelvin and cubic meters and reports pascals. Avogadro's volume-law tool uses liters at unchanged temperature and pressure. No model assumes that a mole of every real gas always occupies the same volume.
- Classical de Broglie calculations are limited to `v ≤ 0.1c`, with the approximation stated. Relativistic energy requires positive rest mass and `0 ≤ v < c`, and computes kinetic energy without subtracting nearly equal total/rest energies.

## Sources and constants

The shared calculator constants are `N_A = 6.02214076e23 mol^-1`, `h = 6.62607015e-34 J s`, `kB = 1.380649e-23 J/K` and `c = 299792458 m/s`. The electronvolt conversion uses `e = 1.602176634e-19 C`. These SI defining values are exact; their binary floating-point representations are approximations. `R = N_A kB = 8.31446261815324 J/(mol K)`. [BIPM SI defining constants](https://www.bipm.org/en/measurement-units/si-defining-constants), [NIST SI definitions](https://www.nist.gov/pml/special-publication-330/sp-330-section-2).

Gravitation uses the measured `G = 6.67430e-11 m³/(kg s²)` from the [NIST CODATA 2022 table](https://physics.nist.gov/cuu/pdf/all.pdf), not an exact SI definition.

The physical models follow standard Newtonian mechanics, ideal electrical components and introductory thermal/modern physics. Sources include [OpenStax University Physics Volume 1](https://openstax.org/details/books/university-physics-volume-1), [series RLC circuits](https://openstax.org/books/university-physics-volume-2/pages/15-3-rlc-series-circuits-with-ac), [relativistic energy](https://openstax.org/books/university-physics-volume-3/pages/5-9-relativistic-energy), and [de Broglie matter waves](https://openstax.org/books/university-physics-volume-3/pages/6-5-de-broglies-matter-waves). Equations and interface copy were written for OpenCalc; textbook prose and illustrations are not bundled.

Chemistry references include [OpenStax molarity and dilution](https://openstax.org/books/chemistry-2e/pages/3-3-molarity), [pH and pOH](https://openstax.org/books/chemistry-2e/pages/14-2-ph-and-poh), [Arrhenius collision theory](https://openstax.org/books/chemistry-2e/pages/12-5-collision-theory), and the [USP Beer–Lambert definition](https://doi.usp.org/USPNF/USPNF_M6960_02_01.html). Model assumptions appear beside the givens rather than being inferred from the subject or learning level.

## Formula parser

The molar-mass table covers 84 elements with conventional atomic weights. It uses classroom approximations for elements with natural-isotope intervals (for example H 1.008, C 12.011, O 15.999, S 32.06 and Cl 35.45) and standard single-valued weights for the others. Al is 26.9815384, Na 22.98976928, Zr 91.222 and Gd 157.249. These are approximate natural-composition molar masses, not isotope-specific exact masses. [CIAAW Standard Atomic Weights 2024](https://ciaaw.org/atomic-weights.htm), [CIAAW abridged values](https://www.ciaaw.org/abridged-atomic-weights.htm).

Examples: `H2O`, `Ca(OH)2`, `Al2(SO4)3`, `K4[Fe(CN)6]` and `CuSO4·5H2O`. Unicode subscript digits are supported. A middle dot or period separates hydrate components; it is not a fractional atom count. Leading reaction coefficients, isotope labels, ion charges and equations must be omitted. Elements without supported standard weights (such as Tc) fail with an explanation.

There is no science file import or execution. Each field is limited to 512 Unicode characters. Formula nesting is limited to 16 levels, individual subscripts/hydrate coefficients to one million, and total atom count to one billion. Count arithmetic is checked before multiplication/addition. Numeric expressions use the existing bounded calculator parser. Science state is ephemeral; the workbench adds no saved user data or new dependency.

## Verification contract

`science::evaluate(id, inputs, decimal_comma)` is the same pure evaluator called by the native UI. `TOOLS` supplies stable IDs, field labels/examples and model assumptions; `ScienceReport.values` exposes numeric values and units to independent acceptance checks. Input order is exactly the visible field order. Multi-output first values are impedance magnitude for `series-rlc`, total energy for `relativistic-energy`, remaining amount for `radioactive-decay`, voltage for `rc-decay` and pH for `strong-acid-base`.

Module tests check all 32 examples, independently known mechanics/electrical values, SI conversions, a relativistic low-speed limit, nested/hydrate formulas, mass conversions, neutral and excess-base mixtures, Beer–Lambert transmittance, decimal-comma input, invalid scientific domains, overflow and parser resource limits. Separate teacher fixtures and UI-driven student/accessibility acceptance are maintained by their owners. Synthetic semantics and keyboard checks cannot establish compatibility with a person's actual screen reader, Braille display, switch or eye tracker.
