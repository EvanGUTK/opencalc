//! Small, explicit physical models with SI givens and auditable numeric results.
use crate::{EqualApp, appearance, egui, math};
use std::collections::BTreeMap;
use std::f64::consts::{LN_2, PI};

const MAX_INPUT_CHARS: usize = 512;
const MAX_FORMULA_DEPTH: usize = 16;
const MAX_ATOM_COUNT: u64 = 1_000_000_000;
const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;
const GRAVITATIONAL_CONSTANT: f64 = 6.674_30e-11;
const WATER_ION_PRODUCT: f64 = 1e-14;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FieldKind {
    Number,
    Formula,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ScienceField {
    pub label: &'static str,
    pub example: &'static str,
    pub kind: FieldKind,
}

const fn number(label: &'static str, example: &'static str) -> ScienceField {
    ScienceField {
        label,
        example,
        kind: FieldKind::Number,
    }
}
const fn formula(example: &'static str) -> ScienceField {
    ScienceField {
        label: "Chemical formula",
        example,
        kind: FieldKind::Formula,
    }
}

pub(crate) struct ScienceTool {
    pub id: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub help: &'static str,
    pub fields: &'static [ScienceField],
}

pub(crate) const TOOLS: &[ScienceTool] = &[
    ScienceTool {
        id: "newton-force",
        name: "Newton's second law",
        group: "Physics",
        help: "F = m a. Net force along one axis for constant mass in an inertial frame. Acceleration can be signed; mass cannot be negative.",
        fields: &[number("Mass (kg)", "2"), number("Acceleration (m/s²)", "3")],
    },
    ScienceTool {
        id: "kinetic-energy",
        name: "Kinetic energy",
        group: "Physics",
        help: "K = ½ m v². Classical translational kinetic energy; use the relativistic tool when speed approaches the speed of light.",
        fields: &[number("Mass (kg)", "2"), number("Velocity (m/s)", "3")],
    },
    ScienceTool {
        id: "momentum",
        name: "Linear momentum",
        group: "Physics",
        help: "p = m v. Classical momentum along one axis. Velocity and momentum retain their direction through their sign.",
        fields: &[number("Mass (kg)", "2"), number("Velocity (m/s)", "3")],
    },
    ScienceTool {
        id: "work",
        name: "Work by a constant force",
        group: "Physics",
        help: "W = F d cos(θ). Force and displacement magnitudes are nonnegative; their angle is explicitly in degrees, regardless of calculator mode.",
        fields: &[
            number("Force magnitude (N)", "10"),
            number("Displacement magnitude (m)", "2"),
            number("Angle (degrees)", "60"),
        ],
    },
    ScienceTool {
        id: "power",
        name: "Average power",
        group: "Physics",
        help: "P = W / Δt. Signed work divided by a strictly positive elapsed time gives average power.",
        fields: &[number("Work (J)", "120"), number("Elapsed time (s)", "4")],
    },
    ScienceTool {
        id: "gravitation",
        name: "Gravitational attraction",
        group: "Physics",
        help: "F = G m₁ m₂ / r². Magnitude for point masses or spherical bodies outside their surfaces. Separation is measured between centers. G = 6.67430 × 10⁻¹¹ m³/(kg s²), a measured CODATA value.",
        fields: &[
            number("First mass (kg)", "5.972e24"),
            number("Second mass (kg)", "1"),
            number("Center separation (m)", "6.371e6"),
        ],
    },
    ScienceTool {
        id: "wave-speed",
        name: "Wave speed",
        group: "Physics",
        help: "v = f λ. Positive frequency and wavelength describe a periodic wave in the same medium.",
        fields: &[
            number("Frequency (Hz)", "440"),
            number("Wavelength (m)", "0.78"),
        ],
    },
    ScienceTool {
        id: "ohms-law",
        name: "Ohm's law",
        group: "Physics",
        help: "I = V / R. Ideal ohmic resistor with positive resistance. Voltage and current can be signed using the same reference direction.",
        fields: &[number("Voltage (V)", "12"), number("Resistance (ohm)", "6")],
    },
    ScienceTool {
        id: "electric-power",
        name: "Electrical power",
        group: "Physics",
        help: "P = V I. DC or instantaneous power; positive power means absorption under the passive sign convention. For AC average power use the RLC tool.",
        fields: &[number("Voltage (V)", "12"), number("Current (A)", "2")],
    },
    ScienceTool {
        id: "capacitor-energy",
        name: "Capacitor charge & energy",
        group: "Physics",
        help: "U = ½ C V² and Q = C V. An ideal linear capacitor has positive capacitance; charge follows the chosen voltage sign.",
        fields: &[
            number("Capacitance (F)", "0.001"),
            number("Voltage (V)", "10"),
        ],
    },
    ScienceTool {
        id: "rc-decay",
        name: "RC discharge",
        group: "Physics",
        help: "V(t) = V₀ exp(−t/RC); time constant τ = RC. Ideal capacitor discharges through a positive resistor, with no voltage source connected. Time is nonnegative.",
        fields: &[
            number("Initial voltage (V)", "10"),
            number("Resistance (ohm)", "1000"),
            number("Capacitance (F)", "0.001"),
            number("Elapsed time (s)", "1"),
        ],
    },
    ScienceTool {
        id: "series-rlc",
        name: "Series RLC phasor",
        group: "Physics",
        help: "Z = R + j(ωL − 1/ωC), ω = 2πf. Sinusoidal steady state with ideal series components and RMS source voltage. The phase is voltage relative to current in degrees. Zero ideal impedance is rejected.",
        fields: &[
            number("Resistance (ohm)", "3"),
            number("Inductance (H)", "0.01"),
            number("Capacitance (F)", "0.001"),
            number("Frequency (Hz)", "50"),
            number("RMS voltage (V)", "10"),
        ],
    },
    ScienceTool {
        id: "ideal-gas",
        name: "Ideal gas pressure",
        group: "Physics",
        help: "P = n R T / V. SI units: volume in cubic meters, temperature in kelvin, pressure in pascals. Assumes an ideal gas; low temperatures or high densities can require a real-gas model.",
        fields: &[
            number("Amount (mol)", "1"),
            number("Temperature (K)", "273.15"),
            number("Volume (m³)", "0.022414"),
        ],
    },
    ScienceTool {
        id: "heat",
        name: "Sensible heat",
        group: "Physics",
        help: "Q = m c ΔT. Constant specific heat, no phase change and no other energy transfer. A temperature difference has the same numerical value in kelvin and degrees Celsius.",
        fields: &[
            number("Mass (kg)", "2"),
            number("Specific heat (J/(kg K))", "4184"),
            number("Temperature change (K)", "10"),
        ],
    },
    ScienceTool {
        id: "photon-energy",
        name: "Photon energy",
        group: "Physics",
        help: "E = h c / λ and f = c / λ. Vacuum wavelength in meters. SI Planck constant and speed of light are exact definitions; electronvolt conversion uses the exact elementary charge.",
        fields: &[number("Vacuum wavelength (m)", "500e-9")],
    },
    ScienceTool {
        id: "de-broglie",
        name: "de Broglie wavelength",
        group: "Physics",
        help: "λ = h / (m v). Positive rest mass and speed; this nonrelativistic approximation is limited here to v ≤ 0.1 c. For higher speeds obtain relativistic momentum and use λ = h/p.",
        fields: &[
            number("Mass (kg)", "9.1093837139e-31"),
            number("Speed (m/s)", "1e6"),
        ],
    },
    ScienceTool {
        id: "relativistic-energy",
        name: "Relativistic energy & momentum",
        group: "Physics",
        help: "E = γ m c², K = (γ − 1)m c² and p = γ m v, where γ = 1/√(1 − v²/c²). Positive rest mass, 0 ≤ v < c. K uses an equivalent expression to avoid cancellation at small speeds.",
        fields: &[
            number("Rest mass (kg)", "1"),
            number("Speed (m/s)", "0.6*lightspeed"),
        ],
    },
    ScienceTool {
        id: "radioactive-decay",
        name: "Half-life decay",
        group: "Physics",
        help: "N(t) = N₀ exp(−ln(2)t/t½). Single exponential decay without replenishment. Elapsed time and half-life must use the same time unit; remaining amount uses the initial amount's unit.",
        fields: &[
            number("Initial amount", "800"),
            number("Elapsed time", "30"),
            number("Half-life", "10"),
        ],
    },
    ScienceTool {
        id: "boltzmann-ratio",
        name: "Boltzmann population ratio",
        group: "Physics",
        help: "N₂/N₁ = exp(−ΔE/(kB T)). Thermal equilibrium with equal state degeneracies. The signed energy difference is E₂ − E₁; temperature is positive kelvin.",
        fields: &[
            number("Energy difference (J)", "1.380649e-23*300"),
            number("Temperature (K)", "300"),
        ],
    },
    ScienceTool {
        id: "moles-particles",
        name: "Moles to particles",
        group: "Chemistry",
        help: "N = n N_A. One mole contains exactly 6.02214076 × 10²³ specified entities. Identify whether the entities are atoms, molecules, ions or formula units.",
        fields: &[number("Amount (mol)", "2")],
    },
    ScienceTool {
        id: "particles-moles",
        name: "Particles to moles",
        group: "Chemistry",
        help: "n = N / N_A. Particle count must refer to specified entities; positive fractional counts can represent an estimated mean.",
        fields: &[number("Particle count", "6.02214076e23")],
    },
    ScienceTool {
        id: "avogadro-volume",
        name: "Avogadro's volume law",
        group: "Chemistry",
        help: "V₂ = V₁ n₂ / n₁. Ideal gas at unchanged temperature and pressure. Both volumes use liters; the initial amount and initial volume must be positive.",
        fields: &[
            number("Initial volume (L)", "2"),
            number("Initial amount (mol)", "1"),
            number("Final amount (mol)", "3"),
        ],
    },
    ScienceTool {
        id: "molar-mass",
        name: "Molar mass from formula",
        group: "Chemistry",
        help: "Sum the conventional atomic weights multiplied by atom counts. Supports case-sensitive element symbols, nested parentheses/brackets, integer subscripts and hydrates such as CuSO4·5H2O. Uses approximate natural-isotope classroom masses, not isotope-specific masses.",
        fields: &[formula("H2O")],
    },
    ScienceTool {
        id: "mass-moles",
        name: "Grams to moles",
        group: "Chemistry",
        help: "n = mass / molar mass. Enter a nonnegative mass in grams and a molecular or formula-unit composition. Assumes a pure sample with the conventional isotope composition.",
        fields: &[number("Mass (g)", "36.03"), formula("H2O")],
    },
    ScienceTool {
        id: "moles-mass",
        name: "Moles to grams",
        group: "Chemistry",
        help: "mass = n × molar mass. Enter a nonnegative amount in moles and the composition of the specified entities.",
        fields: &[number("Amount (mol)", "2"), formula("H2O")],
    },
    ScienceTool {
        id: "molarity",
        name: "Molarity",
        group: "Chemistry",
        help: "c = n / V. Amount of solute in moles divided by the final solution volume in liters, not the volume of solvent alone.",
        fields: &[
            number("Solute amount (mol)", "0.5"),
            number("Solution volume (L)", "2"),
        ],
    },
    ScienceTool {
        id: "dilution",
        name: "Solution dilution",
        group: "Chemistry",
        help: "c₂ = c₁ V₁ / V₂. Moles of solute stay fixed when solvent is added, with no reaction or loss. Final solution volume must be at least the initial volume.",
        fields: &[
            number("Initial concentration (mol/L)", "2"),
            number("Initial volume (L)", "0.1"),
            number("Final volume (L)", "0.5"),
        ],
    },
    ScienceTool {
        id: "ph",
        name: "pH from hydronium",
        group: "Chemistry",
        help: "pH = −log₁₀[H₃O⁺]. Ideal dilute-solution concentration approximation at 25 °C, Kw = 10⁻¹⁴. Uses pH + pOH = 14. Values outside 0–14 are allowed mathematically; concentrated solutions need activities.",
        fields: &[number("Hydronium concentration (mol/L)", "0.001")],
    },
    ScienceTool {
        id: "strong-acid-base",
        name: "Strong acid–base mixture",
        group: "Chemistry",
        help: "Complete neutralization of a strong monoprotic acid and strong monobasic base at 25 °C. Volumes add, concentrations approximate activities. Water autoionization is included near equivalence: [H⁺] − [OH⁻] = excess acid / total volume and [H⁺][OH⁻] = Kw. Not a weak-acid or buffer solver.",
        fields: &[
            number("Acid concentration (mol/L)", "0.1"),
            number("Acid volume (L)", "0.05"),
            number("Base concentration (mol/L)", "0.1"),
            number("Base volume (L)", "0.05"),
        ],
    },
    ScienceTool {
        id: "arrhenius-ratio",
        name: "Arrhenius rate ratio",
        group: "Chemistry",
        help: "k₂/k₁ = exp[(Ea/R)(1/T₁ − 1/T₂)]. Same reaction mechanism, activation energy and pre-exponential factor at both positive absolute temperatures. Activation energy is entered in joules per mole.",
        fields: &[
            number("Activation energy (J/mol)", "50000"),
            number("Initial temperature (K)", "300"),
            number("Final temperature (K)", "310"),
        ],
    },
    ScienceTool {
        id: "beer-lambert",
        name: "Beer–Lambert absorbance",
        group: "Chemistry",
        help: "A = ε c l and transmittance = 10^(−A). Homogeneous, non-scattering solution in the linear range, monochromatic light. Absorptivity is in L/(mol cm), concentration in mol/L and path in cm.",
        fields: &[
            number("Molar absorptivity (L/(mol cm))", "10000"),
            number("Concentration (mol/L)", "0.00002"),
            number("Path length (cm)", "1"),
        ],
    },
    ScienceTool {
        id: "percent-yield",
        name: "Reaction percent yield",
        group: "Chemistry",
        help: "Yield = 100 × actual / theoretical product mass. Use the same product and mass unit for both. Values over 100% are shown so impurities, wet product or measurement errors can be investigated.",
        fields: &[
            number("Actual product mass (g)", "8"),
            number("Theoretical product mass (g)", "10"),
        ],
    },
];

#[derive(Clone, Debug)]
pub(crate) struct ScienceValue {
    pub label: String,
    pub value: f64,
    pub unit: &'static str,
}

#[derive(Clone, Debug)]
pub(crate) struct ScienceReport {
    pub values: Vec<ScienceValue>,
    pub method: String,
    pub assumptions: String,
}

impl ScienceReport {
    fn new(tool: &ScienceTool) -> Self {
        Self {
            values: Vec::new(),
            method: String::new(),
            assumptions: tool.help.into(),
        }
    }

    fn add(&mut self, label: &str, value: f64, unit: &'static str) -> Result<(), String> {
        if !value.is_finite() {
            return Err("The calculation exceeds the finite numeric range. Check units and input magnitudes.".into());
        }
        self.values.push(ScienceValue {
            label: label.into(),
            value,
            unit,
        });
        Ok(())
    }

    pub(crate) fn text(&self, precision: usize) -> String {
        let rows = self
            .values
            .iter()
            .map(|v| {
                format!(
                    "{}: {}{}{}",
                    v.label,
                    math::format(v.value, precision),
                    if v.unit.is_empty() { "" } else { " " },
                    v.unit
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{rows}\n\nMethod: {}\nAssumptions: {}",
            self.method, self.assumptions
        )
    }
}

fn has_variable(expr: &math::Expr) -> bool {
    match expr {
        math::Expr::Var(_) => true,
        math::Expr::Number(_) => false,
        math::Expr::Unary(_, value) => has_variable(value),
        math::Expr::Binary(_, left, right) => has_variable(left) || has_variable(right),
        math::Expr::Call(_, arguments) => arguments.iter().any(has_variable),
    }
}

fn numeric_input(text: &str, decimal_comma: bool, label: &str) -> Result<f64, String> {
    let normalized =
        math::normalize_input(text, decimal_comma).map_err(|e| format!("{label}: {e}"))?;
    let expr = math::parse(&normalized).map_err(|e| format!("{label}: {e}"))?;
    if has_variable(&expr) {
        return Err(format!(
            "{label}: enter a number or constant expression without variables or Ans."
        ));
    }
    let value = expr
        .eval(0.0, 0.0, 0.0, false)
        .map_err(|e| format!("{label}: {e}"))?;
    if !value.is_finite() {
        return Err(format!("{label} must be finite."));
    }
    Ok(value)
}

/// Evaluates the same models used by the native workbench. Unit conversion is
/// explicit in the field labels; this is not a general dimensional solver.
pub(crate) fn evaluate(
    id: &str,
    inputs: &[String],
    decimal_comma: bool,
) -> Result<ScienceReport, String> {
    let tool = TOOLS
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| "Unknown science tool.".to_string())?;
    if inputs.len() != tool.fields.len() {
        return Err(format!("{} needs {} inputs.", tool.name, tool.fields.len()));
    }
    let mut numbers = Vec::with_capacity(inputs.len());
    for (field, input) in tool.fields.iter().zip(inputs) {
        if input.chars().count() > MAX_INPUT_CHARS {
            return Err(format!(
                "{}: use at most {MAX_INPUT_CHARS} characters.",
                field.label
            ));
        }
        numbers.push(if field.kind == FieldKind::Number {
            numeric_input(input, decimal_comma, field.label)?
        } else {
            0.0
        });
    }
    let n = |i: usize| numbers[i];
    let positive = |i: usize| -> Result<f64, String> {
        if n(i) > 0.0 {
            Ok(n(i))
        } else {
            Err(format!(
                "{} must be greater than zero.",
                tool.fields[i].label
            ))
        }
    };
    let nonnegative = |i: usize| -> Result<f64, String> {
        if n(i) >= 0.0 {
            Ok(n(i))
        } else {
            Err(format!("{} cannot be negative.", tool.fields[i].label))
        }
    };
    let mut report = ScienceReport::new(tool);
    match id {
        "newton-force" => {
            report.add("Net force", nonnegative(0)? * n(1), "N")?;
            report.method = "Multiply mass by the signed acceleration: F = m × a.".into();
        }
        "kinetic-energy" => {
            report.add("Kinetic energy", 0.5 * nonnegative(0)? * n(1) * n(1), "J")?;
            report.method =
                "Square velocity, multiply by mass, then divide by two: K = m v² / 2.".into();
        }
        "momentum" => {
            report.add("Momentum", nonnegative(0)? * n(1), "kg m/s")?;
            report.method = "Multiply mass by signed velocity: p = m × v.".into();
        }
        "work" => {
            report.add(
                "Work",
                nonnegative(0)? * nonnegative(1)? * n(2).to_radians().cos(),
                "J",
            )?;
            report.method =
                "Convert the given angle from degrees to radians, then W = F × d × cos(θ).".into();
        }
        "power" => {
            report.add("Average power", n(0) / positive(1)?, "W")?;
            report.method = "Divide signed work by positive elapsed time: P = W / Δt.".into();
        }
        "gravitation" => {
            let distance = positive(2)?;
            report.add(
                "Attractive force magnitude",
                GRAVITATIONAL_CONSTANT
                    * (nonnegative(0)? / distance)
                    * (nonnegative(1)? / distance),
                "N",
            )?;
            report.method = "Use center-to-center separation: F = G × (m₁/r) × (m₂/r), with G = 6.67430e−11 m³/(kg s²). G is measured, not exact.".into();
        }
        "wave-speed" => {
            report.add("Wave speed", positive(0)? * positive(1)?, "m/s")?;
            report.method = "Multiply cycles per second by meters per cycle: v = f × λ.".into();
        }
        "ohms-law" => {
            report.add("Current", n(0) / positive(1)?, "A")?;
            report.method = "Divide voltage by resistance: I = V / R.".into();
        }
        "electric-power" => {
            report.add("Electrical power", n(0) * n(1), "W")?;
            report.method =
                "Multiply voltage by current under the passive sign convention: P = V × I.".into();
        }
        "capacitor-energy" => {
            let charge = positive(0)? * n(1);
            report.add("Stored energy", 0.5 * charge * n(1), "J")?;
            report.add("Charge", charge, "C")?;
            report.method = "First Q = C × V, then stored energy U = Q × V / 2.".into();
        }
        "rc-decay" => {
            let tau = positive(1)? * positive(2)?;
            if !tau.is_finite() || tau <= 0.0 {
                return Err(
                    "The RC time constant is outside the supported positive numeric range.".into(),
                );
            }
            let time = nonnegative(3)?;
            report.add("Voltage", n(0) * (-time / tau).exp(), "V")?;
            report.add("Time constant", tau, "s")?;
            report.method = "Calculate τ = R × C, then V(t) = V₀ × exp(−t/τ). Exponentially small voltages can underflow to zero.".into();
        }
        "series-rlc" => {
            let resistance = nonnegative(0)?;
            let inductance = nonnegative(1)?;
            let capacitance = positive(2)?;
            let omega = 2.0 * PI * positive(3)?;
            let voltage = nonnegative(4)?;
            let reactance = omega * inductance - (1.0 / omega) / capacitance;
            let impedance = resistance.hypot(reactance);
            if !impedance.is_finite() || impedance <= 0.0 {
                return Err("Impedance must be finite and nonzero; an ideal resonant circuit without resistance has no finite current.".into());
            }
            let current = voltage / impedance;
            report.add("Impedance magnitude", impedance, "ohm")?;
            report.add("RMS current", current, "A")?;
            report.add(
                "Voltage phase relative to current",
                reactance.atan2(resistance).to_degrees(),
                "degrees",
            )?;
            report.add("Average real power", current * current * resistance, "W")?;
            report.add("Power factor", resistance / impedance, "")?;
            report.method = "ω = 2πf; X = ωL − 1/(ωC); |Z| = hypot(R, X); I_rms = V_rms/|Z|; phase = atan2(X, R); average P = I_rms² R; power factor = R/|Z|.".into();
        }
        "ideal-gas" => {
            report.add(
                "Pressure",
                nonnegative(0)? * math::GAS_CONSTANT * (positive(1)? / positive(2)?),
                "Pa",
            )?;
            report.method = "P = n × R × T / V, with R = N_A kB = 8.31446261815324 J/(mol K). A liter is 0.001 m³.".into();
        }
        "heat" => {
            report.add(
                "Heat transferred",
                nonnegative(0)? * positive(1)? * n(2),
                "J",
            )?;
            report.method = "Q = m × c × ΔT; a negative temperature change gives negative heat transfer into the material.".into();
        }
        "photon-energy" => {
            let wavelength = positive(0)?;
            let energy = (math::PLANCK * math::LIGHTSPEED) / wavelength;
            report.add("Photon energy", energy, "J")?;
            report.add("Frequency", math::LIGHTSPEED / wavelength, "Hz")?;
            report.add(
                "Photon energy in electronvolts",
                energy / ELEMENTARY_CHARGE,
                "eV",
            )?;
            report.method = "E = h c / λ; f = c / λ; E(eV) = E(J) / 1.602176634e−19. h = 6.62607015e−34 J s and c = 299792458 m/s are exact SI definitions.".into();
        }
        "de-broglie" => {
            let mass = positive(0)?;
            let speed = positive(1)?;
            if speed > 0.1 * math::LIGHTSPEED {
                return Err("This nonrelativistic tool requires speed ≤ 0.1 c. Use relativistic momentum and λ = h/p above this speed.".into());
            }
            report.add("de Broglie wavelength", (math::PLANCK / mass) / speed, "m")?;
            report.method = "Nonrelativistic momentum p = m v, then λ = h / p. The classical approximation has about a 0.5% wavelength error at 0.1 c.".into();
        }
        "relativistic-energy" => {
            let mass = positive(0)?;
            let speed = nonnegative(1)?;
            if speed >= math::LIGHTSPEED {
                return Err(
                    "A massive particle's speed must be strictly less than the speed of light."
                        .into(),
                );
            }
            let beta = speed / math::LIGHTSPEED;
            let inverse_gamma = ((1.0 - beta) * (1.0 + beta)).sqrt();
            let rest_energy = mass * math::LIGHTSPEED * math::LIGHTSPEED;
            report.add("Total energy", rest_energy / inverse_gamma, "J")?;
            report.add(
                "Kinetic energy",
                rest_energy * (beta / inverse_gamma) * (beta / (1.0 + inverse_gamma)),
                "J",
            )?;
            report.add(
                "Momentum magnitude",
                mass * (speed / inverse_gamma),
                "kg m/s",
            )?;
            report.add("Lorentz factor", 1.0 / inverse_gamma, "")?;
            report.method = "β = v/c; s = √((1−β)(1+β)); γ = 1/s; E = mc²/s; K = mc² β²/[s(1+s)]; p = mv/s. The equivalent expression for K avoids subtracting nearly equal energies.".into();
        }
        "radioactive-decay" => {
            let initial = nonnegative(0)?;
            let time = nonnegative(1)?;
            let half_life = positive(2)?;
            report.add(
                "Remaining amount",
                initial * (-LN_2 * (time / half_life)).exp(),
                "initial amount units",
            )?;
            report.add("Decay constant", LN_2 / half_life, "per input time unit")?;
            report.method = "λ = ln(2)/t½; N = N₀ exp(−λt). The amount may be an expected particle count, mass or activity if the same exponential model applies. Very small remaining amounts can underflow to zero.".into();
        }
        "boltzmann-ratio" => {
            report.add(
                "Population ratio N₂/N₁",
                (-(n(0) / math::BOLTZMANN) / positive(1)?).exp(),
                "",
            )?;
            report.method = "Divide signed energy gap by kB T, negate and exponentiate. For unequal degeneracies multiply this ratio by g₂/g₁. Very small ratios can underflow to zero.".into();
        }
        "moles-particles" => {
            report.add(
                "Particle count",
                nonnegative(0)? * math::AVOGADRO,
                "specified entities",
            )?;
            report.method =
                "N = n × 6.02214076e23 mol⁻¹. The Avogadro constant is exact in the SI.".into();
        }
        "particles-moles" => {
            report.add("Amount", nonnegative(0)? / math::AVOGADRO, "mol")?;
            report.method = "n = N / 6.02214076e23 mol⁻¹.".into();
        }
        "avogadro-volume" => {
            report.add(
                "Final volume",
                positive(0)? * (nonnegative(2)? / positive(1)?),
                "L",
            )?;
            report.method =
                "Hold temperature and pressure constant: V₂/V₁ = n₂/n₁, so V₂ = V₁ × n₂/n₁.".into();
        }
        "molar-mass" | "mass-moles" | "moles-mass" => {
            let formula_index = usize::from(id != "molar-mass");
            let (molar_mass, atoms) = molar_mass(&inputs[formula_index])?;
            if id == "mass-moles" {
                report.add("Amount", nonnegative(0)? / molar_mass, "mol")?;
            } else if id == "moles-mass" {
                report.add("Mass", nonnegative(0)? * molar_mass, "g")?;
            }
            report.add("Molar mass", molar_mass, "g/mol")?;
            let composition = atoms
                .iter()
                .map(|(symbol, count)| format!("{symbol}: {count}"))
                .collect::<Vec<_>>()
                .join(", ");
            report.method = format!(
                "Parsed atom counts: {composition}. M = Σ(atom count × conventional atomic weight). This approximate natural-isotope molar mass is suitable for classroom calculations; isotope-enriched samples need their measured composition."
            );
        }
        "molarity" => {
            report.add("Concentration", nonnegative(0)? / positive(1)?, "mol/L")?;
            report.method = "Divide solute amount by final solution volume: c = n/V.".into();
        }
        "dilution" => {
            let initial = positive(1)?;
            let final_volume = positive(2)?;
            if final_volume < initial {
                return Err("For dilution by adding solvent, final volume cannot be smaller than initial volume.".into());
            }
            report.add(
                "Final concentration",
                nonnegative(0)? * (initial / final_volume),
                "mol/L",
            )?;
            report.method = "Conserve solute amount: c₁V₁ = c₂V₂, then c₂ = c₁ × (V₁/V₂).".into();
        }
        "ph" => {
            let hydrogen = positive(0)?;
            let ph = -hydrogen.log10();
            report.add("pH", ph, "")?;
            report.add("pOH", 14.0 - ph, "")?;
            report.add(
                "Hydroxide concentration",
                WATER_ION_PRODUCT / hydrogen,
                "mol/L",
            )?;
            report.method = "pH = −log₁₀[H₃O⁺]; pOH = 14 − pH; [OH⁻] = 10⁻¹⁴/[H₃O⁺], at 25 °C under the dilute concentration approximation.".into();
        }
        "strong-acid-base" => {
            let acid = nonnegative(0)?;
            let va = nonnegative(1)?;
            let base = nonnegative(2)?;
            let vb = nonnegative(3)?;
            let volume = va + vb;
            if !volume.is_finite() || volume <= 0.0 {
                return Err("Total solution volume must be positive and finite.".into());
            }
            // Weighting before multiplication avoids overflow in Ca*Va when the
            // finite concentration after mixing is still representable.
            let excess = acid * (va / volume) - base * (vb / volume);
            let discriminant = excess.hypot(2.0 * WATER_ION_PRODUCT.sqrt());
            let hydrogen = if excess >= 0.0 {
                0.5 * excess + 0.5 * discriminant
            } else {
                WATER_ION_PRODUCT / (0.5 * discriminant - 0.5 * excess)
            };
            report.add("pH", -hydrogen.log10(), "")?;
            report.add("Hydronium concentration", hydrogen, "mol/L")?;
            report.add(
                "Hydroxide concentration",
                WATER_ION_PRODUCT / hydrogen,
                "mol/L",
            )?;
            report.method = "Let D = (Ca Va − Cb Vb)/(Va + Vb). Solve H² − D H − Kw = 0 for the positive H using a cancellation-resistant branch, then pH = −log₁₀ H. This includes water autoionization and returns pH 7 at equivalence at 25 °C.".into();
        }
        "arrhenius-ratio" => {
            let activation = nonnegative(0)?;
            let exponent =
                (activation / math::GAS_CONSTANT) * (1.0 / positive(1)? - 1.0 / positive(2)?);
            report.add("Rate constant ratio k₂/k₁", exponent.exp(), "")?;
            report.method = "Subtract the two logarithmic Arrhenius equations to cancel the shared pre-exponential factor: ln(k₂/k₁) = (Ea/R)(1/T₁ − 1/T₂). Very small ratios can underflow to zero.".into();
        }
        "beer-lambert" => {
            let absorbance = nonnegative(0)? * nonnegative(1)? * positive(2)?;
            report.add("Absorbance", absorbance, "")?;
            report.add("Transmittance", 10.0_f64.powf(-absorbance), "fraction")?;
            report.method = "A = ε × c × l; T = 10^(−A), the transmitted fraction rather than a percentage. Very small transmittances can underflow to zero.".into();
        }
        "percent-yield" => {
            report.add(
                "Percent yield",
                (nonnegative(0)? / positive(1)?) * 100.0,
                "%",
            )?;
            report.method = "Divide actual by theoretical product mass and multiply by 100. This tool does not determine the limiting reagent or balance a reaction.".into();
        }
        _ => return Err("Unknown science tool.".into()),
    }
    Ok(report)
}

// Conventional classroom values: CIAAW abridged natural-isotope weights, with
// extra digits for several single-valued standard weights. Not isotope masses.
const ATOMIC_WEIGHTS: &[(&str, f64)] = &[
    ("H", 1.008),
    ("He", 4.002602),
    ("Li", 6.94),
    ("Be", 9.0121831),
    ("B", 10.81),
    ("C", 12.011),
    ("N", 14.007),
    ("O", 15.999),
    ("F", 18.998403162),
    ("Ne", 20.1797),
    ("Na", 22.98976928),
    ("Mg", 24.305),
    ("Al", 26.9815384),
    ("Si", 28.085),
    ("P", 30.973761998),
    ("S", 32.06),
    ("Cl", 35.45),
    ("Ar", 39.95),
    ("K", 39.0983),
    ("Ca", 40.078),
    ("Sc", 44.955907),
    ("Ti", 47.867),
    ("V", 50.9415),
    ("Cr", 51.9961),
    ("Mn", 54.938043),
    ("Fe", 55.845),
    ("Co", 58.933194),
    ("Ni", 58.6934),
    ("Cu", 63.546),
    ("Zn", 65.38),
    ("Ga", 69.723),
    ("Ge", 72.630),
    ("As", 74.921595),
    ("Se", 78.971),
    ("Br", 79.904),
    ("Kr", 83.798),
    ("Rb", 85.4678),
    ("Sr", 87.62),
    ("Y", 88.905838),
    ("Zr", 91.222),
    ("Nb", 92.90637),
    ("Mo", 95.95),
    ("Ru", 101.07),
    ("Rh", 102.90549),
    ("Pd", 106.42),
    ("Ag", 107.8682),
    ("Cd", 112.414),
    ("In", 114.818),
    ("Sn", 118.710),
    ("Sb", 121.760),
    ("Te", 127.60),
    ("I", 126.90447),
    ("Xe", 131.293),
    ("Cs", 132.90545196),
    ("Ba", 137.327),
    ("La", 138.90547),
    ("Ce", 140.116),
    ("Pr", 140.90766),
    ("Nd", 144.242),
    ("Sm", 150.36),
    ("Eu", 151.964),
    ("Gd", 157.249),
    ("Tb", 158.925354),
    ("Dy", 162.500),
    ("Ho", 164.930329),
    ("Er", 167.259),
    ("Tm", 168.934219),
    ("Yb", 173.045),
    ("Lu", 174.96669),
    ("Hf", 178.486),
    ("Ta", 180.94788),
    ("W", 183.84),
    ("Re", 186.207),
    ("Os", 190.23),
    ("Ir", 192.217),
    ("Pt", 195.084),
    ("Au", 196.966570),
    ("Hg", 200.592),
    ("Tl", 204.38),
    ("Pb", 207.2),
    ("Bi", 208.98040),
    ("Th", 232.0377),
    ("Pa", 231.03588),
    ("U", 238.02891),
];

fn atomic_weight(symbol: &str) -> Option<f64> {
    ATOMIC_WEIGHTS
        .iter()
        .find(|(s, _)| *s == symbol)
        .map(|(_, mass)| *mass)
}

type AtomCounts = BTreeMap<String, u64>;

fn add_atoms(target: &mut AtomCounts, source: AtomCounts, multiplier: u64) -> Result<(), String> {
    for (symbol, count) in source {
        let addition = count
            .checked_mul(multiplier)
            .ok_or("Atom count is too large.")?;
        let current = target.entry(symbol).or_default();
        *current = current
            .checked_add(addition)
            .filter(|v| *v <= MAX_ATOM_COUNT)
            .ok_or("Use at most one billion atoms of each element in a formula.")?;
    }
    if target.values().sum::<u64>() > MAX_ATOM_COUNT {
        return Err("Use at most one billion total atoms in a formula.".into());
    }
    Ok(())
}

struct FormulaParser {
    chars: Vec<char>,
    at: usize,
}

impl FormulaParser {
    fn count(&mut self) -> Result<u64, String> {
        let start = self.at;
        let mut value = 0_u64;
        while let Some(digit) = self.chars.get(self.at).and_then(|c| c.to_digit(10)) {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(digit as u64))
                .filter(|v| *v <= 1_000_000)
                .ok_or("A subscript or hydrate coefficient must be at most one million.")?;
            self.at += 1;
        }
        if self.at == start {
            return Ok(1);
        }
        if value == 0 {
            return Err(
                "Atom subscripts and hydrate coefficients must be positive whole numbers.".into(),
            );
        }
        Ok(value)
    }

    fn group(&mut self, end: Option<char>, depth: usize) -> Result<AtomCounts, String> {
        if depth > MAX_FORMULA_DEPTH {
            return Err("Use at most 16 nested formula groups.".into());
        }
        let mut atoms = AtomCounts::new();
        while let Some(&character) = self.chars.get(self.at) {
            if Some(character) == end {
                if atoms.is_empty() {
                    return Err("Formula groups cannot be empty.".into());
                }
                self.at += 1;
                return Ok(atoms);
            }
            if character == ')' || character == ']' {
                return Err("Formula parentheses or brackets do not match.".into());
            }
            if character == '.' || character == '·' {
                if end.is_some() {
                    return Err("A hydrate separator must be outside formula groups.".into());
                }
                break;
            }
            let part = if character == '(' || character == '[' {
                self.at += 1;
                self.group(Some(if character == '(' { ')' } else { ']' }), depth + 1)?
            } else if character.is_ascii_uppercase() {
                let mut symbol = character.to_string();
                self.at += 1;
                while self
                    .chars
                    .get(self.at)
                    .is_some_and(char::is_ascii_lowercase)
                {
                    symbol.push(self.chars[self.at]);
                    self.at += 1;
                }
                if atomic_weight(&symbol).is_none() {
                    return Err(format!(
                        "Element '{symbol}' has no supported conventional atomic weight. Element symbols are case-sensitive; isotope labels and charges are not supported."
                    ));
                }
                BTreeMap::from([(symbol, 1)])
            } else {
                return Err(format!(
                    "Unexpected '{character}' in formula. Use element symbols, positive whole subscripts, parentheses/brackets and a middle dot for hydrates; omit charges and reaction coefficients."
                ));
            };
            let count = self.count()?;
            add_atoms(&mut atoms, part, count)?;
        }
        if end.is_some() {
            return Err("A formula group is missing its closing parenthesis or bracket.".into());
        }
        if atoms.is_empty() {
            return Err("Enter a chemical formula such as H2O or Ca(OH)2.".into());
        }
        Ok(atoms)
    }
}

fn molar_mass(text: &str) -> Result<(f64, AtomCounts), String> {
    if text.chars().count() > MAX_INPUT_CHARS {
        return Err("Chemical formula is limited to 512 characters.".into());
    }
    let chars = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '₀' => '0',
            '₁' => '1',
            '₂' => '2',
            '₃' => '3',
            '₄' => '4',
            '₅' => '5',
            '₆' => '6',
            '₇' => '7',
            '₈' => '8',
            '₉' => '9',
            other => other,
        })
        .collect::<Vec<_>>();
    let mut parser = FormulaParser { chars, at: 0 };
    let mut atoms = parser.group(None, 0)?;
    while parser.at < parser.chars.len() {
        parser.at += 1; // group() stops only at a top-level hydrate separator.
        let coefficient = parser.count()?;
        let component = parser.group(None, 0)?;
        add_atoms(&mut atoms, component, coefficient)?;
    }
    let mass = atoms
        .iter()
        .map(|(symbol, count)| atomic_weight(symbol).expect("validated symbol") * *count as f64)
        .sum::<f64>();
    Ok((mass, atoms))
}

