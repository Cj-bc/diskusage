# DiskUsage API Documentation

## Module Interfaces

This document describes the internal APIs and interfaces between the different modules of the DiskUsage application.

## Core Data Types

### DirectoryNode
```rust
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
    pub fn new(path: PathBuf) -> Self;
    pub fn add_child(&mut self, child: DirectoryNode);
    pub fn calculate_size(&mut self) -> u64;
    pub fn find_child(&self, name: &str) -> Option<&DirectoryNode>;
    pub fn total_items(&self) -> usize;
    pub fn depth(&self) -> usize;
}
```

### ScanResult
```rust
pub struct ScanResult {
    pub root: DirectoryNode,
    pub total_size: u64,
    pub total_files: usize,
    pub total_dirs: usize,
    pub scan_duration: Duration,
    pub errors: Vec<ScanError>,
}

impl ScanResult {
    pub fn new(root: DirectoryNode) -> Self;
    pub fn is_valid(&self) -> bool;
    pub fn has_errors(&self) -> bool;
}
```

### ScanError
```rust
#[derive(Debug, Clone)]
pub enum ScanError {
    PermissionDenied(PathBuf),
    PathNotFound(PathBuf),
    IoError(PathBuf, String),
    Other(String),
}
```

## File System Scanner (`src/fs/scanner.rs`)

### Scanner
```rust
pub struct Scanner {
    pub progress_callback: Option<Box<dyn Fn(ScanProgress) + Send + Sync>>,
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
    pub filters: Vec<PathFilter>,
}

impl Scanner {
    pub fn new() -> Self;
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where F: Fn(ScanProgress) + Send + Sync + 'static;
    pub fn with_max_depth(mut self, depth: usize) -> Self;
    pub fn scan(&self, path: &Path) -> Result<ScanResult, ScanError>;
    pub fn scan_async(&self, path: &Path) -> impl Future<Output = Result<ScanResult, ScanError>>;
    pub fn cancel(&self);
}
```

### ScanProgress
```rust
pub struct ScanProgress {
    pub current_path: PathBuf,
    pub scanned_files: usize,
    pub scanned_dirs: usize,
    pub total_size: u64,
    pub elapsed: Duration,
    pub estimated_remaining: Option<Duration>,
}
```

### PathFilter
```rust
pub enum PathFilter {
    Extension(String),
    Name(String),
    Size(SizeFilter),
    Modified(TimeFilter),
    Hidden(bool),
}

pub enum SizeFilter {
    MinSize(u64),
    MaxSize(u64),
    Range(u64, u64),
}

pub enum TimeFilter {
    After(SystemTime),
    Before(SystemTime),
    Range(SystemTime, SystemTime),
}
```

## Cache Manager (`src/data/cache.rs`)

### CacheManager
```rust
pub struct CacheManager {
    cache: HashMap<PathBuf, CacheEntry>,
    max_entries: usize,
    max_age: Duration,
}

impl CacheManager {
    pub fn new(max_entries: usize, max_age: Duration) -> Self;
    pub fn get(&self, path: &Path) -> Option<&DirectoryNode>;
    pub fn put(&mut self, path: PathBuf, node: DirectoryNode);
    pub fn invalidate(&mut self, path: &Path);
    pub fn clear(&mut self);
    pub fn size(&self) -> usize;
    pub fn cleanup_expired(&mut self);
}
```

### CacheEntry
```rust
struct CacheEntry {
    node: DirectoryNode,
    created: SystemTime,
    accessed: SystemTime,
    access_count: usize,
}
```

## Export Manager (`src/data/export.rs`)

### ExportManager
```rust
pub struct ExportManager;

impl ExportManager {
    pub fn export_json(&self, node: &DirectoryNode, path: &Path) -> Result<(), ExportError>;
    pub fn export_csv(&self, node: &DirectoryNode, path: &Path) -> Result<(), ExportError>;
    pub fn export_custom<F>(&self, node: &DirectoryNode, path: &Path, formatter: F) -> Result<(), ExportError>
    where F: Fn(&DirectoryNode) -> String;
}
```

