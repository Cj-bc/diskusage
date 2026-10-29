# Contributing to DiskUsage

## Welcome

Thank you for your interest in contributing to DiskUsage! This document provides guidelines and information for contributors to help maintain code quality and streamline the development process.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Code Standards](#code-standards)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Documentation](#documentation)
- [Performance Considerations](#performance-considerations)

## Code of Conduct

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

Examples of behavior that contributes to creating a positive environment include:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported by contacting the project team. All complaints will be reviewed and investigated and will result in a response that is deemed necessary and appropriate to the circumstances.

## Getting Started

### Prerequisites

- **Rust**: Install the latest stable version from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Platform-specific tools**:
  - Windows: Visual Studio Build Tools or Visual Studio
  - Linux: GCC, development headers

### Setting Up Your Development Environment

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/diskusage.git
   cd diskusage
   ```

3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/Cj-bc/diskusage.git
   ```

4. **Install dependencies**:
   ```bash
   cargo build
   ```

5. **Run tests** to ensure everything works:
   ```bash
   cargo test
   ```

6. **Run the application**:
   ```bash
   cargo run
   ```

## Development Environment

### Recommended IDE Setup

#### VS Code
- **Extensions**:
  - rust-analyzer
  - CodeLLDB (for debugging)
  - Better TOML
  - GitLens

#### Other IDEs
- **CLion**: Native Rust support
- **IntelliJ IDEA**: Rust plugin
- **Vim/Neovim**: rust.vim, coc-rust-analyzer

### Code Formatting

We use `rustfmt` with its default settings (there is no `rustfmt.toml`). Configure your editor to format on save, or run:

```bash
cargo fmt
```

### Linting

The code is kept free of clippy warnings, tests included:
```bash
cargo clippy --all-targets -- -D warnings
```

## Code Standards

### Rust Code Guidelines

#### Naming Conventions
```rust
// Types: PascalCase
struct DirectoryNode {}
enum ViewMode {}
trait Visualization {}

// Functions and variables: snake_case
fn calculate_size() {}
let file_count = 0;

// Constants: SCREAMING_SNAKE_CASE
const MAX_DEPTH: usize = 1000;

// Modules: snake_case
mod file_system;
mod ui_components;
```

#### Error Handling
```rust
// Use Result for functions that can fail
fn scan_directory(path: &Path) -> Result<DirectoryNode, ScanError> {
    // Implementation
}

// Use custom error types that keep the context (here, the path)
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("I/O error at {path}: {message}")]
    IoError { path: PathBuf, message: String },
}

// Avoid unwrap()/expect() outside tests, and don't index or slice strings
// by byte offset: names can be non-ASCII. Use chars() instead.
```

#### Documentation
```rust
/// Scans a directory and returns a tree structure representing its contents.
///
/// # Arguments
/// * `path` - The path to the directory to scan
/// * `options` - Scanning options (depth limit, symlink handling)
///
/// # Returns
/// A `DirectoryNode` containing the directory structure and size information.
///
/// # Errors
/// Returns `ScanError` if the directory cannot be accessed or read.
fn scan_directory(path: &Path, options: &ScanOptions) -> Result<DirectoryNode, ScanError> {
    // Implementation
}
```

#### Memory Management
```rust
// Share large data through Arc instead of cloning it. The three views
// share one scan this way; never clone a DirectoryNode tree per frame.
use std::sync::Arc;

struct TreeMapData {
    scan_result: Arc<ScanResult>,
    // ...
}

// Make invalid states unrepresentable: prefer an enum over several
// flags and Options that must be kept in sync by hand
enum ScanState {
    Idle,
    Scanning { progress: Option<ScanProgress> },
    Loaded(Box<LoadedScan>),
}

// Prefer owned data when possible
fn process_node(node: DirectoryNode) -> ProcessedNode {
    // Takes ownership, avoiding unnecessary clones
}

// Use references for read-only access
fn display_node(node: &DirectoryNode) {
    // Read-only access
}
```

### Performance Guidelines

#### Efficient Data Structures
```rust
// Use Vec for sequential access
let mut items = Vec::new();

// Use HashMap for key-value lookup
let mut cache = HashMap::new();

// Use BTreeMap for sorted keys
let mut sorted_items = BTreeMap::new();

// Use HashSet for unique items
let mut visited = HashSet::new();
```

#### Avoid Unnecessary Allocations
```rust
// Good: reuse string
let mut buffer = String::new();
for item in items {
    buffer.clear();
    write!(buffer, "Item: {}", item.name)?;
    println!("{}", buffer);
}

// Avoid: creating new strings
for item in items {
    let text = format!("Item: {}", item.name);
    println!("{}", text);
}
```

#### Keep the UI Thread Responsive
`eframe::App::update` runs once per frame, so it must never block. Long-running work runs on a background thread and reports back through a channel that the UI polls each frame. The scanner is the model to follow:

```rust
// UI thread, every frame
match self.scanner.poll() {
    Some(ScanUpdate::Progress(progress)) => { /* update the status bar */ }
    Some(ScanUpdate::Finished(result)) => { /* switch to the result */ }
    None => {}
}
```

The project has no async runtime or thread-pool dependency. Discuss in an issue before adding one.

## Testing Guidelines

### Unit Tests
Tests live in a `#[cfg(test)] mod tests` next to the code they test. Keep pure logic, such as layout math, slice calculation and formatting, in functions that don't need an `egui::Ui`, so it can be tested directly.

Scanner tests build real directory trees in a temporary directory with `tempfile`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn counts_files_directly_in_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.bin"), [0u8; 100]).unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("b.bin"), [0u8; 30]).unwrap();

        let result = scan_blocking(dir.path(), ScanOptions::default()).unwrap();

        assert_eq!(result.total_size, 130);
        assert_eq!(result.total_files, 2);
    }
}
```

A bug fix should come with a test that fails without the fix.

### Integration Tests, Property Tests and Benchmarks
None exist yet. Integration tests go in `tests/` and can only use the library crate's public API (for example `diskusage::Scanner`). If you add property tests or benchmarks, add `proptest` or `criterion` as dev-dependencies in the same pull request, and for criterion a `[[bench]]` entry with `harness = false`.

## Pull Request Process

### Before Creating a Pull Request

1. **Update your branch**:
   ```bash
   git checkout main
   git pull upstream main
   git checkout your-feature-branch
   git rebase main
   ```

2. **Run the full test suite**:
   ```bash
   cargo test
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   ```

3. **Update documentation** if necessary. User-visible changes go in the README and USER_GUIDE.md; type and module changes go in API.md and ARCHITECTURE.md

4. **Add tests** for new functionality

### Pull Request Guidelines

#### Title Format
```
<type>(<scope>): <description>

