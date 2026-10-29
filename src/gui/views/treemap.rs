use eframe::egui::{self, Color32, Pos2, Rect, Sense, Vec2};
use std::path::PathBuf;
use std::sync::Arc;

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
    pub scan_result: Arc<ScanResult>,
    pub current_path: PathBuf,
    pub zoom_level: f32,
    pub pan_offset: Vec2,
    pub selected_path: Option<PathBuf>,
    pub hovered_path: Option<PathBuf>,
}

#[derive(Default)]
pub struct TreeMapResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
    pub zoom_changed: bool,
    pub pan_changed: bool,
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
        let (rect, interact_response) =
            ui.allocate_exact_size(available_rect.size(), Sense::click_and_drag());

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
        data.hovered_path = None;
        if let Some(pos) = interact_response.hover_pos() {
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

        // Handle scrolling for zoom, but only while the pointer is over the map
        let scroll_delta = ui.input(|i| i.scroll_delta);
        if interact_response.hovered() && scroll_delta.y != 0.0 {
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

            if ui
                .add_enabled(data.can_navigate_up(), egui::Button::new("⬆ Up"))
                .clicked()
            {
                data.navigate_up();
            }

            if ui.button("Reset View").clicked() {
                data.reset_view();
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

        let current_node = data.current_node();

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
        let mut visible_children: Vec<_> = node
            .children
            .iter()
            .filter(|child| child.size > 0)
            .collect();

        visible_children.sort_by_key(|child| std::cmp::Reverse(child.size));

        if visible_children.is_empty() {
            return;
        }

        // Calculate layout using squarified treemap algorithm
        let child_rects = squarified_layout(&visible_children, bounds, node.size);

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
        ui.painter()
            .rect_filled(treemap_rect.rect, egui::Rounding::same(0.0), color);

        // Draw border
        if self.border_width > 0.0 {
            ui.painter().rect_stroke(
                treemap_rect.rect,
                egui::Rounding::same(0.0),
                egui::Stroke::new(self.border_width, Color32::WHITE),
            );
        }

        // Draw label if rectangle is large enough
        if self.show_labels
            && treemap_rect.rect.width() > self.label_threshold
            && treemap_rect.rect.height() > self.label_threshold
        {
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
        rectangles
            .iter()
            .filter(|rect| rect.rect.contains(pos))
            .min_by(|a, b| a.rect.area().total_cmp(&b.rect.area()))
            .map(|rect| rect.path.clone())
    }

    fn get_color_for_depth(&self, depth: usize) -> Color32 {
        let hue = (depth as f32 * 60.0) % 360.0;
        let saturation = 0.6 + (depth as f32 * 0.1) % 0.3;
        let value = 0.8 - (depth as f32 * 0.05) % 0.3;

        hsv_to_rgb(hue, saturation, value)
    }
}

impl TreeMapData {
    pub fn new(scan_result: Arc<ScanResult>) -> Self {
        Self {
            current_path: scan_result.root.path.clone(),
            scan_result,
            zoom_level: 1.0,
            pan_offset: Vec2::ZERO,
            selected_path: None,
            hovered_path: None,
        }
    }

    /// The directory currently shown, falling back to the scan root.
    pub fn current_node(&self) -> &DirectoryNode {
        self.scan_result
            .root
            .find_by_path(&self.current_path)
            .unwrap_or(&self.scan_result.root)
    }

    /// Shows `path` if it is a directory with something to show inside.
    pub fn navigate_to(&mut self, path: PathBuf) {
        let has_children = self
            .scan_result
            .root
            .find_by_path(&path)
            .is_some_and(|node| !node.children.is_empty());
        if !has_children {
            return;
        }

        self.current_path = path;
        self.reset_view();
        self.selected_path = None;
        self.hovered_path = None;
    }

    pub fn can_navigate_up(&self) -> bool {
        self.current_path != self.scan_result.root.path
    }

    pub fn navigate_up(&mut self) {
        if !self.can_navigate_up() {
            return;
        }
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.reset_view();
            self.selected_path = None;
            self.hovered_path = None;
        }
    }

    pub fn reset_view(&mut self) {
        self.zoom_level = 1.0;
        self.pan_offset = Vec2::ZERO;
    }
}

/// Squarified treemap layout (Bruls, Huizing & van Wijk). `total_size` is
/// what the whole of `bounds` represents; it may exceed the children's sum
/// (files directly in the directory), and that share of the area is left
/// empty. `children` must be sorted by descending size.
fn squarified_layout(children: &[&DirectoryNode], bounds: Rect, total_size: u64) -> Vec<Rect> {
    let mut result = Vec::with_capacity(children.len());
    let mut remaining_bounds = bounds;
    let mut remaining_size = total_size;
    let mut remaining_children = children;

    while !remaining_children.is_empty() && remaining_size > 0 {
        let vertical = remaining_bounds.width() > remaining_bounds.height();

        // Grow the row while doing so makes its worst aspect ratio better.
        let mut row_len = 1;
        let mut best_ratio = worst_aspect_ratio(
            &remaining_children[..1],
            remaining_bounds,
            remaining_size,
            vertical,
        );
        while row_len < remaining_children.len() {
            let ratio = worst_aspect_ratio(
                &remaining_children[..=row_len],
                remaining_bounds,
                remaining_size,
                vertical,
            );
            if ratio >= best_ratio {
                break;
            }
            best_ratio = ratio;
            row_len += 1;
        }

        let row = &remaining_children[..row_len];
        let (row_rects, rest) = layout_row(row, remaining_bounds, remaining_size, vertical);
        result.extend(row_rects);

        // Later rows share what is left, so measure them against what is left.
        remaining_bounds = rest;
        remaining_size = remaining_size.saturating_sub(row.iter().map(|node| node.size).sum());
        remaining_children = &remaining_children[row_len..];
    }

    result
}

/// Worst aspect ratio among the rectangles `row` would get if laid out as
/// one strip of `bounds`, where `bounds` represents `bounds_size` bytes.
fn worst_aspect_ratio(
    row: &[&DirectoryNode],
    bounds: Rect,
    bounds_size: u64,
    vertical: bool,
) -> f32 {
    let row_size: u64 = row.iter().map(|node| node.size).sum();
    let row_area = (row_size as f32 / bounds_size as f32) * bounds.area();
    let side_length = if vertical {
        row_area / bounds.height()
    } else {
        row_area / bounds.width()
    };

    row.iter()
        .map(|node| {
            let node_area = (node.size as f32 / row_size as f32) * row_area;
            let other_side = node_area / side_length;
            side_length.max(other_side) / side_length.min(other_side)
        })
        .fold(0.0, f32::max)
}

/// Lays `row` out as one strip along the short side of `bounds` and returns
/// the strip's rectangles together with the bounds left over.
fn layout_row(
    row: &[&DirectoryNode],
    bounds: Rect,
    bounds_size: u64,
    vertical: bool,
) -> (Vec<Rect>, Rect) {
    let row_size: u64 = row.iter().map(|node| node.size).sum();
    let row_ratio = row_size as f32 / bounds_size as f32;
    let mut offset = 0.0f32;

    if vertical {
        let row_width = bounds.width() * row_ratio;
        let rects = row
            .iter()
            .map(|node| {
                let node_height = bounds.height() * (node.size as f32 / row_size as f32);
                let rect = Rect::from_min_size(
                    Pos2::new(bounds.min.x, bounds.min.y + offset),
                    Vec2::new(row_width, node_height),
                );
                offset += node_height;
                rect
            })
            .collect();
        let rest = Rect::from_min_max(
            Pos2::new(bounds.min.x + row_width, bounds.min.y),
            bounds.max,
        );
        (rects, rest)
    } else {
        let row_height = bounds.height() * row_ratio;
        let rects = row
            .iter()
            .map(|node| {
                let node_width = bounds.width() * (node.size as f32 / row_size as f32);
                let rect = Rect::from_min_size(
                    Pos2::new(bounds.min.x + offset, bounds.min.y),
                    Vec2::new(node_width, row_height),
                );
                offset += node_width;
                rect
            })
            .collect();
        let rest = Rect::from_min_max(
            Pos2::new(bounds.min.x, bounds.min.y + row_height),
            bounds.max,
        );
        (rects, rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, size: u64) -> DirectoryNode {
        let mut node = DirectoryNode::new(PathBuf::from(name));
        node.size = size;
        node
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.01,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn areas_are_proportional_to_sizes() {
        let nodes = [node("a", 50), node("b", 30), node("c", 15), node("d", 5)];
        let children: Vec<&DirectoryNode> = nodes.iter().collect();
        let bounds = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 100.0));

        let rects = squarified_layout(&children, bounds, 100);

        assert_eq!(rects.len(), nodes.len());
        for (node, rect) in nodes.iter().zip(&rects) {
            assert_close(rect.area(), bounds.area() * node.size as f32 / 100.0);
            assert!(
                bounds.expand(0.01).contains_rect(*rect),
                "{rect:?} escapes bounds"
            );
        }
    }

    #[test]
    fn leaves_room_for_files_not_in_children() {
        let nodes = [node("a", 30), node("b", 20)];
        let children: Vec<&DirectoryNode> = nodes.iter().collect();
        let bounds = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0));

        let rects = squarified_layout(&children, bounds, 100);

        let covered: f32 = rects.iter().map(|rect| rect.area()).sum();
        assert_close(covered, bounds.area() * 0.5);
    }
}
