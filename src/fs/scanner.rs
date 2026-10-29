use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::data::{DirectoryNode, ScanError, ScanProgress, ScanResult};

/// Minimum time between two progress messages sent by the scan thread.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(50);

/// What the scanner has to report since the last [`Scanner::poll`].
#[derive(Debug)]
pub enum ScanUpdate {
    Progress(ScanProgress),
    Finished(Result<ScanResult, ScanError>),
}

#[derive(Debug)]
enum ScanMessage {
    Progress(ScanProgress),
    Completed(Result<ScanResult, ScanError>),
}

/// A scan running on a background thread.
struct ActiveScan {
    receiver: Receiver<ScanMessage>,
    cancel_flag: Arc<AtomicBool>,
}

/// Runs directory scans on a background thread, one at a time.
#[derive(Default)]
pub struct Scanner {
    active: Option<ActiveScan>,
}

#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
}

impl Scanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts scanning `path`. A scan that is already running is cancelled
    /// and its results are discarded.
    pub fn start_scan(&mut self, path: PathBuf, options: ScanOptions) {
        self.cancel();

        let (sender, receiver) = mpsc::channel();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let thread_cancel_flag = Arc::clone(&cancel_flag);

        thread::spawn(move || {
            let result = scan(&path, &options, &sender, &thread_cancel_flag);
            // The receiver is gone if the scan was superseded; nothing to do then.
            let _ = sender.send(ScanMessage::Completed(result));
        });

        self.active = Some(ActiveScan {
            receiver,
            cancel_flag,
        });
    }

    /// Asks the running scan, if any, to stop. Its result is discarded.
    pub fn cancel(&mut self) {
        if let Some(active) = self.active.take() {
            active.cancel_flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_scanning(&self) -> bool {
        self.active.is_some()
    }

    /// Drains pending messages from the scan thread. Returns the result once
    /// the scan has finished, otherwise the latest progress, if any arrived.
    pub fn poll(&mut self) -> Option<ScanUpdate> {
        let active = self.active.as_ref()?;
        let mut latest_progress = None;

        loop {
            match active.receiver.try_recv() {
                Ok(ScanMessage::Progress(progress)) => latest_progress = Some(progress),
                Ok(ScanMessage::Completed(result)) => {
                    self.active = None;
                    return Some(ScanUpdate::Finished(result));
                }
                Err(TryRecvError::Empty) => return latest_progress.map(ScanUpdate::Progress),
                Err(TryRecvError::Disconnected) => {
                    self.active = None;
                    return Some(ScanUpdate::Finished(Err(ScanError::Other {
                        message: "scan thread terminated unexpectedly".to_string(),
                    })));
                }
            }
        }
    }
}

/// Mutable state shared by every level of one recursive scan.
struct ScanContext<'a> {
    options: &'a ScanOptions,
    sender: &'a Sender<ScanMessage>,
    cancel_flag: &'a AtomicBool,
    started_at: Instant,
    last_progress_at: Option<Instant>,
    scanned_files: usize,
    scanned_dirs: usize,
    total_size: u64,
    errors: Vec<ScanError>,
}

impl ScanContext<'_> {
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }

    fn report_progress(&mut self, current_path: &Path) {
        let now = Instant::now();
        if self
            .last_progress_at
            .is_some_and(|last| now.duration_since(last) < PROGRESS_INTERVAL)
        {
            return;
        }
        self.last_progress_at = Some(now);

        let _ = self.sender.send(ScanMessage::Progress(ScanProgress {
            current_path: current_path.to_path_buf(),
            scanned_files: self.scanned_files,
            scanned_dirs: self.scanned_dirs,
            total_size: self.total_size,
            elapsed: now.duration_since(self.started_at),
            estimated_remaining: None,
        }));
    }
}

fn scan(
    path: &Path,
    options: &ScanOptions,
    sender: &Sender<ScanMessage>,
    cancel_flag: &AtomicBool,
) -> Result<ScanResult, ScanError> {
    let mut context = ScanContext {
        options,
        sender,
        cancel_flag,
        started_at: Instant::now(),
        last_progress_at: None,
        scanned_files: 0,
        scanned_dirs: 0,
        total_size: 0,
        errors: Vec::new(),
    };

    let root = scan_directory(path, 0, &mut context)?;

    let mut result = ScanResult::new(root);
    result.scan_duration = context.started_at.elapsed();
    result.errors = context.errors;
    Ok(result)
}

