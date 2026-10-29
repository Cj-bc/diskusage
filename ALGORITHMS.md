# Data Structures and Algorithms

> **Status: design specification.** This document was written before the implementation and is kept as a design reference; its code samples are sketches, not the actual code. For what exists, see [ARCHITECTURE.md](ARCHITECTURE.md) and [API.md](API.md).
>
> - **Implemented:** squarified treemap layout (`src/gui/views/treemap.rs`; unlike the sketch below, each row is measured against the size that *remains*, and the total may include files directly in the directory), single-key tree view sorting, pie slices (largest subdirectories clockwise from the top plus one "Other" slice), and a recursive depth-first scan on one background thread.
> - **Not implemented:** breadth-first traversal, multi-level sorting, async (Tokio) scanning, the LRU cache and its invalidation, search and filtering, the node memory pool, and batch processing.

## Core Data Structures

### DirectoryNode Structure
```rust
pub struct DirectoryNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub children: Vec<DirectoryNode>,
    pub file_count: usize,
    pub dir_count: usize,
    pub last_modified: SystemTime,
    pub is_expanded: bool,
}
```

**Properties:**
- **Memory Complexity**: O(n) where n is the number of directories
- **Access Time**: O(1) for direct children, O(d) for depth d
- **Space Efficiency**: Compact representation with minimal overhead

**Operations:**
- **Insert Child**: O(1) amortized
- **Find Child**: O(c) where c is the number of children
- **Calculate Size**: O(n) recursive traversal
- **Sort Children**: O(c log c) where c is children count

### Tree Traversal Algorithms

#### Depth-First Search (DFS)
```rust
fn traverse_depth_first<F>(node: &DirectoryNode, mut visitor: F) 
where F: FnMut(&DirectoryNode) {
    visitor(node);
    for child in &node.children {
        traverse_depth_first(child, &mut visitor);
    }
}
```

**Complexity:**
- **Time**: O(n) where n is total nodes
- **Space**: O(d) where d is maximum depth
- **Use Cases**: Size calculation, serialization, deep analysis

#### Breadth-First Search (BFS)
```rust
fn traverse_breadth_first<F>(root: &DirectoryNode, mut visitor: F) 
where F: FnMut(&DirectoryNode) {
    let mut queue = VecDeque::new();
    queue.push_back(root);
    
    while let Some(node) = queue.pop_front() {
        visitor(node);
        for child in &node.children {
            queue.push_back(child);
        }
    }
}
```

**Complexity:**
- **Time**: O(n) where n is total nodes
- **Space**: O(w) where w is maximum width
- **Use Cases**: Level-order processing, UI rendering

## Visualization Algorithms

### TreeMap Layout Algorithm (Squarified)

The squarified treemap algorithm optimizes aspect ratios of rectangles to improve readability.

