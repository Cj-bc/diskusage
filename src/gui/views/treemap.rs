use eframe::egui::{self, Color32, Pos2, Rect, Sense, Vec2};
use std::path::PathBuf;

use crate::data::{DirectoryNode, ScanResult};
use crate::utils::{format_size, hsv_to_rgb};

#[derive(Clone)]
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

pub struct TreeMapData {
    pub scan_result: ScanResult,
    pub current_path: PathBuf,
    pub zoom_level: f32,
    pub pan_offset: Vec2,
    pub selected_path: Option<PathBuf>,
    pub hovered_path: Option<PathBuf>,
}

pub struct TreeMapResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
    pub zoom_changed: bool,
    pub pan_changed: bool,
}

impl Default for TreeMapResponse {
    fn default() -> Self {
        Self {
            clicked: None,
            double_clicked: None,
            right_clicked: None,
            hovered: None,
            zoom_changed: false,
            pan_changed: false,
        }
    }
}

pub struct TreeMapRenderer {
    pub min_rect_size: f32,
    pub border_width: f32,
    pub label_threshold: f32,
    pub show_labels: bool,
    pub show_sizes: bool,
}

impl Default for TreeMapRenderer {
    fn default() -> Self {
        Self {
            min_rect_size: 4.0,
            border_width: 1.0,
            label_threshold: 30.0,
            show_labels: true,
            show_sizes: true,
        }
    }
}

impl TreeMapRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ui: &mut egui::Ui, data: &mut TreeMapData) -> TreeMapResponse {
        let mut response = TreeMapResponse::default();
        
        // Header with controls
        self.render_header(ui, data);
        
        ui.separator();

        // Main treemap area
        let available_rect = ui.available_rect_before_wrap();
        let (rect, interact_response) = ui.allocate_exact_size(
            available_rect.size(),
            Sense::click_and_drag(),
        );

        // Handle zoom and pan
        if interact_response.dragged() {
            data.pan_offset += interact_response.drag_delta();
            response.pan_changed = true;
        }

        // Calculate treemap layout
        let treemap_rects = self.calculate_layout(data, rect);

        // Render rectangles
        for treemap_rect in &treemap_rects {
            self.render_rectangle(ui, treemap_rect, data);
        }

        // Handle interactions
        if let Some(pos) = interact_response.interact_pointer_pos() {
            if let Some(path) = self.get_path_at_position(pos, &treemap_rects) {
                data.hovered_path = Some(path.clone());
                response.hovered = Some(path.clone());

                if interact_response.clicked() {
                    data.selected_path = Some(path.clone());
                    response.clicked = Some(path.clone());
                }

                if interact_response.double_clicked() {
                    response.double_clicked = Some(path.clone());
                }

                if interact_response.secondary_clicked() {
                    response.right_clicked = Some(path);
                }
            }
        }

        // Handle scrolling for zoom
        let scroll_delta = ui.input(|i| i.scroll_delta);
        if scroll_delta.y != 0.0 {
            let zoom_factor = 1.0 + scroll_delta.y * 0.001;
            data.zoom_level = (data.zoom_level * zoom_factor).clamp(0.1, 10.0);
            response.zoom_changed = true;
        }

