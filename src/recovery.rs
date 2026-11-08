//! Opt-in recovery drafts. All encoding and disk access run on a single background worker.
//! A draft contains only a validated notebook, never a target path for a later Save.

use crate::{notebook::Notebook, notebook_files};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const CHECKPOINT_INTERVAL: Duration = Duration::from_secs(5);
const LATEST: &str = "latest.opencalc.json";
const PREVIOUS: &str = "previous.opencalc.json";
const DISCARDED: &str = "discarded";
const DISCARD_MARKER: &[u8] = b"OpenCalc recovery explicitly discarded\n";

pub(crate) enum RecoveryEvent {
    Candidate {
        document: Box<Notebook>,
        from_previous: bool,
    },
    Saved,
    Discarded,
    Missing,
    Failed(String),
}

#[derive(Default)]
struct Requests {
    enabled: bool,
    revision: u64,
    checkpoint: Option<Notebook>,
    check: bool,
    discard: bool,
    awaiting_decision: bool,
    stop: bool,
}

#[derive(Default)]
struct Shared {
    requests: Mutex<Requests>,
    wake: Condvar,
    active: AtomicBool,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Requests> {
        self.requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn current(&self, revision: u64) -> bool {
        let requests = self.lock();
        requests.revision == revision && requests.enabled && !requests.stop
    }
}

pub(crate) struct RecoveryManager {
    app_data_dir: Option<PathBuf>,
    shared: Arc<Shared>,
    events: Receiver<(u64, RecoveryEvent)>,
    worker: Option<JoinHandle<()>>,
    ever_started: bool,
    local_event: Option<(u64, RecoveryEvent)>,
    start_error: Option<String>,
    interval: Duration,
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RecoveryManager {
    /// The caller supplies the application's private data directory, never a notebook directory.
    /// Construction and all public scheduling methods perform no file IO.
    pub(crate) fn new(app_data_dir: Option<PathBuf>) -> Self {
        let (_, events) = mpsc::channel();
        Self {
            app_data_dir,
            shared: Arc::new(Shared::default()),
            events,
            worker: None,
            ever_started: false,
            local_event: None,
            start_error: None,
            interval: CHECKPOINT_INTERVAL,
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.shared.lock().enabled
    }

    pub(crate) fn busy(&self) -> bool {
        if self.worker.is_none() {
            return false;
        }
        let requests = self.shared.lock();
        self.shared.active.load(Ordering::Acquire)
            || requests.check
            || requests.discard
            || (requests.checkpoint.is_some() && !requests.awaiting_decision)
    }

    pub(crate) fn awaiting_decision(&self) -> bool {
        self.shared.lock().awaiting_decision
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        let was_enabled = self.enabled();
        self.shared.lock().enabled = enabled;
        if enabled {
            self.start_worker();
        } else if was_enabled {
            self.discard();
        }
        self.shared.wake.notify_one();
    }

    /// Discovery suspends checkpoints until Missing or an explicit recovery decision.
    pub(crate) fn check(&mut self) {
        if !self.enabled() {
            return;
        }
        self.start_worker();
        let mut requests = self.shared.lock();
        requests.revision = requests.revision.wrapping_add(1);
        requests.checkpoint = None;
        requests.check = true;
        requests.awaiting_decision = true;
        self.shared.wake.notify_one();
    }

    /// New snapshots replace any queued snapshot; encoding and comparison happen off the UI thread.
    pub(crate) fn checkpoint(&mut self, document: Notebook) {
        let mut requests = self.shared.lock();
        if requests.enabled && !requests.awaiting_decision {
            requests.checkpoint = Some(document);
            self.shared.wake.notify_one();
        }
    }

    /// Call only after the user applies the offered document; it does not restore a Save target.
    pub(crate) fn accept_recovery(&mut self) {
        self.shared.lock().awaiting_decision = false;
        self.shared.wake.notify_one();
    }

    /// Wait for Discarded before completing an explicit destructive notebook action.
    /// A durable marker suppresses old backups even if subsequent cleanup fails.
    pub(crate) fn discard(&mut self) {
        let mut requests = self.shared.lock();
        requests.revision = requests.revision.wrapping_add(1);
        requests.checkpoint = None;
        requests.check = false;
        requests.awaiting_decision = false;
        if !requests.enabled && !self.ever_started {
            // Protection was never started, so there are no session drafts to clear.
            self.local_event = Some((requests.revision, RecoveryEvent::Discarded));
            return;
        }
        requests.discard = true;
        drop(requests);
        self.start_worker();
        self.shared.wake.notify_one();
    }

    pub(crate) fn poll(&mut self) -> Option<RecoveryEvent> {
        if let Some(error) = self.start_error.take() {
            return Some(RecoveryEvent::Failed(error));
        }
        if let Some((revision, event)) = self.local_event.take()
            && revision == self.shared.lock().revision
        {
            return Some(event);
        }
        loop {
            match self.events.try_recv() {
                Ok((revision, event)) => {
                    let mut requests = self.shared.lock();
                    if revision != requests.revision {
                        continue;
                    }
                    if matches!(event, RecoveryEvent::Missing | RecoveryEvent::Discarded) {
                        requests.awaiting_decision = false;
                        self.shared.wake.notify_one();
                    }
                    return Some(event);
                }
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    if self.worker.take().is_some() {
                        return Some(RecoveryEvent::Failed(
                            "The local recovery worker stopped. Your notebook files were not changed."
                                .into(),
                        ));
                    }
                    return None;
                }
            }
        }
    }

    fn start_worker(&mut self) {
        if self.worker.is_some() {
            return;
        }
        let (sender, events) = mpsc::channel();
        self.events = events;
        let shared = Arc::clone(&self.shared);
        let directory = self.app_data_dir.clone();
        let interval = self.interval;
        match thread::Builder::new()
            .name("opencalc-recovery".into())
            .spawn(move || worker(shared, sender, directory, interval))
        {
            Ok(worker) => {
                self.ever_started = true;
                self.worker = Some(worker);
            }
            Err(error) => {
                self.start_error = Some(format!("Could not start local recovery: {error}"));
            }
        }
    }

    fn stop(&self) {
        self.shared.lock().stop = true;
        self.shared.wake.notify_one();
    }

    #[cfg(test)]
    fn shutdown(mut self) {
        self.stop();
        if let Some(worker) = self.worker.take() {
            worker.join().expect("recovery worker should finish");
        }
    }
}

impl Drop for RecoveryManager {
    fn drop(&mut self) {
        self.stop();
        // Do not join here: the OS may still be completing disk IO as the UI closes.
    }
}

enum Work {
    Check(u64),
    Discard(u64),
    Checkpoint(u64, Box<Notebook>),
}

fn next_work(shared: &Shared, attempted: Option<Instant>, interval: Duration) -> Option<Work> {
    let mut requests = shared.lock();
    loop {
        if requests.stop {
            return None;
        }
        let work = if requests.discard {
            requests.discard = false;
            Some(Work::Discard(requests.revision))
        } else if requests.enabled && requests.check {
            requests.check = false;
            Some(Work::Check(requests.revision))
        } else if requests.enabled && !requests.awaiting_decision && requests.checkpoint.is_some() {
            let delay = attempted
                .map(|instant| interval.saturating_sub(instant.elapsed()))
                .unwrap_or_default();
            if !delay.is_zero() {
                requests = shared
                    .wake
                    .wait_timeout(requests, delay)
                    .unwrap_or_else(|error| error.into_inner())
                    .0;
                continue;
            }
            requests
                .checkpoint
                .take()
                .map(|document| Work::Checkpoint(requests.revision, Box::new(document)))
        } else {
            None
        };
        if let Some(work) = work {
            shared.active.store(true, Ordering::Release);
            return Some(work);
        }
        requests = shared
            .wake
            .wait(requests)
            .unwrap_or_else(|error| error.into_inner());
    }
}

fn worker(
    shared: Arc<Shared>,
    sender: Sender<(u64, RecoveryEvent)>,
    app_data_dir: Option<PathBuf>,
    interval: Duration,
) {
    let mut attempted = None;
    let mut last_content = None;
    while let Some(work) = next_work(&shared, attempted, interval) {
        let (revision, result) = match work {
            Work::Check(revision) => {
                let result = inspect(app_data_dir.as_deref()).and_then(|event| {
                    if let RecoveryEvent::Candidate { document, .. } = &event {
                        last_content = Some(document.content_bytes()?);
                    }
                    Ok(event)
                });
                (revision, result)
            }
            Work::Discard(revision) => {
                last_content = None;
                attempted = None;
                (
                    revision,
                    discard_files(app_data_dir.as_deref()).map(|()| RecoveryEvent::Discarded),
                )
            }
            Work::Checkpoint(revision, document) => {
                attempted = Some(Instant::now());
                (
                    revision,
                    checkpoint(
                        app_data_dir.as_deref(),
                        &document,
                        &mut last_content,
                        &shared,
                        revision,
                    )
                    .map(|()| RecoveryEvent::Saved),
                )
            }
        };
        shared.active.store(false, Ordering::Release);
        let event = result.unwrap_or_else(RecoveryEvent::Failed);
        if sender.send((revision, event)).is_err() {
            return;
        }
    }
}

fn recovery_directory(
    app_data_dir: Option<&Path>,
    create: bool,
) -> Result<Option<PathBuf>, String> {
    let app_data_dir = app_data_dir.ok_or_else(|| {
        "A private application data directory is unavailable. Local recovery is off.".to_owned()
    })?;
    if !app_data_dir.is_absolute() {
        return Err("Local recovery needs an absolute application data directory.".into());
    }
    reject_source_directory(app_data_dir)?;
    let directory = app_data_dir.join("recovery");
    if !directory
        .try_exists()
        .map_err(|error| format!("Could not inspect the local recovery directory: {error}"))?
        && !create
    {
        return Ok(None);
    }
    if create {
        fs::create_dir_all(&directory)
            .map_err(|error| format!("Could not create the local recovery directory: {error}"))?;
    }
    let canonical = fs::canonicalize(&directory)
        .map_err(|error| format!("Could not access the local recovery directory: {error}"))?;
    let canonical_parent = fs::canonicalize(app_data_dir)
        .map_err(|error| format!("Could not access the application data directory: {error}"))?;
    reject_source_directory(&canonical)?;
    if canonical.parent() != Some(canonical_parent.as_path()) || !canonical.is_dir() {
        return Err(
            "The recovery directory must stay inside the application data directory.".into(),
        );
    }
    Ok(Some(canonical))
}

fn reject_source_directory(path: &Path) -> Result<(), String> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let canonical_source = fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
    if path.starts_with(source) || path.ancestors().any(|parent| parent.join(".git").exists()) {
        return Err("Recovery drafts cannot be stored in a source repository.".into());
    }
    // Resolve existing ancestors before creating anything, including paths through symlinks.
    if let Some(existing) = path.ancestors().find(|parent| parent.exists()) {
        let canonical = fs::canonicalize(existing)
            .map_err(|error| format!("Could not check the recovery location: {error}"))?;
        if canonical.starts_with(&canonical_source)
            || canonical
                .ancestors()
                .any(|parent| parent.join(".git").exists())
        {
            return Err("Recovery drafts cannot be stored in a source repository.".into());
        }
    }
    Ok(())
}

fn read_generation(path: &Path) -> Result<Option<(Notebook, Vec<u8>)>, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            let bytes = notebook_files::read_notebook(path)?;
            let document = Notebook::decode(&bytes)?;
            Ok(Some((document, bytes)))
        }
        Ok(_) => Err("A recovery draft is not a regular file.".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Could not inspect a local recovery draft: {error}")),
    }
}

