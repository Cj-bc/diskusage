# Performance and Memory Usage Specifications

> **Status: design specification.** These are targets set before the implementation; none of them has been measured, and there is no performance test suite.
>
> - **In place:** scanning runs on a background thread with throttled progress messages; the three views share one `Arc<ScanResult>` and the tree view no longer copies the tree each frame.
> - **Not implemented:** parallel scanning, object pooling and compact node layouts, viewport culling and level of detail, caching, metrics collection, and the benchmark/load-test suites.

## Overview

This document defines the performance targets, memory usage requirements, and optimization strategies for the DiskUsage application to ensure responsive operation across various system configurations.

## Performance Targets

### Response Time Requirements

#### User Interface Responsiveness
- **Frame Rate**: Maintain 60 FPS for smooth animations and interactions
- **Interaction Latency**: < 16ms for immediate UI feedback
- **Startup Time**: < 2 seconds for application launch
- **View Switching**: < 100ms for changing visualization modes

#### File System Operations
- **Small Directories** (< 1,000 files): < 100ms scan time
- **Medium Directories** (1,000 - 10,000 files): < 1 second scan time
- **Large Directories** (10,000 - 100,000 files): < 10 seconds scan time
- **Very Large Directories** (> 100,000 files): Progressive loading with 1-second updates

### Throughput Requirements

#### Scanning Performance
```rust
pub struct ScanPerformanceTargets {
    pub files_per_second: u32,
    pub directories_per_second: u32,
    pub bytes_per_second: u64,
    pub max_concurrent_operations: u32,
}

impl Default for ScanPerformanceTargets {
    fn default() -> Self {
        Self {
            files_per_second: 10_000,      // 10K files/second
            directories_per_second: 1_000,  // 1K directories/second
            bytes_per_second: 100_000_000,  // 100MB/second metadata processing
            max_concurrent_operations: 4,   // 4 concurrent scan threads
        }
    }
}
```

#### Rendering Performance
```rust
pub struct RenderPerformanceTargets {
    pub max_rectangles_per_frame: u32,
    pub max_tree_nodes_per_frame: u32,
    pub max_pie_segments_per_frame: u32,
    pub viewport_culling_threshold: f32,
}

impl Default for RenderPerformanceTargets {
    fn default() -> Self {
        Self {
            max_rectangles_per_frame: 10_000,   // TreeMap rectangles
            max_tree_nodes_per_frame: 1_000,    // TreeView visible nodes
            max_pie_segments_per_frame: 100,    // PieChart segments
            viewport_culling_threshold: 0.5,    // 0.5 pixel minimum size
        }
    }
}
```

## Memory Usage Specifications

### Memory Budgets

#### Base Memory Requirements
```rust
pub struct MemoryBudgets {
    pub base_application: usize,
    pub per_directory_node: usize,
    pub per_file_entry: usize,
    pub ui_rendering: usize,
    pub cache_overhead: usize,
    pub maximum_total: usize,
}

impl Default for MemoryBudgets {
    fn default() -> Self {
        Self {
            base_application: 50 * 1024 * 1024,    // 50MB base
            per_directory_node: 200,               // 200 bytes per directory
            per_file_entry: 100,                   // 100 bytes per file reference
            ui_rendering: 100 * 1024 * 1024,      // 100MB for UI buffers
            cache_overhead: 20,                    // 20% cache overhead
            maximum_total: 1024 * 1024 * 1024,    // 1GB maximum
        }
    }
}
```

#### Memory Scaling Analysis
```rust
pub fn calculate_memory_usage(
    directories: usize,
    files: usize,
    cache_size: usize,
) -> MemoryUsageEstimate {
    let budgets = MemoryBudgets::default();
    
    let directory_memory = directories * budgets.per_directory_node;
    let file_memory = files * budgets.per_file_entry;
    let cache_memory = cache_size * budgets.per_directory_node;
    let cache_overhead = (cache_memory as f32 * 0.2) as usize;
    
    let total_data_memory = directory_memory + file_memory + cache_memory + cache_overhead;
    let total_memory = budgets.base_application + budgets.ui_rendering + total_data_memory;
    
    MemoryUsageEstimate {
        base_memory: budgets.base_application,
        data_memory: total_data_memory,
        ui_memory: budgets.ui_rendering,
        total_memory,
        exceeds_budget: total_memory > budgets.maximum_total,
    }
}
```

