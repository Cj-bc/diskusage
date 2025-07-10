use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::data::{DirectoryNode, ScanError, ScanProgress, ScanResult};

pub struct Scanner {
    sender: Option<Sender<ScanMessage>>,
    receiver: Arc<Mutex<Receiver<ScanMessage>>>,
    is_scanning: Arc<Mutex<bool>>,
    cancel_flag: Arc<Mutex<bool>>,
    current_progress: Arc<Mutex<Option<ScanProgress>>>,
    scan_result: Arc<Mutex<Option<Result<ScanResult, ScanError>>>>,
}

#[derive(Debug)]
enum ScanMessage {
    Progress(ScanProgress),
    Completed(Result<ScanResult, ScanError>),
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
    pub current_depth: usize,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            follow_symlinks: false,
            max_depth: None,
            current_depth: 0,
        }
    }
}

impl Scanner {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            sender: Some(sender),
            receiver: Arc::new(Mutex::new(receiver)),
            is_scanning: Arc::new(Mutex::new(false)),
            cancel_flag: Arc::new(Mutex::new(false)),
            current_progress: Arc::new(Mutex::new(None)),
            scan_result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_scan(&mut self, path: PathBuf) {
        if *self.is_scanning.lock().unwrap() {
            log::warn!("Scan already in progress, ignoring new scan request");
            return;
        }

        *self.is_scanning.lock().unwrap() = true;
        *self.cancel_flag.lock().unwrap() = false;
        *self.current_progress.lock().unwrap() = None;
        *self.scan_result.lock().unwrap() = None;

        let sender = self.sender.as_ref().unwrap().clone();
        let cancel_flag = Arc::clone(&self.cancel_flag);
        let is_scanning = Arc::clone(&self.is_scanning);

        thread::spawn(move || {
            let start_time = Instant::now();
            let result = Self::scan_directory_internal(&path, &ScanOptions::default(), &sender, &cancel_flag);
            
            let scan_result = match result {
                Ok(mut root) => {
                    let scan_duration = start_time.elapsed();
                    root.calculate_size();
                    
                    Ok(ScanResult {
                        total_size: root.size,
                        total_files: root.file_count,
                        total_dirs: root.dir_count,
                        root,
                        scan_duration,
                        errors: Vec::new(),
                    })
                }
                Err(error) => Err(error),
            };

            let _ = sender.send(ScanMessage::Completed(scan_result));
            *is_scanning.lock().unwrap() = false;
        });
    }

    pub fn cancel(&self) {
        *self.cancel_flag.lock().unwrap() = true;
    }

    pub fn is_scanning(&self) -> bool {
        *self.is_scanning.lock().unwrap()
    }

    pub fn get_progress(&self) -> Option<ScanProgress> {
        // Check for new messages
        if let Ok(receiver) = self.receiver.try_lock() {
            while let Ok(message) = receiver.try_recv() {
                match message {
                    ScanMessage::Progress(progress) => {
                        *self.current_progress.lock().unwrap() = Some(progress);
                    }
                    ScanMessage::Completed(result) => {
                        *self.scan_result.lock().unwrap() = Some(result);
                        *self.current_progress.lock().unwrap() = None;
                    }
                }
            }
        }

        self.current_progress.lock().unwrap().clone()
    }

    pub fn take_result(&self) -> Option<Result<ScanResult, ScanError>> {
        self.scan_result.lock().unwrap().take()
    }

    fn scan_directory_internal(
        path: &Path,
        options: &ScanOptions,
        sender: &Sender<ScanMessage>,
        cancel_flag: &Arc<Mutex<bool>>,
    ) -> Result<DirectoryNode, ScanError> {
        if *cancel_flag.lock().unwrap() {
            return Err(ScanError::Cancelled);
        }

        let mut node = DirectoryNode::new(path.to_path_buf());
        
        // Check if we've reached maximum depth
        if let Some(max_depth) = options.max_depth {
            if options.current_depth >= max_depth {
                return Ok(node);
            }
        }

        // Send progress update
        let progress = ScanProgress {
            current_path: path.to_path_buf(),
            scanned_files: 0,
            scanned_dirs: options.current_depth,
            total_size: 0,
            elapsed: Duration::default(),
            estimated_remaining: None,
        };
        let _ = sender.send(ScanMessage::Progress(progress));

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                return match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        Err(ScanError::PermissionDenied { path: path.to_path_buf() })
                    }
                    std::io::ErrorKind::NotFound => {
                        Err(ScanError::PathNotFound { path: path.to_path_buf() })
                    }
                    _ => Err(ScanError::IoError {
                        path: path.to_path_buf(),
                        message: e.to_string(),
                    }),
                };
            }
        };

        for entry in entries {
            if *cancel_flag.lock().unwrap() {
                return Err(ScanError::Cancelled);
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue, // Skip invalid entries
            };

            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Handle symlinks
                if !options.follow_symlinks && entry_path.is_symlink() {
                    continue;
                }

                let mut child_options = options.clone();
                child_options.current_depth += 1;

                match Self::scan_directory_internal(&entry_path, &child_options, sender, cancel_flag) {
                    Ok(child_node) => {
                        node.size += child_node.size;
                        node.dir_count += 1 + child_node.dir_count;
                        node.file_count += child_node.file_count;
                        node.children.push(child_node);
                    }
                    Err(ScanError::Cancelled) => {
                        return Err(ScanError::Cancelled);
                    }
                    Err(e) => {
                        log::warn!("Error scanning {}: {}", entry_path.display(), e);
                        // Continue scanning other directories
                    }
                }
            } else {
                // Regular file
                let file_size = entry
                    .metadata()
                    .map(|m| m.len())
                    .unwrap_or(0);

                node.size += file_size;
                node.file_count += 1;
            }
        }

        // Update last modified time
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                node.last_modified = modified;
            }
        }

        Ok(node)
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}