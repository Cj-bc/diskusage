# Build and Deployment Guide

## Overview

This document provides comprehensive instructions for building, testing, and deploying the DiskUsage application across different platforms and environments.

## Prerequisites

### Development Environment

#### Required Tools
- **Rust**: Version 1.70.0 or later
- **Cargo**: Included with Rust
- **Git**: For version control

#### Platform-Specific Requirements

##### Windows
- **Visual Studio Build Tools 2019** or later
- **Windows SDK**: Latest version
- **PowerShell**: For build scripts

##### Linux
- **GCC**: Version 7.0 or later
- **CMake**: Version 3.16 or later
- **Development packages**:
  ```bash
  # Ubuntu/Debian
  sudo apt-get install build-essential libgtk-3-dev libxcb-shape0-dev libxcb-xfixes0-dev
  
  # Fedora/CentOS
  sudo dnf install gcc gcc-c++ gtk3-devel libxcb-devel
  
  # Arch Linux
  sudo pacman -S base-devel gtk3 libxcb
  ```

### Installing Rust

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Additional Tools

```bash
# Install additional Rust tools
cargo install cargo-audit        # Security auditing
cargo install cargo-outdated     # Dependency updates
cargo install cargo-watch       # File watching
cargo install cargo-criterion   # Benchmarking
```

## Project Structure

```
diskusage/
├── src/                    # Source code
│   ├── main.rs            # Application entry point
│   ├── app.rs             # Main application logic
│   ├── gui/               # GUI components
│   ├── fs/                # File system operations
│   ├── data/              # Data structures
│   ├── visualization/     # Visualization components
│   └── utils/             # Utility functions
├── tests/                 # Integration tests
├── benches/               # Benchmarks
├── examples/              # Example code
├── docs/                  # Documentation
├── assets/                # Static assets
├── scripts/               # Build scripts
├── .github/               # GitHub workflows
├── Cargo.toml             # Project configuration
├── Cargo.lock             # Dependency lock file
├── build.rs               # Build script
└── README.md              # Project overview
```

## Build Configuration

### Cargo.toml

```toml
[package]
name = "diskusage"
version = "0.1.0"
edition = "2021"
authors = ["Cj-bc <cj.bc-sd@outlook.jp>"]
description = "A cross-platform GUI application for visualizing directory space usage"
repository = "https://github.com/Cj-bc/diskusage"
license = "MIT"
keywords = ["gui", "filesystem", "visualization", "disk-usage"]
categories = ["filesystem", "gui", "visualization"]

[dependencies]
# GUI Framework
egui = "0.24"
eframe = { version = "0.24", features = ["persistence"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# File system utilities
dirs = "5.0"
walkdir = "2.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.10"

# Performance
rayon = "1.7"

# Platform-specific dependencies
[target.'cfg(windows)'.dependencies]
winreg = "0.50"
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_System_SystemInformation",
    "Win32_Storage_FileSystem",
    "Win32_System_Threading",
] }

[target.'cfg(unix)'.dependencies]
libc = "0.2"
nix = "0.27"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.8"
proptest = "1.4"

[build-dependencies]
vergen = "8.2"

# Profile configurations
[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"
debug-assertions = true
overflow-checks = true
lto = false
panic = "unwind"
incremental = true
codegen-units = 256
rpath = false

[profile.release]
opt-level = 3
debug = false
split-debuginfo = "packed"
debug-assertions = false
overflow-checks = false
lto = true
panic = "abort"
incremental = false
codegen-units = 1
rpath = false

[profile.bench]
inherits = "release"
debug = true

[[bench]]
name = "directory_scan"
harness = false

[[bench]]
name = "treemap_layout"
harness = false
```

### Build Script (build.rs)