fn inspect(app_data_dir: Option<&Path>) -> Result<RecoveryEvent, String> {
    let Some(directory) = recovery_directory(app_data_dir, false)? else {
        return Ok(RecoveryEvent::Missing);
    };
    if has_discard_marker(&directory)? {
        return Ok(RecoveryEvent::Missing);
    }
    let latest = read_generation(&directory.join(LATEST));
    if let Ok(Some((document, _))) = latest {
        return Ok(RecoveryEvent::Candidate {
            document: Box::new(document),
            from_previous: false,
        });
    }
    let previous = read_generation(&directory.join(PREVIOUS));
    if let Ok(Some((document, _))) = previous {
        return Ok(RecoveryEvent::Candidate {
            document: Box::new(document),
            from_previous: true,
        });
    }
    match (latest, previous) {
        (Ok(None), Ok(None)) => Ok(RecoveryEvent::Missing),
        (Err(error), _) | (_, Err(error)) => Err(format!(
            "No valid local recovery draft could be opened: {error}. Your notebook files were not changed."
        )),
        _ => unreachable!("valid recovery candidates returned above"),
    }
}

fn checkpoint(
    app_data_dir: Option<&Path>,
    document: &Notebook,
    last_content: &mut Option<Vec<u8>>,
    shared: &Shared,
    revision: u64,
) -> Result<(), String> {
    let content = document.content_bytes()?;
    if last_content.as_ref() == Some(&content) || !shared.current(revision) {
        return Ok(());
    }
    let bytes = document.to_bytes()?;
    let directory = recovery_directory(app_data_dir, true)?
        .ok_or_else(|| "The recovery directory is unavailable.".to_owned())?;
    if !shared.current(revision) {
        return Ok(());
    }
    let marker = directory.join(DISCARDED);
    if !has_discard_marker(&directory)?
        && let Ok(Some((old_document, old_bytes))) = read_generation(&directory.join(LATEST))
    {
        if old_document.content_bytes()? == content {
            *last_content = Some(content);
            return Ok(());
        }
        notebook_files::write_notebook(&directory.join(PREVIOUS), &old_bytes)?;
    }
    if !shared.current(revision) {
        return Ok(());
    }
    notebook_files::write_notebook(&directory.join(LATEST), &bytes)?;
    remove_draft(&marker)?;
    *last_content = Some(content);
    Ok(())
}