### Memory Management Strategy

#### Object Pooling
```rust
pub struct ObjectPool<T> {
    pool: Vec<T>,
    active: Vec<T>,
    factory: Box<dyn Fn() -> T>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    pub fn new(factory: impl Fn() -> T + 'static, max_size: usize) -> Self {
        Self {
            pool: Vec::with_capacity(max_size),
            active: Vec::new(),
            factory: Box::new(factory),
            max_size,
        }
    }
    
    pub fn acquire(&mut self) -> T {
        self.pool.pop().unwrap_or_else(|| (self.factory)())
    }
    
    pub fn release(&mut self, item: T) {
        if self.pool.len() < self.max_size {
            self.pool.push(item);
        }
    }
    
    pub fn clear(&mut self) {
        self.pool.clear();
        self.active.clear();
    }
}
```

#### Memory-Efficient Data Structures
```rust
// Compact directory node representation
#[repr(C)]
pub struct CompactDirectoryNode {
    pub name_offset: u32,      // Offset into string table
    pub name_len: u16,         // Length of name
    pub size: u64,             // Directory size
    pub children_offset: u32,  // Offset into children array
    pub children_count: u16,   // Number of children
    pub flags: u8,             // Packed flags (expanded, hidden, etc.)
}

// String interning for efficient memory usage
pub struct StringInterner {
    strings: Vec<u8>,
    map: HashMap<String, u32>,
}

impl StringInterner {
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.map.get(s) {
            return offset;
        }
        
        let offset = self.strings.len() as u32;
        self.strings.extend_from_slice(s.as_bytes());
        self.strings.push(0); // null terminator
        self.map.insert(s.to_string(), offset);
        offset
    }
    
    pub fn get(&self, offset: u32) -> &str {
        let start = offset as usize;
        let end = self.strings[start..].iter().position(|&b| b == 0).unwrap() + start;
        std::str::from_utf8(&self.strings[start..end]).unwrap()
    }
}
```

## Optimization Strategies

### File System Scanning Optimizations

#### Parallel Scanning
```rust
pub struct ParallelScanner {
    thread_pool: ThreadPool,
    max_concurrent_scans: usize,
    scan_queue: VecDeque<ScanJob>,
    results_channel: mpsc::Receiver<ScanResult>,
}

impl ParallelScanner {
    pub fn scan_directory_parallel(
        &mut self,
        root_path: &Path,
        options: ScanOptions,
    ) -> impl Future<Output = Result<DirectoryNode, ScanError>> {
        let (tx, rx) = mpsc::channel();
        let mut job_counter = Arc::new(AtomicUsize::new(0));
        
        // Initial job
        let initial_job = ScanJob {
            path: root_path.to_path_buf(),
            depth: 0,
            options: options.clone(),
            result_sender: tx.clone(),
            job_counter: job_counter.clone(),
        };
        
        self.submit_job(initial_job);
        
        async move {
            let mut root_node = None;
            let mut completed_jobs = HashMap::new();
            
            while let Ok(result) = rx.recv().await {
                match result {
                    ScanJobResult::DirectoryScanned { path, node } => {
                        if path == root_path {
                            root_node = Some(node);
                        } else {
                            completed_jobs.insert(path, node);
                        }
                    },
                    ScanJobResult::Error { path, error } => {
                        eprintln!("Error scanning {}: {}", path.display(), error);
                    },
                }
                
                // Check if all jobs are complete
                if job_counter.load(Ordering::Relaxed) == 0 {
                    break;
                }
            }
            
            root_node.ok_or(ScanError::Other("No root node found".to_string()))
        }
    }
}
```

