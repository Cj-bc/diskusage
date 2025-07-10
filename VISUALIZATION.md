# Visualization Component Specifications

## Overview

This document specifies the requirements and design for the three main visualization components: TreeMap, TreeView, and PieChart. Each component provides a unique perspective on directory space usage data.

## Common Visualization Interface

### Base Visualization Trait
```rust
pub trait Visualization {
    type Data;
    type Response;
    
    fn render(&mut self, ui: &mut egui::Ui, data: &Self::Data) -> Self::Response;
    fn update(&mut self, data: &Self::Data);
    fn handle_interaction(&mut self, response: &egui::Response, data: &Self::Data);
    fn get_tooltip_text(&self, pos: Pos2, data: &Self::Data) -> Option<String>;
}
```

### Common Data Types
```rust
pub struct InteractionState {
    pub hovered_path: Option<PathBuf>,
    pub selected_path: Option<PathBuf>,
    pub mouse_pos: Pos2,
    pub is_dragging: bool,
    pub drag_start: Option<Pos2>,
}

pub struct ViewState {
    pub zoom_level: f32,
    pub pan_offset: Vec2,
    pub view_bounds: Rect,
    pub animation_progress: f32,
}
```

## TreeMap Visualization

### Purpose
Display directory sizes as proportional rectangles using the squarified treemap algorithm for optimal aspect ratios.

### Component Specification

#### TreeMapRenderer
```rust
pub struct TreeMapRenderer {
    // Configuration
    pub min_rect_size: f32,          // Minimum rectangle size in pixels
    pub border_width: f32,           // Border width between rectangles
    pub label_threshold: f32,        // Minimum size to show labels
    pub color_scheme: ColorScheme,   // Color scheme for visualization
    
    // State
    pub view_state: ViewState,       // Zoom and pan state
    pub interaction_state: InteractionState,
    pub layout_cache: HashMap<PathBuf, Vec<TreeMapRect>>,
    
    // Animation
    pub animation_duration: f32,     // Duration for transitions
    pub easing_function: EasingFunction,
}
```

#### TreeMapData
```rust
pub struct TreeMapData {
    pub root: Arc<DirectoryNode>,
    pub current_path: PathBuf,
    pub display_threshold: u64,      // Minimum size to display
    pub sort_order: SortOrder,
    pub filter: Option<PathFilter>,
}
```

#### TreeMapRect
```rust
pub struct TreeMapRect {
    pub rect: Rect,
    pub path: PathBuf,
    pub size: u64,
    pub percentage: f32,
    pub color: Color32,
    pub label: String,
    pub depth: usize,
    pub is_leaf: bool,
}
```

#### TreeMapResponse
```rust
pub struct TreeMapResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
    pub zoom_changed: bool,
    pub pan_changed: bool,
}
```

### Visual Design

#### Color Schemes
```rust
pub enum ColorScheme {
    DepthBased,    // Colors based on directory depth
    SizeBased,     // Colors based on relative size
    TypeBased,     // Colors based on file type
    Custom(Vec<Color32>),
}

impl ColorScheme {
    pub fn get_color(&self, rect: &TreeMapRect) -> Color32 {
        match self {
            ColorScheme::DepthBased => {
                let hue = (rect.depth as f32 * 30.0) % 360.0;
                hsv_to_rgb(hue, 0.6, 0.8)
            },
            ColorScheme::SizeBased => {
                let intensity = (rect.percentage * 255.0) as u8;
                Color32::from_rgb(intensity, 100, 255 - intensity)
            },
            ColorScheme::TypeBased => {
                // Implement file type-based coloring
                self.get_type_color(&rect.path)
            },
            ColorScheme::Custom(colors) => {
                colors[rect.depth % colors.len()]
            }
        }
    }
}
```

