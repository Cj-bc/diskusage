# DiskUsage User Guide

## Getting Started

### Installation

#### Prerequisites
- **Rust Toolchain**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For cloning the repository

#### Building from Source
```bash
# Clone the repository
git clone https://github.com/Cj-bc/diskusage.git
cd diskusage

# Build the application
cargo build --release

# Run the application
./target/release/diskusage
```

#### Windows-specific Notes
- Run from Command Prompt or PowerShell
- The executable will be located at `target\release\diskusage.exe`

#### Linux-specific Notes
- Ensure you have development tools installed
- May require `build-essential` or equivalent package

## Interface Overview

### Main Window Layout
```
┌─────────────────────────────────────────────────────────────┐
│ File  View  Tools  Help                           [ ][-][×] │
├─────────────────────────────────────────────────────────────┤
│ [📁] /home/user/documents     [🔍] Search    [⚙️] Settings │
├─────────────────────────────────────────────────────────────┤
│ Path: /home/user/documents    Size: 2.5 GB   Files: 1,240  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│              Visualization Area                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ [TreeMap] [TreeView] [PieChart]           Status: Ready    │
└─────────────────────────────────────────────────────────────┘
```

### Navigation Bar
- **Folder Button**: Opens directory browser
- **Path Display**: Shows current directory path
- **Search Box**: Filter directories and files
- **Settings Button**: Access application preferences

### Status Bar
- **View Mode Tabs**: Switch between visualization modes
- **Status Display**: Shows current operation status
- **Progress Bar**: Appears during directory scans

## Core Features

### 1. Directory Selection

#### Browse for Directory
1. Click the folder button (📁) in the navigation bar
2. Navigate to desired directory using the file browser
3. Click "Select" to analyze the chosen directory

#### Recent Directories
- Access recently analyzed directories from the File menu
- Use **Ctrl+R** to quickly reopen recent locations

#### Favorites
- Add frequently used directories to favorites
- Access via File → Favorites or **Ctrl+F**

### 2. Directory Analysis

#### Starting a Scan
1. Select a directory using the folder button
2. The scan will start automatically
3. Progress is shown in the status bar

#### Scan Options
- **Follow Symlinks**: Include symbolic links in analysis
- **Max Depth**: Limit recursion depth for large directories
- **File Filters**: Include/exclude specific file types

#### Canceling a Scan
- Press **Escape** or click the "Cancel" button
- Partial results will be displayed if available

### 3. Visualization Modes

#### TreeMap View
The TreeMap displays directories as rectangles, where:
- **Size**: Rectangle area proportional to directory size
- **Color**: Indicates directory depth or file type
- **Labels**: Directory names shown when space permits

**Interactions:**
- **Click**: Navigate into subdirectory
- **Right-click**: Context menu with options
- **Mouse wheel**: Zoom in/out
- **Drag**: Pan when zoomed

#### TreeView Mode
A hierarchical list showing:
- **Tree Structure**: Expandable directory hierarchy
- **Size Information**: File sizes and percentages
- **File Counts**: Number of files and subdirectories

**Interactions:**
- **Click arrow**: Expand/collapse directories
- **Click name**: Select directory
- **Double-click**: Navigate to directory
- **Right-click**: Context menu

#### PieChart View
Shows immediate subdirectories as pie slices:
- **Slice Size**: Proportional to directory size
- **Colors**: Different colors for each directory
- **Labels**: Directory names and sizes

**Interactions:**
- **Click slice**: Navigate to subdirectory
- **Hover**: Show detailed information
- **Right-click**: Context menu

### 4. Navigation

#### Drilling Down
- **TreeMap**: Click on any rectangle to zoom into that directory
- **TreeView**: Double-click on directory name or expand arrow
- **PieChart**: Click on pie slice to navigate

#### Going Back
- Use the back button (◀) in the navigation bar
- Press **Alt+Left** keyboard shortcut
- Use breadcrumb navigation in the path bar

#### Breadcrumb Navigation
Click any part of the path to navigate to that level:
```
/home/user/documents/projects/myproject
  ↑     ↑        ↑         ↑
  └─────┼────────┼─────────┼───── Click any part to navigate
```

### 5. Search and Filtering

#### Search Box
- Type to filter visible directories and files
- Use wildcards: `*.txt`, `photo*`, `*2023*`
- Case-insensitive by default

#### Filter Options
Access advanced filtering through Tools → Filters:
- **File Size**: Filter by minimum/maximum size
- **Date Modified**: Filter by date range
- **File Type**: Show only specific file extensions
- **Hidden Files**: Show/hide hidden files and directories

#### Quick Filters
- **Large Files**: Show files larger than 100MB
- **Recent**: Show files modified in last 7 days
- **Empty Directories**: Show directories with no files

### 6. Information Display

#### Size Formatting
Choose how sizes are displayed:
- **Bytes**: 1,024 bytes
- **Binary**: 1 KiB, 1 MiB, 1 GiB
- **Decimal**: 1 KB, 1 MB, 1 GB
- **Auto**: Automatically choose appropriate unit

#### Detailed Information
Hover over items to see:
- Full path
- Exact size in bytes
- File count
- Last modified date
- Permissions (on supported systems)

### 7. Context Menus