#### I/O Optimization
```rust
pub struct IOOptimizer {
    read_ahead_buffer: Vec<u8>,
    batch_size: usize,
    concurrent_reads: usize,
}

impl IOOptimizer {
    pub fn batch_stat_operations(&self, paths: &[PathBuf]) -> Vec<Result<Metadata, io::Error>> {
        // Batch multiple stat operations for better performance
        paths.par_iter()
            .map(|path| std::fs::metadata(path))
            .collect()
    }
    
    pub fn prefetch_directory_entries(&self, path: &Path) -> Result<Vec<DirEntry>, io::Error> {
        // Prefetch directory entries to improve locality
        let mut entries = Vec::new();
        
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            entries.push(entry);
        }
        
        // Sort by inode number for better disk access patterns
        entries.sort_by_key(|entry| {
            entry.metadata()
                .map(|m| m.ino())
                .unwrap_or(0)
        });
        
        Ok(entries)
    }
}
```

### Rendering Optimizations

#### Viewport Culling
```rust
pub struct ViewportCuller {
    viewport: Rect,
    min_size_threshold: f32,
    occlusion_buffer: Vec<Rect>,
}

impl ViewportCuller {
    pub fn cull_rectangles(&self, rectangles: &[TreeMapRect]) -> Vec<&TreeMapRect> {
        rectangles.iter()
            .filter(|rect| self.is_visible(rect))
            .collect()
    }
    
    fn is_visible(&self, rect: &TreeMapRect) -> bool {
        // Check if rectangle intersects with viewport
        if !self.viewport.intersects(rect.rect) {
            return false;
        }
        
        // Check if rectangle is large enough to be visible
        if rect.rect.width() < self.min_size_threshold || 
           rect.rect.height() < self.min_size_threshold {
            return false;
        }
        
        // Check occlusion
        !self.is_occluded(&rect.rect)
    }
    
    fn is_occluded(&self, rect: &Rect) -> bool {
        self.occlusion_buffer.iter()
            .any(|occluder| occluder.contains_rect(*rect))
    }
}
```

#### Level of Detail (LOD)
```rust
pub struct LODRenderer {
    detail_levels: Vec<DetailLevel>,
    distance_thresholds: Vec<f32>,
}

#[derive(Clone)]
pub struct DetailLevel {
    pub show_labels: bool,
    pub show_borders: bool,
    pub min_rect_size: f32,
    pub text_size: f32,
    pub simplify_shapes: bool,
}

impl LODRenderer {
    pub fn get_detail_level(&self, rect: &Rect, zoom_level: f32) -> &DetailLevel {
        let size = rect.width().min(rect.height()) * zoom_level;
        
        for (i, &threshold) in self.distance_thresholds.iter().enumerate() {
            if size > threshold {
                return &self.detail_levels[i];
            }
        }
        
        // Return lowest detail level
        self.detail_levels.last().unwrap()
    }
    
    pub fn render_with_lod(&self, ui: &mut egui::Ui, rect: &TreeMapRect, zoom_level: f32) {
        let detail = self.get_detail_level(&rect.rect, zoom_level);
        
        // Render based on detail level
        if detail.simplify_shapes {
            self.render_simplified_rect(ui, rect);
        } else {
            self.render_detailed_rect(ui, rect);
        }
        
        if detail.show_labels && rect.rect.width() > 50.0 {
            self.render_label(ui, rect, detail.text_size);
        }
    }
}
```

### Cache Optimization

#### Hierarchical Caching
```rust
pub struct HierarchicalCache {
    l1_cache: LRUCache<PathBuf, CompactDirectoryNode>,
    l2_cache: LRUCache<PathBuf, Arc<DirectoryNode>>,
    l3_cache: DiskCache,
    hit_stats: CacheStats,
}

impl HierarchicalCache {
    pub fn get(&mut self, path: &Path) -> Option<Arc<DirectoryNode>> {
        // L1 cache (compact representation)
        if let Some(compact) = self.l1_cache.get(path) {
            self.hit_stats.l1_hits += 1;
            return Some(self.expand_compact_node(compact));
        }
        
        // L2 cache (full representation)
        if let Some(node) = self.l2_cache.get(path) {
            self.hit_stats.l2_hits += 1;
            return Some(node.clone());
        }
        
        // L3 cache (disk)
        if let Some(node) = self.l3_cache.get(path) {
            self.hit_stats.l3_hits += 1;
            self.l2_cache.put(path.to_path_buf(), node.clone());
            return Some(node);
        }
        
        self.hit_stats.misses += 1;
        None
    }
    
    pub fn put(&mut self, path: PathBuf, node: Arc<DirectoryNode>) {
        // Store in L2 cache
        self.l2_cache.put(path.clone(), node.clone());
        
        // Store compact version in L1 cache
        let compact = self.compact_node(&node);
        self.l1_cache.put(path.clone(), compact);
        
        // Async write to L3 cache
        self.l3_cache.put_async(path, node);
    }
}
```

