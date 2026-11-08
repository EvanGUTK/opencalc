use crate::math;
use std::fmt::Write;
use std::path::PathBuf;

const CURVE_SAMPLES: usize = 401;
const SURFACE_SAMPLES: usize = 41;

pub fn graph_script(
    first: &str,
    second: &str,
    surface: bool,
    center: [f64; 2],
    range: f64,
    degrees: bool,
) -> Result<String, String> {
    let a = math::parse(first).map_err(|e| format!("First function: {e}"))?;
    let b = if surface || second.trim().is_empty() {
        None
    } else {
        Some(math::parse(second).map_err(|e| format!("Second function: {e}"))?)
    };
    // Export evaluated numbers, so learner input never becomes executable MATLAB code.
    let mut script = format!(
        "% OpenCalc graph export: numeric samples; angles in {}.\n% Undefined real values are NaN. OpenCalc remains usable without MATLAB.\nopencalc_data = [\n",
        if degrees { "degrees" } else { "radians" }
    );
    let mut valid = [0, 0];
    let mut sample = |expression: &math::Expr, x, y, index: usize| {
        expression
            .eval(x, y, 0.0, degrees)
            .ok()
            .filter(|v| v.is_finite())
            .map(|v| {
                valid[index] += 1;
                format!("{v:.16e}")
            })
            .unwrap_or_else(|| "NaN".into())
    };
    if surface {
        for row in 0..SURFACE_SAMPLES {
            let y = -10.0 + 20.0 * row as f64 / (SURFACE_SAMPLES - 1) as f64;
            for column in 0..SURFACE_SAMPLES {
                let x = -10.0 + 20.0 * column as f64 / (SURFACE_SAMPLES - 1) as f64;
                let z = sample(&a, x, y, 0);
                writeln!(script, "{x:.16e} {y:.16e} {z};").expect("write to string");
            }
        }
    } else {
        for i in 0..CURVE_SAMPLES {
            let x = center[0] - range + 2.0 * range * i as f64 / (CURVE_SAMPLES - 1) as f64;
            let av = sample(&a, x, 0.0, 0);
            let bv = b
                .as_ref()
                .map(|b| sample(b, x, 0.0, 1))
                .unwrap_or_else(|| "NaN".into());
            writeln!(script, "{x:.16e} {av} {bv};").expect("write to string");
        }
    }
    if valid[0] == 0 || (b.is_some() && valid[1] == 0) {
        return Err("No real values to send to MATLAB. Check your function and range.".into());
    }
    script.push_str("] ;\nfigure('Name','OpenCalc - MATLAB graph','Color','w');\n");
    if surface {
        writeln!(script, "opencalc_X = reshape(opencalc_data(:,1),{SURFACE_SAMPLES},{SURFACE_SAMPLES})';\nopencalc_Y = reshape(opencalc_data(:,2),{SURFACE_SAMPLES},{SURFACE_SAMPLES})';\nopencalc_Z = reshape(opencalc_data(:,3),{SURFACE_SAMPLES},{SURFACE_SAMPLES})';").expect("write to string");
        script.push_str("surf(opencalc_X,opencalc_Y,opencalc_Z,'FaceColor',[0.75 0.85 0.82],'EdgeColor',[0.08 0.25 0.22]);\nylabel('y'); zlabel('z');\nopencalc_table = array2table(opencalc_data,'VariableNames',{'x','y','z'});\n");
    } else {
        script.push_str(
            "plot(opencalc_data(:,1),opencalc_data(:,2),'-','Color',[0.08 0.35 0.65],'LineWidth',2);\n",
        );
        if b.is_some() {
            script.push_str("hold on; plot(opencalc_data(:,1),opencalc_data(:,3),'--','Color',[0.65 0.25 0.05],'LineWidth',2); hold off;\nlegend('f(x) - solid','g(x) - dashed','Location','best');\n");
        }
        script.push_str("ylabel('Function value');\nopencalc_table = array2table(opencalc_data,'VariableNames',{'x','f','g'});\n");
    }
    script.push_str("xlabel('x'); grid on; title('OpenCalc graph - numeric samples');\nset(gca,'FontSize',14);\ndisp('Graph samples are available as opencalc_table in the workspace.');\ndisp(opencalc_table(1:min(9,height(opencalc_table)),:));\n");
    Ok(script)
}

