# DiskUsage - Directory Space Visualization Tool

A cross-platform GUI application for visualizing directory space usage with interactive treemaps, tree views, and comprehensive analysis tools.

## Overview

DiskUsage helps you understand how storage space is distributed across your file system by providing intuitive visualizations of directory sizes. Built with Rust and egui for native performance and cross-platform compatibility.

## Features

### 📊 Multiple Visualization Types
- **Treemap View**: Proportional rectangles showing relative directory sizes
- **Tree View**: Hierarchical list with size information
- **Pie Chart**: Visual breakdown of immediate subdirectories
- **Interactive Navigation**: Click to drill down into subdirectories

### 🔍 Analysis Tools
- **Recursive Size Calculation**: Accurate directory size analysis
- **Sorting Options**: By size, name, modification date, or file type
- **Search & Filter**: Find specific files or directories
- **Export Functionality**: Save analysis results as JSON or CSV

### 🖥️ Cross-Platform Support
- **Windows**: Native Windows GUI with proper file system handling
- **Linux**: Native Linux GUI with Unix file system support
- **Single Executable**: No runtime dependencies required

### ⚡ Performance Features
- **Efficient Memory Usage**: Optimized for large directory structures
- **Progress Tracking**: Real-time progress for large scans
- **Caching System**: Intelligent caching for faster re-analysis
- **Responsive UI**: Non-blocking operations with smooth interactions

## Technology Stack

- **Language**: Rust (for performance and safety)
- **GUI Framework**: egui (immediate mode GUI)
- **Visualization**: Custom rendering with egui graphics
- **File System**: Cross-platform path handling with std::fs
- **Data Export**: JSON and CSV serialization

## Quick Start

### Prerequisites
- Rust toolchain (1.70+)
- Git

### Installation
```bash
git clone https://github.com/Cj-bc/diskusage.git
cd diskusage
cargo build --release
```

### Usage
```bash
# Run the application
cargo run --release

# Or use the built executable
./target/release/diskusage
```

## Project Status

🚧 **In Development** - This project is currently in the documentation and planning phase.

### Planned Milestones
1. **Phase 1**: Core directory analysis engine
2. **Phase 2**: Basic GUI implementation
3. **Phase 3**: Visualization components
4. **Phase 4**: Advanced features and polish
5. **Phase 5**: Cross-platform testing and packaging

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Architecture

For detailed system design information, see [ARCHITECTURE.md](ARCHITECTURE.md).

## User Guide

For comprehensive usage instructions, see [USER_GUIDE.md](USER_GUIDE.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [egui](https://github.com/emilk/egui)
- Inspired by tools like WinDirStat, Baobab, and ncdu