#### Cache Warm-up Strategy
```rust
pub struct CacheWarmupStrategy {
    predictor: AccessPredictor,
    background_loader: BackgroundLoader,
}

impl CacheWarmupStrategy {
    pub fn predict_and_warmup(&mut self, current_path: &Path) {
        let predicted_paths = self.predictor.predict_next_accesses(current_path);
        
        for path in predicted_paths {
            if !self.is_cached(&path) {
                self.background_loader.schedule_load(path);
            }
        }
    }
}

pub struct AccessPredictor {
    access_history: VecDeque<(PathBuf, SystemTime)>,
    pattern_weights: HashMap<String, f32>,
}

impl AccessPredictor {
    pub fn predict_next_accesses(&self, current_path: &Path) -> Vec<PathBuf> {
        let mut predictions = Vec::new();
        
        // Predict parent directory access
        if let Some(parent) = current_path.parent() {
            predictions.push(parent.to_path_buf());
        }
        
        // Predict sibling directory access
        if let Some(parent) = current_path.parent() {
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() && entry.path() != current_path {
                        predictions.push(entry.path());
                    }
                }
            }
        }
        
        // Predict based on historical patterns
        predictions.extend(self.predict_from_history(current_path));
        
        predictions
    }
}
```

## Performance Monitoring

### Metrics Collection
```rust
pub struct PerformanceMetrics {
    pub scan_times: RollingAverage,
    pub render_times: RollingAverage,
    pub memory_usage: MemoryTracker,
    pub cache_hit_rates: CacheMetrics,
    pub file_system_latency: LatencyTracker,
}

impl PerformanceMetrics {
    pub fn record_scan_time(&mut self, duration: Duration, file_count: usize) {
        self.scan_times.add(duration.as_millis() as f64);
        
        // Check if performance target is met
        let files_per_second = file_count as f64 / duration.as_secs_f64();
        if files_per_second < 1000.0 {
            self.report_performance_issue(PerformanceIssue::SlowScan {
                duration,
                file_count,
                files_per_second,
            });
        }
    }
    
    pub fn record_render_time(&mut self, duration: Duration, object_count: usize) {
        self.render_times.add(duration.as_millis() as f64);
        
        // Check frame rate
        let fps = 1.0 / duration.as_secs_f64();
        if fps < 30.0 {
            self.report_performance_issue(PerformanceIssue::LowFrameRate {
                fps,
                object_count,
            });
        }
    }
}
```

### Performance Profiling
```rust
pub struct PerformanceProfiler {
    active_spans: HashMap<String, Instant>,
    completed_spans: Vec<ProfileSpan>,
    enabled: bool,
}

#[derive(Debug)]
pub struct ProfileSpan {
    pub name: String,
    pub duration: Duration,
    pub start_time: Instant,
    pub memory_before: usize,
    pub memory_after: usize,
}

impl PerformanceProfiler {
    pub fn start_span(&mut self, name: &str) {
        if !self.enabled {
            return;
        }
        
        self.active_spans.insert(name.to_string(), Instant::now());
    }
    
    pub fn end_span(&mut self, name: &str) {
        if !self.enabled {
            return;
        }
        
        if let Some(start_time) = self.active_spans.remove(name) {
            let duration = start_time.elapsed();
            
            self.completed_spans.push(ProfileSpan {
                name: name.to_string(),
                duration,
                start_time,
                memory_before: 0, // TODO: Get actual memory usage
                memory_after: 0,
            });
        }
    }
    
    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            spans: self.completed_spans.clone(),
            total_time: self.completed_spans.iter().map(|s| s.duration).sum(),
            memory_peak: self.completed_spans.iter().map(|s| s.memory_after).max().unwrap_or(0),
        }
    }
}
```

