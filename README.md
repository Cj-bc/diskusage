# DiskUsage - Directory Space Visualization Tool

A cross-platform GUI application that shows where the space in a directory goes. Pick a folder, let it scan in the background, then explore the result as a tree, a treemap, or a pie chart. Built with Rust and [egui](https://github.com/emilk/egui).

## Features

### Three views of the same scan
- **🌳 Tree View**: expandable directory hierarchy with sizes, percentages and subdirectory counts; sort by name, size, modification time or type, and pick how sizes are formatted
- **🗺️ TreeMap**: squarified treemap where each rectangle's area is proportional to its size, colored by depth; zoom with the mouse wheel, pan by dragging, double-click to drill down
- **🥧 Pie Chart**: the largest subdirectories as slices (5–20, configurable), with everything else, including files directly in the directory, grouped into a grey "Other" slice; double-click to drill down

Switching views keeps the scan; nothing is rescanned.

### Scanning
- Runs on a background thread, so the window stays responsive
- Live progress in the status bar (files, directories and bytes so far) and a Cancel button
- Unreadable entries (e.g. permission denied) are skipped rather than aborting the scan; the status bar shows how many were skipped and lists them on hover
- Symbolic links and Windows junctions are not followed, so link cycles can't trap the scan

### Convenience
- **Recent directories**: the last 10 scanned folders under File → Recent
- **Persistence**: the selected view, last directory and recent list are remembered between sessions. Scan results are not, so open a directory again (or use Tools → Refresh) after restarting

## Quick Start

### Prerequisites
- A recent stable Rust toolchain ([rustup.rs](https://rustup.rs/))
- **Linux**: GTK 3 development files for the folder picker (e.g. `libgtk-3-dev` on Debian/Ubuntu)
- **Windows**: the MSVC build tools that `rustup` asks for

See [BUILD.md](BUILD.md) for details.

### Build and run
```bash
git clone https://github.com/Cj-bc/diskusage.git
cd diskusage
cargo run --release
```

The executable ends up in `target/release/diskusage` (`diskusage.exe` on Windows).

### Basic workflow
1. Click **📁 Open Directory** (or File → Open Directory...) and choose a folder
2. Wait for the scan; progress is shown in the status bar
3. Switch between **TreeMap**, **Tree View** and **Pie Chart** with the buttons on the right of the toolbar
4. Double-click a directory in the TreeMap or Pie Chart to look inside it, and use **⬆ Up** to go back

See the [User Guide](USER_GUIDE.md) for everything else.

## Project Status

The scanner and all three views work. Tested on Windows; Linux should work but gets less testing; macOS is untested.

Not implemented yet (the menu items exist but are disabled):
- Export (JSON/CSV)
- Settings dialog: `follow_symlinks` and `max_scan_depth` exist in the settings model but can't be changed from the UI
- Context menus on right-click
- Search and filtering

## Documentation

| Document | Contents |
|---|---|
| [USER_GUIDE.md](USER_GUIDE.md) | How to use the application |
| [BUILD.md](BUILD.md) | Building, testing and troubleshooting |
| [BUILD_CONFIG.md](BUILD_CONFIG.md) | What `build.rs` and the Cargo profiles do |
| [ARCHITECTURE.md](ARCHITECTURE.md) | How the code is organised and how a scan flows through it |
| [API.md](API.md) | The main types and functions, module by module |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

[ALGORITHMS.md](ALGORITHMS.md), [VISUALIZATION.md](VISUALIZATION.md), [PERFORMANCE.md](PERFORMANCE.md) and [PLATFORM.md](PLATFORM.md) are the original design specifications. They describe intended behaviour, much of which is not implemented; each starts with a note on its status.

## License

MIT, as declared in `Cargo.toml`.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [egui](https://github.com/emilk/egui)
- Inspired by tools like WinDirStat, Baobab, and ncdu