fn remove_draft(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Could not remove a local recovery draft: {error}")),
    }
}

fn has_discard_marker(directory: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(directory.join(DISCARDED)) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "Could not inspect the recovery discard marker: {error}"
        )),
    }
}

fn discard_files(app_data_dir: Option<&Path>) -> Result<(), String> {
    let Some(directory) = recovery_directory(app_data_dir, false)? else {
        return Ok(());
    };
    notebook_files::write_notebook(&directory.join(DISCARDED), DISCARD_MARKER)?;
    let latest = remove_draft(&directory.join(LATEST));
    let previous = remove_draft(&directory.join(PREVIOUS));
    latest.and(previous)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "opencalc-recovery-test-{}-{timestamp}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn manager(&self, interval: Duration) -> RecoveryManager {
            let mut manager = RecoveryManager::new(Some(self.0.clone()));
            manager.interval = interval;
            manager
        }

        fn draft(&self, name: &str) -> PathBuf {
            self.0.join("recovery").join(name)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            // Every entry is in this test's create_new-equivalent unique directory.
            // Cleanup is shallow; never recursively delete or follow directory links.
            let recovery = self.0.join("recovery");
            if recovery.is_dir() {
                for entry in fs::read_dir(&recovery).unwrap() {
                    fs::remove_file(entry.unwrap().path()).unwrap();
                }
                fs::remove_dir(&recovery).unwrap();
            } else if recovery.exists() {
                fs::remove_file(&recovery).unwrap();
            }
            for entry in fs::read_dir(&self.0).unwrap() {
                fs::remove_file(entry.unwrap().path()).unwrap();
            }
            fs::remove_dir(&self.0).unwrap();
        }
    }

    fn document(title: &str) -> Notebook {
        crate::EqualApp::with_settings(crate::Settings::default(), None).capture_notebook(
            title,
            "recovery notes",
            false,
        )
    }

    fn event(manager: &mut RecoveryManager) -> RecoveryEvent {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(event) = manager.poll() {
                return event;
            }
            assert!(Instant::now() < deadline, "recovery worker did not respond");
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn saved(manager: &mut RecoveryManager) {
        match event(manager) {
            RecoveryEvent::Saved => {}
            RecoveryEvent::Failed(error) => panic!("checkpoint failed: {error}"),
            _ => panic!("expected checkpoint completion"),
        }
    }

    #[test]
    fn disabled_by_default_and_restart_requires_an_explicit_recovery_decision() {
        let directory = TestDirectory::new();
        let mut manager = directory.manager(Duration::ZERO);
        assert!(!manager.enabled());
        manager.checkpoint(document("Not opted in"));
        assert!(!directory.draft(LATEST).exists());
        manager.set_enabled(true);
        manager.checkpoint(document("Recover this"));
        saved(&mut manager);
        manager.shutdown();

        let mut restarted = directory.manager(Duration::ZERO);
        restarted.set_enabled(true);
        restarted.check();
        match event(&mut restarted) {
            RecoveryEvent::Candidate {
                document,
                from_previous,
            } => {
                assert_eq!(document.title, "Recover this");
                assert!(!from_previous);
            }
            _ => panic!("expected a validated recovery candidate after restart"),
        }
        assert!(restarted.awaiting_decision());
        restarted.checkpoint(document("Blocked until decision"));
        assert_eq!(
            Notebook::decode(&fs::read(directory.draft(LATEST)).unwrap())
                .unwrap()
                .title,
            "Recover this"
        );
        restarted.accept_recovery();
        restarted.checkpoint(document("Explicitly recovered and edited"));
        saved(&mut restarted);
        restarted.shutdown();
    }

    #[test]
    fn unchanged_content_does_not_rotate_and_throttle_keeps_only_the_latest_snapshot() {
        let directory = TestDirectory::new();
        let mut manager = directory.manager(Duration::from_millis(80));
        manager.set_enabled(true);
        manager.checkpoint(document("First"));
        saved(&mut manager);
        manager.checkpoint(document("First"));
        saved(&mut manager);
        assert!(!directory.draft(PREVIOUS).exists());
        manager.checkpoint(document("Intermediate"));
        manager.checkpoint(document("Latest"));
        saved(&mut manager);
        manager.shutdown();
        assert_eq!(
            Notebook::decode(&fs::read(directory.draft(LATEST)).unwrap())
                .unwrap()
                .title,
            "Latest"
        );
        assert_eq!(
            Notebook::decode(&fs::read(directory.draft(PREVIOUS)).unwrap())
                .unwrap()
                .title,
            "First"
        );
        assert_eq!(
            fs::read_dir(directory.0.join("recovery")).unwrap().count(),
            2
        );
    }

    #[test]
    fn a_truncated_latest_uses_the_previous_valid_generation_after_restart() {
        let directory = TestDirectory::new();
        let mut manager = directory.manager(Duration::ZERO);
        manager.set_enabled(true);
        manager.checkpoint(document("Previous valid"));
        saved(&mut manager);
        manager.checkpoint(document("Latest before crash"));
        saved(&mut manager);
        manager.shutdown();
        fs::write(directory.draft(LATEST), b"{\"format\":").unwrap();

        let mut restarted = directory.manager(Duration::ZERO);
        restarted.set_enabled(true);
        restarted.check();
        match event(&mut restarted) {
            RecoveryEvent::Candidate {
                document,
                from_previous,
            } => {
                assert_eq!(document.title, "Previous valid");
                assert!(from_previous);
            }
            _ => panic!("expected the previous validated generation"),
        }
        restarted.shutdown();
    }

    #[test]
    fn explicit_discard_cancels_queued_work_and_survives_a_new_manager() {
        let directory = TestDirectory::new();
        let mut manager = directory.manager(Duration::from_secs(1));
        manager.set_enabled(true);
        manager.checkpoint(document("Old work"));
        saved(&mut manager);
        manager.checkpoint(document("Queued unwanted work"));
        manager.discard();
        assert!(matches!(event(&mut manager), RecoveryEvent::Discarded));
        manager.shutdown();
        assert!(!directory.draft(LATEST).exists());
        assert!(!directory.draft(PREVIOUS).exists());
        assert!(directory.draft(DISCARDED).exists());
        // Even a leftover backup must remain suppressed after an explicit discard.
        fs::write(
            directory.draft(PREVIOUS),
            document("Leftover").to_bytes().unwrap(),
        )
        .unwrap();
        let mut restarted = directory.manager(Duration::ZERO);
        restarted.set_enabled(true);
        restarted.check();
        assert!(matches!(event(&mut restarted), RecoveryEvent::Missing));
        restarted.shutdown();
    }

    #[test]
    fn failed_recovery_write_preserves_an_unrelated_original_file() {
        let directory = TestDirectory::new();
        let original = directory.0.join("student.opencalc.json");
        fs::write(&original, b"original student file").unwrap();
        fs::write(directory.0.join("recovery"), b"directory unavailable").unwrap();
        let mut manager = directory.manager(Duration::ZERO);
        manager.set_enabled(true);
        manager.checkpoint(document("Cannot be checkpointed"));
        assert!(matches!(event(&mut manager), RecoveryEvent::Failed(_)));
        manager.shutdown();
        assert_eq!(fs::read(original).unwrap(), b"original student file");
    }

    #[test]
    fn turning_protection_off_clears_drafts_and_acknowledges_once() {
        let directory = TestDirectory::new();
        let mut manager = directory.manager(Duration::ZERO);
        manager.discard();
        assert!(matches!(event(&mut manager), RecoveryEvent::Discarded));
        assert!(!directory.0.join("recovery").exists());
        manager.set_enabled(true);
        manager.checkpoint(document("Opted in work"));
        saved(&mut manager);
        manager.set_enabled(false);
        assert!(matches!(event(&mut manager), RecoveryEvent::Discarded));
        assert!(manager.poll().is_none());
        assert!(!manager.enabled());
        manager.shutdown();
        let mut restarted = directory.manager(Duration::ZERO);
        restarted.set_enabled(true);
        restarted.check();
        assert!(matches!(event(&mut restarted), RecoveryEvent::Missing));
        assert!(!restarted.awaiting_decision());
        restarted.shutdown();
    }

    #[test]
    fn invalid_generations_report_an_error_without_modifying_their_bytes() {
        let directory = TestDirectory::new();
        fs::create_dir(directory.0.join("recovery")).unwrap();
        let invalid = b"{\"format\":\"unrecognized\",\"version\":999}";
        fs::write(directory.draft(LATEST), invalid).unwrap();
        fs::write(directory.draft(PREVIOUS), b"truncated").unwrap();
        let mut manager = directory.manager(Duration::ZERO);
        manager.set_enabled(true);
        manager.check();
        assert!(matches!(event(&mut manager), RecoveryEvent::Failed(_)));
        assert!(manager.awaiting_decision());
        manager.shutdown();
        assert_eq!(fs::read(directory.draft(LATEST)).unwrap(), invalid);
        assert_eq!(fs::read(directory.draft(PREVIOUS)).unwrap(), b"truncated");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn denied_atomic_replacement_preserves_last_valid_draft_for_restart() {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;
        let directory = TestDirectory::new();
        let mut manager = directory.manager(Duration::ZERO);
        manager.set_enabled(true);
        manager.checkpoint(document("Last valid"));
        saved(&mut manager);
        let held = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(directory.draft(LATEST))
            .unwrap();
        manager.checkpoint(document("Write denied"));
        assert!(matches!(event(&mut manager), RecoveryEvent::Failed(_)));
        manager.shutdown();
        drop(held);
        let mut restarted = directory.manager(Duration::ZERO);
        restarted.set_enabled(true);
        restarted.check();
        match event(&mut restarted) {
            RecoveryEvent::Candidate { document, .. } => assert_eq!(document.title, "Last valid"),
            _ => panic!("expected preserved valid work"),
        }
        restarted.shutdown();
    }

    #[test]
    fn relative_storage_is_rejected_without_writing_to_the_working_directory() {
        assert!(recovery_directory(Some(Path::new("relative-profile")), true).is_err());
    }
}
