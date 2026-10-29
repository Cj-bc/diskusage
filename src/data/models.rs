use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub children: Vec<DirectoryNode>,
    pub file_count: usize,
    pub dir_count: usize,
    pub last_modified: SystemTime,
    pub is_expanded: bool,
}

impl DirectoryNode {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        Self {
            name,
            path,
            size: 0,
            children: Vec::new(),
            file_count: 0,
            dir_count: 0,
            last_modified: SystemTime::UNIX_EPOCH,
            is_expanded: false,
        }
    }

    pub fn add_child(&mut self, child: DirectoryNode) {
        self.children.push(child);
    }

    pub fn find_child(&self, name: &str) -> Option<&DirectoryNode> {
        self.children.iter().find(|child| child.name == name)
    }

    pub fn find_child_mut(&mut self, name: &str) -> Option<&mut DirectoryNode> {
        self.children.iter_mut().find(|child| child.name == name)
    }

    pub fn total_items(&self) -> usize {
        self.file_count + self.dir_count
    }

    pub fn depth(&self) -> usize {
        self.children
            .iter()
            .map(|child| child.depth())
            .max()
            .unwrap_or(0) + 1
    }

    pub fn sort_by_size(&mut self, ascending: bool) {
        self.children.sort_by(|a, b| {
            let comparison = a.size.cmp(&b.size);
            if ascending { comparison } else { comparison.reverse() }
        });
        
        for child in &mut self.children {
            child.sort_by_size(ascending);
        }
    }

    pub fn sort_by_name(&mut self, ascending: bool) {
        self.children.sort_by(|a, b| {
            let comparison = a.name.to_lowercase().cmp(&b.name.to_lowercase());
            if ascending { comparison } else { comparison.reverse() }
        });
        
        for child in &mut self.children {
            child.sort_by_name(ascending);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub root: DirectoryNode,
    pub total_size: u64,
    pub total_files: usize,
    pub total_dirs: usize,
    pub scan_duration: Duration,
    pub errors: Vec<ScanError>,
}

impl ScanResult {
    pub fn new(root: DirectoryNode) -> Self {
        Self {
            total_size: root.size,
            total_files: root.file_count,
            total_dirs: root.dir_count,
            root,
            scan_duration: Duration::default(),
            errors: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.total_size > 0 || self.total_files > 0 || self.total_dirs > 0
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current_path: PathBuf,
    pub scanned_files: usize,
    pub scanned_dirs: usize,
    pub total_size: u64,
    pub elapsed: Duration,
    pub estimated_remaining: Option<Duration>,
}

impl ScanProgress {
    pub fn new() -> Self {
        Self {
            current_path: PathBuf::new(),
            scanned_files: 0,
            scanned_dirs: 0,
            total_size: 0,
            elapsed: Duration::default(),
            estimated_remaining: None,
        }
    }
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ScanError {
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },
    
    #[error("Path too long: {path}")]
    PathTooLong { path: PathBuf },
    
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },
    
    #[error("I/O error at {path}: {message}")]
    IoError { path: PathBuf, message: String },
    
    #[error("File system error: {message}")]
    FileSystemError { message: String },
    
    #[error("Scan cancelled")]
    Cancelled,
    
    #[error("Other error: {message}")]
    Other { message: String },
}

impl ScanError {
    /// Classifies an I/O error that occurred while accessing `path`.
    pub fn from_io(path: &Path, error: &std::io::Error) -> Self {
        let path = path.to_path_buf();
        match error.kind() {
            std::io::ErrorKind::PermissionDenied => ScanError::PermissionDenied { path },
            std::io::ErrorKind::NotFound => ScanError::PathNotFound { path },
            _ => ScanError::IoError { path, message: error.to_string() },
        }
    }
}

impl From<std::io::Error> for ScanError {
    fn from(error: std::io::Error) -> Self {
        ScanError::Other { message: error.to_string() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Size
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Descending
    }
}