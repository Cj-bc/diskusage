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

We use `rustfmt` for code formatting. Configure your editor to format on save:

```toml
# .rustfmt.toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
```

### Linting

Use `clippy` for linting:
```bash
cargo clippy -- -D warnings
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

// Use custom error types
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

#### Documentation
```rust
/// Scans a directory and returns a tree structure representing its contents.
///
/// # Arguments
/// * `path` - The path to the directory to scan
/// * `options` - Scanning options (depth limit, filters, etc.)
///
/// # Returns
/// A `DirectoryNode` containing the directory structure and size information.
///
/// # Errors
/// Returns `ScanError` if the directory cannot be accessed or read.
///
/// # Examples
/// ```
/// use diskusage::Scanner;
/// 
/// let scanner = Scanner::new();
/// let result = scanner.scan_directory("/home/user")?;
/// println!("Total size: {}", result.size);
/// ```
pub fn scan_directory(path: &Path, options: &ScanOptions) -> Result<DirectoryNode, ScanError> {
    // Implementation
}
```

#### Memory Management
```rust
// Use Arc for shared data
use std::sync::Arc;

struct AppState {
    current_data: Arc<DirectoryNode>,
    cache: Arc<Mutex<Cache>>,
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

#### Parallel Processing
```rust
use rayon::prelude::*;

// Use parallel iterators for CPU-intensive work
let results: Vec<_> = items
    .par_iter()
    .map(|item| expensive_computation(item))
    .collect();
```

## Testing Guidelines

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_directory_scanning() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        let scanner = Scanner::new();
        let result = scanner.scan_directory(temp_dir.path()).unwrap();
        
        assert_eq!(result.file_count, 1);
        assert!(result.size > 0);
    }
    
    #[test]
    fn test_error_handling() {
        let scanner = Scanner::new();
        let result = scanner.scan_directory(Path::new("/nonexistent"));
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError::PathNotFound { .. } => {}, // Expected
            _ => panic!("Wrong error type"),
        }
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use diskusage::*;
use std::process::Command;

#[test]
fn test_full_workflow() {
    // Test complete application workflow
    let app = App::new();
    let result = app.scan_and_visualize("/tmp");
    assert!(result.is_ok());
}
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_size_calculation_properties(sizes in prop::collection::vec(0u64..1000000, 0..100)) {
        let total = sizes.iter().sum::<u64>();
        let calculated = calculate_directory_size(&sizes);
        
        prop_assert_eq!(total, calculated);
    }
}
```

### Benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_directory_scan(c: &mut Criterion) {
    let test_dir = setup_test_directory();
    
    c.bench_function("directory_scan", |b| {
        b.iter(|| {
            let scanner = Scanner::new();
            scanner.scan_directory(black_box(&test_dir))
        })
    });
}

criterion_group!(benches, bench_directory_scan);
criterion_main!(benches);
```

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
   cargo clippy
   cargo fmt -- --check
   ```

3. **Update documentation** if necessary

4. **Add tests** for new functionality

### Pull Request Guidelines

#### Title Format
```
<type>(<scope>): <description>

Examples:
feat(scanner): Add parallel directory scanning
fix(ui): Fix treemap rendering on high DPI displays
docs(api): Update API documentation for new cache system
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

4. **Parallel processing**:
   - Use `rayon` for CPU-intensive tasks
   - Use `tokio` for I/O-bound operations
   - Be mindful of thread pool sizes

### Memory Usage Guidelines

1. **Monitor memory usage**:
   ```rust
   #[cfg(feature = "memory-profiling")]
   fn track_memory_usage() {
       // Memory tracking code
   }
   ```

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

// Use feature flags for optional platform features
#[cfg(feature = "windows-registry")]
fn use_registry() {
    // Windows registry code
}
```

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

### Release Checklist

- [ ] All tests pass on all platforms
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated
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