# DiskUsage User Guide

## Getting Started

Build and run the application as described in the [README](README.md#quick-start):

```bash
cargo run --release
```

## Interface Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│ File  View  Tools  Help                                             │  menu bar
├─────────────────────────────────────────────────────────────────────┤
│ [📁 Open Directory] │ Path: C:\Users\me\Documents                    │  toolbar
│                                  [TreeMap] [Tree View] [Pie Chart]  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                         visualization area                          │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ Total: 1520 | Files: 1240 | Dirs: 280 | Size: 2.50 GiB | Took: 1.20s│  status bar
└─────────────────────────────────────────────────────────────────────┘
```

### Menu bar
| Menu | Item | What it does |
|---|---|---|
| File | Open Directory... | Pick a folder and scan it |
| File | Recent | Rescan one of the last 10 folders (disabled until you have scanned one) |
| File | Exit | Close the application |
| View | TreeMap / Tree View / Pie Chart | Switch the visualization |
| Tools | Refresh | Rescan the current folder |
| Tools | Export..., Settings | Not implemented yet (disabled) |
| Help | About | Version and project link |

### Toolbar
- **📁 Open Directory** opens the folder picker
- **Path** shows the folder that was scanned
- The buttons on the right switch between the three views

### Status bar
- **While scanning**: a spinner, the number of files and directories scanned so far, the bytes counted, and a **Cancel** button
- **After a scan**: totals for the whole scan and how long it took. If some entries could not be read, a **⚠ N skipped** marker appears; hover over it to see which (up to 10)
- **Errors**, such as a folder that could not be opened at all, are shown in red on the right

## Scanning

1. Choose a folder with **📁 Open Directory**, File → Open Directory..., or File → Recent
2. The scan starts immediately and runs in the background
3. Click **Cancel** in the status bar to stop it. Nothing from a cancelled scan is kept

Starting a new scan while one is running cancels the running one.

### What is measured
- **Sizes are file lengths** as reported by the file system, not the space allocated on disk. Compressed or sparse files may take less space than shown, and files with several hard links are counted once per link
- **Directories only** appear as items in the views. Files are counted and their sizes added to their directory, but they are not listed individually
- **Symbolic links and Windows junctions are not followed.** A link is counted as a small file, so the same data is never counted twice and link cycles can't trap the scan
- **Unreadable entries are skipped.** A folder you are not allowed to read is left out (and listed under ⚠ skipped); the rest of the scan continues. Only failing to read the folder you picked stops the scan

## Views

### Tree View
A hierarchical list of directories, starting with the scanned folder expanded.

Each row shows, from left to right:
- **▶ / ▼** to expand or collapse (only on directories that contain subdirectories)
- The directory name
- On the right: its size, its share of the whole scan (when 0.1% or more), and `(N items)`, the number of subdirectories it directly contains

**Header controls**
- **Sort by**: Name, Size, Modified, or Type
- **↑ Asc / ↓ Desc**: toggle the sort order
- **Size format**: Auto, Bytes, Binary (KiB, MiB…), or Decimal (KB, MB…)

**Interactions**
- **Click a name**: select it
- **Double-click a name**: select it and expand it

### TreeMap
Each directory is a rectangle whose area is proportional to its size, so big directories are easy to spot. Subdirectories are drawn inside their parent once the parent is large enough. Colors indicate nesting depth. Empty space inside a rectangle is taken up by files sitting directly in that directory.

**Header controls**
- **⬆ Up**: go back to the parent directory (disabled at the scanned folder)
- **Reset View**: undo zoom and pan
- **Zoom**: the current zoom factor

**Interactions**
- **Hover**: tooltip with the path, size, and share of the directory being shown
- **Click**: select (highlighted)
- **Double-click**: show that directory's contents (only for directories that have subdirectories)
- **Mouse wheel** over the map: zoom between 0.1× and 10×
- **Drag**: pan

### Pie Chart
The directory's largest subdirectories as slices of a ring, starting at the top and running clockwise. Everything that does not get its own slice is grouped into a grey **Other** slice: subdirectories beyond the slice limit, slices smaller than about 3°, and files directly in the directory. The center shows the current directory's name and size.

**Header controls**
- **⬆ Up**: go back to the parent directory
- **Show Labels** / **Show Percentages**: label slices that are wide enough (20° or more)
- **Max slices**: 5–20 (default 12). When there are more subdirectories than this, one of the slots goes to Other

**Interactions**
- **Hover**: the slice pops out and a tooltip shows its name, path, size and percentage
- **Click**: select (the slice stays popped out)
- **Double-click**: show that directory's contents (not available on the Other slice)

## Saved state

When you close the application it remembers:
- the selected view
- the last scanned folder
- the recent folders list

Scan results are **not** saved, because they can be large and would be out of date by the next launch. After restarting, use File → Recent or Tools → Refresh to scan again.

On Windows this is stored in `%APPDATA%\DiskUsage\data\app.ron`; on other platforms, in the platform's application data directory. Deleting that file resets everything to the defaults.

## Troubleshooting

### Many entries are skipped
Folders that need administrator rights (for example, other users' profiles or system folders) can't be read and are skipped. Run the application with elevated rights if you need them included.

### The scan is slow
Network drives and folders with very many small files take a while; antivirus software can also slow down file system access. You can cancel at any time.

### The folder picker does not open (Linux)
The picker uses GTK 3. Install your distribution's GTK 3 runtime (and, to build, its development package such as `libgtk-3-dev`).

### Seeing what the application is doing
Run it with logging enabled to see scans, skipped entries and clicks:

```bash
RUST_LOG=info cargo run --release            # Linux/macOS
$env:RUST_LOG="info"; cargo run --release    # Windows PowerShell
```

## Reporting Issues

Please open an issue at <https://github.com/Cj-bc/diskusage/issues> and include:
- your operating system and version
- the application version (Help → About)
- steps to reproduce
- any error messages, and the `RUST_LOG=info` output if you can
