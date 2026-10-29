use eframe::egui::{self, Color32, Pos2, Sense};
use std::f32::consts::{PI, TAU};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::data::{DirectoryNode, ScanResult};
use crate::utils::{format_size, hsv_to_rgb};

/// Space kept between the pie and the edge of its area for labels.
const LABEL_MARGIN: f32 = 40.0;
/// Longest directory name shown in the center circle, in characters.
const CENTER_NAME_MAX_CHARS: usize = 10;

/// What a slice stands for.
#[derive(Debug, Clone, PartialEq)]
pub enum SliceTarget {
    Directory(PathBuf),
    /// Everything without a slice of its own: files directly in the current
    /// directory and subdirectories that were too small or too many.
    Other,
}

#[derive(Debug, Clone)]
pub struct PieSlice {
    pub target: SliceTarget,
    pub name: String,
    pub size: u64,
    pub percentage: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub color: Color32,
}

impl PieSlice {
    /// The directory this slice represents, if it represents a single one.
    pub fn path(&self) -> Option<&Path> {
        match &self.target {
            SliceTarget::Directory(path) => Some(path),
            SliceTarget::Other => None,
        }
    }

    fn contains_angle(&self, angle: f32) -> bool {
        (self.start_angle..=self.end_angle).contains(&angle)
    }
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

#[derive(Debug, Default)]
pub struct PieChartResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub hovered: Option<PathBuf>,
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

        self.render_header(ui, data);

        ui.separator();

        let available_size = ui.available_rect_before_wrap().size();
        let (rect, interact_response) = ui.allocate_exact_size(available_size, Sense::click());

        let slices = calculate_slices(data);
        let center = rect.center();
        let radius = (rect.width().min(rect.height()) / 2.0 - LABEL_MARGIN).max(self.center_radius + 10.0);

        // Hit-test first so this frame already draws the hover state.
        let hovered_slice = interact_response
            .hover_pos()
            .and_then(|pos| self.slice_at_position(pos, center, radius, &slices));
        data.hovered_path = hovered_slice.and_then(PieSlice::path).map(Path::to_path_buf);

        for slice in &slices {
            self.render_slice(ui, slice, center, radius, data);
        }
        if let Some(slice) = hovered_slice {
            show_slice_tooltip(ui, slice);
        }

        if self.show_center_info {
            self.render_center_info(ui, center, data);
        }

        if let Some(path) = &data.hovered_path {
            response.hovered = Some(path.clone());

            if interact_response.clicked() {
                data.selected_path = Some(path.clone());
                response.clicked = Some(path.clone());
            }
            if interact_response.double_clicked() {
                response.double_clicked = Some(path.clone());
            }
            if interact_response.secondary_clicked() {
                response.right_clicked = Some(path.clone());
            }
        }