### ExportError
```rust
#[derive(Debug)]
pub enum ExportError {
    IoError(std::io::Error),
    SerializationError(String),
    InvalidPath(PathBuf),
}
```

## Visualization Components

### TreeMap (`src/visualization/treemap.rs`)

### TreeMapRenderer
```rust
pub struct TreeMapRenderer {
    pub color_scheme: ColorScheme,
    pub border_width: f32,
    pub min_rect_size: f32,
}

impl TreeMapRenderer {
    pub fn new() -> Self;
    pub fn render(&self, ui: &mut egui::Ui, data: &TreeMapData) -> TreeMapResponse;
    pub fn calculate_layout(&self, node: &DirectoryNode, bounds: Rect) -> Vec<TreeMapRect>;
}
```

### TreeMapData
```rust
pub struct TreeMapData {
    pub root: DirectoryNode,
    pub selected_path: Option<PathBuf>,
    pub hover_info: Option<HoverInfo>,
    pub zoom_level: f32,
    pub view_bounds: Rect,
}
```

### TreeMapResponse
```rust
pub struct TreeMapResponse {
    pub clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
    pub context_menu: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
}
```

### TreeMapRect
```rust
pub struct TreeMapRect {
    pub rect: Rect,
    pub path: PathBuf,
    pub size: u64,
    pub color: Color32,
    pub label: String,
}
```

### TreeView (`src/visualization/tree_view.rs`)

### TreeViewRenderer
```rust
pub struct TreeViewRenderer {
    pub indent_size: f32,
    pub icon_size: f32,
    pub show_icons: bool,
    pub show_sizes: bool,
}

impl TreeViewRenderer {
    pub fn new() -> Self;
    pub fn render(&self, ui: &mut egui::Ui, data: &TreeViewData) -> TreeViewResponse;
}
```

### TreeViewData
```rust
pub struct TreeViewData {
    pub root: DirectoryNode,
    pub expanded_paths: HashSet<PathBuf>,
    pub selected_path: Option<PathBuf>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}
```

### TreeViewResponse
```rust
pub struct TreeViewResponse {
    pub clicked: Option<PathBuf>,
    pub expanded: Option<PathBuf>,
    pub collapsed: Option<PathBuf>,
    pub context_menu: Option<PathBuf>,
}
```

### PieChart (`src/visualization/pie_chart.rs`)

### PieChartRenderer
```rust
pub struct PieChartRenderer {
    pub center: Pos2,
    pub radius: f32,
    pub start_angle: f32,
    pub show_labels: bool,
}

impl PieChartRenderer {
    pub fn new(center: Pos2, radius: f32) -> Self;
    pub fn render(&self, ui: &mut egui::Ui, data: &PieChartData) -> PieChartResponse;
}
```

### PieChartData
```rust
pub struct PieChartData {
    pub segments: Vec<PieSegment>,
    pub total_size: u64,
    pub selected_segment: Option<usize>,
}
```

### PieSegment
```rust
pub struct PieSegment {
    pub label: String,
    pub size: u64,
    pub color: Color32,
    pub path: PathBuf,
}
```

## Application State (`src/app.rs`)

### AppState
```rust
pub struct AppState {
    pub current_path: Option<PathBuf>,
    pub scan_result: Option<ScanResult>,
    pub view_mode: ViewMode,
    pub is_scanning: bool,
    pub scan_progress: Option<ScanProgress>,
    pub cache_manager: CacheManager,
    pub export_manager: ExportManager,
    pub settings: AppSettings,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self;
    pub fn set_current_path(&mut self, path: PathBuf);
    pub fn start_scan(&mut self, path: PathBuf);
    pub fn set_scan_result(&mut self, result: ScanResult);
    pub fn set_view_mode(&mut self, mode: ViewMode);
    pub fn clear_error(&mut self);
    pub fn set_error(&mut self, message: String);
}
```