pub(crate) struct ScienceWorkbench {
    pub(crate) selected: usize,
    pub(crate) inputs: Vec<String>,
    pub(crate) report: Option<Result<ScienceReport, String>>,
    search: String,
    group: &'static str,
    browser_open: bool,
}

impl Default for ScienceWorkbench {
    fn default() -> Self {
        let mut this = Self {
            selected: 0,
            inputs: Vec::new(),
            report: None,
            search: String::new(),
            group: "All",
            browser_open: false,
        };
        this.select(0);
        this
    }
}

impl ScienceWorkbench {
    pub(crate) fn select(&mut self, index: usize) {
        self.selected = index;
        self.inputs = TOOLS[index]
            .fields
            .iter()
            .map(|field| field.example.into())
            .collect();
        self.report = None;
    }
    fn run(&mut self, decimal_comma: bool) {
        self.report = Some(evaluate(
            TOOLS[self.selected].id,
            &self.inputs,
            decimal_comma,
        ));
    }
}

// Egui's max width is soft: a long selectable label can expand it before the
// next input is laid out. Prelayout every string against the original column,
// including unbroken units and long chemical formulas at large text sizes.
#[derive(Clone, Copy)]
struct ScienceColumn {
    right: f32,
    line_spacing: f32,
}

impl ScienceColumn {
    fn width(self, ui: &egui::Ui) -> f32 {
        let left = if ui.layout().is_horizontal() && ui.layout().main_wrap() {
            ui.max_rect().left()
        } else {
            ui.next_widget_position().x
        };
        (self.right - left).max(1.0)
    }

