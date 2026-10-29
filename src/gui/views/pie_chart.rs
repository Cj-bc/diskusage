use eframe::egui::{self, Color32, Pos2, Rect, Sense};
use std::f32::consts::PI;
use std::path::PathBuf;
use std::sync::Arc;

use crate::data::{DirectoryNode, ScanResult};
use crate::utils::{format_size, hsv_to_rgb};

#[derive(Clone)]
pub struct PieSlice {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub percentage: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub color: Color32,
    pub is_directory: bool,
}

pub struct PieChartData {
    pub scan_result: Arc<ScanResult>,
    pub current_path: PathBuf,
    pub selected_path: Option<PathBuf>,
    pub hovered_path: Option<PathBuf>,
    pub show_labels: bool,
    pub show_percentages: bool,
    pub min_slice_angle: f32,
    pub max_slices: usize,
}

pub struct PieChartResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
}

impl Default for PieChartResponse {
    fn default() -> Self {
        Self {
            clicked: None,
            double_clicked: None,
            right_clicked: None,
            hovered: None,
        }
    }
}

pub struct PieChartRenderer {
    pub center_radius: f32,
    pub show_center_info: bool,
    pub label_distance: f32,
    pub min_label_size: f32,
}

impl Default for PieChartRenderer {
    fn default() -> Self {
        Self {
            center_radius: 30.0,
            show_center_info: true,
            label_distance: 1.3,
            min_label_size: 20.0,
        }
    }
}

impl PieChartRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ui: &mut egui::Ui, data: &mut PieChartData) -> PieChartResponse {
        let mut response = PieChartResponse::default();
        
        // Header with controls
        self.render_header(ui, data);
        
        ui.separator();

        // Main pie chart area
        let available_rect = ui.available_rect_before_wrap();
        let (rect, interact_response) = ui.allocate_exact_size(
            available_rect.size(),
            Sense::click(),
        );

        // Calculate pie chart layout
        let pie_slices = self.calculate_slices(data, rect);
        
        // Draw pie chart
        let center = rect.center();
        let radius = (rect.width().min(rect.height()) / 2.0) - 40.0; // Leave margin for labels
        
        // Draw slices
        for slice in &pie_slices {
            self.render_slice(ui, slice, center, radius, data);
        }

        // Draw center circle with info
        if self.show_center_info {
            self.render_center_info(ui, center, data);
        }

