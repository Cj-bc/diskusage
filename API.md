# DiskUsage API Reference

The main public types and functions of the `diskusage` library crate, module by module. The binary (`src/main.rs`) only creates a window and runs `DiskUsageApp`. For how the pieces fit together, see [ARCHITECTURE.md](ARCHITECTURE.md). `cargo doc --open` renders the same items from the source.

The crate root re-exports `DiskUsageApp` and everything in `data` and `fs`.

## `data::models`

### `DirectoryNode`
One directory in the scanned tree. Files are not nodes; they are folded into `size` and `file_count`.

```rust
pub struct DirectoryNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,                    // recursive, includes files directly inside
    pub children: Vec<DirectoryNode>, // subdirectories only
    pub file_count: usize,            // recursive
    pub dir_count: usize,             // recursive
    pub last_modified: SystemTime,
    pub is_expanded: bool,            // unused by the views
}

impl DirectoryNode {
    pub fn new(path: PathBuf) -> Self;             // empty node named after the last path component
    pub fn add_child(&mut self, child: DirectoryNode);
    pub fn find_child(&self, name: &str) -> Option<&DirectoryNode>;
    pub fn find_child_mut(&mut self, name: &str) -> Option<&mut DirectoryNode>;
    pub fn find_by_path(&self, path: &Path) -> Option<&DirectoryNode>; // searches only the matching branch
    pub fn total_items(&self) -> usize;            // file_count + dir_count
    pub fn depth(&self) -> usize;                  // levels in this subtree, 1 for a leaf
    pub fn sort_by_size(&mut self, ascending: bool);   // recursive, in place
    pub fn sort_by_name(&mut self, ascending: bool);   // recursive, case-insensitive
}
```

### `ScanResult`
```rust
pub struct ScanResult {
    pub root: DirectoryNode,
    pub total_size: u64,
    pub total_files: usize,
    pub total_dirs: usize,         // excludes the root itself
    pub scan_duration: Duration,
    pub errors: Vec<ScanError>,    // entries skipped during the scan
}

impl ScanResult {
    pub fn new(root: DirectoryNode) -> Self;  // totals taken from root; no duration or errors
    pub fn is_valid(&self) -> bool;           // anything at all was found
    pub fn has_errors(&self) -> bool;
}
```

### `ScanProgress`
Sent periodically while scanning. `estimated_remaining` is always `None` for now.

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

### `ScanError`
```rust
pub enum ScanError {
    PermissionDenied { path: PathBuf },
    PathNotFound { path: PathBuf },
    PathTooLong { path: PathBuf },
    InvalidPath { path: PathBuf },
    IoError { path: PathBuf, message: String },
    FileSystemError { message: String },
    Cancelled,
    Other { message: String },
}

impl ScanError {
    /// Maps an I/O error on `path` to PermissionDenied, PathNotFound or IoError.
    pub fn from_io(path: &Path, error: &std::io::Error) -> Self;
}
impl From<std::io::Error> for ScanError; // -> Other, loses the path; prefer from_io
```

It implements `std::error::Error` and `Display` (via `thiserror`) as well as `Serialize`/`Deserialize`.

### `SortBy`, `SortOrder`
```rust
pub enum SortBy { Name, #[default] Size, Modified, Type }
pub enum SortOrder { Ascending, #[default] Descending }
```

## `fs::scanner`

### `Scanner`
Runs one scan at a time on a background thread.

```rust
pub struct Scanner { /* private */ }

impl Scanner {
    pub fn new() -> Self;
    /// Starts scanning `path`, cancelling and discarding any scan already running.
    pub fn start_scan(&mut self, path: PathBuf, options: ScanOptions);
    /// Stops the running scan, if any. No result will be delivered for it.
    pub fn cancel(&mut self);
    pub fn is_scanning(&self) -> bool;
    /// Drains pending messages: the result once finished, else the latest progress, else None.
    pub fn poll(&mut self) -> Option<ScanUpdate>;
}

pub enum ScanUpdate {
    Progress(ScanProgress),
    Finished(Result<ScanResult, ScanError>),
}

#[derive(Default)]
pub struct ScanOptions {
    pub follow_symlinks: bool,     // default false
    pub max_depth: Option<usize>,  // None = unlimited; Some(n) reads n levels,
                                   // so Some(1) counts only the root's own files
}
```

Typical use, once per frame:

```rust
match scanner.poll() {
    Some(ScanUpdate::Progress(progress)) => { /* show progress */ }
    Some(ScanUpdate::Finished(Ok(result))) => { /* show result */ }
    Some(ScanUpdate::Finished(Err(error))) => { /* show error */ }
    None => {}
}
```

A scan fails only when the root cannot be read or it is cancelled. Problems below the root are collected in `ScanResult::errors`.

