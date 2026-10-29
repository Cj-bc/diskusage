# Cross-Platform Compatibility Requirements

> **Status: design specification.** Written before the implementation and kept as a design reference. Currently the application is tested on Windows; Linux is expected to work; macOS is untested.
>
> - **As implemented:** paths go through `std::path` with no platform-specific handling; symlinks and Windows junctions are not followed; settings are stored by eframe in the platform's application data directory (`%APPDATA%\DiskUsage\data\app.ron` on Windows), not in the registry or a config file of our own.
> - **Not implemented:** registry and configuration-file integration, special handling of long Windows paths, platform-specific error types, and the platform test suites sketched below.

## Overview

This document outlines the cross-platform compatibility requirements for the DiskUsage application, ensuring consistent behavior across Windows and Linux operating systems.

## Supported Platforms

### Primary Platforms
- **Windows 10/11** (x64)
- **Linux** (x64) - Ubuntu 20.04+ LTS, Debian 11+, Fedora 35+

### Secondary Platforms (Future Support)
- **macOS** (Intel/Apple Silicon)
- **Windows** (ARM64)
- **Linux** (ARM64)

## Platform-Specific Considerations

### File System Differences

#### Windows File Systems
```rust
// Windows-specific file system handling
pub mod windows {
    use std::path::Path;
    
    pub fn get_drive_info(path: &Path) -> Result<DriveInfo, PlatformError> {
        // Handle drive letters (C:, D:, etc.)
        // Support UNC paths (\\server\share)
        // Handle junction points and symbolic links
    }
    
    pub fn get_file_attributes(path: &Path) -> Result<FileAttributes, PlatformError> {
        // Windows-specific attributes:
        // - Hidden files
        // - System files
        // - Read-only files
        // - Archive bit
        // - Compressed files
        // - Encrypted files
    }
    
    pub fn format_path(path: &Path) -> String {
        // Convert forward slashes to backslashes
        // Handle long path names (>260 characters)
        // Properly escape special characters
    }
}
```

#### Linux File Systems
```rust
// Linux-specific file system handling
pub mod linux {
    use std::path::Path;
    
    pub fn get_mount_info(path: &Path) -> Result<MountInfo, PlatformError> {
        // Handle mount points
        // Support various file systems (ext4, xfs, btrfs, etc.)
        // Handle symbolic links and hard links
    }
    
    pub fn get_file_permissions(path: &Path) -> Result<FilePermissions, PlatformError> {
        // Unix permissions (rwxrwxrwx)
        // File ownership (uid/gid)
        // Extended attributes
        // SELinux contexts
    }
    
    pub fn format_path(path: &Path) -> String {
        // Handle case-sensitive file names
        // Support Unicode file names
        // Handle special characters in names
    }
}
```

### Path Handling

#### Cross-Platform Path Utilities
```rust
pub mod path_utils {
    use std::path::{Path, PathBuf};
    
    pub fn normalize_path(path: &Path) -> PathBuf {
        // Remove redundant components (., ..)
        // Handle different path separators
        // Convert to canonical form
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }
    
    pub fn get_parent_safely(path: &Path) -> Option<PathBuf> {
        // Handle root directories appropriately
        // Windows: C:\ -> None
        // Linux: / -> None
        match path.parent() {
            Some(parent) if parent != path => Some(parent.to_path_buf()),
            _ => None,
        }
    }
    
    pub fn join_paths(base: &Path, relative: &Path) -> PathBuf {
        // Safe path joining that handles edge cases
        // Prevent directory traversal attacks
        base.join(relative)
    }
    
    pub fn get_display_name(path: &Path) -> String {
        // Platform-appropriate display names
        // Windows: Show drive letters
        // Linux: Show mount point info
        match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => path.to_string_lossy().to_string(),
        }
    }
}
```

### File System Operations

