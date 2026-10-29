# DiskUsage Architecture

## System Overview

DiskUsage is a single-window [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)/egui application. A background thread scans the chosen directory into an in-memory tree; the UI thread polls it each frame and, once the scan completes, renders the tree in one of three views.

```
┌───────────────────────────────────────────────────────────────────┐
│ UI thread                                                          │
│                                                                    │
│  DiskUsageApp (app.rs)                                             │
│   ├─ AppState            persisted preferences (gui/app_state.rs)  │
│   ├─ ScanState           Idle | Scanning | Loaded                  │
│   │    └─ LoadedScan     Arc<ScanResult> + per-view data           │
│   ├─ TreeMapRenderer ┐                                             │
│   ├─ TreeViewRenderer├─ display settings only (gui/views/)         │
│   ├─ PieChartRenderer┘                                             │
│   └─ Scanner ──────── poll() each frame                            │
│          ▲                                                         │
└──────────┼─────────────────────────────────────────────────────────┘
           │ mpsc channel: Progress(ScanProgress) / Completed(Result)
┌──────────┴─────────────────────────────────────────────────────────┐
│ Scan thread (fs/scanner.rs)                                         │
│   recursive std::fs::read_dir walk → DirectoryNode tree             │
└─────────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs                 # Binary entry point: window options, runs DiskUsageApp
├── lib.rs                  # Library crate root; the binary depends on it
├── app.rs                  # DiskUsageApp: scan lifecycle, menus, toolbar, status bar
├── data/
│   ├── models.rs           # DirectoryNode, ScanResult, ScanProgress, ScanError, SortBy/SortOrder
│   ├── cache.rs            # Placeholder (CacheManager, no behaviour yet)
│   ├── export.rs           # Placeholder (ExportManager, no behaviour yet)
│   └── filter.rs           # Placeholder (PathFilter, matches everything)
├── fs/
│   ├── scanner.rs          # Scanner, ScanOptions, ScanUpdate; the scan thread
│   ├── path_utils.rs       # Small path helpers (currently unused)
│   └── permissions.rs      # can_read / is_hidden helpers (currently unused)
├── gui/
│   ├── app_state.rs        # AppState, AppSettings, ViewMode, Theme
│   ├── components/         # Reusable widgets: SizeDisplay, ProgressBar, DirectoryBrowser (unused)
│   └── views/
│       ├── tree_view.rs    # TreeViewData + TreeViewRenderer
│       ├── treemap.rs      # TreeMapData + TreeMapRenderer + squarified layout
│       └── pie_chart.rs    # PieChartData + PieChartRenderer + slice calculation
├── visualization/          # ColorScheme; the other files are empty placeholders
└── utils/
    ├── format.rs           # SizeFormat, format_size, format_duration, …
    └── math.rs             # hsv_to_rgb and other numeric helpers
```

Dependencies point downwards: `app` → `gui` → `data` / `utils`, and `fs` → `data`. `utils` depends on nothing in the crate.

## Data Model

The whole scan is one tree of `DirectoryNode`s held in memory.

```rust
pub struct DirectoryNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,               // bytes in this directory, recursively
    pub children: Vec<DirectoryNode>, // subdirectories only
    pub file_count: usize,       // files, recursively
    pub dir_count: usize,        // subdirectories, recursively
    pub last_modified: SystemTime,
    pub is_expanded: bool,       // unused; the tree view tracks expansion itself
}
```

- **Only directories are nodes.** Files are folded into their directory's `size` and `file_count`. This keeps memory proportional to the number of directories rather than files. Consequently, a directory's `size` is usually larger than the sum of its children's sizes; the difference is files directly inside it. Views must use `node.size`, not the children's sum, as the whole.
- `ScanResult` wraps the root with totals, the scan duration, and `errors: Vec<ScanError>` for entries that were skipped.

## Scan Lifecycle

### Scanner (`fs/scanner.rs`)

`Scanner::start_scan(path, options)`:
1. Cancels any running scan (its result will be discarded)
2. Creates a fresh `mpsc` channel and `Arc<AtomicBool>` cancel flag for this scan
3. Spawns a thread that walks the tree and finally sends `Completed(Result<ScanResult, ScanError>)`

