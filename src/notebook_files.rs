//! Local notebook file operations. Serialization and notebook validation live in the model.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(any(target_os = "windows", test))]
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::{
    os::windows::process::CommandExt,
    process::{Command, Stdio},
};

pub(crate) const MAX_NOTEBOOK_BYTES: usize = 2 * 1024 * 1024;
const TEMP_FILE_ATTEMPTS: usize = 32;
static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn read_notebook(path: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(path)
        .map_err(|error| format!("Could not open notebook '{}': {error}", path.display()))?;
    read_bounded(file)
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, String> {
    // Do not trust file metadata: the file may change or be a special file.
    let mut bytes = Vec::new();
    reader
        .take((MAX_NOTEBOOK_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the notebook: {error}"))?;
    if bytes.len() > MAX_NOTEBOOK_BYTES {
        return Err("The notebook is larger than the 2 MiB file limit.".into());
    }
    Ok(bytes)
}

pub(crate) fn write_notebook(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_NOTEBOOK_BYTES {
        return Err("The notebook is larger than the 2 MiB file limit.".into());
    }
    if path.file_name().is_none() {
        return Err("Choose a notebook file name.".into());
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let (mut file, mut temporary) = create_temporary(parent, path)?;
    let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
    // Windows cannot replace a file while our own temporary write handle remains open.
    drop(file);
    if let Err(error) = write_result {
        return Err(temporary.failure(format!("Could not write the notebook: {error}")));
    }
    // Keep the old destination in place until the same-directory rename succeeds.
    // Never remove the destination first: a failed save must preserve its contents.
    if let Err(error) = fs::rename(&temporary.path, path) {
        return Err(temporary.failure(format!(
            "Could not replace notebook '{}': {error}",
            path.display()
        )));
    }
    temporary.owned = false;
    Ok(())
}

fn create_temporary(parent: &Path, destination: &Path) -> Result<(File, OwnedTemporary), String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for _ in 0..TEMP_FILE_ATTEMPTS {
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".equal-notebook-{}-{timestamp}-{sequence}.tmp",
            std::process::id()
        ));
        if path == destination {
            continue;
        }
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((file, OwnedTemporary { path, owned: true })),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Could not create a temporary notebook file in '{}': {error}",
                    parent.display()
                ));
            }
        }
    }
    Err("Could not create a unique temporary notebook file. Try saving again.".into())
}

struct OwnedTemporary {
    path: PathBuf,
    owned: bool,
}

impl OwnedTemporary {
    fn cleanup(&mut self) -> Result<(), std::io::Error> {
        if self.owned {
            match fs::remove_file(&self.path) {
                Ok(()) => self.owned = false,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.owned = false,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn failure(&mut self, message: String) -> String {
        match self.cleanup() {
            Ok(()) => message,
            Err(error) => format!(
                "{message} The temporary file '{}' could not be removed: {error}",
                self.path.display()
            ),
        }
    }
}

impl Drop for OwnedTemporary {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            eprintln!(
                "Could not remove owned notebook temporary file '{}': {error}",
                self.path.display()
            );
        }
    }
}

#[cfg(any(target_os = "windows", test))]
#[derive(Serialize)]
struct DialogRequest<'a> {
    save: bool,
    current: Option<&'a str>,
}

#[cfg(any(target_os = "windows", test))]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DialogResponse {
    cancelled: bool,
    path: Option<String>,
}

#[cfg(any(target_os = "windows", test))]
fn dialog_payload(save: bool, current: Option<&Path>) -> Result<Vec<u8>, String> {
    let current = current
        .map(|path| {
            path.to_str()
                .ok_or_else(|| "The notebook path cannot be represented as UTF-8.".to_owned())
        })
        .transpose()?;
    serde_json::to_vec(&DialogRequest { save, current })
        .map_err(|error| format!("Could not prepare the notebook file dialog: {error}"))
}