        response
    }

    fn render_header(&self, ui: &mut egui::Ui, data: &mut PieChartData) {
        ui.horizontal(|ui| {
            ui.label(format!("Path: {}", data.current_path.display()));

            ui.separator();

            if ui
                .add_enabled(data.can_navigate_up(), egui::Button::new("⬆ Up"))
                .clicked()
            {
                data.navigate_up();
            }

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

    fn render_slice(&self, ui: &mut egui::Ui, slice: &PieSlice, center: Pos2, radius: f32, data: &PieChartData) {
        let is_selected = slice.path().is_some() && slice.path() == data.selected_path.as_deref();
        let is_hovered = slice.path().is_some() && slice.path() == data.hovered_path.as_deref();

        // Pop the slice out a little when selected or hovered
        let slice_radius = if is_selected {
            radius + 10.0
        } else if is_hovered {
            radius + 5.0
        } else {
            radius
        };

        self.draw_pie_slice(ui, center, slice_radius, slice);

        if data.show_labels && slice.end_angle - slice.start_angle > self.min_label_size.to_radians() {
            self.draw_slice_label(ui, slice, center, slice_radius, data.show_percentages);
        }
    }

    fn draw_pie_slice(&self, ui: &mut egui::Ui, center: Pos2, outer_radius: f32, slice: &PieSlice) {
        let inner_radius = self.center_radius;
        let sweep = slice.end_angle - slice.start_angle;

        // Two segments per degree keep the arc smooth
        let segments = (sweep.to_degrees() * 2.0).max(8.0) as usize;
        let angle_step = sweep / segments as f32;
        let point_at = |radius: f32, i: usize| {
            let angle = slice.start_angle + i as f32 * angle_step;
            Pos2::new(center.x + radius * angle.cos(), center.y + radius * angle.sin())
        };

        // Inner arc forwards, then the outer arc backwards, closes the ring segment
        let vertices: Vec<Pos2> = (0..=segments)
            .map(|i| point_at(inner_radius, i))
            .chain((0..=segments).rev().map(|i| point_at(outer_radius, i)))
            .collect();

        ui.painter().add(egui::Shape::convex_polygon(
            vertices,
            slice.color,
            egui::Stroke::new(1.0_f32, Color32::WHITE),
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

        // Labels sit outside the pie, on the panel background
        ui.painter().text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::default(),
            ui.visuals().text_color(),
        );
    }

    fn render_center_info(&self, ui: &mut egui::Ui, center: Pos2, data: &PieChartData) {
        ui.painter().circle_filled(center, self.center_radius, Color32::WHITE);
        ui.painter()
            .circle_stroke(center, self.center_radius, egui::Stroke::new(2.0_f32, Color32::BLACK));

        let current_node = data.current_node();
        let center_text = format!(
            "{}\n{}",
            truncate_chars(&current_node.name, CENTER_NAME_MAX_CHARS),
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

    fn slice_at_position<'a>(&self, pos: Pos2, center: Pos2, radius: f32, slices: &'a [PieSlice]) -> Option<&'a PieSlice> {
        let offset = pos - center;
        let distance = offset.length();
        if distance < self.center_radius || distance > radius {
            return None;
        }

        // Slices run clockwise from the top, i.e. over [-PI/2, 3PI/2)
        let angle = offset.y.atan2(offset.x);
        let angle = if angle < -PI / 2.0 { angle + TAU } else { angle };

        slices.iter().find(|slice| slice.contains_angle(angle))
    }
}

fn show_slice_tooltip(ui: &egui::Ui, slice: &PieSlice) {
    egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("pie_tooltip"), |ui| {
        ui.label(format!("Name: {}", slice.name));
        if let Some(path) = slice.path() {
            ui.label(format!("Path: {}", path.display()));
        }
        ui.label(format!("Size: {}", format_size(slice.size)));
        ui.label(format!("Percentage: {:.1}%", slice.percentage));
    });
}

/// Splits the current directory into slices for its largest subdirectories
/// and one "Other" slice for the rest, starting at the top and going clockwise.
fn calculate_slices(data: &PieChartData) -> Vec<PieSlice> {
    let current_node = data.current_node();
    let total_size = current_node.size;
    if total_size == 0 {
        return Vec::new();
    }

    let mut children: Vec<&DirectoryNode> = current_node.children.iter().filter(|child| child.size > 0).collect();
    children.sort_by_key(|child| std::cmp::Reverse(child.size));

    // Reserve one slot for "Other" when not every child fits
    let slots = if children.len() > data.max_slices {
        data.max_slices.saturating_sub(1)
    } else {
        children.len()
    };

    let fraction_of = |size: u64| size as f64 / total_size as f64;
    let mut slices = Vec::new();
    let mut current_angle = -PI / 2.0;
    let mut shown_size = 0u64;

    for (index, child) in children.iter().take(slots).enumerate() {
        let angle_size = (fraction_of(child.size) * TAU as f64) as f32;
        if angle_size < data.min_slice_angle {
            // Children are sorted, so every remaining one is smaller still
            break;
        }

        slices.push(PieSlice {
            target: SliceTarget::Directory(child.path.clone()),
            name: child.name.clone(),
            size: child.size,
            percentage: (fraction_of(child.size) * 100.0) as f32,
            start_angle: current_angle,
            end_angle: current_angle + angle_size,
            color: color_for_index(index),
        });
        current_angle += angle_size;
        shown_size += child.size;
    }

    let other_size = total_size.saturating_sub(shown_size);
    let other_angle = (fraction_of(other_size) * TAU as f64) as f32;
    if other_size > 0 && other_angle >= data.min_slice_angle {
        slices.push(PieSlice {
            target: SliceTarget::Other,
            name: "Other".to_string(),
            size: other_size,
            percentage: (fraction_of(other_size) * 100.0) as f32,
            start_angle: current_angle,
            end_angle: current_angle + other_angle,
            color: Color32::GRAY,
        });
    }

    slices
}

fn color_for_index(index: usize) -> Color32 {
    // Step the hue by the golden angle so neighbouring slices differ clearly
    let hue = (index as f32 * 137.5) % 360.0;
    hsv_to_rgb(hue, 0.7, 0.9)
}

/// Shortens `text` to at most `max_chars` characters, marking any cut with "…".
fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
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
            self.selected_path = None;
            self.hovered_path = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(path: &str, size: u64, children: Vec<DirectoryNode>) -> DirectoryNode {
        let mut node = DirectoryNode::new(PathBuf::from(path));
        node.size = size;
        node.children = children;
        node
    }

    #[test]
    fn truncate_chars_handles_multibyte_names() {
        assert_eq!(truncate_chars("短い", 10), "短い");
        assert_eq!(truncate_chars("ドキュメントとダウンロード", 10), "ドキュメントとダウ…");
    }

    #[test]
    fn slices_cover_the_whole_circle() {
        // 40 bytes of files sit directly in the root, outside any child
        let root = directory("root", 100, vec![directory("root/a", 40, vec![]), directory("root/b", 20, vec![])]);
        let data = PieChartData::new(Arc::new(crate::data::ScanResult::new(root)));

        let slices = calculate_slices(&data);

        let targets: Vec<_> = slices.iter().map(|slice| slice.target.clone()).collect();
        assert_eq!(
            targets,
            [
                SliceTarget::Directory(PathBuf::from("root/a")),
                SliceTarget::Directory(PathBuf::from("root/b")),
                SliceTarget::Other,
            ]
        );
        let swept = slices.last().unwrap().end_angle - slices[0].start_angle;
        assert!((swept - TAU).abs() < 1e-4, "slices sweep {swept} rad");
    }

    #[test]
    fn navigate_up_stops_at_root() {
        let root = directory("root", 10, vec![directory("root/a", 10, vec![directory("root/a/b", 10, vec![])])]);
        let mut data = PieChartData::new(Arc::new(crate::data::ScanResult::new(root)));

        data.navigate_to(PathBuf::from("root/a"));
        assert_eq!(data.current_path, PathBuf::from("root/a"));

        // A directory without subdirectories has nothing to show
        data.navigate_to(PathBuf::from("root/a/b"));
        assert_eq!(data.current_path, PathBuf::from("root/a"));

        data.navigate_up();
        data.navigate_up();
        assert_eq!(data.current_path, PathBuf::from("root"));
        assert!(!data.can_navigate_up());
    }
}
