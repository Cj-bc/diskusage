use eframe::egui;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::data::{DirectoryNode, ScanResult, SortBy, SortOrder};
use crate::utils::{format_size_with_format, SizeFormat};

pub struct TreeViewData {
    pub scan_result: Arc<ScanResult>,
    pub expanded_paths: HashSet<PathBuf>,
    pub selected_path: Option<PathBuf>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub size_format: SizeFormat,
}

#[derive(Default)]
pub struct TreeViewResponse {
    pub clicked: Option<PathBuf>,
    pub double_clicked: Option<PathBuf>,
    pub right_clicked: Option<PathBuf>,
    pub expanded: Vec<PathBuf>,
    pub collapsed: Vec<PathBuf>,
}


pub struct TreeViewRenderer {
    pub indent_size: f32,
    pub line_height: f32,
    pub show_icons: bool,
    pub show_sizes: bool,
    pub show_percentages: bool,
    pub show_file_counts: bool,
}

impl Default for TreeViewRenderer {
    fn default() -> Self {
        Self {
            indent_size: 20.0,
            line_height: 20.0,
            show_icons: true,
            show_sizes: true,
            show_percentages: true,
            show_file_counts: true,
        }
    }
}

impl TreeViewRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ui: &mut egui::Ui, data: &mut TreeViewData) -> TreeViewResponse {
        let mut response = TreeViewResponse::default();

        // Header with sort controls
        self.render_header(ui, data);

        ui.separator();

