# Build Configuration

This document describes the platform-specific build configurations implemented in `build.rs`.

## Platform Support

### Windows
- **Target**: `windows`
- **Features**:
  - High DPI awareness
  - Windows 7+ compatibility
  - Console subsystem for debug builds
  - Link-time optimization (LTO) for release builds
  - Required system libraries: user32, shell32, ole32, comctl32

### Linux
- **Target**: `linux`
- **Features**:
  - GTK3 detection and configuration
  - Unix permissions support
  - Linux filesystem-specific optimizations
  - Required system libraries: pthread, dl, m

### macOS
- **Target**: `macos`
- **Features**:
  - Cocoa framework integration
  - App bundle support
  - macOS 10.12+ deployment target
  - ARM64 (Apple Silicon) and x86_64 support

## Build Features

### Conditional Compilation Flags

The build script sets various `cfg` flags that can be used for conditional compilation:

- `target_windows` - Windows-specific code
- `target_linux` - Linux-specific code  
- `target_macos` - macOS-specific code
- `unix_permissions` - Unix permission handling (Linux/macOS)
- `gtk3_available` - GTK3 libraries detected (Linux)
- `arch_x86_64` - x86_64 architecture
- `arch_aarch64` - ARM64 architecture
- `simd_sse2`, `simd_avx`, `simd_neon` - Available SIMD instruction sets

### Build Profiles

#### Debug Profile
- Console subsystem enabled on Windows
- Full debug information
- No optimizations
- Incremental compilation

#### Release Profile
- Optimized for size and speed
- Link-time optimization (LTO)
- Stripped debug information
- Single codegen unit for maximum optimization

## Optional Features

### Bundle (`--features bundle`)
- Enables application bundle creation
- Sets bundle metadata for macOS applications

### Embed Resources (`--features embed_resources`)  
- Embeds application resources into the binary
- Includes icons and other assets
- Windows resource file support

## Usage Examples

```bash
# Standard build
cargo build

# Release build with all optimizations
cargo build --release

# Build with bundle feature (macOS)
cargo build --features bundle

# Build with embedded resources
cargo build --features embed_resources

# Cross-compilation example (Linux to Windows)
cargo build --target x86_64-pc-windows-gnu
```

## Architecture-Specific Optimizations

The build script detects the target architecture and enables appropriate optimizations:

- **x86_64**: SSE2/AVX SIMD instructions where available
- **ARM64**: NEON SIMD instructions where available
- **Platform-specific**: Native optimization flags per OS

## Dependencies

### Build Dependencies
- `vergen` - Build information generation
- `pkg-config` - System library detection (Linux)

### Runtime Dependencies
Platform-specific dependencies are automatically handled through Cargo.toml target-specific sections.