```rust
pub fn squarified_treemap(items: &[TreeMapItem], bounds: Rect) -> Vec<Rect> {
    if items.is_empty() {
        return vec![];
    }
    
    let mut sorted_items = items.to_vec();
    sorted_items.sort_by(|a, b| b.size.cmp(&a.size));
    
    let total_size: u64 = sorted_items.iter().map(|item| item.size).sum();
    let mut result = Vec::new();
    let mut remaining_bounds = bounds;
    
    squarify(&sorted_items, &mut result, remaining_bounds, total_size);
    result
}

fn squarify(items: &[TreeMapItem], result: &mut Vec<Rect>, bounds: Rect, total_size: u64) {
    if items.is_empty() {
        return;
    }
    
    let vertical = bounds.width() > bounds.height();
    let mut current_row = Vec::new();
    let mut remaining_items = items;
    
    while !remaining_items.is_empty() {
        let item = &remaining_items[0];
        current_row.push(item.clone());
        
        let current_worst = worst_aspect_ratio(&current_row, bounds, total_size, vertical);
        
        if remaining_items.len() > 1 {
            let next_item = &remaining_items[1];
            let mut next_row = current_row.clone();
            next_row.push(next_item.clone());
            let next_worst = worst_aspect_ratio(&next_row, bounds, total_size, vertical);
            
            if current_worst <= next_worst {
                // Layout current row and continue with remaining items
                let (used_bounds, remaining_bounds) = layout_row(&current_row, bounds, total_size, vertical);
                result.extend(used_bounds);
                
                let remaining_size: u64 = remaining_items[1..].iter().map(|item| item.size).sum();
                squarify(&remaining_items[1..], result, remaining_bounds, remaining_size);
                break;
            }
        }
        
        remaining_items = &remaining_items[1..];
    }
}

fn worst_aspect_ratio(row: &[TreeMapItem], bounds: Rect, total_size: u64, vertical: bool) -> f32 {
    let row_size: u64 = row.iter().map(|item| item.size).sum();
    let row_area = (row_size as f32 / total_size as f32) * bounds.area();
    
    let side_length = if vertical {
        row_area / bounds.height()
    } else {
        row_area / bounds.width()
    };
    
    let mut worst_ratio = 0.0_f32;
    
    for item in row {
        let item_area = (item.size as f32 / row_size as f32) * row_area;
        let other_side = item_area / side_length;
        let ratio = side_length.max(other_side) / side_length.min(other_side);
        worst_ratio = worst_ratio.max(ratio);
    }
    
    worst_ratio
}
```

**Algorithm Properties:**
- **Time Complexity**: O(n log n) due to sorting
- **Space Complexity**: O(n) for result storage
- **Aspect Ratio**: Optimized for better readability
- **Stability**: Consistent layout for same input

### Tree View Sorting

#### Size-Based Sorting
```rust
fn sort_by_size(nodes: &mut [DirectoryNode], ascending: bool) {
    nodes.sort_by(|a, b| {
        let comparison = a.size.cmp(&b.size);
        if ascending { comparison } else { comparison.reverse() }
    });
}
```

#### Name-Based Sorting
```rust
fn sort_by_name(nodes: &mut [DirectoryNode], ascending: bool) {
    nodes.sort_by(|a, b| {
        let comparison = a.name.to_lowercase().cmp(&b.name.to_lowercase());
        if ascending { comparison } else { comparison.reverse() }
    });
}
```

#### Multi-Level Sorting
```rust
fn sort_hierarchical(nodes: &mut [DirectoryNode], criteria: &[SortCriteria]) {
    nodes.sort_by(|a, b| {
        for criterion in criteria {
            let comparison = match criterion {
                SortCriteria::Size(ascending) => {
                    let cmp = a.size.cmp(&b.size);
                    if *ascending { cmp } else { cmp.reverse() }
                },
                SortCriteria::Name(ascending) => {
                    let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                    if *ascending { cmp } else { cmp.reverse() }
                },
                SortCriteria::Modified(ascending) => {
                    let cmp = a.last_modified.cmp(&b.last_modified);
                    if *ascending { cmp } else { cmp.reverse() }
                },
            };
            
            if comparison != Ordering::Equal {
                return comparison;
            }
        }
        Ordering::Equal
    });
}
```

### Pie Chart Calculation

```rust
pub fn calculate_pie_segments(node: &DirectoryNode, center: Pos2, radius: f32) -> Vec<PieSegment> {
    let total_size = node.size;
    let mut segments = Vec::new();
    let mut current_angle = 0.0;
    
    for child in &node.children {
        let percentage = child.size as f32 / total_size as f32;
        let angle = percentage * 2.0 * std::f32::consts::PI;
        
        segments.push(PieSegment {
            label: child.name.clone(),
            size: child.size,
            start_angle: current_angle,
            end_angle: current_angle + angle,
            color: generate_color(segments.len()),
            path: child.path.clone(),
        });
        
        current_angle += angle;
    }
    
    segments
}

fn generate_color(index: usize) -> Color32 {
    // HSV color wheel for distinct colors
    let hue = (index as f32 * 137.5) % 360.0; // Golden angle distribution
    let saturation = 0.7;
    let value = 0.9;
    
    hsv_to_rgb(hue, saturation, value)
}
```