## Benchmarking and Testing

### Performance Test Suite
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use test::Bencher;
    
    #[bench]
    fn bench_directory_scan_small(b: &mut Bencher) {
        let test_dir = create_test_directory(100, 10); // 100 files, 10 dirs
        b.iter(|| {
            let scanner = Scanner::new();
            scanner.scan(&test_dir).unwrap()
        });
    }
    
    #[bench]
    fn bench_directory_scan_large(b: &mut Bencher) {
        let test_dir = create_test_directory(10000, 1000); // 10K files, 1K dirs
        b.iter(|| {
            let scanner = Scanner::new();
            scanner.scan(&test_dir).unwrap()
        });
    }
    
    #[bench]
    fn bench_treemap_layout(b: &mut Bencher) {
        let test_data = create_test_treemap_data(1000);
        let renderer = TreeMapRenderer::new();
        b.iter(|| {
            renderer.calculate_layout(&test_data, Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 1000.0)))
        });
    }
    
    #[bench]
    fn bench_cache_operations(b: &mut Bencher) {
        let mut cache = HierarchicalCache::new(1000);
        let test_paths = generate_test_paths(100);
        
        b.iter(|| {
            for path in &test_paths {
                cache.get(path);
            }
        });
    }
}
```

### Load Testing
```rust
pub struct LoadTester {
    test_scenarios: Vec<TestScenario>,
    metrics_collector: MetricsCollector,
}

#[derive(Clone)]
pub struct TestScenario {
    pub name: String,
    pub directory_count: usize,
    pub file_count: usize,
    pub depth: usize,
    pub concurrent_operations: usize,
}

impl LoadTester {
    pub fn run_load_test(&mut self, scenario: &TestScenario) -> LoadTestResult {
        let start_time = Instant::now();
        let initial_memory = self.get_memory_usage();
        
        // Create test directory structure
        let test_dir = self.create_test_structure(scenario);
        
        // Run concurrent scan operations
        let mut handles = Vec::new();
        for _ in 0..scenario.concurrent_operations {
            let test_dir = test_dir.clone();
            let handle = std::thread::spawn(move || {
                let scanner = Scanner::new();
                scanner.scan(&test_dir)
            });
            handles.push(handle);
        }
        
        // Wait for completion and collect results
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        let total_time = start_time.elapsed();
        let peak_memory = self.get_peak_memory_usage();
        
        LoadTestResult {
            scenario: scenario.clone(),
            total_time,
            peak_memory,
            memory_increase: peak_memory - initial_memory,
            success_rate: results.iter().filter(|r| r.is_ok()).count() as f64 / results.len() as f64,
            average_scan_time: total_time / scenario.concurrent_operations as u32,
        }
    }
}
```

## System Requirements

### Minimum System Requirements
- **CPU**: 1.5 GHz dual-core processor
- **Memory**: 2 GB RAM
- **Storage**: 100 MB available space
- **Graphics**: DirectX 11 compatible (Windows) or OpenGL 3.3 (Linux)

### Recommended System Requirements
- **CPU**: 2.5 GHz quad-core processor
- **Memory**: 8 GB RAM
- **Storage**: 500 MB available space (for cache)
- **Graphics**: Dedicated graphics card with 1GB VRAM

### Performance Scaling
```rust
pub struct SystemCapabilities {
    pub cpu_cores: usize,
    pub available_memory: u64,
    pub disk_io_speed: u64,
    pub graphics_memory: u64,
}

impl SystemCapabilities {
    pub fn get_optimal_settings(&self) -> OptimalSettings {
        OptimalSettings {
            scan_threads: self.cpu_cores.min(8),
            cache_size: (self.available_memory / 8).min(256 * 1024 * 1024), // 1/8 of RAM, max 256MB
            render_quality: if self.graphics_memory > 1024 * 1024 * 1024 { 
                RenderQuality::High 
            } else { 
                RenderQuality::Medium 
            },
            prefetch_enabled: self.disk_io_speed > 50 * 1024 * 1024, // 50MB/s
        }
    }
}
```

This comprehensive performance specification ensures that the DiskUsage application will deliver responsive performance across a wide range of system configurations while making efficient use of available resources.