        // Handle interactions
        if let Some(pos) = interact_response.interact_pointer_pos() {
            if let Some(path) = self.get_path_at_position(pos, center, radius, &pie_slices) {
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

        response
    }

    fn render_header(&self, ui: &mut egui::Ui, data: &mut PieChartData) {
        ui.horizontal(|ui| {
            ui.label(format!("Path: {}", data.current_path.display()));
            
            ui.separator();
            
            ui.checkbox(&mut data.show_labels, "Show Labels");
            ui.checkbox(&mut data.show_percentages, "Show Percentages");

            ui.separator();

            ui.label("Max slices:");
            ui.add(egui::Slider::new(&mut data.max_slices, 5..=20));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Total: {} ({} items)",
                    format_size(data.scan_result.total_size),
                    data.scan_result.total_files + data.scan_result.total_dirs
                ));
            });
        });
    }

    fn calculate_slices(&self, data: &PieChartData, _rect: Rect) -> Vec<PieSlice> {
        let mut slices = Vec::new();
        
        // Find the current directory to display
        let current_node = self.find_node_by_path(&data.scan_result.root, &data.current_path)
            .unwrap_or(&data.scan_result.root);

        if current_node.children.is_empty() {
            return slices;
        }

        // Sort children by size
        let mut sorted_children: Vec<_> = current_node.children.iter()
            .filter(|child| child.size > 0)
            .collect();
        sorted_children.sort_by(|a, b| b.size.cmp(&a.size));

        // Take only the largest slices
        let visible_children = if sorted_children.len() > data.max_slices {
            sorted_children.into_iter().take(data.max_slices - 1).collect::<Vec<_>>()
        } else {
            sorted_children
        };

        let total_visible_size: u64 = visible_children.iter().map(|child| child.size).sum();
        let total_size = current_node.size.max(1); // Avoid division by zero

        let mut current_angle = -PI / 2.0; // Start at top

        // Create slices for visible children
        for (index, child) in visible_children.iter().enumerate() {
            let percentage = (child.size as f64 / total_size as f64 * 100.0) as f32;
            let angle_size = (child.size as f64 / total_size as f64 * 2.0 * PI as f64) as f32;
            
            // Skip very small slices
            if angle_size < data.min_slice_angle {
                continue;
            }

            slices.push(PieSlice {
                path: child.path.clone(),
                name: child.name.clone(),
                size: child.size,
                percentage,
                start_angle: current_angle,
                end_angle: current_angle + angle_size,
                color: self.get_color_for_index(index),
                is_directory: !child.children.is_empty(),
            });

            current_angle += angle_size;
        }

        // Add "Others" slice if we have more children than max_slices
        if current_node.children.len() > data.max_slices {
            let others_size = current_node.size - total_visible_size;
            if others_size > 0 {
                let percentage = (others_size as f64 / total_size as f64 * 100.0) as f32;
                let angle_size = (others_size as f64 / total_size as f64 * 2.0 * PI as f64) as f32;
                
                if angle_size >= data.min_slice_angle {
                    slices.push(PieSlice {
                        path: current_node.path.clone(), // Use parent path for "Others"
                        name: "Others".to_string(),
                        size: others_size,
                        percentage,
                        start_angle: current_angle,
                        end_angle: current_angle + angle_size,
                        color: Color32::GRAY,
                        is_directory: true,
                    });
                }
            }
        }

        slices
    }

    fn render_slice(&self, ui: &mut egui::Ui, slice: &PieSlice, center: Pos2, radius: f32, data: &PieChartData) {
        let is_selected = data.selected_path.as_ref() == Some(&slice.path);
        let is_hovered = data.hovered_path.as_ref() == Some(&slice.path);

        // Adjust radius for selection/hover
        let slice_radius = if is_selected {
            radius + 10.0
        } else if is_hovered {
            radius + 5.0
        } else {
            radius
        };

        // Draw the pie slice
        self.draw_pie_slice(ui, center, self.center_radius, slice_radius, slice.start_angle, slice.end_angle, slice.color);

        // Draw label if enabled and slice is large enough
        if data.show_labels && slice.end_angle - slice.start_angle > self.min_label_size * PI / 180.0 {
            self.draw_slice_label(ui, slice, center, slice_radius, data.show_percentages);
        }

        // Show tooltip on hover
        if is_hovered {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("pie_tooltip"), |ui| {
                ui.label(format!("Name: {}", slice.name));
                ui.label(format!("Path: {}", slice.path.display()));
                ui.label(format!("Size: {}", format_size(slice.size)));
                ui.label(format!("Percentage: {:.1}%", slice.percentage));
                ui.label(format!("Type: {}", if slice.is_directory { "Directory" } else { "File" }));
            });
        }
    }

    fn draw_pie_slice(&self, ui: &mut egui::Ui, center: Pos2, inner_radius: f32, outer_radius: f32, start_angle: f32, end_angle: f32, color: Color32) {
        let painter = ui.painter();
        
        // Number of segments for smooth arc
        let segments = ((end_angle - start_angle) * 180.0 / PI * 2.0).max(8.0) as usize;
        let angle_step = (end_angle - start_angle) / segments as f32;
        
        // Generate vertices for the slice
        let mut vertices = Vec::new();
        
        // Inner arc vertices
        for i in 0..=segments {
            let angle = start_angle + i as f32 * angle_step;
            let x = center.x + inner_radius * angle.cos();
            let y = center.y + inner_radius * angle.sin();
            vertices.push(Pos2::new(x, y));
        }
        
        // Outer arc vertices (in reverse order)
        for i in (0..=segments).rev() {
            let angle = start_angle + i as f32 * angle_step;
            let x = center.x + outer_radius * angle.cos();
            let y = center.y + outer_radius * angle.sin();
            vertices.push(Pos2::new(x, y));
        }
        
        // Draw filled polygon
        painter.add(egui::Shape::convex_polygon(
            vertices,
            color,
            egui::Stroke::new(1.0, Color32::WHITE),
        ));
    }

    fn draw_slice_label(&self, ui: &mut egui::Ui, slice: &PieSlice, center: Pos2, radius: f32, show_percentage: bool) {
        let mid_angle = (slice.start_angle + slice.end_angle) / 2.0;
        let label_radius = radius * self.label_distance;
        
        let label_pos = Pos2::new(
            center.x + label_radius * mid_angle.cos(),
            center.y + label_radius * mid_angle.sin(),
        );

        let text = if show_percentage {
            format!("{}\n{:.1}%", slice.name, slice.percentage)
        } else {
            slice.name.clone()
        };

        ui.painter().text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::default(),
            Color32::BLACK,
        );
    }

    fn render_center_info(&self, ui: &mut egui::Ui, center: Pos2, data: &PieChartData) {
        // Draw center circle
        ui.painter().circle_filled(center, self.center_radius, Color32::WHITE);
        ui.painter().circle_stroke(center, self.center_radius, egui::Stroke::new(2.0, Color32::BLACK));

        // Draw center text
        let current_node = self.find_node_by_path(&data.scan_result.root, &data.current_path)
            .unwrap_or(&data.scan_result.root);
        
        let center_text = format!("{}\n{}", 
            if current_node.name.len() > 10 { 
                format!("{}...", &current_node.name[..7])
            } else { 
                current_node.name.clone() 
            },
            format_size(current_node.size)
        );

        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            center_text,
            egui::FontId::proportional(10.0),
            Color32::BLACK,
        );
    }

    fn get_path_at_position(&self, pos: Pos2, center: Pos2, radius: f32, slices: &[PieSlice]) -> Option<PathBuf> {
        let dx = pos.x - center.x;
        let dy = pos.y - center.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Check if position is within the pie chart (but outside center)
        if distance < self.center_radius || distance > radius {
            return None;
        }
        
        // Calculate angle
        let angle = dy.atan2(dx);
        let normalized_angle = if angle < -PI / 2.0 {
            angle + 2.0 * PI
        } else {
            angle
        };
        
        // Find which slice contains this angle
        for slice in slices {
            if normalized_angle >= slice.start_angle && normalized_angle <= slice.end_angle {
                return Some(slice.path.clone());
            }
        }
        
        None
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

    fn get_color_for_index(&self, index: usize) -> Color32 {
        let hue = (index as f32 * 137.5) % 360.0; // Golden angle distribution
        let saturation = 0.7;
        let value = 0.9;
        
        hsv_to_rgb(hue, saturation, value)
    }
}

impl PieChartData {
    pub fn new(scan_result: Arc<ScanResult>) -> Self {
        Self {
            current_path: scan_result.root.path.clone(),
            scan_result,
            selected_path: None,
            hovered_path: None,
            show_labels: true,
            show_percentages: true,
            min_slice_angle: 0.05, // ~3 degrees minimum
            max_slices: 12,
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        self.current_path = path;
        self.selected_path = None;
        self.hovered_path = None;
    }
}