/// Scans `path` recursively. Only failing to read `path` itself is an error;
/// problems further down are recorded in the context and skipped.
fn scan_directory(
    path: &Path,
    depth: usize,
    context: &mut ScanContext<'_>,
) -> Result<DirectoryNode, ScanError> {
    if context.is_cancelled() {
        return Err(ScanError::Cancelled);
    }

    let mut node = DirectoryNode::new(path.to_path_buf());
    if let Ok(modified) = std::fs::metadata(path).and_then(|metadata| metadata.modified()) {
        node.last_modified = modified;
    }

    if context
        .options
        .max_depth
        .is_some_and(|max_depth| depth >= max_depth)
    {
        return Ok(node);
    }

    context.report_progress(path);

    let entries = std::fs::read_dir(path).map_err(|error| ScanError::from_io(path, &error))?;

    for entry in entries {
        if context.is_cancelled() {
            return Err(ScanError::Cancelled);
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                context.errors.push(ScanError::from_io(path, &error));
                continue;
            }
        };
        let entry_path = entry.path();

        // `DirEntry::metadata` does not follow symlinks; resolve them only when asked to.
        let metadata = match entry.metadata() {
            Ok(metadata) if metadata.is_symlink() && context.options.follow_symlinks => {
                std::fs::metadata(&entry_path)
            }
            other => other,
        };
        let metadata = match metadata {
            Ok(metadata) => metadata,
            Err(error) => {
                context.errors.push(ScanError::from_io(&entry_path, &error));
                continue;
            }
        };

        if metadata.is_dir() {
            match scan_directory(&entry_path, depth + 1, context) {
                Ok(child) => {
                    node.size += child.size;
                    node.dir_count += 1 + child.dir_count;
                    node.file_count += child.file_count;
                    node.add_child(child);
                }
                Err(ScanError::Cancelled) => return Err(ScanError::Cancelled),
                Err(error) => {
                    log::warn!("Skipping {}: {}", entry_path.display(), error);
                    context.errors.push(error);
                }
            }
        } else {
            node.size += metadata.len();
            node.file_count += 1;
            context.scanned_files += 1;
            context.total_size += metadata.len();
        }
    }

    context.scanned_dirs += 1;
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scan_blocking(path: &Path, options: ScanOptions) -> Result<ScanResult, ScanError> {
        let (sender, _receiver) = mpsc::channel();
        scan(path, &options, &sender, &AtomicBool::new(false))
    }

    #[test]
    fn counts_files_directly_in_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.bin"), [0u8; 100]).unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("b.bin"), [0u8; 30]).unwrap();

        let result = scan_blocking(dir.path(), ScanOptions::default()).unwrap();

        assert_eq!(result.total_size, 130);
        assert_eq!(result.total_files, 2);
        assert_eq!(result.total_dirs, 1);
        assert_eq!(result.root.children.len(), 1);
        assert_eq!(result.root.children[0].size, 30);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn respects_max_depth() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("deep.bin"), [0u8; 10]).unwrap();

        let options = ScanOptions {
            max_depth: Some(1),
            ..Default::default()
        };
        let result = scan_blocking(dir.path(), options).unwrap();

        assert_eq!(result.total_size, 0);
        assert_eq!(result.root.children[0].children.len(), 0);
    }

    #[test]
    fn missing_root_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");

        let result = scan_blocking(&missing, ScanOptions::default());

        assert!(matches!(result, Err(ScanError::PathNotFound { .. })));
    }

    #[test]
    fn cancelled_scan_reports_cancellation() {
        let dir = tempfile::tempdir().unwrap();
        let (sender, _receiver) = mpsc::channel();

        let result = scan(
            dir.path(),
            &ScanOptions::default(),
            &sender,
            &AtomicBool::new(true),
        );

        assert!(matches!(result, Err(ScanError::Cancelled)));
    }

    #[test]
    fn scanner_delivers_result_through_poll() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.bin"), [0u8; 7]).unwrap();
        let mut scanner = Scanner::new();

        scanner.start_scan(dir.path().to_path_buf(), ScanOptions::default());
        let deadline = Instant::now() + Duration::from_secs(10);
        let result = loop {
            if let Some(ScanUpdate::Finished(result)) = scanner.poll() {
                break result;
            }
            assert!(Instant::now() < deadline, "scan did not finish in time");
            thread::sleep(Duration::from_millis(5));
        };

        assert_eq!(result.unwrap().total_size, 7);
        assert!(!scanner.is_scanning());
    }
}