#### Layout Algorithm
```rust
impl TreeMapRenderer {
    pub fn calculate_layout(&self, data: &TreeMapData, bounds: Rect) -> Vec<TreeMapRect> {
        let mut rectangles = Vec::new();
        let total_size = data.root.size;
        
        self.layout_recursive(
            &data.root,
            bounds,
            total_size,
            0,
            &mut rectangles,
            &data.current_path
        );
        
        rectangles
    }
    
    fn layout_recursive(
        &self,
        node: &DirectoryNode,
        bounds: Rect,
        total_size: u64,
        depth: usize,
        rectangles: &mut Vec<TreeMapRect>,
        current_path: &Path,
    ) {
        if bounds.area() < self.min_rect_size {
            return;
        }
        
        let visible_children: Vec<_> = node.children.iter()
            .filter(|child| child.size > 0)
            .collect();
        
        if visible_children.is_empty() {
            // Leaf node
            rectangles.push(TreeMapRect {
                rect: bounds,
                path: node.path.clone(),
                size: node.size,
                percentage: node.size as f32 / total_size as f32,
                color: self.color_scheme.get_color_for_node(node, depth),
                label: node.name.clone(),
                depth,
                is_leaf: true,
            });
            return;
        }
        
        // Apply squarified treemap algorithm
        let child_rectangles = self.squarified_layout(&visible_children, bounds, node.size);
        
        for (child, rect) in visible_children.iter().zip(child_rectangles.iter()) {
            if child.path == *current_path {
                // Recursively layout children of current directory
                self.layout_recursive(child, *rect, total_size, depth + 1, rectangles, current_path);
            } else {
                // Add as single rectangle
                rectangles.push(TreeMapRect {
                    rect: *rect,
                    path: child.path.clone(),
                    size: child.size,
                    percentage: child.size as f32 / total_size as f32,
                    color: self.color_scheme.get_color_for_node(child, depth + 1),
                    label: child.name.clone(),
                    depth: depth + 1,
                    is_leaf: child.children.is_empty(),
                });
            }
        }
    }
}
```

#### Interaction Handling
```rust
impl TreeMapRenderer {
    pub fn handle_interaction(&mut self, response: &egui::Response, data: &TreeMapData) -> TreeMapResponse {
        let mut result = TreeMapResponse::default();
        
        if response.clicked() {
            if let Some(path) = self.get_path_at_position(response.interact_pointer_pos().unwrap()) {
                result.clicked = Some(path);
            }
        }
        
        if response.double_clicked() {
            if let Some(path) = self.get_path_at_position(response.interact_pointer_pos().unwrap()) {
                result.double_clicked = Some(path);
            }
        }
        
        if response.secondary_clicked() {
            if let Some(path) = self.get_path_at_position(response.interact_pointer_pos().unwrap()) {
                result.right_clicked = Some(path);
            }
        }
        
        if response.hovered() {
            if let Some(path) = self.get_path_at_position(response.interact_pointer_pos().unwrap()) {
                result.hovered = Some(path);
                self.interaction_state.hovered_path = Some(path);
            }
        }
        
        // Handle zoom and pan
        if let Some(scroll_delta) = response.scroll_delta {
            self.handle_zoom(scroll_delta.y);
            result.zoom_changed = true;
        }
        
        if response.dragged() {
            self.handle_pan(response.drag_delta());
            result.pan_changed = true;
        }
        
        result
    }
    
    fn handle_zoom(&mut self, delta: f32) {
        let zoom_factor = 1.0 + delta * 0.001;
        self.view_state.zoom_level *= zoom_factor;
        self.view_state.zoom_level = self.view_state.zoom_level.clamp(0.1, 10.0);
    }
    
    fn handle_pan(&mut self, delta: Vec2) {
        self.view_state.pan_offset += delta;
    }
}
```

## TreeView Visualization

### Purpose
Display directory structure as a hierarchical tree with expandable nodes and size information.

### Component Specification

#### TreeViewRenderer
```rust
pub struct TreeViewRenderer {
    // Configuration
    pub indent_size: f32,           // Indentation per level
    pub line_height: f32,           // Height of each line
    pub icon_size: f32,             // Size of expand/collapse icons
    pub show_icons: bool,           // Show file type icons
    pub show_sizes: bool,           // Show size information
    pub show_percentages: bool,     // Show size percentages
    
    // State
    pub expanded_nodes: HashSet<PathBuf>,
    pub selected_node: Option<PathBuf>,
    pub scroll_offset: f32,
    pub visible_range: Range<usize>,
    
    // Styling
    pub font_size: f32,
    pub alternate_row_colors: bool,
    pub highlight_color: Color32,
    pub selection_color: Color32,
}
```

