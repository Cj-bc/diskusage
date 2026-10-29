# Build Guide

How to build, run and test DiskUsage. What the build script and profiles do is described in [BUILD_CONFIG.md](BUILD_CONFIG.md).

## Prerequisites

- A recent stable **Rust** toolchain from [rustup.rs](https://rustup.rs/) (`rustc --version` to check)
- **Git**

### Windows
- The MSVC toolchain (`x86_64-pc-windows-msvc`, rustup's default) with the Visual Studio C++ build tools that rustup asks for. The GNU toolchain is not supported; see [BUILD_CONFIG.md](BUILD_CONFIG.md#windows-msvc)

### Linux
A C toolchain, `pkg-config`, GTK 3 (for the folder picker), and the X11/Wayland libraries eframe links against:

```bash
# Debian / Ubuntu
sudo apt-get install build-essential pkg-config libgtk-3-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev

# Fedora
sudo dnf install gcc pkgconf-pkg-config gtk3-devel libxcb-devel libxkbcommon-devel

# Arch Linux
sudo pacman -S base-devel gtk3 libxcb libxkbcommon
```

### macOS
Xcode command line tools (`xcode-select --install`). macOS builds are untested.

## Building and Running

```bash
git clone https://github.com/Cj-bc/diskusage.git
cd diskusage

# Debug build: fast to compile, slow to run on large directories
cargo run

# Release build: fat LTO, slow to link, much faster scans
cargo run --release
```

The binary is written to `target/debug/` or `target/release/` as `diskusage` (`diskusage.exe` on Windows).

### Logging
Logging goes through `env_logger` and is controlled with `RUST_LOG`:

```bash
RUST_LOG=info cargo run              # scans, skipped entries, clicks
RUST_LOG=diskusage=debug cargo run   # only this crate, more detail
```

In PowerShell, set it first: `$env:RUST_LOG="info"`.

## Project Layout

```
diskusage/
├── src/            # Source code; see ARCHITECTURE.md
├── assets/         # icon.png
├── build.rs        # Platform-specific build directives; see BUILD_CONFIG.md
├── Cargo.toml
├── Cargo.lock      # Committed: this is an application
└── *.md            # Documentation
```

`target/` is git-ignored.

## Testing

Tests are unit tests next to the code; there are no integration tests or benchmarks yet.

```bash
cargo test                       # everything
cargo test scanner               # tests whose path contains "scanner"
cargo test -- --nocapture        # show println! output
```

The scanner tests create real files in temporary directories (via `tempfile`), so they also exercise the platform's file system. Rendering is not covered by tests; check UI changes by running the app.

## Linting and Formatting

The code is kept free of compiler and clippy warnings and is formatted with default `rustfmt` settings:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Run both, plus `cargo test`, before committing.

### Optional tools
```bash
cargo install cargo-audit     # cargo audit: check dependencies for advisories
cargo install cargo-outdated  # cargo outdated: list newer dependency versions
cargo doc --no-deps --open    # API docs from the source
```

## Continuous Integration

There is no CI configured yet. A workflow would need to run the three commands from [Linting and Formatting](#linting-and-formatting) and [Testing](#testing) on Windows and Linux, installing the Linux packages listed above first.

## Troubleshooting

### `unresolved import winapi::um::winuser` in eframe (Windows)
eframe 0.24 relies on another crate enabling winapi's `winuser` feature. `Cargo.toml` does that with a `winapi` dependency; don't remove it while on eframe 0.24.

### Linker errors about `/OPT:REF` or `/LTCG` (Windows)
You are probably using the GNU toolchain. Switch to MSVC: `rustup default stable-x86_64-pc-windows-msvc`.

### `GTK3 development libraries not found` warning (Linux)
`build.rs` could not find GTK 3 with `pkg-config`. Install `libgtk-3-dev` (or your distribution's equivalent); without it the folder picker cannot be built.

### Missing `xcb` / `xkbcommon` libraries (Linux)
Install the packages listed under [Linux](#linux).

### Release builds take a long time
That is fat LTO with one codegen unit. Use `cargo build` (debug) while developing.