    fn galley(
        self,
        ui: &egui::Ui,
        text: &str,
        style: egui::TextStyle,
        color: egui::Color32,
        padding: f32,
    ) -> std::sync::Arc<egui::Galley> {
        let font = style.resolve(ui.style());
        let line_height = font.size * 1.2 * self.line_spacing;
        let mut job = egui::text::LayoutJob::simple(
            text.into(),
            font,
            color,
            (self.width(ui) - padding).max(1.0),
        );
        // Prefer words; epaint still breaks a token that cannot fit on a row.
        job.wrap.break_anywhere = false;
        job.sections[0].format.line_height = Some(line_height);
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    }

    fn label(self, ui: &mut egui::Ui, text: &str) -> egui::Response {
        let galley = self.galley(
            ui,
            text,
            egui::TextStyle::Body,
            appearance::palette(ui.ctx()).ink,
            0.0,
        );
        ui.label(galley)
    }

    fn heading(self, ui: &mut egui::Ui, text: &str) {
        let galley = self.galley(
            ui,
            text,
            egui::TextStyle::Heading,
            appearance::palette(ui.ctx()).ink,
            0.0,
        );
        let response = ui.label(galley);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Heading)
        });
    }

    fn button(self, ui: &mut egui::Ui, text: &str, primary: bool) -> egui::Response {
        let palette = appearance::palette(ui.ctx());
        let galley = self.galley(
            ui,
            text,
            egui::TextStyle::Button,
            if primary {
                palette.selected_text
            } else {
                palette.ink
            },
            ui.spacing().button_padding.x * 2.0,
        );
        let mut button = egui::Button::new(galley).wrap();
        if primary {
            button = button.fill(palette.accent);
        }
        focus_visible(ui.add(button))
    }

    fn choice<Value: PartialEq>(
        self,
        ui: &mut egui::Ui,
        current: &mut Value,
        value: Value,
        text: &str,
    ) -> egui::Response {
        let selected = *current == value;
        let palette = appearance::palette(ui.ctx());
        let galley = self.galley(
            ui,
            text,
            egui::TextStyle::Button,
            if selected {
                palette.selected_text
            } else {
                palette.ink
            },
            ui.spacing().button_padding.x * 2.0,
        );
        let mut response = focus_visible(ui.selectable_label(selected, galley));
        if response.clicked() && !selected {
            *current = value;
            response.mark_changed();
        }
        response
    }

    fn announce(self, ui: &mut egui::Ui, text: &str) {
        let response = self.label(ui, text);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::Status);
            node.set_label(text);
            node.clear_value();
            node.set_live(egui::accesskit::Live::Polite);
            node.set_live_atomic();
        });
    }
}