## File System Scanning

### Recursive Directory Traversal

```rust
pub fn scan_directory(path: &Path, options: &ScanOptions) -> Result<DirectoryNode, ScanError> {
    let mut node = DirectoryNode::new(path.to_path_buf());
    
    if let Some(max_depth) = options.max_depth {
        if options.current_depth >= max_depth {
            return Ok(node);
        }
    }
    
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => return Err(ScanError::IoError(path.to_path_buf(), e.to_string())),
    };
    
    for entry in entries {
        let entry = entry.map_err(|e| ScanError::IoError(path.to_path_buf(), e.to_string()))?;
        let entry_path = entry.path();
        
        if entry_path.is_dir() {
            if !options.follow_symlinks && entry_path.is_symlink() {
                continue;
            }
            
            let mut child_options = options.clone();
            child_options.current_depth += 1;
            
            match scan_directory(&entry_path, &child_options) {
                Ok(child_node) => {
                    node.size += child_node.size;
                    node.dir_count += 1 + child_node.dir_count;
                    node.file_count += child_node.file_count;
                    node.children.push(child_node);
                },
                Err(e) => {
                    // Log error but continue scanning
                    eprintln!("Error scanning {}: {}", entry_path.display(), e);
                }
            }
        } else {
            let file_size = entry.metadata()
                .map(|m| m.len())
                .unwrap_or(0);
            
            node.size += file_size;
            node.file_count += 1;
        }
    }
    
    Ok(node)
}
```

### Asynchronous Scanning

```rust
pub async fn scan_directory_async(
    path: &Path,
    options: &ScanOptions,
    progress_callback: Option<&dyn Fn(ScanProgress)>,
) -> Result<DirectoryNode, ScanError> {
    let (tx, rx) = mpsc::channel();
    let path = path.to_path_buf();
    let options = options.clone();
    
    tokio::spawn(async move {
        let result = scan_directory_recursive(&path, &options, &tx).await;
        tx.send(ScanMessage::Complete(result)).unwrap();
    });
    
    let mut node = DirectoryNode::new(path);
    let mut progress = ScanProgress::new();
    
    while let Ok(message) = rx.recv() {
        match message {
            ScanMessage::Progress(p) => {
                progress = p;
                if let Some(callback) = progress_callback {
                    callback(progress);
                }
            },
            ScanMessage::NodeFound(child) => {
                node.children.push(child);
            },
            ScanMessage::Complete(result) => {
                return result;
            },
        }
    }
    
    Ok(node)
}
```

## Caching Algorithms

### LRU Cache Implementation

```rust
pub struct LRUCache<K, V> {
    cache: HashMap<K, (V, usize)>,
    access_order: VecDeque<K>,
    max_size: usize,
    access_counter: usize,
}

impl<K: Clone + Hash + Eq, V> LRUCache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
            access_counter: 0,
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, access_time)) = self.cache.get_mut(key) {
            self.access_counter += 1;
            *access_time = self.access_counter;
            
            // Move to front of access order
            if let Some(pos) = self.access_order.iter().position(|x| x == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_front(key.clone());
            
            Some(value)
        } else {
            None
        }
    }
    
    pub fn put(&mut self, key: K, value: V) {
        if self.cache.len() >= self.max_size && !self.cache.contains_key(&key) {
            // Remove least recently used item
            if let Some(lru_key) = self.access_order.pop_back() {
                self.cache.remove(&lru_key);
            }
        }
        
        self.access_counter += 1;
        self.cache.insert(key.clone(), (value, self.access_counter));
        
        // Update access order
        if let Some(pos) = self.access_order.iter().position(|x| x == &key) {
            self.access_order.remove(pos);
        }
        self.access_order.push_front(key);
    }
}
```