Examples:
feat(scanner): Add parallel directory scanning
fix(ui): Fix treemap rendering on high DPI displays
docs(api): Document the Scanner polling protocol
test(scanner): Add integration tests for permission handling
```

#### Description Template
```markdown
## Summary
Brief description of the changes made.

## Changes Made
- List of specific changes
- New features added
- Bug fixes
- Performance improvements

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Performance benchmarks run (if applicable)

## Breaking Changes
List any breaking changes and migration instructions.

## Screenshots
Include screenshots for UI changes.

## Related Issues
Closes #123
Related to #456
```

#### Review Process

1. **Automated checks** must pass (CI/CD pipeline)
2. **Code review** by at least one maintainer
3. **Testing** on multiple platforms if applicable
4. **Documentation** review for user-facing changes

### Merge Requirements

- [ ] All tests pass
- [ ] Code follows style guidelines
- [ ] Documentation is updated
- [ ] No merge conflicts
- [ ] Approved by maintainer

## Issue Reporting

### Bug Reports

Use the bug report template:

```markdown
**Bug Description**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected Behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Environment:**
- OS: [e.g. Windows 10, Ubuntu 20.04]
- Rust version: [e.g. 1.70.0]
- Application version: [e.g. 1.0.0]

**Additional Context**
Add any other context about the problem here.
```

### Feature Requests

Use the feature request template:

```markdown
**Feature Description**
A clear and concise description of what you want to happen.

**Use Case**
Describe the use case that would benefit from this feature.

**Proposed Solution**
Describe the solution you'd like to see implemented.

**Alternative Solutions**
Describe any alternative solutions or features you've considered.

