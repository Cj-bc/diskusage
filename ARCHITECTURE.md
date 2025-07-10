# DiskUsage Architecture

## System Overview

DiskUsage is designed as a modular, cross-platform desktop application with clear separation of concerns between data analysis, visualization, and user interface components.

```
┌─────────────────────────────────────────────────────────────┐
│                        GUI Layer (egui)                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   TreeMap       │  │   TreeView      │  │   PieChart      │ │
│  │   Component     │  │   Component     │  │   Component     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Application State                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   FileSystem    │  │   Cache         │  │   Export        │ │
│  │   Scanner       │  │   Manager       │  │   Manager       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                     Core Data Layer                         │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. GUI Layer (`src/gui/`)
- **Framework**: egui immediate mode GUI
- **Responsibilities**: User interface rendering, event handling, layout management
- **Components**:
  - `app.rs`: Main application state and event loop
  - `components/`: Reusable UI components
  - `views/`: Different visualization modes

### 2. Visualization Components (`src/visualization/`)
- **TreeMap**: Proportional rectangle visualization
- **TreeView**: Hierarchical list with size information
- **PieChart**: Circular sector visualization
- **Interactive**: Click-to-drill-down functionality

### 3. File System Analysis (`src/fs/`)
- **Scanner**: Directory traversal and size calculation
- **Cross-platform**: Windows and Linux path handling
- **Performance**: Async scanning with progress tracking
- **Error handling**: Graceful handling of permission issues

### 4. Data Management (`src/data/`)
- **Models**: Core data structures for directories and files
- **Cache**: Intelligent caching for performance
- **Export**: JSON and CSV serialization
- **Filtering**: Search and filter functionality

## Data Flow

```
User Input → GUI Events → Application State → File System Scanner
                                            ↓
Export Manager ← Cache Manager ← Data Models ← Scanner Results
      ↓
GUI Components ← Visualization Engine ← Processed Data
```

## Key Design Principles

### 1. Separation of Concerns
- **GUI**: Pure presentation logic
- **Business Logic**: Data processing and analysis
- **Data Access**: File system operations

### 2. Performance Optimization
- **Lazy Loading**: Only scan directories when needed
- **Caching**: Store computed results for fast re-access
- **Async Operations**: Non-blocking UI during scans
- **Memory Efficiency**: Stream processing for large directories

### 3. Cross-Platform Compatibility
- **Path Handling**: Platform-agnostic path operations
- **File System**: Handle different file system types
- **UI Consistency**: Consistent look and feel across platforms

## Module Structure

```
src/
├── main.rs                 # Application entry point
├── app.rs                  # Main application state
├── gui/                    # GUI components
│   ├── mod.rs
│   ├── app.rs              # Main application window
│   ├── components/         # Reusable UI components
│   │   ├── mod.rs
│   │   ├── directory_browser.rs
│   │   ├── size_display.rs
│   │   └── progress_bar.rs
│   └── views/              # Different visualization views
│       ├── mod.rs
│       ├── treemap.rs
│       ├── tree_view.rs
│       └── pie_chart.rs
├── fs/                     # File system operations
│   ├── mod.rs
│   ├── scanner.rs          # Directory scanning
│   ├── path_utils.rs       # Cross-platform path handling
│   └── permissions.rs      # Permission handling
├── data/                   # Data models and management
│   ├── mod.rs
│   ├── models.rs           # Core data structures
│   ├── cache.rs            # Caching system
│   ├── export.rs           # Export functionality
│   └── filter.rs           # Search and filtering
├── visualization/          # Visualization algorithms
│   ├── mod.rs
│   ├── treemap.rs          # Treemap algorithm
│   ├── layout.rs           # Layout calculations
│   └── color.rs            # Color schemes
└── utils/                  # Utility functions
    ├── mod.rs
    ├── format.rs           # Size formatting
    └── math.rs             # Mathematical utilities
```

## Data Structures

### DirectoryNode
```rust
struct DirectoryNode {
    name: String,
    path: PathBuf,
    size: u64,
    children: Vec<DirectoryNode>,
    file_count: usize,
    dir_count: usize,
    last_modified: SystemTime,
    is_expanded: bool,
}
```

### ScanResult
```rust
struct ScanResult {
    root: DirectoryNode,
    total_size: u64,
    total_files: usize,
    total_dirs: usize,
    scan_duration: Duration,
    errors: Vec<ScanError>,
}
```

### VisualizationData
```rust
struct VisualizationData {
    nodes: Vec<RenderNode>,
    bounds: Rect,
    color_scheme: ColorScheme,
    interaction_state: InteractionState,
}
```

## Concurrency Model

### Async Scanning
- **Background Tasks**: Directory scanning runs in background threads
- **Progress Updates**: Regular progress updates to UI
- **Cancellation**: User can cancel long-running scans
- **Error Handling**: Graceful handling of file system errors

### State Management
- **Single Source of Truth**: Application state managed centrally
- **Immutable Updates**: State updates create new state objects
- **Event-Driven**: UI updates triggered by state changes

## Performance Considerations

### Memory Usage
- **Streaming**: Process large directories in chunks
- **Garbage Collection**: Efficient memory cleanup
- **Caching Strategy**: LRU cache for frequently accessed directories

### Rendering Performance
- **Viewport Culling**: Only render visible elements
- **Level of Detail**: Simplified rendering for distant objects
- **Batch Updates**: Group UI updates for efficiency

## Error Handling

### File System Errors
- **Permission Denied**: Graceful handling with user notification
- **Path Not Found**: Clear error messages
- **I/O Errors**: Retry logic with exponential backoff

### UI Error States
- **Loading States**: Progress indicators during operations
- **Error Messages**: User-friendly error descriptions
- **Recovery Options**: Suggestions for resolving issues

## Testing Strategy

### Unit Tests
- **Data Models**: Core data structure validation
- **Algorithms**: Visualization and calculation algorithms
- **Utilities**: Helper function correctness

### Integration Tests
- **File System**: Cross-platform file operations
- **Visualization**: Rendering accuracy and performance
- **Export**: Data serialization integrity

### Platform Testing
- **Windows**: NTFS file system handling
- **Linux**: ext4/xfs file system compatibility
- **UI Testing**: Cross-platform UI consistency

## Security Considerations

### File System Access
- **Sandboxing**: Limit file system access to selected directories
- **Permission Validation**: Check permissions before operations
- **Path Traversal**: Prevent directory traversal attacks

### Data Privacy
- **No Network Access**: All operations are local
- **No Data Collection**: No telemetry or analytics
- **Secure Export**: Safe handling of exported data