### Helpers
`fs::path_utils` (`normalize_path`, `get_parent_safely`, `join_paths`, `get_display_name`) and `fs::permissions` (`can_read`, `is_hidden`) are small helpers that the application does not currently use.

## `gui::app_state`

### `AppState`
Persisted between sessions by eframe. Scan data is not part of it.

```rust
#[serde(default)]
pub struct AppState {
    pub current_path: Option<PathBuf>,
    pub view_mode: ViewMode,
    pub settings: AppSettings,
    pub recent_paths: Vec<PathBuf>,   // most recent first, at most 10
    #[serde(skip)]
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self;
    /// Sets the current path and moves it to the front of recent_paths.
    pub fn set_current_path(&mut self, path: PathBuf);
    pub fn set_view_mode(&mut self, mode: ViewMode);
    pub fn set_error(&mut self, message: String);
    pub fn clear_error(&mut self);
    pub fn update_settings(&mut self, settings: AppSettings);
}
```

### `AppSettings`
`follow_symlinks` and `max_scan_depth` are passed to every scan. The other fields are not used yet, and there is no settings UI.

```rust
#[serde(default)]
pub struct AppSettings {
    pub theme: Theme,                  // Light | Dark (default) | System
    pub default_view: ViewMode,
    pub cache_size: usize,             // 1000
    pub follow_symlinks: bool,         // false
    pub max_scan_depth: Option<usize>, // None
    pub auto_refresh: bool,            // false
}
```

### `ViewMode`
```rust
pub enum ViewMode { #[default] TreeMap, TreeView, PieChart }

impl ViewMode {
    pub const ALL: [ViewMode; 3];
    pub fn label(self) -> &'static str; // "TreeMap", "Tree View", "Pie Chart"
}
```

## `gui::views`

Every view has a **data** struct that holds per-scan state and shares the scan through `Arc<ScanResult>`, and a **renderer** that holds display parameters. All three renderers have the same shape:

```rust
impl XRenderer {
    pub fn new() -> Self;  // same as Default
    pub fn render(&mut self, ui: &mut egui::Ui, data: &mut XData) -> XResponse;
}
```

Every response carries `clicked`, `double_clicked` and `right_clicked: Option<PathBuf>` for the frame. Navigation on double-click is up to the caller.

### Tree View (`tree_view.rs`)
```rust
pub struct TreeViewData {
    pub scan_result: Arc<ScanResult>,
    pub expanded_paths: HashSet<PathBuf>,  // root is expanded initially
    pub selected_path: Option<PathBuf>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub size_format: SizeFormat,
}

impl TreeViewData {
    pub fn new(scan_result: Arc<ScanResult>) -> Self;
    pub fn expand_path(&mut self, path: PathBuf);
    pub fn collapse_path(&mut self, path: &Path);
    pub fn is_expanded(&self, path: &Path) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);  // keeps the root expanded
}

pub struct TreeViewRenderer {
    pub indent_size: f32,        // 20.0
    pub line_height: f32,        // 20.0
    pub show_icons: bool,
    pub show_sizes: bool,
    pub show_percentages: bool,
    pub show_file_counts: bool,
}

pub struct TreeViewResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub expanded: Vec<PathBuf>,
    pub collapsed: Vec<PathBuf>,
}
```

### TreeMap (`treemap.rs`)
```rust
pub struct TreeMapData {
    pub scan_result: Arc<ScanResult>,
    pub current_path: PathBuf,
    pub zoom_level: f32,       // clamped to 0.1..=10.0
    pub pan_offset: Vec2,
    pub selected_path: Option<PathBuf>,
    pub hovered_path: Option<PathBuf>,  // reset every frame
}

impl TreeMapData {
    pub fn new(scan_result: Arc<ScanResult>) -> Self;
    pub fn current_node(&self) -> &DirectoryNode;  // falls back to the root
    /// Shows `path` if it exists and has subdirectories; otherwise does nothing.
    pub fn navigate_to(&mut self, path: PathBuf);
    pub fn can_navigate_up(&self) -> bool;         // false at the scan root
    pub fn navigate_up(&mut self);
    pub fn reset_view(&mut self);                  // zoom 1.0, no pan
}

pub struct TreeMapRenderer {
    pub min_rect_size: f32,    // 4.0, smaller rectangles are not drawn
    pub border_width: f32,     // 1.0
    pub label_threshold: f32,  // 30.0, minimum width and height for a label
    pub show_labels: bool,
    pub show_sizes: bool,
}

pub struct TreeMapResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
    pub zoom_changed: bool,
    pub pan_changed: bool,
}

pub struct TreeMapRect {
    pub rect: Rect,
    pub path: PathBuf,
    pub size: u64,
    pub percentage: f32,  // of the directory being shown
    pub color: Color32,
    pub label: String,
    pub depth: usize,
    pub is_leaf: bool,
}
```

