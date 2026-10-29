# Build Configuration

What `build.rs` and the Cargo profiles in `Cargo.toml` do. For how to build, see [BUILD.md](BUILD.md).

## `build.rs`

The build script runs before every compile of the crate and prints Cargo directives depending on the target OS and architecture.

### Windows (MSVC)
- Linker flags `/OPT:REF` and `/OPT:ICF` (drop and fold unused code), and `/MANIFESTUAC:level='asInvoker'` (run without requesting elevation)
- In release builds, `/LTCG` (link-time code generation)
- Links `user32`, `shell32`, `ole32` and `comctl32`
- `/SUBSYSTEM:CONSOLE` when the build script itself is compiled with debug assertions. Console is the default subsystem anyway: `main.rs` does not set `#![windows_subsystem = "windows"]`, so release builds also open a console window alongside the GUI

These are MSVC linker flags and are passed for every Windows target, so the GNU toolchain (`x86_64-pc-windows-gnu`) is not expected to link.

### Linux
- Probes for `gtk+-3.0` with `pkg-config` and prints a warning if it is missing. GTK 3 is needed by `rfd` for the native folder picker
- Links `pthread`, `dl` and `m`

### macOS
- Links the `Cocoa`, `Foundation` and `CoreServices` frameworks
- Sets `MACOSX_DEPLOYMENT_TARGET=10.12` with `cargo:rustc-env`. This only makes the variable visible to `env!()` in the crate; it does not change the deployment target the binary is built for

macOS builds are untested.

### Other targets
The script prints a warning for operating systems other than Windows, Linux and macOS, and for architectures other than `x86_64` and `aarch64`. It does not stop the build.

### `cfg` flags
The script emits these flags, but no code reads them yet:

| Flag | When |
|---|---|
| `target_windows`, `target_linux`, `target_macos` | Per target OS |
| `windows_7_compatible` | Windows |
| `gtk3_available` | Linux, if `pkg-config` finds GTK 3 |
| `unix_permissions`, `linux_filesystems`, `macos_bundle` | Linux / macOS |
| `arch_x86_64`, `arch_aarch64` | Per target architecture |
| `windows_x64_optimized`, `linux_x64_optimized`, `macos_x64_optimized`, `macos_arm64_optimized` | Per OS/architecture pair |
| `lto_enabled` | Always |
| `release_build` / `debug_build` | Per `PROFILE` |
| `simd_sse2`, `simd_avx`, `simd_neon` | If the target enables that feature |

If you start using one, declare it so the `unexpected_cfgs` lint knows about it, e.g. `println!("cargo::rustc-check-cfg=cfg(target_linux)");`.

## Cargo features

`Cargo.toml` declares `profiling`, `debug-ui`, `bundle` and `embed_resources`. None has an effect yet. `build.rs` contains `setup_app_bundle` (for `bundle`) and `embed_resources` (for `embed_resources`), but nothing calls them.

## Profiles

| Setting | `dev` | `release` |
|---|---|---|
| `opt-level` | 0 | 3 |
| `debug` | true | false |
| `debug-assertions`, `overflow-checks` | on | off |
| `lto` | off | on (fat) |
| `codegen-units` | 256 | 1 |
| `incremental` | on | off |
| `panic` | unwind | abort |

`bench` inherits from `release` with debug info enabled. There are no benchmarks yet.

Release builds are slow to link because of fat LTO and a single codegen unit; use the `dev` profile while iterating.

## Dependencies

### Build dependencies
- `pkg-config`: GTK 3 detection on Linux

### Runtime dependencies
| Crate | Used for |
|---|---|
| `eframe` / `egui` 0.24 | Window, rendering and widgets; `persistence` feature for saved state |
| `rfd` | Native folder picker |
| `serde` | Serializing persisted state |
| `thiserror` | `ScanError` |
| `dirs` | Home/documents/downloads folders in `DirectoryBrowser` |
| `log`, `env_logger` | Logging, controlled with `RUST_LOG` |
| `winapi` (Windows only) | Not used directly. eframe 0.24 imports `winapi::um::winuser` without enabling that feature, so this crate enables it |

### Dev dependencies
- `tempfile`: temporary directories in scanner tests