        // Hold our own handle to the result so the tree can be borrowed
        // while `data` is mutated, without copying the tree every frame.
        let scan_result = Arc::clone(&data.scan_result);

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.render_node(
                    ui,
                    &scan_result.root,
                    0,
                    scan_result.total_size,
                    data,
                    &mut response,
                );
            });

        response
    }

    fn render_header(&self, ui: &mut egui::Ui, data: &mut TreeViewData) {
        ui.horizontal(|ui| {
            ui.label("Sort by:");
            
            egui::ComboBox::from_id_source("sort_by_combo")
                .selected_text(format!("{:?}", data.sort_by))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut data.sort_by, SortBy::Name, "Name");
                    ui.selectable_value(&mut data.sort_by, SortBy::Size, "Size");
                    ui.selectable_value(&mut data.sort_by, SortBy::Modified, "Modified");
                    ui.selectable_value(&mut data.sort_by, SortBy::Type, "Type");
                });

            if ui.button(match data.sort_order {
                SortOrder::Ascending => "↑ Asc",
                SortOrder::Descending => "↓ Desc",
            }).clicked() {
                data.sort_order = match data.sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
            }

            ui.separator();

            ui.label("Size format:");
            egui::ComboBox::from_id_source("size_format_combo")
                .selected_text(format!("{:?}", data.size_format))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut data.size_format, SizeFormat::Auto, "Auto");
                    ui.selectable_value(&mut data.size_format, SizeFormat::Bytes, "Bytes");
                    ui.selectable_value(&mut data.size_format, SizeFormat::Binary, "Binary");
                    ui.selectable_value(&mut data.size_format, SizeFormat::Decimal, "Decimal");
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Total: {} ({} files, {} dirs)", 
                    format_size_with_format(data.scan_result.total_size, data.size_format),
                    data.scan_result.total_files,
                    data.scan_result.total_dirs
                ));
            });
        });
    }

    fn render_node(
        &self,
        ui: &mut egui::Ui,
        node: &DirectoryNode,
        depth: usize,
        total_size: u64,
        data: &mut TreeViewData,
        response: &mut TreeViewResponse,
    ) {
        let is_expanded = data.expanded_paths.contains(&node.path);
        let is_selected = data.selected_path.as_ref() == Some(&node.path);

        // Calculate percentage
        let percentage = if total_size > 0 {
            (node.size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        // Row background
        let (row_rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(ui.available_width(), self.line_height),
            egui::Sense::click(),
        );

        if is_selected {
            ui.painter().rect_filled(
                row_rect,
                egui::Rounding::same(2.0),
                ui.style().visuals.selection.bg_fill,
            );
        }

        ui.allocate_ui_at_rect(row_rect, |ui| {
            ui.horizontal(|ui| {
                // Indentation
                ui.add_space(depth as f32 * self.indent_size);

                // Expand/collapse button for directories with children
                if !node.children.is_empty() {
                    let expand_button = if is_expanded {
                        ui.button("▼")
                    } else {
                        ui.button("▶")
                    };

                    if expand_button.clicked() {
                        if is_expanded {
                            data.expanded_paths.remove(&node.path);
                            response.collapsed.push(node.path.clone());
                        } else {
                            data.expanded_paths.insert(node.path.clone());
                            response.expanded.push(node.path.clone());
                        }
                    }
                } else {
                    ui.add_space(20.0); // Space for alignment
                }

                // Icon
                if self.show_icons {
                    // Every node is a directory; files are only counted, not listed
                    let icon = if is_expanded { "📂" } else { "📁" };
                    ui.label(icon);
                }

                // Name (clickable)
                let name_response = ui.selectable_label(is_selected, &node.name);
                
                if name_response.clicked() {
                    data.selected_path = Some(node.path.clone());
                    response.clicked = Some(node.path.clone());
                }
                
                if name_response.double_clicked() {
                    response.double_clicked = Some(node.path.clone());
                    // Auto-expand on double-click
                    if !node.children.is_empty() && !is_expanded {
                        data.expanded_paths.insert(node.path.clone());
                        response.expanded.push(node.path.clone());
                    }
                }

                if name_response.secondary_clicked() {
                    response.right_clicked = Some(node.path.clone());
                }

                // Right-aligned size information
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.show_file_counts && !node.children.is_empty() {
                        ui.label(format!("({} items)", node.children.len()));
                    }

                    if self.show_percentages && percentage >= 0.1 {
                        ui.label(format!("{:.1}%", percentage));
                    }

                    if self.show_sizes {
                        ui.label(format_size_with_format(node.size, data.size_format));
                    }
                });
            });
        });

        // Show children if expanded
        if is_expanded && !node.children.is_empty() {
            // Sort children based on current sort criteria
            let mut sorted_children: Vec<&DirectoryNode> = node.children.iter().collect();
            self.sort_children(&mut sorted_children, data.sort_by, data.sort_order);

            for child in sorted_children {
                self.render_node(ui, child, depth + 1, total_size, data, response);
            }
        }
    }

    fn sort_children(&self, children: &mut [&DirectoryNode], sort_by: SortBy, sort_order: SortOrder) {
        children.sort_by(|a, b| {
            let comparison = match sort_by {
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Modified => a.last_modified.cmp(&b.last_modified),
                SortBy::Type => {
                    // Directories first, then files
                    let a_is_dir = !a.children.is_empty();
                    let b_is_dir = !b.children.is_empty();
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    }
                }
            };

            match sort_order {
                SortOrder::Ascending => comparison,
                SortOrder::Descending => comparison.reverse(),
            }
        });
    }
}

impl TreeViewData {
    pub fn new(scan_result: Arc<ScanResult>) -> Self {
        let mut expanded_paths = HashSet::new();
        // Auto-expand the root
        expanded_paths.insert(scan_result.root.path.clone());

        Self {
            scan_result,
            expanded_paths,
            selected_path: None,
            sort_by: SortBy::Size,
            sort_order: SortOrder::Descending,
            size_format: SizeFormat::Auto,
        }
    }

    pub fn expand_path(&mut self, path: PathBuf) {
        self.expanded_paths.insert(path);
    }

    pub fn collapse_path(&mut self, path: &Path) {
        self.expanded_paths.remove(path);
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        self.expanded_paths.contains(path)
    }

    pub fn expand_all(&mut self) {
        fn collect_paths(node: &DirectoryNode, paths: &mut HashSet<PathBuf>) {
            if !node.children.is_empty() {
                paths.insert(node.path.clone());
                for child in &node.children {
                    collect_paths(child, paths);
                }
            }
        }

        collect_paths(&self.scan_result.root, &mut self.expanded_paths);
    }

    pub fn collapse_all(&mut self) {
        self.expanded_paths.clear();
        // Keep root expanded
        self.expanded_paths.insert(self.scan_result.root.path.clone());
    }
}