fn focus_visible(response: egui::Response) -> egui::Response {
    if response.gained_focus() {
        response.scroll_to_me(Some(egui::Align::Center));
    }
    response
}

impl EqualApp {
    pub(crate) fn science_workspace(&mut self, ui: &mut egui::Ui) {
        let compact = ui.available_width() < 360.0;
        let column = ScienceColumn {
            right: ui.max_rect().right(),
            line_spacing: self.settings.access.line_spacing,
        };
        column.heading(ui, "Physics & chemistry");
        if compact {
            ui.horizontal_wrapped(|ui| {
                column.label(ui, "32 tools");
                if column.button(ui, "Math tools", false).clicked() {
                    self.toolkit.science_open = false;
                }
            });
            column.label(ui, "Enter values in the displayed units.");
        } else {
            column.label(ui, "32 tools");
            if column.button(ui, "Math tools", false).clicked() {
                self.toolkit.science_open = false;
            }
            column.label(ui, "Choose a model, check its assumptions, and enter quantities in the displayed units. Constant expressions are supported. Trigonometric expressions use radians; the Work angle field uses degrees. These tools cover selected school and university calculations, not a full symbolic physics or chemistry system.");
        }
        let decimal_comma = self.settings.access.decimal_comma;
        let input_note = if decimal_comma {
            "Numeric input: decimal comma or decimal point; use semicolons between function arguments."
        } else {
            "Numeric input: decimal point. Scientific notation such as 500e-9 and constants such as planck are supported."
        };
        if !compact {
            column.label(ui, input_note);
        }
        let precision = self.settings.precision;
        let mut read_text = None;
        let mut stop_audio = false;
        let state = &mut self.toolkit.science;
        if compact {
            let toggle = column.button(ui, "Choose science tool", false);
            if toggle.clicked() {
                state.browser_open = !state.browser_open;
            }
            ui.ctx().accesskit_node_builder(toggle.id, |node| {
                node.set_expanded(state.browser_open);
            });
        }
        let mut focus_selected_input = false;
        if !compact || state.browser_open {
            ui.horizontal_wrapped(|ui| {
                column.label(ui, "Find a science tool");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.search)
                        .desired_width((column.width(ui) - 10.0).clamp(1.0, 300.0))
                        .char_limit(100)
                        .hint_text("Search physics and chemistry"),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::TextEdit,
                        true,
                        "Find a science tool",
                    )
                });
                if column.button(ui, "Clear science search", false).clicked() {
                    state.search.clear();
                    state.group = "All";
                }
            });
            let mut category_changed = false;
            ui.horizontal_wrapped(|ui| {
                for group in ["All", "Physics", "Chemistry"] {
                    category_changed |= column.choice(ui, &mut state.group, group, group).changed();
                }
            });
            if category_changed {
                state.search.clear();
                if let Some(index) = TOOLS
                    .iter()
                    .position(|tool| state.group == "All" || state.group == tool.group)
                {
                    state.select(index);
                }
            }
            let query = state.search.to_lowercase();
            let mut selection = None;
            let mut matches = 0;
            ui.horizontal_wrapped(|ui| {
                for (index, tool) in TOOLS.iter().enumerate().filter(|(_, tool)| {
                    (state.group == "All" || state.group == tool.group)
                        && format!("{} {} {}", tool.name, tool.group, tool.help)
                            .to_lowercase()
                            .contains(&query)
                }) {
                    matches += 1;
                    let mut selected = state.selected;
                    if column.choice(ui, &mut selected, index, tool.name).clicked() {
                        selection = Some(index);
                    }
                }
            });
            if matches == 0 {
                column.label(
                    ui,
                    "No matching science tools. Clear the search or change the subject.",
                );
            }
            if let Some(index) = selection {
                state.select(index);
                if compact {
                    state.browser_open = false;
                    focus_selected_input = true;
                }
            }
        }
        let tool = &TOOLS[state.selected];
        ui.add_space(8.0);
        crate::ui::card(ui, |ui| {
            let body = ScienceColumn {
                right: ui.max_rect().right().min(column.right - 22.0),
                ..column
            };
            body.heading(ui, tool.name);
            if !compact {
                body.label(ui, tool.help);
            }
            for (index, field) in tool.fields.iter().enumerate() {
                body.label(ui, field.label);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.inputs[index])
                        .desired_width((body.width(ui) - 10.0).clamp(1.0, 550.0))
                        .char_limit(MAX_INPUT_CHARS)
                        .id_salt(("science-input", tool.id, index)),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, field.label)
                });
                if index == 0 && focus_selected_input {
                    response.request_focus();
                }
                if response.changed() {
                    state.report = None;
                }
                if (response.has_focus() || response.lost_focus())
                    && !focus_selected_input
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
                    response.request_focus();
                    state.run(decimal_comma);
                }
            }
            if compact {
                let notes = body.galley(
                    ui,
                    "Model & input notes",
                    egui::TextStyle::Button,
                    appearance::palette(ui.ctx()).ink,
                    40.0,
                );
                let notes_response = egui::CollapsingHeader::new(notes).id_salt(("science-model-notes", tool.id)).show(ui, |ui| {
                    body.label(ui, tool.help);
                    body.label(ui, input_note);
                    body.label(ui, "Constant expressions use radians for trigonometric functions. The Work angle field uses degrees.");
                });
                focus_visible(notes_response.header_response);
            }
            ui.horizontal_wrapped(|ui| {
                if body.button(ui, "Calculate science", true).clicked() {
                    state.run(decimal_comma);
                }
                if body.button(ui, "Reset science example", false).clicked() {
                    state.select(state.selected);
                }
            });
        });
        if let Some(result) = &state.report {
            ui.add_space(8.0);
            match result {
                Err(error) => column.announce(ui, &format!("Check science inputs: {error}")),
                Ok(report) => {
                    let text = format!("{}\n{}", tool.name, report.text(precision));
                    crate::ui::card(ui, |ui| {
                        let body = ScienceColumn {
                            right: ui.max_rect().right().min(column.right - 22.0),
                            ..column
                        };
                        body.heading(ui, "Science result");
                        body.announce(ui, &text);
                        ui.horizontal_wrapped(|ui| {
                            if body.button(ui, "Copy science result", false).clicked() {
                                ui.ctx().copy_text(text.clone());
                            }
                            if body.button(ui, "Read science result", false).clicked() {
                                read_text = Some(text.clone());
                            }
                            if body.button(ui, "Stop audio", false).clicked() {
                                stop_audio = true;
                            }
                        });
                    });
                }
            }
        }
        let sources = column.galley(
            ui,
            "Science sources & scope",
            egui::TextStyle::Button,
            appearance::palette(ui.ctx()).ink,
            40.0,
        );
        let source_response = egui::CollapsingHeader::new(sources).show(ui, |ui| {
            column.label(ui, "SI defining constants are exact; G and conventional atomic weights are measured or approximate. Floating-point calculations do not propagate uncertainty or infer significant figures. The formula parser supports 84 elements with conventional atomic weights; radioactive elements without such weights, isotope labels, charges and reaction equations are excluded. Calculations use only local code; the links below open reference pages if requested.");
            for (label, url) in [
                ("BIPM: SI defining constants", "https://www.bipm.org/en/measurement-units/si-defining-constants"),
                ("NIST: CODATA 2022 constants", "https://physics.nist.gov/cuu/Constants/index.html"),
                ("CIAAW: standard atomic weights", "https://ciaaw.org/atomic-weights.htm"),
                ("OpenStax: University Physics", "https://openstax.org/subjects/science"),
                ("OpenStax: Chemistry 2e", "https://openstax.org/details/books/chemistry-2e"),
            ] {
                let galley = column.galley(ui, label, egui::TextStyle::Body, ui.visuals().hyperlink_color, 0.0);
                focus_visible(ui.hyperlink_to(galley, url));
            }
        });
        focus_visible(source_response.header_response);
        if let Some(text) = read_text {
            self.speak(&text);
        }
        if stop_audio {
            self.stop_audio();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(id: &str, inputs: &[&str]) -> ScienceReport {
        evaluate(
            id,
            &inputs.iter().map(|v| (*v).into()).collect::<Vec<_>>(),
            false,
        )
        .unwrap()
    }
    fn close(actual: f64, expected: f64, relative: f64) {
        assert!(
            (actual - expected).abs() <= relative * expected.abs().max(f64::MIN_POSITIVE),
            "actual={actual}, expected={expected}"
        );
    }

    #[test]
    fn every_example_is_finite_and_has_a_method() {
        for tool in TOOLS {
            let inputs = tool
                .fields
                .iter()
                .map(|field| field.example.into())
                .collect::<Vec<_>>();
            let report =
                evaluate(tool.id, &inputs, false).unwrap_or_else(|e| panic!("{}: {e}", tool.id));
            assert!(!report.values.is_empty(), "{}", tool.id);
            assert!(report.values.iter().all(|v| v.value.is_finite()));
            assert!(!report.method.is_empty());
        }
        assert_eq!(TOOLS.len(), 32);
    }

    #[test]
    fn mechanics_and_electricity_known_values() {
        assert_eq!(run("newton-force", &["2", "3"]).values[0].value, 6.0);
        assert_eq!(run("kinetic-energy", &["2", "3"]).values[0].value, 9.0);
        close(run("work", &["10", "2", "60"]).values[0].value, 10.0, 1e-14);
        assert_eq!(run("ohms-law", &["12", "6"]).values[0].value, 2.0);
        close(
            run("capacitor-energy", &["0.001", "10"]).values[0].value,
            0.05,
            1e-14,
        );
        let rc = run("rc-decay", &["10", "1000", "0.001", "1"]);
        close(rc.values[0].value, 3.6787944117144233, 1e-14);
        assert_eq!(rc.values[1].value, 1.0);
        let rlc = run("series-rlc", &["3", "2", "0.5", "1/(2*pi)", "12"]);
        close(rlc.values[0].value, 3.0, 1e-14);
        close(rlc.values[1].value, 4.0, 1e-14);
        close(rlc.values[3].value, 48.0, 1e-14);
    }

    #[test]
    fn modern_physics_and_exact_si_constants() {
        close(
            run("photon-energy", &["500e-9"]).values[0].value,
            3.972891714297857e-19,
            1e-14,
        );
        let relativity = run("relativistic-energy", &["1", "0.6*lightspeed"]);
        close(relativity.values[0].value, 1.123443973421022e17, 1e-14);
        close(relativity.values[1].value, 2.246887946842044e16, 1e-14);
        close(relativity.values[3].value, 1.25, 1e-14);
        close(
            run("relativistic-energy", &["2", "0.001"]).values[1].value,
            1e-6,
            1e-14,
        );
        close(
            run("radioactive-decay", &["800", "30", "10"]).values[0].value,
            100.0,
            1e-14,
        );
        close(
            run("boltzmann-ratio", &["boltzmann*300", "300"]).values[0].value,
            0.36787944117144233,
            1e-14,
        );
        assert_eq!(
            run("moles-particles", &["1"]).values[0].value,
            6.02214076e23
        );
    }

    #[test]
    fn formula_groups_hydrates_and_mass_conversion() {
        close(run("molar-mass", &["H₂O"]).values[0].value, 18.015, 1e-14);
        close(
            run("molar-mass", &["Ca(OH)2"]).values[0].value,
            74.092,
            1e-14,
        );
        close(
            run("molar-mass", &["Al2(SO4)3"]).values[0].value,
            342.1310768,
            1e-14,
        );
        close(
            run("molar-mass", &["CuSO4·5H2O"]).values[0].value,
            249.677,
            1e-14,
        );
        close(
            run("molar-mass", &["K4[Fe(CN)6]"]).values[0].value,
            368.3462,
            1e-14,
        );
        close(
            run("mass-moles", &["36.03", "H2O"]).values[0].value,
            2.0,
            1e-14,
        );
        close(
            run("moles-mass", &["2", "H2O"]).values[0].value,
            36.03,
            1e-14,
        );
        assert_eq!(ATOMIC_WEIGHTS.len(), 84);
    }

    #[test]
    fn chemical_concentrations_equivalence_and_spectroscopy() {
        assert_eq!(
            run("avogadro-volume", &["2", "1", "3"]).values[0].value,
            6.0
        );
        assert_eq!(run("molarity", &["0.5", "2"]).values[0].value, 0.25);
        close(
            run("dilution", &["2", "0.1", "0.5"]).values[0].value,
            0.4,
            1e-14,
        );
        assert_eq!(run("ph", &["0.001"]).values[0].value, 3.0);
        let neutral = run("strong-acid-base", &["0.1", "0.05", "0.1", "0.05"]);
        assert_eq!(neutral.values[0].value, 7.0);
        close(neutral.values[1].value, 1e-7, 1e-14);
        let base = run("strong-acid-base", &["0", "0", "0.1", "1"]);
        close(base.values[0].value, 13.0, 1e-12);
        let beer = run("beer-lambert", &["10000", "0.00002", "1"]);
        close(beer.values[0].value, 0.2, 1e-14);
        close(beer.values[1].value, 0.6309573444801932, 1e-14);
        close(
            run("percent-yield", &["11", "10"]).values[0].value,
            110.0,
            1e-14,
        );
    }

    #[test]
    fn invalid_domains_and_nonfinite_results_are_explained() {
        for (id, input) in [
            ("ohms-law", vec!["12", "0"]),
            ("ideal-gas", vec!["1", "0", "1"]),
            ("kinetic-energy", vec!["-1", "2"]),
            ("relativistic-energy", vec!["1", "lightspeed"]),
            ("de-broglie", vec!["1", "0.5*lightspeed"]),
            ("ph", vec!["0"]),
            ("dilution", vec!["1", "2", "1"]),
            ("strong-acid-base", vec!["1", "0", "1", "0"]),
            ("series-rlc", vec!["0", "2", "0.5", "1/(2*pi)", "1"]),
            ("moles-particles", vec!["1e300"]),
            ("arrhenius-ratio", vec!["1e300", "1", "2"]),
            ("ohms-law", vec!["x", "2"]),
            ("ohms-law", vec!["Ans", "2"]),
        ] {
            let values = input.iter().map(|v| (*v).into()).collect::<Vec<_>>();
            assert!(evaluate(id, &values, false).is_err(), "{id} {input:?}");
        }
        assert!(evaluate("not-a-tool", &[], false).is_err());
        assert!(evaluate("ohms-law", &["1".into()], false).is_err());
        assert!(evaluate("ph", &["1".repeat(513)], false).is_err());
    }

    #[test]
    fn malformed_formula_and_resource_limits_are_rejected() {
        for formula in [
            "",
            "H0",
            "2H2O",
            "h2o",
            "Ca(OH]2",
            "H()",
            "H2O·",
            "H2O..H2O",
            "H2O+",
            "Xx2",
            "Tc",
            "H1000001",
            "(H1000000)1000000",
            "(((((((((((((((((H)))))))))))))))))",
        ] {
            assert!(molar_mass(formula).is_err(), "{formula}");
        }
    }

    #[test]
    fn localized_numeric_input_is_explicit() {
        let report = evaluate("molarity", &["0,5".into(), "2".into()], true).unwrap();
        assert_eq!(report.values[0].value, 0.25);
        close(run("work", &["10", "2", "60"]).values[0].value, 10.0, 1e-14);
    }
}