#### TreeViewData
```rust
pub struct TreeViewData {
    pub root: Arc<DirectoryNode>,
    pub sort_criteria: Vec<SortCriteria>,
    pub filter: Option<PathFilter>,
    pub search_query: Option<String>,
    pub show_hidden: bool,
}
```

#### TreeViewItem
```rust
pub struct TreeViewItem {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub percentage: f32,
    pub depth: usize,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub children_count: usize,
    pub icon: Option<String>,
}
```

#### TreeViewResponse
```rust
pub struct TreeViewResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub expanded: Vec<PathBuf>,
    pub collapsed: Vec<PathBuf>,
    pub selection_changed: bool,
}
```

### Visual Design

#### Node Rendering
```rust
impl TreeViewRenderer {
    pub fn render_node(&self, ui: &mut egui::Ui, item: &TreeViewItem, row_index: usize) {
        let row_height = self.line_height;
        let total_width = ui.available_width();
        
        // Alternate row colors
        if self.alternate_row_colors && row_index % 2 == 1 {
            ui.painter().rect_filled(
                Rect::from_min_size(ui.cursor().min, Vec2::new(total_width, row_height)),
                0.0,
                Color32::from_gray(248)
            );
        }
        
        // Selection highlight
        if Some(&item.path) == self.selected_node.as_ref() {
            ui.painter().rect_filled(
                Rect::from_min_size(ui.cursor().min, Vec2::new(total_width, row_height)),
                0.0,
                self.selection_color
            );
        }
        
        ui.horizontal(|ui| {
            // Indentation
            ui.add_space(item.depth as f32 * self.indent_size);
            
            // Expand/collapse icon
            if item.is_directory && item.children_count > 0 {
                let icon = if item.is_expanded { "▼" } else { "▶" };
                if ui.button(icon).clicked() {
                    // Toggle expansion
                }
            } else {
                ui.add_space(self.icon_size);
            }
            
            // File type icon
            if self.show_icons {
                if let Some(icon) = &item.icon {
                    ui.label(icon);
                }
            }
            
            // Name
            ui.label(&item.name);
            
            // Size information
            if self.show_sizes {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.show_percentages {
                        ui.label(format!("{:.1}%", item.percentage * 100.0));
                    }
                    ui.label(format_size(item.size));
                });
            }
        });
    }
}
```

#### Virtualization
```rust
impl TreeViewRenderer {
    pub fn render_virtualized(&mut self, ui: &mut egui::Ui, data: &TreeViewData) -> TreeViewResponse {
        let available_height = ui.available_height();
        let visible_rows = (available_height / self.line_height).ceil() as usize;
        
        let flat_items = self.flatten_tree(data);
        let total_items = flat_items.len();
        
        // Calculate scroll position
        let scroll_area = egui::ScrollArea::vertical()
            .max_height(available_height)
            .show(ui, |ui| {
                ui.allocate_space(Vec2::new(
                    ui.available_width(),
                    total_items as f32 * self.line_height
                ));
                
                // Determine visible range
                let scroll_top = ui.clip_rect().min.y;
                let start_index = ((scroll_top - ui.cursor().min.y) / self.line_height).floor().max(0.0) as usize;
                let end_index = (start_index + visible_rows + 1).min(total_items);
                
                // Render only visible items
                for (i, item) in flat_items[start_index..end_index].iter().enumerate() {
                    let row_index = start_index + i;
                    self.render_node(ui, item, row_index);
                }
            });
        
        TreeViewResponse::default()
    }
    
    fn flatten_tree(&self, data: &TreeViewData) -> Vec<TreeViewItem> {
        let mut items = Vec::new();
        self.flatten_recursive(&data.root, 0, &mut items);
        items
    }
    
    fn flatten_recursive(&self, node: &DirectoryNode, depth: usize, items: &mut Vec<TreeViewItem>) {
        items.push(TreeViewItem {
            path: node.path.clone(),
            name: node.name.clone(),
            size: node.size,
            percentage: 0.0, // Calculate based on parent
            depth,
            is_directory: !node.children.is_empty(),
            is_expanded: self.expanded_nodes.contains(&node.path),
            children_count: node.children.len(),
            icon: self.get_icon_for_node(node),
        });
        
        if self.expanded_nodes.contains(&node.path) {
            for child in &node.children {
                self.flatten_recursive(child, depth + 1, items);
            }
        }
    }
}
```

