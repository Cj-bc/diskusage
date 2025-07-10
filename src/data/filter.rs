// Placeholder for filter implementation
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum PathFilter {
    Extension(String),
    Name(String),
    Hidden(bool),
}

impl PathFilter {
    pub fn matches(&self, _path: &PathBuf) -> bool {
        // TODO: Implement filter logic
        true
    }
}