```rust
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Generate version information
    generate_version_info();
    
    // Platform-specific build configuration
    configure_platform_specific();
    
    // Handle static assets
    bundle_assets();
    
    // Set up conditional compilation
    setup_feature_flags();
}

fn generate_version_info() {
    // Use vergen to generate version information
    let mut config = vergen::Config::default();
    *config.git_mut().sha_kind_mut() = vergen::ShaKind::Short;
    *config.git_mut().commit_timestamp_kind_mut() = vergen::TimestampKind::DateOnly;
    
    if let Err(e) = vergen::vergen(config) {
        println!("cargo:warning=Failed to generate version info: {}", e);
    }
}

fn configure_platform_specific() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    match target_os.as_str() {
        "windows" => {
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=kernel32");
            println!("cargo:rustc-link-lib=dwmapi");
            
            // Embed Windows manifest
            embed_windows_manifest();
        }
        "linux" => {
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=pthread");
            
            // Check for required libraries
            check_linux_dependencies();
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=AppKit");
            println!("cargo:rustc-link-lib=framework=Foundation");
        }
        _ => {}
    }
}

fn embed_windows_manifest() {
    #[cfg(windows)]
    {
        let manifest = r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity
        version="1.0.0.0"
        processorArchitecture="*"
        name="DiskUsage"
        type="win32"/>
    <description>DiskUsage - Directory Space Visualization</description>
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
    <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
        <application>
            <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
            <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
            <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
            <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
            <supportedOS Id="{e2011457-1546-43c5-a5fe-008deee3d3f0}"/>
        </application>
    </compatibility>
    <application xmlns="urn:schemas-microsoft-com:asm.v3">
        <windowsSettings>
            <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
            <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
        </windowsSettings>
    </application>
</assembly>
"#;
        
        fs::write("app.manifest", manifest).unwrap();
        println!("cargo:rustc-link-arg=/MANIFEST:app.manifest");
    }
}

fn check_linux_dependencies() {
    // Check for required libraries
    let required_libs = ["gtk-3", "gdk-3", "glib-2.0"];
    
    for lib in &required_libs {
        if !pkg_config::probe(lib).is_ok() {
            println!("cargo:warning=Missing required library: {}", lib);
        }
    }
}

fn bundle_assets() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let assets_dir = Path::new("assets");
    
    if assets_dir.exists() {
        // Copy assets to output directory
        copy_dir_all(assets_dir, Path::new(&out_dir).join("assets")).unwrap();
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    
    Ok(())
}

fn setup_feature_flags() {
    // Enable features based on build configuration
    if cfg!(feature = "profiling") {
        println!("cargo:rustc-cfg=profiling_enabled");
    }
    
    if cfg!(feature = "debug-ui") {
        println!("cargo:rustc-cfg=debug_ui_enabled");
    }
}
```

## Building the Application

### Development Build

```bash
# Clone the repository
git clone https://github.com/Cj-bc/diskusage.git
cd diskusage

# Build in debug mode
cargo build

# Run the application
cargo run

# Build and run with logging
RUST_LOG=debug cargo run
```

### Release Build

```bash
# Build optimized release version
cargo build --release

# Run release version
./target/release/diskusage

# Build with specific features
cargo build --release --features="profiling,debug-ui"
```

### Cross-Platform Builds

#### Windows from Linux

```bash
# Install cross-compilation target
rustup target add x86_64-pc-windows-gnu

# Install MinGW
sudo apt-get install gcc-mingw-w64-x86-64

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu
```

#### Linux from Windows

```bash
# Install WSL2 with Ubuntu
wsl --install -d Ubuntu

# Install cross-compilation target
rustup target add x86_64-unknown-linux-gnu

# Build for Linux
cargo build --release --target x86_64-unknown-linux-gnu
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_directory_scan

# Run with output
cargo test -- --nocapture

# Run tests with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --workspace --timeout 120
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration_tests

# Run with specific test directory
TEST_DIR=/tmp/test_data cargo test --test integration_tests
```

### Benchmarks

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench directory_scan

# Generate benchmark report
cargo bench -- --output-format html
```

### Performance Testing

```bash
# Install performance tools
cargo install cargo-profiler

# Profile the application
cargo profiler callgrind --bin diskusage
```

## Static Analysis

### Code Quality

```bash
# Run Clippy for linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Security Auditing

```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Documentation

```bash
# Generate documentation
cargo doc --no-deps --open

# Check documentation
cargo doc --document-private-items
```

## Continuous Integration

### GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        override: true
        components: rustfmt, clippy
    
    - name: Install system dependencies (Linux)
      if: runner.os == 'Linux'
      run: |
        sudo apt-get update
        sudo apt-get install -y libgtk-3-dev libxcb-shape0-dev libxcb-xfixes0-dev
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: cargo test --all-features
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Check formatting
      run: cargo fmt -- --check

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y libgtk-3-dev libxcb-shape0-dev libxcb-xfixes0-dev
    
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    
    - name: Generate coverage report
      run: cargo tarpaulin --all-features --workspace --timeout 120 --out Xml
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./cobertura.xml
        flags: unittests
        name: codecov-umbrella
        fail_ci_if_error: true

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Install cargo-audit
      run: cargo install cargo-audit
    
    - name: Run security audit
      run: cargo audit
```

### Release Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: diskusage
            asset_name: diskusage-linux-x86_64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: diskusage.exe
            asset_name: diskusage-windows-x86_64.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: diskusage
            asset_name: diskusage-macos-x86_64
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true
    
    - name: Install system dependencies (Linux)
      if: runner.os == 'Linux'
      run: |
        sudo apt-get update
        sudo apt-get install -y libgtk-3-dev libxcb-shape0-dev libxcb-xfixes0-dev
    
    - name: Build release
      run: cargo build --release --target ${{ matrix.target }}
    
    - name: Upload binaries to release
      uses: svenstaro/upload-release-action@v2
      with:
        repo_token: ${{ secrets.GITHUB_TOKEN }}
        file: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
        asset_name: ${{ matrix.asset_name }}
        tag: ${{ github.ref }}
        overwrite: true
```

## Deployment

### Binary Distribution

#### Linux

```bash
# Create AppImage
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