### Cache Invalidation Strategy

```rust
pub fn invalidate_cache_entry(cache: &mut LRUCache<PathBuf, DirectoryNode>, path: &Path) {
    // Remove direct entry
    cache.remove(path);
    
    // Remove all entries that are subdirectories of this path
    let keys_to_remove: Vec<PathBuf> = cache.keys()
        .filter(|key| key.starts_with(path))
        .cloned()
        .collect();
    
    for key in keys_to_remove {
        cache.remove(&key);
    }
}
```

## Search and Filtering

### Pattern Matching

```rust
pub fn matches_pattern(name: &str, pattern: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let name = name.to_lowercase();
    
    if pattern.contains('*') {
        // Wildcard matching
        wildcard_match(&name, &pattern)
    } else if pattern.contains('?') {
        // Single character matching
        single_char_match(&name, &pattern)
    } else {
        // Simple substring matching
        name.contains(&pattern)
    }
}

fn wildcard_match(text: &str, pattern: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    
    wildcard_match_recursive(&text_chars, &pattern_chars, 0, 0)
}

fn wildcard_match_recursive(text: &[char], pattern: &[char], text_idx: usize, pattern_idx: usize) -> bool {
    if pattern_idx >= pattern.len() {
        return text_idx >= text.len();
    }
    
    if pattern[pattern_idx] == '*' {
        // Try matching zero or more characters
        for i in text_idx..=text.len() {
            if wildcard_match_recursive(text, pattern, i, pattern_idx + 1) {
                return true;
            }
        }
        false
    } else if text_idx >= text.len() {
        false
    } else if pattern[pattern_idx] == '?' || pattern[pattern_idx] == text[text_idx] {
        wildcard_match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
    } else {
        false
    }
}
```

### Range Filtering

```rust
pub fn filter_by_size(nodes: &[DirectoryNode], min_size: Option<u64>, max_size: Option<u64>) -> Vec<&DirectoryNode> {
    nodes.iter()
        .filter(|node| {
            if let Some(min) = min_size {
                if node.size < min {
                    return false;
                }
            }
            if let Some(max) = max_size {
                if node.size > max {
                    return false;
                }
            }
            true
        })
        .collect()
}
```

## Performance Optimizations

### Memory Pool for Nodes

```rust
pub struct NodePool {
    nodes: Vec<DirectoryNode>,
    free_indices: Vec<usize>,
    next_index: usize,
}

impl NodePool {
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(initial_capacity),
            free_indices: Vec::new(),
            next_index: 0,
        }
    }
    
    pub fn allocate(&mut self) -> usize {
        if let Some(index) = self.free_indices.pop() {
            index
        } else {
            let index = self.next_index;
            self.next_index += 1;
            
            if index >= self.nodes.len() {
                self.nodes.resize_with(index + 1, Default::default);
            }
            
            index
        }
    }
    
    pub fn deallocate(&mut self, index: usize) {
        if index < self.nodes.len() {
            self.free_indices.push(index);
        }
    }
}
```

### Batch Processing

```rust
pub fn process_nodes_batch<F>(nodes: &[DirectoryNode], batch_size: usize, mut processor: F) 
where F: FnMut(&[DirectoryNode]) {
    for chunk in nodes.chunks(batch_size) {
        processor(chunk);
    }
}
```

## Algorithm Complexity Summary

| Algorithm | Time Complexity | Space Complexity | Use Case |
|-----------|----------------|------------------|----------|
| Directory Scan | O(n) | O(d) | File system traversal |
| TreeMap Layout | O(n log n) | O(n) | Visualization |
| Tree Sorting | O(n log n) | O(1) | UI display |
| Cache Lookup | O(1) | O(k) | Performance optimization |
| Pattern Matching | O(m*n) | O(1) | Search functionality |
| Size Calculation | O(n) | O(d) | Directory analysis |

Where:
- n = number of nodes/files
- d = maximum directory depth
- k = cache size
- m = pattern length