Because every scan has its own channel, a superseded scan cannot deliver its result to a newer one; its sends simply fail once the receiver is dropped.

Inside the thread, a `ScanContext` is threaded through the recursion. It holds running counters (files, directories, bytes), collected non-fatal errors, and the time of the last progress message. Progress messages are throttled to one every 50 ms.

Error policy:
- Failing to read the **root** directory fails the scan (`PathNotFound`, `PermissionDenied`, …)
- Failing to read anything **below** the root is recorded in `ScanResult::errors` and skipped
- `Cancelled` propagates straight up

Symlinks are not followed unless `ScanOptions::follow_symlinks` is set (it is never set from the UI today). `DirEntry::metadata` does not follow links, so a link is counted as a small file. On Windows, Rust reports junctions as symlinks too.

### UI side (`app.rs`)

The app holds the scan as an enum, so view data cannot exist without a result or outlive it:

```rust
enum ScanState {
    Idle,
    Scanning { progress: Option<ScanProgress> },
    Loaded(Box<LoadedScan>),
}

struct LoadedScan {
    result: Arc<ScanResult>,
    tree_view: TreeViewData,
    tree_map: TreeMapData,
    pie_chart: PieChartData,
}
```

Each frame, `handle_events` calls `Scanner::poll()`, which drains the channel and returns either the final result or the latest progress:

```
Idle ──start_scan──▶ Scanning ──Finished(Ok)──▶ Loaded
  ▲                    │  │                        │
  └──Finished(Err)─────┘  └──cancel──▶ Idle        └──start_scan──▶ Scanning
```

While a scan is running the app requests a repaint every 100 ms, so progress keeps updating without user input.

The three view-data structs share a single `Arc<ScanResult>`; the tree is never copied.

## Views

Each view is split into a **data** struct (per-scan state: current directory, selection, hover, view options) and a **renderer** (display parameters). `Renderer::render(ui, &mut data)` draws one frame and returns a `…Response` with `clicked`, `double_clicked` and `right_clicked` paths. `app.rs` reacts to double-clicks in the TreeMap and Pie Chart by calling `navigate_to`; right-clicks are only logged for now.

| View | Layout | Notes |
|---|---|---|
| Tree View | One row per expanded node, children sorted on the fly | Sorts references to children each frame, never clones nodes |
| TreeMap | Squarified layout (Bruls, Huizing & van Wijk), recursing into rectangles larger than 50×50 px | Rows are measured against the size *remaining*, so leftover area represents files directly in the directory |
| Pie Chart | Largest subdirectories clockwise from the top, then one `SliceTarget::Other` slice | `Other` covers smaller subdirectories and direct files, so slices always fill the circle |

All three use `DirectoryNode::find_by_path` to resolve the current directory, which only descends into the child whose path is a prefix of the target.

Hit-testing uses `Response::hover_pos()`, and `hovered_path` is reset at the start of every frame.

## Persistence

`DiskUsageApp::save` stores `AppState` through eframe's persistence (`app.ron` in the platform's application data directory). `AppState` contains preferences only: the view mode, the current and recent paths, and `AppSettings`. Scan results are deliberately not persisted. They can be very large and are stale by the next launch. `AppState` and `AppSettings` are `#[serde(default)]`, so state saved by an older version still loads.

## Testing

Unit tests live next to the code (`cargo test`):
- `fs/scanner.rs`: scanning real temporary directories (via `tempfile`), including root-level files, max depth, a missing root, cancellation, and the `poll` round trip
- `gui/views/treemap.rs`: squarified areas are proportional to sizes and stay inside the bounds
- `gui/views/pie_chart.rs`: slices cover the full circle, Unicode-safe truncation, and Up navigation
- `gui/app_state.rs`: recent-path ordering and capping
- `utils/`: size, duration and percentage formatting, and hue wrapping

The rendering code itself is not covered by automated tests.

## Known Gaps

- `data::{cache, export, filter}`, most of `visualization/`, and `gui::components::DirectoryBrowser` are placeholders that nothing calls
- `AppSettings` has no editor; `theme`, `default_view`, `cache_size` and `auto_refresh` are not read anywhere
- The scan is single-threaded and keeps the whole tree in memory