# Create directory structure
mkdir -p DiskUsage.AppDir/usr/bin
mkdir -p DiskUsage.AppDir/usr/share/applications
mkdir -p DiskUsage.AppDir/usr/share/icons/hicolor/256x256/apps

# Copy binary and assets
cp target/release/diskusage DiskUsage.AppDir/usr/bin/
cp assets/diskusage.desktop DiskUsage.AppDir/usr/share/applications/
cp assets/diskusage.png DiskUsage.AppDir/usr/share/icons/hicolor/256x256/apps/

# Create AppImage
./appimagetool-x86_64.AppImage DiskUsage.AppDir
```

#### Windows

```bash
# Create installer using NSIS
makensis installer.nsi

# Create portable version
mkdir portable
cp target/release/diskusage.exe portable/
cp -r assets portable/
```

#### macOS

```bash
# Create application bundle
mkdir -p DiskUsage.app/Contents/MacOS
mkdir -p DiskUsage.app/Contents/Resources

# Copy binary and assets
cp target/release/diskusage DiskUsage.app/Contents/MacOS/
cp assets/Info.plist DiskUsage.app/Contents/
cp assets/diskusage.icns DiskUsage.app/Contents/Resources/

# Create DMG
hdiutil create -volname "DiskUsage" -srcfolder DiskUsage.app -ov -format UDZO DiskUsage.dmg
```

### Package Managers

#### Cargo

```bash
# Publish to crates.io
cargo publish
```

#### Homebrew (macOS)

```ruby
# diskusage.rb
class Diskusage < Formula
  desc "Cross-platform GUI application for visualizing directory space usage"
  homepage "https://github.com/Cj-bc/diskusage"
  url "https://github.com/Cj-bc/diskusage/archive/v1.0.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/diskusage", "--version"
  end
end
```

#### Chocolatey (Windows)

```xml
<!-- diskusage.nuspec -->
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>diskusage</id>
    <version>1.0.0</version>
    <title>DiskUsage</title>
    <authors>Cj-bc</authors>
    <projectUrl>https://github.com/Cj-bc/diskusage</projectUrl>
    <licenseUrl>https://github.com/Cj-bc/diskusage/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <description>Cross-platform GUI application for visualizing directory space usage</description>
    <tags>filesystem gui visualization disk-usage</tags>
  </metadata>
  <files>
    <file src="target\release\diskusage.exe" target="tools\diskusage.exe" />
  </files>
</package>
```

## Docker Support

### Dockerfile

```dockerfile
# Multi-stage build for smaller image
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libgtk-3-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    && rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libgtk-3-0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/diskusage /usr/local/bin/

# Set up user
RUN useradd -m -s /bin/bash diskusage
USER diskusage

ENTRYPOINT ["diskusage"]
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  diskusage:
    build: .
    volumes:
      - /tmp/.X11-unix:/tmp/.X11-unix:rw
      - $HOME/.Xauthority:/root/.Xauthority:rw
      - /data:/data:ro
    environment:
      - DISPLAY
    network_mode: host
```

## Development Scripts

### Build Scripts

```bash
#!/bin/bash
# scripts/build.sh

set -e

echo "Building DiskUsage..."

# Clean previous builds
cargo clean

# Run tests
echo "Running tests..."
cargo test --all-features

# Run clippy
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Format code
echo "Formatting code..."
cargo fmt

# Build release
echo "Building release..."
cargo build --release

echo "Build completed successfully!"
```

### Release Script

```bash
#!/bin/bash
# scripts/release.sh

set -e

VERSION=${1:-patch}

echo "Preparing release..."

# Bump version
cargo install cargo-edit
cargo set-version --bump $VERSION

# Update changelog
git cliff --tag $(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "diskusage") | .version') > CHANGELOG.md

# Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to $(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "diskusage") | .version')"

# Create tag
git tag -a v$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "diskusage") | .version') -m "Release v$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "diskusage") | .version')"

echo "Release prepared. Run 'git push --tags' to publish."
```

## Troubleshooting

### Common Build Issues

#### Missing Dependencies

```bash
# Ubuntu/Debian
sudo apt-get install build-essential libgtk-3-dev pkg-config

# Fedora/CentOS
sudo dnf install gcc-c++ gtk3-devel pkgconfig

# macOS
brew install gtk+3 pkg-config
```

#### Rust Version Issues

```bash
# Update Rust
rustup update

# Install specific version
rustup install 1.70.0
rustup default 1.70.0
```

#### Link Errors

```bash
# Clear cargo cache
cargo clean
rm -rf ~/.cargo/registry/cache

# Rebuild
cargo build --release
```

### Performance Issues

```bash
# Enable optimizations
export RUSTFLAGS="-C target-cpu=native"
cargo build --release

# Use parallel compilation
export CARGO_BUILD_JOBS=4
cargo build --release
```

This comprehensive build and deployment guide provides everything needed to develop, build, test, and deploy the DiskUsage application across multiple platforms and distribution channels.