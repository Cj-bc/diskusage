use std::path::{Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub fn get_parent_safely(path: &Path) -> Option<PathBuf> {
    match path.parent() {
        Some(parent) if parent != path => Some(parent.to_path_buf()),
        _ => None,
    }
}

pub fn join_paths(base: &Path, relative: &Path) -> PathBuf {
    base.join(relative)
}

pub fn get_display_name(path: &Path) -> String {
    match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => path.to_string_lossy().to_string(),
    }
}