## PieChart Visualization

### Purpose
Display immediate subdirectories as pie slices with proportional sizes and clear labels.

### Component Specification

#### PieChartRenderer
```rust
pub struct PieChartRenderer {
    // Configuration
    pub center: Pos2,
    pub radius: f32,
    pub inner_radius: f32,        // For donut chart
    pub start_angle: f32,         // Starting angle in radians
    pub show_labels: bool,
    pub show_percentages: bool,
    pub label_distance: f32,      // Distance of labels from center
    
    // State
    pub selected_segment: Option<usize>,
    pub hovered_segment: Option<usize>,
    pub rotation_angle: f32,
    
    // Animation
    pub animation_progress: f32,
    pub animate_on_data_change: bool,
}
```

#### PieChartData
```rust
pub struct PieChartData {
    pub segments: Vec<PieSegment>,
    pub total_size: u64,
    pub center_text: Option<String>,
    pub title: Option<String>,
}
```

#### PieSegment
```rust
pub struct PieSegment {
    pub label: String,
    pub size: u64,
    pub percentage: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub color: Color32,
    pub path: PathBuf,
    pub is_selected: bool,
    pub is_hovered: bool,
}
```

#### PieChartResponse
```rust
pub struct PieChartResponse {
    pub clicked: Option<usize>,
    pub double_clicked: Option<usize>,
    pub right_clicked: Option<usize>,
    pub hovered: Option<usize>,
    pub selection_changed: bool,
}
```

### Visual Design

#### Segment Rendering
```rust
impl PieChartRenderer {
    pub fn render_segment(&self, ui: &mut egui::Ui, segment: &PieSegment, index: usize) {
        let painter = ui.painter();
        let center = self.center;
        let radius = self.radius;
        let inner_radius = self.inner_radius;
        
        // Calculate segment path
        let points = self.calculate_segment_points(segment, center, radius, inner_radius);
        
        // Draw filled segment
        let mut color = segment.color;
        if segment.is_selected {
            color = color.gamma_multiply(1.2);
        }
        if segment.is_hovered {
            color = color.gamma_multiply(1.1);
        }
        
        painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
        
        // Draw segment border
        painter.add(egui::Shape::line_segment(
            [points[0], points[1]],
            egui::Stroke::new(1.0, Color32::WHITE)
        ));
        
        // Draw labels
        if self.show_labels {
            self.render_segment_label(ui, segment, index);
        }
    }
    
    fn calculate_segment_points(&self, segment: &PieSegment, center: Pos2, radius: f32, inner_radius: f32) -> Vec<Pos2> {
        let mut points = Vec::new();
        let angle_step = (segment.end_angle - segment.start_angle) / 20.0;
        
        // Outer arc
        let mut angle = segment.start_angle;
        while angle <= segment.end_angle {
            points.push(Pos2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin()
            ));
            angle += angle_step;
        }
        
        // Inner arc (reverse order)
        angle = segment.end_angle;
        while angle >= segment.start_angle {
            points.push(Pos2::new(
                center.x + inner_radius * angle.cos(),
                center.y + inner_radius * angle.sin()
            ));
            angle -= angle_step;
        }
        
        points
    }
    
    fn render_segment_label(&self, ui: &mut egui::Ui, segment: &PieSegment, index: usize) {
        let center = self.center;
        let mid_angle = (segment.start_angle + segment.end_angle) / 2.0;
        
        let label_pos = Pos2::new(
            center.x + self.label_distance * mid_angle.cos(),
            center.y + self.label_distance * mid_angle.sin()
        );
        
        let text = if self.show_percentages {
            format!("{}\n{:.1}%", segment.label, segment.percentage * 100.0)
        } else {
            segment.label.clone()
        };
        
        ui.painter().text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::default(),
            Color32::BLACK
        );
    }
}
```