        response
    }

    fn render_header(&self, ui: &mut egui::Ui, data: &mut TreeMapData) {
        ui.horizontal(|ui| {
            ui.label(format!("Path: {}", data.current_path.display()));
            
            ui.separator();
            
            if ui.button("Reset View").clicked() {
                data.zoom_level = 1.0;
                data.pan_offset = Vec2::ZERO;
            }

            ui.separator();

            ui.label(format!("Zoom: {:.1}x", data.zoom_level));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Total: {} ({} items)",
                    format_size(data.scan_result.total_size),
                    data.scan_result.total_files + data.scan_result.total_dirs
                ));
            });
        });
    }

    fn calculate_layout(&self, data: &TreeMapData, bounds: Rect) -> Vec<TreeMapRect> {
        let mut rectangles = Vec::new();
        
        // Apply zoom and pan transformations
        let transformed_bounds = Rect::from_min_size(
            bounds.min + data.pan_offset,
            bounds.size() * data.zoom_level,
        );

        // Find the current directory to display
        let current_node = self.find_node_by_path(&data.scan_result.root, &data.current_path)
            .unwrap_or(&data.scan_result.root);

        self.layout_recursive(
            current_node,
            transformed_bounds,
            current_node.size,
            0,
            &mut rectangles,
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
    ) {
        if bounds.width() < self.min_rect_size || bounds.height() < self.min_rect_size {
            return;
        }

        if node.children.is_empty() {
            // Leaf node
            rectangles.push(TreeMapRect {
                rect: bounds,
                path: node.path.clone(),
                size: node.size,
                percentage: if total_size > 0 { 
                    node.size as f32 / total_size as f32 * 100.0 
                } else { 
                    0.0 
                },
                color: self.get_color_for_depth(depth),
                label: node.name.clone(),
                depth,
                is_leaf: true,
            });
            return;
        }

        // Filter and sort children by size
        let mut visible_children: Vec<_> = node.children.iter()
            .filter(|child| child.size > 0)
            .collect();
        
        visible_children.sort_by(|a, b| b.size.cmp(&a.size));

        if visible_children.is_empty() {
            return;
        }

        // Calculate layout using squarified treemap algorithm
        let child_rects = self.squarified_layout(&visible_children, bounds, node.size);

        for (child, rect) in visible_children.iter().zip(child_rects.iter()) {
            if rect.width() >= self.min_rect_size && rect.height() >= self.min_rect_size {
                rectangles.push(TreeMapRect {
                    rect: *rect,
                    path: child.path.clone(),
                    size: child.size,
                    percentage: if total_size > 0 { 
                        child.size as f32 / total_size as f32 * 100.0 
                    } else { 
                        0.0 
                    },
                    color: self.get_color_for_depth(depth + 1),
                    label: child.name.clone(),
                    depth: depth + 1,
                    is_leaf: child.children.is_empty(),
                });

                // Recursively layout children if they're large enough
                if rect.width() > 50.0 && rect.height() > 50.0 && !child.children.is_empty() {
                    self.layout_recursive(child, *rect, total_size, depth + 1, rectangles);
                }
            }
        }
    }

    fn squarified_layout(
        &self,
        children: &[&DirectoryNode],
        bounds: Rect,
        total_size: u64,
    ) -> Vec<Rect> {
        if children.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut remaining_bounds = bounds;
        let mut remaining_children = children;

        while !remaining_children.is_empty() {
            let vertical = remaining_bounds.width() > remaining_bounds.height();
            
            // Find the best row
            let mut best_row = vec![remaining_children[0]];
            let mut best_ratio = f32::INFINITY;
            
            for i in 1..=remaining_children.len() {
                let row = &remaining_children[0..i];
                let ratio = self.worst_aspect_ratio(row, remaining_bounds, total_size, vertical);
                
                if ratio < best_ratio {
                    best_ratio = ratio;
                    best_row = row.to_vec();
                } else {
                    break;
                }
            }

            // Layout the best row
            let (row_rects, new_remaining_bounds) = self.layout_row(
                &best_row,
                remaining_bounds,
                total_size,
                vertical,
            );
            
            result.extend(row_rects);
            remaining_bounds = new_remaining_bounds;
            remaining_children = &remaining_children[best_row.len()..];
        }

        result
    }

    fn worst_aspect_ratio(
        &self,
        row: &[&DirectoryNode],
        bounds: Rect,
        total_size: u64,
        vertical: bool,
    ) -> f32 {
        let row_size: u64 = row.iter().map(|node| node.size).sum();
        let row_area = (row_size as f32 / total_size as f32) * bounds.width() * bounds.height();
        
        let side_length = if vertical {
            row_area / bounds.height()
        } else {
            row_area / bounds.width()
        };

        let mut worst_ratio = 0.0f32;
        
        for node in row {
            let node_area = (node.size as f32 / row_size as f32) * row_area;
            let other_side = node_area / side_length;
            let ratio = side_length.max(other_side) / side_length.min(other_side);
            worst_ratio = worst_ratio.max(ratio);
        }

        worst_ratio
    }

    fn layout_row(
        &self,
        row: &[&DirectoryNode],
        bounds: Rect,
        total_size: u64,
        vertical: bool,
    ) -> (Vec<Rect>, Rect) {
        let row_size: u64 = row.iter().map(|node| node.size).sum();
        let row_ratio = row_size as f32 / total_size as f32;
        
        let mut rects = Vec::new();
        let mut offset = 0.0f32;

        if vertical {
            // Lay out vertically
            let row_width = bounds.width() * row_ratio;
            
            for node in row {
                let node_ratio = node.size as f32 / row_size as f32;
                let node_height = bounds.height() * node_ratio;
                
                rects.push(Rect::from_min_size(
                    Pos2::new(bounds.min.x, bounds.min.y + offset),
                    Vec2::new(row_width, node_height),
                ));
                
                offset += node_height;
            }

            let remaining_bounds = Rect::from_min_size(
                Pos2::new(bounds.min.x + row_width, bounds.min.y),
                Vec2::new(bounds.width() - row_width, bounds.height()),
            );
            
            (rects, remaining_bounds)
        } else {
            // Lay out horizontally
            let row_height = bounds.height() * row_ratio;
            
            for node in row {
                let node_ratio = node.size as f32 / row_size as f32;
                let node_width = bounds.width() * node_ratio;
                
                rects.push(Rect::from_min_size(
                    Pos2::new(bounds.min.x + offset, bounds.min.y),
                    Vec2::new(node_width, row_height),
                ));
                
                offset += node_width;
            }

            let remaining_bounds = Rect::from_min_size(
                Pos2::new(bounds.min.x, bounds.min.y + row_height),
                Vec2::new(bounds.width(), bounds.height() - row_height),
            );
            
            (rects, remaining_bounds)
        }
    }

    fn render_rectangle(&self, ui: &mut egui::Ui, treemap_rect: &TreeMapRect, data: &TreeMapData) {
        let is_selected = data.selected_path.as_ref() == Some(&treemap_rect.path);
        let is_hovered = data.hovered_path.as_ref() == Some(&treemap_rect.path);

        // Adjust color for selection/hover
        let mut color = treemap_rect.color;
        if is_selected {
            color = Color32::from_rgb(
                (color.r() as f32 * 1.3).min(255.0) as u8,
                (color.g() as f32 * 1.3).min(255.0) as u8,
                (color.b() as f32 * 1.3).min(255.0) as u8,
            );
        } else if is_hovered {
            color = Color32::from_rgb(
                (color.r() as f32 * 1.1).min(255.0) as u8,
                (color.g() as f32 * 1.1).min(255.0) as u8,
                (color.b() as f32 * 1.1).min(255.0) as u8,
            );
        }

        // Draw filled rectangle
        ui.painter().rect_filled(
            treemap_rect.rect,
            egui::Rounding::same(0.0),
            color,
        );

        // Draw border
        if self.border_width > 0.0 {
            ui.painter().rect_stroke(
                treemap_rect.rect,
                egui::Rounding::same(0.0),
                egui::Stroke::new(self.border_width, Color32::WHITE),
            );
        }

        // Draw label if rectangle is large enough
        if self.show_labels && 
           treemap_rect.rect.width() > self.label_threshold && 
           treemap_rect.rect.height() > self.label_threshold {
            
            let text = if self.show_sizes {
                format!("{}\n{}", treemap_rect.label, format_size(treemap_rect.size))
            } else {
                treemap_rect.label.clone()
            };

            ui.painter().text(
                treemap_rect.rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                Color32::BLACK,
            );
        }

        // Show tooltip on hover
        if is_hovered {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("treemap_tooltip"), |ui| {
                ui.label(format!("Path: {}", treemap_rect.path.display()));
                ui.label(format!("Size: {}", format_size(treemap_rect.size)));
                ui.label(format!("Percentage: {:.1}%", treemap_rect.percentage));
            });
        }
    }

    fn get_path_at_position(&self, pos: Pos2, rectangles: &[TreeMapRect]) -> Option<PathBuf> {
        // Find the smallest rectangle that contains the position
        rectangles.iter()
            .filter(|rect| rect.rect.contains(pos))
            .min_by_key(|rect| (rect.rect.width() * rect.rect.height()) as u64)
            .map(|rect| rect.path.clone())
    }

    fn find_node_by_path<'a>(&self, node: &'a DirectoryNode, path: &PathBuf) -> Option<&'a DirectoryNode> {
        if &node.path == path {
            return Some(node);
        }

        for child in &node.children {
            if let Some(found) = self.find_node_by_path(child, path) {
                return Some(found);
            }
        }

        None
    }

    fn get_color_for_depth(&self, depth: usize) -> Color32 {
        let hue = (depth as f32 * 60.0) % 360.0;
        let saturation = 0.6 + (depth as f32 * 0.1) % 0.3;
        let value = 0.8 - (depth as f32 * 0.05) % 0.3;
        
        hsv_to_rgb(hue, saturation, value)
    }
}

impl TreeMapData {
    pub fn new(scan_result: ScanResult) -> Self {
        Self {
            current_path: scan_result.root.path.clone(),
            scan_result,
            zoom_level: 1.0,
            pan_offset: Vec2::ZERO,
            selected_path: None,
            hovered_path: None,
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        self.current_path = path;
        self.zoom_level = 1.0;
        self.pan_offset = Vec2::ZERO;
        self.selected_path = None;
        self.hovered_path = None;
    }

    pub fn reset_view(&mut self) {
        self.zoom_level = 1.0;
        self.pan_offset = Vec2::ZERO;
    }
}