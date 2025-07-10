use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::data::{ScanProgress, ScanResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    TreeMap,
    TreeView,
    PieChart,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::TreeMap
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub default_view: ViewMode,
    pub cache_size: usize,
    pub follow_symlinks: bool,
    pub max_scan_depth: Option<usize>,
    pub auto_refresh: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppState {
    pub current_path: Option<PathBuf>,
    pub scan_result: Option<ScanResult>,
    pub view_mode: ViewMode,
    pub is_scanning: bool,
    #[serde(skip)]
    pub scan_progress: Option<ScanProgress>,
    pub settings: AppSettings,
    #[serde(skip)]
    pub error_message: Option<String>,
    pub recent_paths: Vec<PathBuf>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_current_path(&mut self, path: Option<PathBuf>) {
        if let Some(ref path) = path {
            // Add to recent paths if not already present
            if !self.recent_paths.contains(path) {
                self.recent_paths.insert(0, path.clone());
                // Keep only last 10 recent paths
                self.recent_paths.truncate(10);
            }
        }
        self.current_path = path;
    }

    pub fn set_scan_result(&mut self, result: Option<ScanResult>) {
        self.scan_result = result;
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    pub fn set_scanning(&mut self, scanning: bool) {
        self.is_scanning = scanning;
        if !scanning {
            self.scan_progress = None;
        }
    }

    pub fn set_scan_progress(&mut self, progress: Option<ScanProgress>) {
        self.scan_progress = progress;
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