pub fn installed_matlab() -> Option<PathBuf> {
    let executable = if cfg!(windows) {
        "matlab.exe"
    } else {
        "matlab"
    };
    let mut candidates = vec![PathBuf::from("E:/MATLAB/bin").join(executable)];
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|p| p.join(executable)));
    }
    for base in [
        "C:/Program Files/MATLAB",
        "D:/Program Files/MATLAB",
        "E:/Program Files/MATLAB",
        "/usr/local/MATLAB",
    ] {
        if let Ok(entries) = std::fs::read_dir(base) {
            candidates.extend(
                entries
                    .filter_map(Result::ok)
                    .map(|p| p.path().join("bin").join(executable)),
            );
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}

pub fn open_graph(script: &str) -> Result<PathBuf, String> {
    let executable = installed_matlab().ok_or(
        "MATLAB was not found. Use Copy MATLAB script, then paste into your MATLAB Command Window.",
    )?;
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("exports");
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create graph folder: {e}"))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Could not name graph: {e}"))?
        .as_nanos();
    let path = folder.join(format!("opencalc_graph_{stamp}.m"));
    std::fs::write(&path, script).map_err(|e| format!("Could not save MATLAB graph: {e}"))?;
    let escaped = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''");
    let mut command = std::process::Command::new(executable);
    command.args(["-desktop", "-r", &format!("run('{escaped}')")]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|e| {
        format!(
            "Could not open MATLAB: {e}. Your graph is saved at {}",
            path.display()
        )
    })?;
    std::thread::spawn(move || match child.wait() {
        Ok(status) if !status.success() => eprintln!("MATLAB exited with {status}"),
        Err(error) => eprintln!("Could not wait for MATLAB: {error}"),
        _ => {}
    });
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_numeric_curves_and_distinguishable_styles() {
        let script = graph_script("sin(x)", "cos(x)", false, [2.0, 0.0], 4.0, false).unwrap();
        assert!(script.contains("-2.0000000000000000e0"));
        assert!(script.contains("g(x) - dashed"));
        assert!(!script.contains("sin(x)"));
        assert_eq!(
            script
                .split("opencalc_data = [\n")
                .nth(1)
                .unwrap()
                .split("] ;")
                .next()
                .unwrap()
                .lines()
                .filter(|line| line.ends_with(';') && line.split_whitespace().count() == 3)
                .count(),
            CURVE_SAMPLES
        );
    }

    #[test]
    fn handles_domains_empty_second_function_and_invalid_input() {
        let script = graph_script("sqrt(x)", "", false, [0.0, 0.0], 1.0, false).unwrap();
        assert!(script.contains("NaN"));
        assert!(!script.contains("hold on"));
        assert!(graph_script("sqrt(-1)", "", false, [0.0, 0.0], 1.0, false).is_err());
        assert!(graph_script("x", "nope(x)", false, [0.0, 0.0], 1.0, false).is_err());
        assert!(graph_script("x;system('bad')", "", false, [0.0, 0.0], 1.0, false).is_err());
    }

    #[test]
    fn surface_matches_the_apps_domain_and_exports_text_data() {
        let script = graph_script("x^2-y^2", "ignored", true, [0.0, 0.0], 1.0, false).unwrap();
        assert!(script.contains("reshape(opencalc_data(:,3),41,41)'"));
        assert!(script.contains("surf(opencalc_X,opencalc_Y,opencalc_Z"));
        assert!(script.contains("{'x','y','z'}"));
    }

    #[test]
    fn exported_samples_match_calculator_angle_mode() {
        let script = graph_script("sin(x)", "", false, [90.0, 0.0], 1.0, true).unwrap();
        assert!(script.contains("angles in degrees"));
        let rows = script.split("opencalc_data = [\n").nth(1).unwrap();
        let middle = rows.lines().nth(CURVE_SAMPLES / 2).unwrap();
        let value: f64 = middle.split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!((value - 1.0).abs() < 1e-12);
    }
}