The squarified layout is a private function, `squarified_layout(children, bounds, total_size)`. `total_size` may exceed the children's sum, and that share of `bounds` is left empty.

### Pie Chart (`pie_chart.rs`)
```rust
pub struct PieChartData {
    pub scan_result: Arc<ScanResult>,
    pub current_path: PathBuf,
    pub selected_path: Option<PathBuf>,
    pub hovered_path: Option<PathBuf>,  // reset every frame
    pub show_labels: bool,
    pub show_percentages: bool,
    pub min_slice_angle: f32,  // radians, 0.05 (~3°)
    pub max_slices: usize,     // 12
}

impl PieChartData {
    pub fn new(scan_result: Arc<ScanResult>) -> Self;
    pub fn current_node(&self) -> &DirectoryNode;
    pub fn navigate_to(&mut self, path: PathBuf);  // same rules as TreeMapData
    pub fn can_navigate_up(&self) -> bool;
    pub fn navigate_up(&mut self);
}

pub struct PieChartRenderer {
    pub center_radius: f32,   // 30.0, radius of the hole in the middle
    pub show_center_info: bool,
    pub label_distance: f32,  // 1.3, label radius as a multiple of the pie radius
    pub min_label_size: f32,  // 20.0, degrees, narrower slices get no label
}

pub struct PieChartResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
}

pub struct PieSlice {
    pub target: SliceTarget,
    pub name: String,
    pub size: u64,
    pub percentage: f32,   // of the current directory
    pub start_angle: f32,  // radians, slices run clockwise from -PI/2 (top)
    pub end_angle: f32,
    pub color: Color32,
}

impl PieSlice {
    pub fn path(&self) -> Option<&Path>;  // None for SliceTarget::Other
}

pub enum SliceTarget {
    Directory(PathBuf),
    /// Files directly in the directory, plus subdirectories too small or too many for their own slice.
    Other,
}
```

The `Other` slice never appears in a response, so it cannot be selected or navigated into.

## `gui::components`

```rust
pub struct SizeDisplay;
impl SizeDisplay {
    pub fn render_size(ui: &mut egui::Ui, size: u64, format: SizeFormat);
    pub fn render_size_with_percentage(ui: &mut egui::Ui, size: u64, total: u64, format: SizeFormat);
    pub fn render_size_comparison(ui: &mut egui::Ui, size1: u64, size2: u64, label1: &str, label2: &str, format: SizeFormat);
}

pub struct ProgressBar { pub progress: f32, pub text: Option<String>, pub show_percentage: bool, pub indeterminate: bool }
impl ProgressBar {
    pub fn new(progress: f32) -> Self;  // clamped to 0.0..=1.0
    pub fn indeterminate() -> Self;
    pub fn with_text(self, text: String) -> Self;
    pub fn with_percentage(self, show: bool) -> Self;
    pub fn render(&self, ui: &mut egui::Ui);
}
```

`SizeFormat` is re-exported here from `utils`. `DirectoryBrowser` (an in-app folder browser with history and favorites) exists but is not used; the app uses the native folder picker from `rfd` instead.

## `utils`

### `format`
```rust
pub enum SizeFormat {
    Bytes,        // "1536 bytes"
    Binary,       // "1.50 KiB"
    Decimal,      // "1.54 KB"
    #[default]
    Auto,         // Binary, but "512 B" below 1 KiB
}

pub fn format_size(size: u64) -> String;                       // SizeFormat::Auto
pub fn format_size_with_format(size: u64, format: SizeFormat) -> String;
pub fn format_duration(duration: Duration) -> String;          // "0.50s", "1m 30s", "1h 01m 01s"
pub fn format_percentage(value: f64) -> String;                // "12.3%", "<0.1%", "100%"
pub fn format_number(number: usize) -> String;                 // "999", "1.5K", "2.0M"
```

### `math`
```rust
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Color32; // hue in degrees, any sign
pub fn color_interpolate(start: Color32, end: Color32, t: f32) -> Color32;
pub fn calculate_aspect_ratio(rect: Rect) -> f32;   // width / height, infinity for zero height
pub fn normalize_values(values: &[f64]) -> Vec<f64>; // scale to sum 1, unchanged if the sum is 0
```

## Placeholders

These exist so the module layout matches the design, but have no behaviour yet:

| Item | Status |
|---|---|
| `data::CacheManager` | Unit struct, no methods besides `new` |
| `data::ExportManager` | Unit struct, no methods besides `new` |
| `data::PathFilter` | `matches` always returns `true` |
| `visualization::ColorScheme` | Defined, not used by any view |
| `visualization::{layout, treemap, tree_view, pie_chart}` | Empty files |