#### Safe File Operations
```rust
pub mod fs_ops {
    use std::path::Path;
    use std::fs::Metadata;
    
    pub fn get_file_size(path: &Path) -> Result<u64, PlatformError> {
        // Handle large files (>4GB)
        // Support sparse files
        // Handle file system errors gracefully
        match std::fs::metadata(path) {
            Ok(metadata) => Ok(metadata.len()),
            Err(e) => Err(PlatformError::IoError(e)),
        }
    }
    
    pub fn is_hidden(path: &Path) -> Result<bool, PlatformError> {
        #[cfg(windows)]
        {
            // Check Windows FILE_ATTRIBUTE_HIDDEN
            use std::os::windows::fs::MetadataExt;
            let metadata = std::fs::metadata(path)?;
            Ok(metadata.file_attributes() & 0x2 != 0)
        }
        
        #[cfg(unix)]
        {
            // Unix: files starting with '.' are hidden
            Ok(path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with('.'))
                .unwrap_or(false))
        }
    }
    
    pub fn can_read(path: &Path) -> bool {
        // Check read permissions
        // Handle permission errors gracefully
        match std::fs::metadata(path) {
            Ok(metadata) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    mode & 0o444 != 0
                }
                #[cfg(windows)]
                {
                    // Windows: if we can get metadata, we can usually read
                    true
                }
            },
            Err(_) => false,
        }
    }
}
```

### GUI Platform Integration

#### Native Look and Feel
```rust
pub mod gui_platform {
    use egui::{Context, Style};
    
    pub fn apply_platform_theme(ctx: &Context) {
        let mut style = Style::default();
        
        #[cfg(windows)]
        {
            // Windows 11 design system
            style.spacing.button_padding = egui::Vec2::new(12.0, 8.0);
            style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
            style.visuals.window_rounding = egui::Rounding::same(8.0);
        }
        
        #[cfg(unix)]
        {
            // Linux/GTK theme integration
            style.spacing.button_padding = egui::Vec2::new(10.0, 6.0);
            style.spacing.item_spacing = egui::Vec2::new(6.0, 4.0);
            style.visuals.window_rounding = egui::Rounding::same(4.0);
        }
        
        ctx.set_style(style);
    }
    
    pub fn get_system_colors() -> PlatformColors {
        #[cfg(windows)]
        {
            // Query Windows system colors
            PlatformColors {
                background: egui::Color32::from_rgb(240, 240, 240),
                foreground: egui::Color32::from_rgb(0, 0, 0),
                accent: egui::Color32::from_rgb(0, 120, 215),
            }
        }
        
        #[cfg(unix)]
        {
            // Query GTK theme colors
            PlatformColors {
                background: egui::Color32::from_rgb(255, 255, 255),
                foreground: egui::Color32::from_rgb(0, 0, 0),
                accent: egui::Color32::from_rgb(52, 132, 228),
            }
        }
    }
}
```

#### Window Management
```rust
pub mod window_platform {
    pub fn set_platform_window_properties(window: &mut Window) {
        #[cfg(windows)]
        {
            // Windows-specific window properties
            window.set_window_icon(load_windows_icon());
            window.set_title_bar_transparent(true);
        }
        
        #[cfg(unix)]
        {
            // Linux window manager integration
            window.set_window_icon(load_linux_icon());
            window.set_class_name("diskusage");
        }
    }
}
```

## Performance Considerations

### Platform-Specific Optimizations

#### Windows Optimizations
```rust
pub mod windows_perf {
    pub fn optimize_file_scanning() {
        // Use Windows-specific APIs for better performance
        // Enable FILE_FLAG_BACKUP_SEMANTICS for directory access
        // Use GetFileAttributesEx for efficient metadata retrieval
        // Implement directory change notifications
    }
    
    pub fn handle_long_paths() {
        // Support paths longer than MAX_PATH (260 characters)
        // Use extended-length path syntax (\\?\)
        // Handle UNC paths properly
    }
}
```

#### Linux Optimizations
```rust
pub mod linux_perf {
    pub fn optimize_file_scanning() {
        // Use sendfile() for efficient file operations
        // Implement inotify for directory change monitoring
        // Use readdir64() for large directories
        // Leverage filesystem-specific optimizations
    }
    
    pub fn handle_permissions() {
        // Gracefully handle permission denied errors
        // Support ACLs and extended attributes
        // Handle different filesystem types
    }
}
```

### Memory Management