### ViewMode
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    TreeMap,
    TreeView,
    PieChart,
}
```

### AppSettings
```rust
pub struct AppSettings {
    pub theme: Theme,
    pub default_view: ViewMode,
    pub cache_size: usize,
    pub cache_duration: Duration,
    pub follow_symlinks: bool,
    pub max_scan_depth: Option<usize>,
    pub auto_refresh: bool,
    pub refresh_interval: Duration,
}
```

## GUI Components (`src/gui/components/`)

### DirectoryBrowser
```rust
pub struct DirectoryBrowser {
    pub current_path: PathBuf,
    pub favorites: Vec<PathBuf>,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
}

impl DirectoryBrowser {
    pub fn new() -> Self;
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<PathBuf>;
    pub fn navigate_to(&mut self, path: PathBuf);
    pub fn go_back(&mut self) -> Option<PathBuf>;
    pub fn go_forward(&mut self) -> Option<PathBuf>;
    pub fn add_favorite(&mut self, path: PathBuf);
}
```

### SizeDisplay
```rust
pub struct SizeDisplay;

impl SizeDisplay {
    pub fn render_size(ui: &mut egui::Ui, size: u64, format: SizeFormat);
    pub fn render_size_with_percentage(ui: &mut egui::Ui, size: u64, total: u64, format: SizeFormat);
}

pub enum SizeFormat {
    Bytes,
    Binary,
    Decimal,
    Auto,
}
```

### ProgressBar
```rust
pub struct ProgressBar {
    pub progress: f32,
    pub text: Option<String>,
    pub show_percentage: bool,
    pub indeterminate: bool,
}

impl ProgressBar {
    pub fn new(progress: f32) -> Self;
    pub fn with_text(mut self, text: String) -> Self;
    pub fn render(&self, ui: &mut egui::Ui);
}
```

## Utility Functions (`src/utils/`)

### Format (`src/utils/format.rs`)
```rust
pub fn format_size(size: u64, format: SizeFormat) -> String;
pub fn format_duration(duration: Duration) -> String;
pub fn format_timestamp(time: SystemTime) -> String;
pub fn format_percentage(value: f64) -> String;
```

### Math (`src/utils/math.rs`)
```rust
pub fn calculate_treemap_layout(items: &[TreeMapItem], bounds: Rect) -> Vec<Rect>;
pub fn calculate_aspect_ratio(rect: Rect) -> f32;
pub fn normalize_values(values: &[f64]) -> Vec<f64>;
pub fn color_interpolate(start: Color32, end: Color32, t: f32) -> Color32;
```

## Error Handling

### Common Result Types
```rust
pub type ScanResult = Result<data::ScanResult, ScanError>;
pub type ExportResult = Result<(), ExportError>;
pub type CacheResult<T> = Result<T, CacheError>;
```

### Error Traits
```rust
pub trait UserFriendlyError {
    fn user_message(&self) -> String;
    fn suggested_action(&self) -> Option<String>;
}
```

## Event System

### AppEvent
```rust
#[derive(Debug, Clone)]
pub enum AppEvent {
    PathSelected(PathBuf),
    ScanStarted(PathBuf),
    ScanCompleted(ScanResult),
    ScanError(ScanError),
    ScanCancelled,
    ViewModeChanged(ViewMode),
    ExportRequested(PathBuf, ExportFormat),
    SettingsChanged(AppSettings),
    CacheCleared,
    ErrorOccurred(String),
}
```

### EventHandler
```rust
pub trait EventHandler {
    fn handle_event(&mut self, event: AppEvent);
}
```

This API documentation provides a comprehensive overview of the internal interfaces and data structures used throughout the DiskUsage application. Each module maintains clear boundaries and well-defined interfaces for maintainability and testability.