**Additional Context**
Add any other context, mockups, or screenshots about the feature request here.
```

## Documentation

### Code Documentation

- **Public APIs**: Must have comprehensive documentation
- **Complex algorithms**: Should include implementation notes
- **Configuration options**: Must be documented with examples

### User Documentation

- **README**: Keep up-to-date with current features
- **User Guide**: Update when UI changes
- **Architecture**: Update when system design changes

### Documentation Standards

```rust
/// Brief one-line description.
///
/// Longer description explaining the purpose and behavior.
/// Can span multiple lines and include examples.
///
/// # Arguments
/// * `param1` - Description of parameter
/// * `param2` - Description of parameter
///
/// # Returns
/// Description of return value
///
/// # Errors
/// Description of error conditions
///
/// # Examples
/// ```
/// let result = function(param1, param2);
/// assert_eq!(result, expected_value);
/// ```
///
/// # Safety
/// (For unsafe functions) Description of safety requirements
///
/// # Panics
/// Description of panic conditions
pub fn function(param1: Type1, param2: Type2) -> ReturnType {
    // Implementation
}
```

## Performance Considerations

### Guidelines for Performance-Critical Code

1. **Profile before optimizing**:
   ```bash
   cargo build --release
   perf record target/release/diskusage
   perf report
   ```

2. **Use appropriate data structures**:
   - `Vec` for sequential access
   - `HashMap` for key-value lookup
   - `BTreeMap` for sorted access

3. **Minimize allocations**:
   - Reuse buffers when possible
   - Use `String::with_capacity()` for known sizes
   - Consider using `Cow<str>` for string data

4. **Background work**:
   - Never block in `update`; move slow work to a thread and poll a channel
   - Anything that runs per frame must be cheap: borrow the scan tree, don't clone it, and avoid walking the whole tree

### Memory Usage Guidelines

1. **Monitor memory usage**: the whole scan tree is held in memory, one `DirectoryNode` per directory. Before and after a change that touches the data model, compare the process's memory after scanning a large directory (e.g. your home directory) in a release build

2. **Use appropriate collection sizes**:
   ```rust
   // Pre-allocate when size is known
   let mut items = Vec::with_capacity(expected_size);
   
   // Use iterators to avoid intermediate collections
   let result: Vec<_> = items.iter()
       .filter(|item| item.is_valid())
       .map(|item| item.process())
       .collect();
   ```

3. **Cleanup resources**:
   ```rust
   impl Drop for Resource {
       fn drop(&mut self) {
           // Cleanup code
       }
   }
   ```

## Platform-Specific Considerations

### Windows Development

- Test with different Windows versions (10, 11)
- Handle long path names (>260 characters)
- Consider Windows-specific file attributes
- Test with different file systems (NTFS, FAT32)

### Linux Development

- Test with different distributions (Ubuntu, Fedora, Arch)
- Handle different file systems (ext4, xfs, btrfs)
- Consider permission models and ACLs
- Test with different desktop environments

### Cross-Platform Code

```rust
#[cfg(windows)]
fn platform_specific_function() {
    // Windows implementation
}

#[cfg(unix)]
fn platform_specific_function() {
    // Linux/Unix implementation
}

```

`std` hides most differences: `std::fs`, `Path` and `PathBuf` work the same everywhere. Remember that Windows paths are case-insensitive, that symlinks and junctions are not followed during scans, and that `build.rs` passes MSVC-only linker flags on Windows (see [BUILD_CONFIG.md](BUILD_CONFIG.md)).

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

### Release Checklist

- [ ] All tests pass on all platforms
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (create it with the first release)
- [ ] Version numbers are bumped
- [ ] Release notes are prepared
- [ ] Binaries are built and tested
- [ ] GitHub release is created

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Discord** (if applicable): Real-time chat

### Recognition

We recognize contributors through:
- GitHub contributor graphs
- Release notes acknowledgments
- Hall of fame in documentation

## Getting Help

If you need help with contributing:

1. Check existing issues and documentation
2. Create a discussion thread for questions
3. Reach out to maintainers directly for urgent issues

## License

By contributing to DiskUsage, you agree that your contributions will be licensed under the same license as the project (MIT License).

---

Thank you for contributing to DiskUsage! Your efforts help make this project better for everyone.