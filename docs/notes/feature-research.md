---
tags: [research, calculator, accessibility, features]
updated: 2026-11-07
---

# Learning toolkit research

The existing app already provides arithmetic, trigonometry, numerical calculus, graphing, matrices, MATLAB sample export, local worked lessons, and optional local AI tutoring. This addition fills practical classroom gaps while keeping the supplied TI-84 face drawn from vector shapes and text. The handheld remains the startup screen; Apps opens the new toolkit and Stat opens statistics. No dependencies or network requests were added to the tools.

## Sources and decisions

| Primary source | Observation | Implementation decision |
| --- | --- | --- |
| [TI-84 Plus CE feature overview](https://education.ti.com/en/products/calculators/graphing-calculators/ti-84-plus-ce-python) | TI documents data/statistics and time-value-of-money workflows alongside graphing. | Add data summaries, least-squares regression, probability tools, and three financial exercises instead of duplicating existing graphing. |
| [NIST binomial distribution](https://www.itl.nist.gov/div898/handbook/eda/section3/eda366i.htm) | The binomial mass function describes exact successes; its cumulative function sums terms through a specified count. | Offer separately labeled exact and cumulative tools; require independent trials with constant p, valid integer counts, and p in [0,1]. |
| [NIST normal data](https://www.itl.nist.gov/div898/handbook/pmc/section5/pmc51.htm) | A normal distribution is parameterized by its mean and standard deviation. | Require positive standard deviation; expose both bounds and label probability as approximate. |
| [Desmos accessibility](https://www.desmos.com/accessibility) and [introduction to accessibility features](https://help.desmos.com/hc/en-us/articles/4404860698253-Introduction-to-Accessibility-Features) | Desmos documents keyboard access, spoken graph exploration, and Braille workflows. | Use native focusable controls, meaningful field labels, visible feedback, copyable text reports, and the app's existing local speech. Preserve contrast, readable fonts, and scaling. Braille and audio graph tracing are not implemented by this addition. |
| [CFPB amortization explanation](https://www.consumerfinance.gov/ask-cfpb/what-is-amortization-and-how-could-it-affect-my-auto-loan-en-771/) | Fixed amortizing payments combine principal and interest; their proportions change over time. | Show the fixed monthly payment, total repaid, and interest for an educational fixed-rate model. State that fees, insurance, and taxes are excluded. |

These observations guide the product selection; they do not establish TI firmware compatibility, exam acceptance, accessibility certification, or mathematical correctness of every possible input.

## Mathematical conventions

- Lists accept commas, semicolons, spaces, and new lines; each list is capped at 1,000 finite values. A full list is validated before producing a report.
- Quartiles use medians of the lower and upper halves, excluding the overall median for odd sample sizes. A one-value sample has both quartiles equal to that value. Sample variance is omitted for a one-value sample. Outliers lie strictly outside the 1.5-IQR fences. Modes show every tied repeated value; unique data have no repeated mode.
- Regression uses centered sums. X values must vary and list lengths must match. Correlation and R squared are omitted for constant Y. Regression is descriptive and does not establish causation.
- Combination, permutation, and binomial counts are capped at 170 to keep calculations bounded in double precision. Counts may display rounded scientific notation for large results; they are not arbitrary-precision integers.
- Normal probabilities use Abramowitz and Stegun approximation 26.2.17 with an error below approximately 0.000001 for a difference of CDFs. Very small tail differences may round to zero. This approximation is not suitable for extreme-tail statistical inference.
- Fraction operations, GCD, LCM, prime factorization, and base conversion accept whole inputs up to one billion in magnitude. Fraction numerators and denominators use i128 intermediate arithmetic and return exact reduced fractions; decimal equivalents are approximate. Zero denominators and gcd(0,0) are rejected.
- Quadratic coefficients are scaled to reduce overflow and use a cancellation-resistant formula. Real roots only. A zero leading coefficient becomes a linear equation. Two-equation systems with zero determinant have no unique solution.
- Length uses exact international-inch/foot/mile definitions. Mass uses the international pound, 0.45359237 kg. Temperatures below absolute zero are rejected. Angle conversion always starts from degrees, independent of calculator mode.
- Finance assumes nonnegative rates. Savings compound monthly with end-of-month deposits; loans have fixed monthly payments. Durations must be whole months, up to 12,000. Zero interest is handled directly. Log1p/expm1 formulas reduce cancellation at small rates.
- Reports clear when input changes, so an earlier answer is not displayed as if it belongs to edited values. Reset restores the selected tool's example. All nonfinite inputs and nonfinite numeric results are rejected with visible messages.
- Practice uses deterministic varied questions at four levels. Only the first submitted answer per question affects the session score. Revealing an answer counts as an attempt. No practice or tool data is added to saved preferences.

## Verification and remaining work

Automated tests cover every default tool example, known reference values, malformed data, probability boundaries, zero-rate finance, exact fraction/integer results, degenerate equations, numeric overflow, input bounds, native control navigation, clipboard output, and practice feedback. Existing calculator and tutor regressions remain part of the suite. Rebuild and fresh screenshots check that the original handheld presentation remains intact.

Actual screen-reader, switch-device, Braille-display, and disabled-learner usability testing remain open. Future research should prioritize audio graph tracing and tactile export with users who need them, richer practice curricula, complex roots, and interactive statistical plots. No claim is made that these future items are included in this toolkit.