#[cfg(any(target_os = "windows", test))]
fn parse_dialog_response(bytes: &[u8]) -> Result<Option<PathBuf>, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "The notebook file dialog returned invalid UTF-8.".to_owned())?;
    let response: DialogResponse = serde_json::from_str(text.trim_start_matches('\u{feff}').trim())
        .map_err(|error| {
            format!("The notebook file dialog returned an invalid response: {error}")
        })?;
    match (response.cancelled, response.path) {
        (true, None) => Ok(None),
        (false, Some(path)) if !path.trim().is_empty() && !path.contains('\0') => {
            Ok(Some(PathBuf::from(path)))
        }
        _ => Err("The notebook file dialog returned an inconsistent selection.".into()),
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn choose_notebook_file(
    save: bool,
    current: Option<&Path>,
) -> Result<Option<PathBuf>, String> {
    // Only this fixed script is executable. Paths and dialog mode are JSON data on stdin.
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$dialog = $null
$owner = $null
try {
    Add-Type -AssemblyName System.Windows.Forms
    $request = [Console]::In.ReadToEnd() | ConvertFrom-Json
    if ($request.save) {
        $dialog = New-Object System.Windows.Forms.SaveFileDialog
        $dialog.Title = 'Save notebook'
        $dialog.OverwritePrompt = $true
        $dialog.FileName = 'Notebook.opencalc.json'
    } else {
        $dialog = New-Object System.Windows.Forms.OpenFileDialog
        $dialog.Title = 'Open notebook'
        $dialog.CheckFileExists = $true
        $dialog.Multiselect = $false
    }
    $dialog.Filter = 'OpenCalc notebooks (*.opencalc.json)|*.opencalc.json'
    $dialog.DefaultExt = 'opencalc.json'
    $dialog.AddExtension = $true
    $dialog.SupportMultiDottedExtensions = $true
    $dialog.CheckPathExists = $true
    $dialog.ValidateNames = $true
    $dialog.RestoreDirectory = $true
    if (-not [string]::IsNullOrWhiteSpace([string]$request.current)) {
        $suggested = [System.IO.Path]::GetFullPath([string]$request.current)
        $directory = [System.IO.Path]::GetDirectoryName($suggested)
        if ([System.IO.Directory]::Exists($directory)) {
            $dialog.InitialDirectory = $directory
        }
        $dialog.FileName = [System.IO.Path]::GetFileName($suggested)
    }
    $owner = New-Object System.Windows.Forms.Form
    $owner.ShowInTaskbar = $false
    $owner.Opacity = 0
    $owner.TopMost = $true
    $owner.Show()
    $owner.Activate()
    if ($dialog.ShowDialog($owner) -eq [System.Windows.Forms.DialogResult]::OK) {
        [Console]::Out.Write((@{cancelled=$false;path=$dialog.FileName} | ConvertTo-Json -Compress))
    } else {
        [Console]::Out.Write((@{cancelled=$true;path=$null} | ConvertTo-Json -Compress))
    }
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
} finally {
    if ($null -ne $dialog) { $dialog.Dispose() }
    if ($null -ne $owner) { $owner.Dispose() }
}
"#;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let payload = dialog_payload(save, current)?;
    let mut child = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-STA", "-Command", SCRIPT])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|error| {
            format!("Could not open the notebook file dialog: {error}. Enter a file path instead.")
        })?;
    let send_result = match child.stdin.take() {
        Some(mut stdin) => stdin.write_all(&payload),
        None => Err(std::io::Error::other(
            "The file dialog input is unavailable",
        )),
    };
    if let Err(error) = send_result {
        let stop_error = child.kill().err();
        let cleanup_detail = match child.wait() {
            Ok(_) => String::new(),
            Err(wait_error) => match stop_error {
                Some(stop_error) => format!(
                    " The dialog could not be stopped ({stop_error}) or closed ({wait_error})."
                ),
                None => format!(" The dialog could not be closed: {wait_error}."),
            },
        };
        return Err(format!(
            "Could not send the notebook path to the file dialog: {error}.{cleanup_detail} Enter a file path instead."
        ));
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("Could not wait for the notebook file dialog: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = if detail.trim().is_empty() {
            output.status.to_string()
        } else {
            detail.trim().to_owned()
        };
        return Err(format!(
            "The notebook file dialog failed: {detail}. Enter a file path instead."
        ));
    }
    parse_dialog_response(&output.stdout)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn choose_notebook_file(
    _save: bool,
    _current: Option<&Path>,
) -> Result<Option<PathBuf>, String> {
    Err("Native notebook file dialogs are available on Windows. Enter a file path instead.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Result as IoResult};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "equal-notebook-tests-{}-{timestamp}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test directory");
            Self(path)
        }

        fn entries(&self) -> Vec<PathBuf> {
            fs::read_dir(&self.0)
                .expect("read test directory")
                .map(|entry| entry.expect("read entry").path())
                .collect()
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            for path in self.entries() {
                fs::remove_file(path).expect("remove owned test file");
            }
            fs::remove_dir(&self.0).expect("remove empty test directory");
        }
    }

    #[test]
    fn reads_actual_file_and_rejects_files_over_limit() {
        let directory = TestDirectory::new();
        let path = directory.0.join("read.opencalc.json");
        fs::write(&path, b"{\"version\":1}").unwrap();
        assert_eq!(read_notebook(&path).unwrap(), b"{\"version\":1}");
        let limit = vec![b'x'; MAX_NOTEBOOK_BYTES];
        fs::write(&path, &limit).unwrap();
        assert_eq!(read_notebook(&path).unwrap(), limit);
        fs::write(&path, vec![b'x'; MAX_NOTEBOOK_BYTES + 1]).unwrap();
        assert!(read_notebook(&path).unwrap_err().contains("2 MiB"));
    }

    #[test]
    fn bounded_reader_does_not_trust_size_or_read_past_limit_plus_one() {
        struct CountingReader {
            read: usize,
        }
        impl Read for CountingReader {
            fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
                assert!(self.read < MAX_NOTEBOOK_BYTES + 1, "read beyond the limit");
                let count = buffer.len().min(MAX_NOTEBOOK_BYTES + 1 - self.read);
                buffer[..count].fill(b'x');
                self.read += count;
                Ok(count)
            }
        }
        assert!(read_bounded(CountingReader { read: 0 }).is_err());
        assert_eq!(read_bounded(Cursor::new(b"small")).unwrap(), b"small");
    }

    #[test]
    fn creates_and_replaces_notebook_without_leaving_temporary_files() {
        let directory = TestDirectory::new();
        let path = directory.0.join("Sam's café 数学.opencalc.json");
        write_notebook(&path, b"first").unwrap();
        write_notebook(&path, b"replacement").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        assert_eq!(directory.entries(), vec![path]);
    }

    #[test]
    fn oversized_write_preserves_existing_destination() {
        let directory = TestDirectory::new();
        let path = directory.0.join("keep.opencalc.json");
        fs::write(&path, b"original").unwrap();
        assert!(write_notebook(&path, &vec![0; MAX_NOTEBOOK_BYTES + 1]).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(directory.entries(), vec![path]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn denied_windows_replacement_preserves_destination_and_cleans_owned_temporary() {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 1;
        let directory = TestDirectory::new();
        let path = directory.0.join("locked.opencalc.json");
        fs::write(&path, b"original").unwrap();
        let held = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&path)
            .unwrap();
        assert!(write_notebook(&path, b"replacement").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(directory.entries(), vec![path]);
        drop(held);
    }

    #[test]
    fn dialog_paths_are_utf8_json_data_and_round_trip_exactly() {
        let path = Path::new("C:\\school\\Sam's \"quoted\" $name `tick` 数学.opencalc.json");
        let payload = dialog_payload(true, Some(path)).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(value["save"], true);
        assert_eq!(value["current"], path.to_str().unwrap());
        let response = serde_json::json!({"cancelled":false,"path":path.to_str().unwrap()});
        assert_eq!(
            parse_dialog_response(&serde_json::to_vec(&response).unwrap()).unwrap(),
            Some(path.to_path_buf())
        );
    }

    #[test]
    fn cancellation_and_malformed_dialog_payloads_are_distinct() {
        assert_eq!(
            parse_dialog_response(b"{\"cancelled\":true,\"path\":null}").unwrap(),
            None
        );
        for response in [
            r#"{"cancelled":false,"path":null}"#,
            r#"{"cancelled":false,"path":" "}"#,
            r#"{"cancelled":true,"path":"unexpected"}"#,
            r#"{"cancelled":false,"path":"bad\u0000path"}"#,
            "not JSON",
        ] {
            assert!(parse_dialog_response(response.as_bytes()).is_err());
        }
        assert!(parse_dialog_response(&[0xff]).is_err());
    }
}