Right-click on any directory to access:
- **Open in File Manager**: Open system file manager
- **Open Terminal**: Open command prompt/terminal
- **Copy Path**: Copy full path to clipboard
- **Properties**: Show detailed information
- **Export**: Export directory information

## Advanced Features

### 8. Export Functionality

#### Export Formats
- **JSON**: Machine-readable format with full structure
- **CSV**: Spreadsheet-compatible format
- **Text**: Human-readable tree structure

#### Export Options
1. Right-click on directory and select "Export"
2. Choose export format
3. Select output location
4. Configure export options:
   - Include subdirectories
   - Size thresholds
   - File count limits

### 9. Settings and Preferences

#### Appearance
- **Theme**: Light, Dark, or System default
- **Font Size**: Adjust text size
- **Color Scheme**: Choose visualization colors
- **Animation**: Enable/disable transitions

#### Performance
- **Cache Size**: Number of directories to cache
- **Cache Duration**: How long to keep cached data
- **Scan Threads**: Number of concurrent scan operations
- **Memory Limit**: Maximum memory usage

#### Behavior
- **Auto-refresh**: Automatically update on file changes
- **Confirmation Dialogs**: Show confirmations for actions
- **Default View**: Choose default visualization mode
- **Startup Directory**: Directory to show on startup

### 10. Keyboard Shortcuts

#### Navigation
- **Ctrl+O**: Open directory browser
- **Ctrl+R**: Recent directories
- **Ctrl+F**: Favorites
- **Alt+Left**: Go back
- **Alt+Right**: Go forward
- **Ctrl+Home**: Go to root directory

#### View Controls
- **1**: TreeMap view
- **2**: TreeView view
- **3**: PieChart view
- **Ctrl+Plus**: Zoom in
- **Ctrl+Minus**: Zoom out
- **Ctrl+0**: Reset zoom

#### General
- **Ctrl+Q**: Quit application
- **F5**: Refresh current directory
- **Escape**: Cancel current operation
- **F11**: Toggle fullscreen

## Troubleshooting

### Common Issues

#### "Permission Denied" Errors
- Some directories require administrator privileges
- Use "Run as Administrator" on Windows
- Use `sudo` on Linux if necessary
- Check file permissions

#### Slow Performance
- **Large Directories**: Use depth limiting
- **Network Drives**: May be slower than local drives
- **Antivirus**: May slow down file system access
- **Memory**: Close other applications if low on RAM

#### Application Won't Start
- **Rust Installation**: Ensure Rust is properly installed
- **Dependencies**: Check for missing system libraries
- **Permissions**: Ensure executable permissions on Linux
- **Compatibility**: Check OS version compatibility

### Error Messages

#### "Path Not Found"
- Verify the directory path exists
- Check for typos in manual path entry
- Ensure drive is connected (removable media)

#### "Scan Cancelled"
- User cancelled the operation
- Click "Retry" to start scan again
- Check available disk space

#### "Export Failed"
- Insufficient disk space for export file
- Check write permissions for output directory
- Verify export path is valid

### Performance Tips

#### For Large Directories
- Use depth limiting to avoid deep recursion
- Enable caching for frequently accessed directories
- Close unnecessary applications to free memory
- Use filtering to focus on specific areas

#### For Network Drives
- Copy directory to local drive first
- Use wired connection instead of wireless
- Increase timeout settings
- Consider using alternative tools for very large network shares

## Tips and Best Practices

### Efficient Workflow
1. **Start with Overview**: Use TreeMap to identify large directories
2. **Drill Down**: Navigate to suspicious areas
3. **Use Filters**: Focus on specific file types or sizes
4. **Export Results**: Save findings for future reference

### Maintenance Tasks
- **Regular Scans**: Check disk usage weekly
- **Cache Cleanup**: Clear cache periodically
- **Update Application**: Keep application updated
- **Backup Settings**: Export preferences before major changes

### Privacy and Security
- **Sensitive Data**: Be cautious when sharing exported data
- **Network Drives**: Be aware of network visibility
- **Temporary Files**: Application doesn't create persistent logs
- **File Access**: Application only reads file metadata

## Getting Help

### Built-in Help
- **F1**: Context-sensitive help
- **Help Menu**: Access user guide and documentation
- **Tool Tips**: Hover over buttons for quick help

### Online Resources
- **Documentation**: Latest guides and tutorials
- **Issue Tracker**: Report bugs and request features
- **Community**: User forums and discussions

### Reporting Issues
When reporting problems, include:
- Operating system and version
- Application version
- Steps to reproduce the issue
- Error messages (if any)
- Directory size and structure (if relevant)

## Appendices

### Supported File Systems
- **Windows**: NTFS, FAT32, exFAT
- **Linux**: ext4, ext3, xfs, btrfs
- **Network**: SMB/CIFS, NFS (with limitations)

### File Size Limits
- **Maximum File Size**: Limited by file system
- **Maximum Directory Depth**: 1000 levels (configurable)
- **Maximum Files**: Limited by available memory

### Keyboard Reference Card
Print this reference for quick access to shortcuts:
```
Navigation          View Controls       General
Ctrl+O - Open       1 - TreeMap        Ctrl+Q - Quit
Ctrl+R - Recent     2 - TreeView       F5 - Refresh
Alt+← - Back        3 - PieChart       Esc - Cancel
Alt+→ - Forward     Ctrl+0 - Reset     F11 - Fullscreen
```