#### Platform-Specific Memory Handling
```rust
pub mod memory_platform {
    pub fn get_available_memory() -> u64 {
        #[cfg(windows)]
        {
            // Use GlobalMemoryStatusEx
            unsafe {
                let mut mem_info = std::mem::zeroed::<MEMORYSTATUSEX>();
                mem_info.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
                GlobalMemoryStatusEx(&mut mem_info);
                mem_info.ullAvailPhys
            }
        }
        
        #[cfg(unix)]
        {
            // Parse /proc/meminfo or use sysinfo
            let mut info = std::mem::MaybeUninit::<libc::sysinfo>::uninit();
            unsafe {
                if libc::sysinfo(info.as_mut_ptr()) == 0 {
                    let info = info.assume_init();
                    info.freeram as u64 * info.mem_unit as u64
                } else {
                    0
                }
            }
        }
    }
}
```

## Configuration and Settings

### Platform-Specific Settings Storage

#### Windows Settings
```rust
pub mod windows_settings {
    use std::path::PathBuf;
    
    pub fn get_config_dir() -> PathBuf {
        // Use %APPDATA%\DiskUsage
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("DiskUsage")
    }
    
    pub fn get_cache_dir() -> PathBuf {
        // Use %LOCALAPPDATA%\DiskUsage\Cache
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("DiskUsage")
    }
}
```

#### Linux Settings
```rust
pub mod linux_settings {
    use std::path::PathBuf;
    
    pub fn get_config_dir() -> PathBuf {
        // Use $XDG_CONFIG_HOME/diskusage or ~/.config/diskusage
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("diskusage")
    }
    
    pub fn get_cache_dir() -> PathBuf {
        // Use $XDG_CACHE_HOME/diskusage or ~/.cache/diskusage
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~/.cache"))
            .join("diskusage")
    }
}
```

### Registry and Configuration Files

#### Windows Registry Integration
```rust
pub mod windows_registry {
    use winreg::RegKey;
    
    pub fn save_settings(settings: &AppSettings) -> Result<(), PlatformError> {
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let key = hkcu.create_subkey("SOFTWARE\\DiskUsage")?;
        
        key.set_value("Theme", &settings.theme.to_string())?;
        key.set_value("DefaultView", &settings.default_view.to_string())?;
        
        Ok(())
    }
    
    pub fn load_settings() -> Result<AppSettings, PlatformError> {
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let key = hkcu.open_subkey("SOFTWARE\\DiskUsage")?;
        
        let theme: String = key.get_value("Theme")?;
        let default_view: String = key.get_value("DefaultView")?;
        
        Ok(AppSettings {
            theme: theme.parse()?,
            default_view: default_view.parse()?,
            ..Default::default()
        })
    }
}
```

#### Linux Configuration Files
```rust
pub mod linux_config {
    use std::path::Path;
    
    pub fn save_settings(settings: &AppSettings) -> Result<(), PlatformError> {
        let config_dir = get_config_dir();
        std::fs::create_dir_all(&config_dir)?;
        
        let config_file = config_dir.join("config.toml");
        let toml = toml::to_string(settings)?;
        std::fs::write(config_file, toml)?;
        
        Ok(())
    }
    
    pub fn load_settings() -> Result<AppSettings, PlatformError> {
        let config_file = get_config_dir().join("config.toml");
        
        if config_file.exists() {
            let content = std::fs::read_to_string(config_file)?;
            let settings: AppSettings = toml::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(AppSettings::default())
        }
    }
}
```

## Error Handling

### Platform-Specific Error Types
```rust
#[derive(Debug, Clone)]
pub enum PlatformError {
    IoError(std::io::Error),
    PermissionDenied(PathBuf),
    PathNotFound(PathBuf),
    PathTooLong(PathBuf),
    InvalidPath(PathBuf),
    FileSystemError(String),
    
    #[cfg(windows)]
    WindowsError(u32),
    
    #[cfg(unix)]
    UnixError(i32),
}

impl std::error::Error for PlatformError {}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformError::IoError(e) => write!(f, "I/O error: {}", e),
            PlatformError::PermissionDenied(path) => {
                write!(f, "Permission denied: {}", path.display())
            },
            PlatformError::PathNotFound(path) => {
                write!(f, "Path not found: {}", path.display())
            },
            PlatformError::PathTooLong(path) => {
                write!(f, "Path too long: {}", path.display())
            },
            PlatformError::InvalidPath(path) => {
                write!(f, "Invalid path: {}", path.display())
            },
            PlatformError::FileSystemError(msg) => {
                write!(f, "File system error: {}", msg)
            },
            
            #[cfg(windows)]
            PlatformError::WindowsError(code) => {
                write!(f, "Windows error: {}", code)
            },
            
            #[cfg(unix)]
            PlatformError::UnixError(code) => {
                write!(f, "Unix error: {}", code)
            },
        }
    }
}
```

