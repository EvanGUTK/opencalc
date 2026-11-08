//! Runtime-owned profile locations; release builds never write into a build checkout.
use std::path::{Path, PathBuf};

const PORTABLE_MARKER: &str = "opencalc.portable";

fn absolute(path: PathBuf) -> Result<PathBuf, String> {
    if path.as_os_str().is_empty() {
        return Err("Choose a nonempty OpenCalc data directory.".into());
    }
    if path.is_absolute() {
        Ok(path)
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(path))
            .map_err(|error| format!("Could not resolve the OpenCalc data directory: {error}"))
    }
}

fn resolve_data_dir(
    override_path: Option<PathBuf>,
    portable: bool,
    executable_directory: &Path,
    user_data_root: Option<PathBuf>,
) -> Result<PathBuf, String> {
    if let Some(path) = override_path {
        return absolute(path);
    }
    if portable || executable_directory.join(PORTABLE_MARKER).is_file() {
        return absolute(executable_directory.join("data"));
    }
    user_data_root
        .map(|root| root.join("OpenCalc"))
        .ok_or_else(|| {
            "The user data directory is unavailable. Start OpenCalc with --data-dir PATH or --portable."
                .into()
        })
        .and_then(absolute)
}

/// Select a profile location without creating or reading profile files.
/// An explicit directory takes precedence over portable mode and the OS default.
pub(crate) fn data_dir(override_path: Option<PathBuf>, portable: bool) -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("Could not locate the OpenCalc executable: {error}"))?;
    let executable_directory = executable
        .parent()
        .ok_or_else(|| "The OpenCalc executable has no parent directory.".to_string())?;
    #[cfg(target_os = "windows")]
    let user_data_root = std::env::var_os("LOCALAPPDATA")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let user_data_root = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let user_data_root = std::env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|value| PathBuf::from(value).join(".local/share"))
        });
    resolve_data_dir(
        override_path,
        portable,
        executable_directory,
        user_data_root,
    )
}

/// Import only an existing development checkout's preferences, without overwriting.
/// Callers must skip this for explicit isolated profiles and screenshot runs.
pub(crate) fn migrate_legacy_preferences(directory: &Path) -> Result<(), String> {
    use std::io::{Read, Write};
    const MAX_PREFERENCES_BYTES: u64 = 1024 * 1024;
    let destination = directory.join("preferences.ron");
    if destination.exists() {
        return Ok(());
    }
    let mut candidates = Vec::new();
    if let Ok(current) = std::env::current_dir()
        && current.join("Cargo.toml").is_file()
        && current.join("src/main.rs").is_file()
    {
        candidates.push(current.join("preferences.ron"));
    }
    #[cfg(debug_assertions)]
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("preferences.ron"));
    for source in candidates {
        if source == destination || !source.is_file() {
            continue;
        }
        let metadata = std::fs::metadata(&source)
            .map_err(|error| format!("Could not inspect legacy preferences: {error}"))?;
        if metadata.len() > MAX_PREFERENCES_BYTES {
            return Err("Legacy preferences exceed the 1 MiB import limit.".into());
        }
        let mut content = Vec::new();
        std::fs::File::open(&source)
            .and_then(|file| {
                file.take(MAX_PREFERENCES_BYTES + 1)
                    .read_to_end(&mut content)
            })
            .map_err(|error| format!("Could not read legacy preferences: {error}"))?;
        if content.len() as u64 > MAX_PREFERENCES_BYTES {
            return Err("Legacy preferences exceed the 1 MiB import limit.".into());
        }
        std::fs::create_dir_all(directory)
            .map_err(|error| format!("Could not create the OpenCalc data directory: {error}"))?;
        let mut output = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
        {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
            Err(error) => return Err(format!("Could not import legacy preferences: {error}")),
        };
        output
            .write_all(&content)
            .and_then(|()| output.sync_all())
            .map_err(|error| format!("Could not save imported preferences: {error}"))?;
        return Ok(());
    }
    Ok(())
}

/// Optional references are beside the installed executable, never required assets.
pub(crate) fn reference_image_root() -> PathBuf {
    let installed = std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(Path::to_path_buf))
        .map_or_else(|| PathBuf::from("assets/ti"), |root| root.join("assets/ti"));
    if installed.is_dir() {
        return installed;
    }
    #[cfg(debug_assertions)]
    {
        let development = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ti");
        if development.is_dir() {
            return development;
        }
    }
    installed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_profile_isolated_from_portable_and_os_defaults() {
        let profile = std::env::temp_dir().join(format!(
            "opencalc-profile-choice-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let selected = resolve_data_dir(
            Some(profile.clone()),
            true,
            Path::new("ignored-installation"),
            Some(PathBuf::from("ignored-os-profile")),
        )
        .unwrap();
        assert_eq!(selected, profile);
        assert!(
            !selected.exists(),
            "Location selection must not create data"
        );
    }

    #[test]
    fn normal_profile_and_portable_locations_have_distinct_owners() {
        let root = std::env::temp_dir().join("opencalc-path-owner-test");
        let executable = root.join("installation");
        let os_data = root.join("user-data");
        assert_eq!(
            resolve_data_dir(None, false, &executable, Some(os_data.clone())).unwrap(),
            os_data.join("OpenCalc")
        );
        assert_eq!(
            resolve_data_dir(None, true, &executable, None).unwrap(),
            executable.join("data")
        );
        assert!(resolve_data_dir(None, false, &executable, None).is_err());
        assert!(resolve_data_dir(Some(PathBuf::new()), false, &executable, None).is_err());
    }

    #[test]
    fn marker_enables_portable_mode_without_environment_mutation() {
        let directory = std::env::temp_dir().join(format!(
            "opencalc-portable-marker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let marker = directory.join(PORTABLE_MARKER);
        std::fs::write(&marker, b"OpenCalc portable profile\n").unwrap();
        let selected = resolve_data_dir(None, false, &directory, None).unwrap();
        assert_eq!(selected, directory.join("data"));
        assert!(!selected.exists());
        std::fs::remove_file(marker).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
