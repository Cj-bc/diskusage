use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Number of entries kept in [`AppState::recent_paths`].
const MAX_RECENT_PATHS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ViewMode {
    #[default]
    TreeMap,
    TreeView,
    PieChart,
}

impl ViewMode {
    pub const ALL: [ViewMode; 3] = [ViewMode::TreeMap, ViewMode::TreeView, ViewMode::PieChart];

    pub fn label(self) -> &'static str {
        match self {
            ViewMode::TreeMap => "TreeMap",
            ViewMode::TreeView => "Tree View",
            ViewMode::PieChart => "Pie Chart",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme: Theme,
    pub default_view: ViewMode,
    pub cache_size: usize,
    pub follow_symlinks: bool,
    pub max_scan_depth: Option<usize>,
    pub auto_refresh: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    Light,
    #[default]
    Dark,
    System,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            default_view: ViewMode::default(),
            cache_size: 1000,
            follow_symlinks: false,
            max_scan_depth: None,
            auto_refresh: false,
        }
    }
}

/// UI state that survives restarts. Scan data is deliberately not part of
/// it: it can be huge and is stale by the next launch.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub current_path: Option<PathBuf>,
    pub view_mode: ViewMode,
    pub settings: AppSettings,
    pub recent_paths: Vec<PathBuf>,
    #[serde(skip)]
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes `path` the current directory and moves it to the front of the
    /// recent paths.
    pub fn set_current_path(&mut self, path: PathBuf) {
        self.recent_paths.retain(|recent| recent != &path);
        self.recent_paths.insert(0, path.clone());
        self.recent_paths.truncate(MAX_RECENT_PATHS);
        self.current_path = Some(path);
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn update_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_paths_are_most_recent_first_without_duplicates() {
        let mut state = AppState::new();
        state.set_current_path(PathBuf::from("a"));
        state.set_current_path(PathBuf::from("b"));
        state.set_current_path(PathBuf::from("a"));

        assert_eq!(state.recent_paths, [PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[test]
    fn recent_paths_are_capped() {
        let mut state = AppState::new();
        for i in 0..MAX_RECENT_PATHS + 5 {
            state.set_current_path(PathBuf::from(i.to_string()));
        }

        assert_eq!(state.recent_paths.len(), MAX_RECENT_PATHS);
    }
}