## Testing Strategy

### Platform-Specific Testing

#### Windows Testing
```rust
#[cfg(test)]
#[cfg(windows)]
mod windows_tests {
    use super::*;
    
    #[test]
    fn test_drive_detection() {
        // Test detection of available drives
        // Test UNC path handling
        // Test long path support
    }
    
    #[test]
    fn test_file_attributes() {
        // Test hidden file detection
        // Test system file handling
        // Test compressed file support
    }
    
    #[test]
    fn test_registry_operations() {
        // Test settings save/load
        // Test registry key creation
        // Test error handling
    }
}
```

#### Linux Testing
```rust
#[cfg(test)]
#[cfg(unix)]
mod linux_tests {
    use super::*;
    
    #[test]
    fn test_mount_point_detection() {
        // Test mount point enumeration
        // Test symlink handling
        // Test permission checking
    }
    
    #[test]
    fn test_file_permissions() {
        // Test permission bit reading
        // Test ownership detection
        // Test extended attributes
    }
    
    #[test]
    fn test_config_file_operations() {
        // Test TOML serialization
        // Test XDG directory handling
        // Test file creation/reading
    }
}
```

### Cross-Platform Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_path_normalization() {
        // Test path handling across platforms
        // Test special character handling
        // Test Unicode support
    }
    
    #[test]
    fn test_file_scanning() {
        // Test directory traversal
        // Test error handling
        // Test performance characteristics
    }
    
    #[test]
    fn test_settings_persistence() {
        // Test settings save/load cycle
        // Test migration between versions
        // Test error recovery
    }
}
```

## Build and Deployment

### Platform-Specific Build Configuration

#### Cargo.toml Platform Dependencies
```toml
[dependencies]
# Cross-platform dependencies
egui = "0.24"
eframe = "0.24"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
dirs = "5.0"

# Windows-specific dependencies
[target.'cfg(windows)'.dependencies]
winreg = "0.50"
windows = { version = "0.52", features = ["Win32_System_SystemInformation"] }

# Linux-specific dependencies
[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

#### Build Scripts
```rust
// build.rs
fn main() {
    #[cfg(windows)]
    {
        // Windows-specific build configuration
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=kernel32");
        
        // Embed Windows manifest
        embed_manifest();
    }
    
    #[cfg(unix)]
    {
        // Linux-specific build configuration
        println!("cargo:rustc-link-lib=dl");
        
        // Check for required system libraries
        check_system_dependencies();
    }
}
```

### Distribution

#### Windows Distribution
- **Installer**: NSIS or WiX-based installer
- **Portable**: Single executable with embedded resources
- **Microsoft Store**: MSIX package
- **Chocolatey**: Package manager integration

#### Linux Distribution
- **AppImage**: Portable application format
- **Flatpak**: Sandboxed application distribution
- **Snap**: Universal package format
- **Native Packages**: .deb, .rpm, AUR packages

## Security Considerations

### Platform-Specific Security

#### Windows Security
```rust
pub mod windows_security {
    pub fn check_file_permissions(path: &Path) -> Result<bool, PlatformError> {
        // Check Windows ACLs
        // Verify file integrity
        // Handle UAC requirements
    }
    
    pub fn sanitize_path(path: &Path) -> Result<PathBuf, PlatformError> {
        // Prevent directory traversal
        // Handle junction points safely
        // Validate path components
    }
}
```

#### Linux Security
```rust
pub mod linux_security {
    pub fn check_file_permissions(path: &Path) -> Result<bool, PlatformError> {
        // Check Unix permissions
        // Verify file ownership
        // Handle SELinux contexts
    }
    
    pub fn sanitize_path(path: &Path) -> Result<PathBuf, PlatformError> {
        // Prevent symlink attacks
        // Validate path components
        // Handle special files safely
    }
}
```

This comprehensive cross-platform compatibility specification ensures that the DiskUsage application will work consistently and efficiently across both Windows and Linux platforms while respecting platform-specific conventions and capabilities.