#### Interaction Handling
```rust
impl PieChartRenderer {
    pub fn handle_interaction(&mut self, response: &egui::Response, data: &PieChartData) -> PieChartResponse {
        let mut result = PieChartResponse::default();
        
        if let Some(pos) = response.interact_pointer_pos() {
            let segment_index = self.get_segment_at_position(pos, data);
            
            if response.clicked() {
                result.clicked = segment_index;
            }
            
            if response.double_clicked() {
                result.double_clicked = segment_index;
            }
            
            if response.secondary_clicked() {
                result.right_clicked = segment_index;
            }
            
            if response.hovered() {
                if self.hovered_segment != segment_index {
                    self.hovered_segment = segment_index;
                    result.hovered = segment_index;
                }
            }
        }
        
        result
    }
    
    fn get_segment_at_position(&self, pos: Pos2, data: &PieChartData) -> Option<usize> {
        let center = self.center;
        let dx = pos.x - center.x;
        let dy = pos.y - center.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Check if position is within pie chart
        if distance < self.inner_radius || distance > self.radius {
            return None;
        }
        
        // Calculate angle
        let angle = dy.atan2(dx);
        let normalized_angle = if angle < 0.0 { angle + 2.0 * std::f32::consts::PI } else { angle };
        
        // Find segment
        for (index, segment) in data.segments.iter().enumerate() {
            if normalized_angle >= segment.start_angle && normalized_angle <= segment.end_angle {
                return Some(index);
            }
        }
        
        None
    }
}
```

## Common Utilities

### Animation System
```rust
pub struct AnimationManager {
    pub active_animations: HashMap<String, Animation>,
    pub default_duration: f32,
    pub default_easing: EasingFunction,
}

pub struct Animation {
    pub start_time: f32,
    pub duration: f32,
    pub easing: EasingFunction,
    pub start_value: f32,
    pub end_value: f32,
    pub current_value: f32,
}

pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}
```

### Color Management
```rust
pub struct ColorManager {
    pub schemes: HashMap<String, ColorScheme>,
    pub current_scheme: String,
    pub accent_color: Color32,
}

impl ColorManager {
    pub fn generate_colors(&self, count: usize) -> Vec<Color32> {
        let mut colors = Vec::new();
        for i in 0..count {
            let hue = (i as f32 * 360.0 / count as f32) % 360.0;
            colors.push(hsv_to_rgb(hue, 0.7, 0.8));
        }
        colors
    }
}
```

### Measurement and Formatting
```rust
pub struct MeasurementFormatter {
    pub size_format: SizeFormat,
    pub decimal_places: usize,
    pub use_binary_prefixes: bool,
}

impl MeasurementFormatter {
    pub fn format_size(&self, size: u64) -> String {
        match self.size_format {
            SizeFormat::Bytes => format!("{} bytes", size),
            SizeFormat::Auto => self.auto_format(size),
            SizeFormat::Binary => self.binary_format(size),
            SizeFormat::Decimal => self.decimal_format(size),
        }
    }
}
```

## Performance Considerations

### Rendering Optimization
- **Viewport Culling**: Only render visible elements
- **Level of Detail**: Simplified rendering for small elements
- **Batch Rendering**: Group similar drawing operations
- **Dirty Tracking**: Only redraw when data changes

### Memory Management
- **Data Sharing**: Use Arc<T> for shared data
- **Lazy Loading**: Load visualization data on demand
- **Cache Management**: Intelligent caching of computed layouts
- **Memory Pools**: Reuse allocated objects

### Interaction Performance
- **Spatial Indexing**: Use quad-trees for fast hit testing
- **Debouncing**: Limit update frequency during interactions
- **Async Updates**: Non-blocking data processing
- **Progressive Loading**: Show partial results during computation

This specification provides a comprehensive foundation for implementing the three visualization components while maintaining consistency, performance, and extensibility.