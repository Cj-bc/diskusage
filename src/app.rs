use eframe::egui;
use std::path::PathBuf;

use crate::fs::{ScanOptions, ScanUpdate, Scanner};
use crate::gui::AppState;
use crate::gui::views::{TreeViewData, TreeViewRenderer, TreeMapData, TreeMapRenderer, PieChartData, PieChartRenderer};

pub struct DiskUsageApp {
    state: AppState,
    scanner: Scanner,
    tree_view_renderer: TreeViewRenderer,
    tree_view_data: Option<TreeViewData>,
    tree_map_renderer: TreeMapRenderer,
    tree_map_data: Option<TreeMapData>,
    pie_chart_renderer: PieChartRenderer,
    pie_chart_data: Option<PieChartData>,
}

impl DiskUsageApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::default()
        };

        Self {
            state,
            scanner: Scanner::new(),
            tree_view_renderer: TreeViewRenderer::new(),
            tree_view_data: None,
            tree_map_renderer: TreeMapRenderer::new(),
            tree_map_data: None,
            pie_chart_renderer: PieChartRenderer::new(),
            pie_chart_data: None,
        }
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        if self.scanner.is_scanning() {
            // Keep polling the background scan even when no input arrives.
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        match self.scanner.poll() {
            None => {}
            Some(ScanUpdate::Progress(progress)) => {
                self.state.set_scan_progress(Some(progress));
                ctx.request_repaint();
            }
            Some(ScanUpdate::Finished(result)) => match result {
                Ok(scan_result) => {
                    // Create TreeView, TreeMap, and PieChart data
                    self.tree_view_data = Some(TreeViewData::new(scan_result.clone()));
                    self.tree_map_data = Some(TreeMapData::new(scan_result.clone()));
                    self.pie_chart_data = Some(PieChartData::new(scan_result.clone()));
                    self.state.set_scan_result(Some(scan_result));
                    self.state.set_scanning(false);
                    self.state.clear_error();
                }
                Err(error) => {
                    self.state.set_error(format!("Scan failed: {}", error));
                    self.state.set_scanning(false);
                    self.tree_view_data = None;
                    self.tree_map_data = None;
                    self.pie_chart_data = None;
                }
            },
        }
    }

    fn show_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Directory...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.start_scan(path);
                    }
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Recent").clicked() {
                    // TODO: Show recent directories
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("TreeMap").clicked() {
                    self.state.set_view_mode(crate::gui::ViewMode::TreeMap);
                    ui.close_menu();
                }
                if ui.button("Tree View").clicked() {
                    self.state.set_view_mode(crate::gui::ViewMode::TreeView);
                    ui.close_menu();
                }
                if ui.button("Pie Chart").clicked() {
                    self.state.set_view_mode(crate::gui::ViewMode::PieChart);
                    ui.close_menu();
                }
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Refresh").clicked() {
                    if let Some(path) = &self.state.current_path {
                        self.start_scan(path.clone());
                    }
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Export...").clicked() {
                    // TODO: Show export dialog
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Settings").clicked() {
                    // TODO: Show settings dialog
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    // TODO: Show about dialog
                    ui.close_menu();
                }
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Directory selection
            if ui.button("📁 Open Directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.start_scan(path);
                }
            }

            ui.separator();

            // Current path display
            if let Some(path) = &self.state.current_path {
                ui.label(format!("Path: {}", path.display()));
            } else {
                ui.label("No directory selected");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // View mode buttons
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.state.view_mode, crate::gui::ViewMode::TreeMap, "TreeMap");
                    ui.selectable_value(&mut self.state.view_mode, crate::gui::ViewMode::TreeView, "Tree");
                    ui.selectable_value(&mut self.state.view_mode, crate::gui::ViewMode::PieChart, "Pie");
                });
            });
        });
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.state.is_scanning {
                ui.spinner();
                ui.label("Scanning...");
                
                if let Some(progress) = &self.state.scan_progress {
                    ui.label(format!("Files: {}, Dirs: {}", progress.scanned_files, progress.scanned_dirs));
                    
                    if ui.button("Cancel").clicked() {
                        self.scanner.cancel();
                        self.state.set_scanning(false);
                    }
                }
            } else if let Some(result) = &self.state.scan_result {
                ui.label(format!(
                    "Total: {} | Files: {} | Dirs: {} | Size: {}",
                    result.total_files + result.total_dirs,
                    result.total_files,
                    result.total_dirs,
                    crate::utils::format_size(result.total_size)
                ));
            } else {
                ui.label("Ready");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(error) = &self.state.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                }
            });
        });
    }

    fn start_scan(&mut self, path: PathBuf) {
        log::info!("Starting scan of: {}", path.display());
        self.state.set_current_path(Some(path.clone()));
        self.state.set_scanning(true);
        self.state.set_scan_progress(None);
        self.state.clear_error();
        
        let options = ScanOptions {
            follow_symlinks: self.state.settings.follow_symlinks,
            max_depth: self.state.settings.max_scan_depth,
        };
        self.scanner.start_scan(path, options);
    }
}

impl eframe::App for DiskUsageApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_events(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ctx, ui);
        });

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.show_toolbar(ui);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(scan_result) = &self.state.scan_result {
                match self.state.view_mode {
                    crate::gui::ViewMode::TreeMap => {
                        if let Some(ref mut tree_map_data) = self.tree_map_data {
                            let response = self.tree_map_renderer.render(ui, tree_map_data);
                            
                            // Handle TreeMap interactions
                            if let Some(clicked_path) = response.clicked {
                                // Handle item selection
                                log::info!("Selected: {}", clicked_path.display());
                            }
                            
                            if let Some(double_clicked_path) = response.double_clicked {
                                // Handle navigation on double-click
                                log::info!("Navigate to: {}", double_clicked_path.display());
                                tree_map_data.navigate_to(double_clicked_path);
                            }
                            
                            if let Some(right_clicked_path) = response.right_clicked {
                                // Handle context menu
                                log::info!("Context menu for: {}", right_clicked_path.display());
                                // TODO: Show context menu
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("No data available for TreeMap view");
                            });
                        }
                    }
                    crate::gui::ViewMode::TreeView => {
                        if let Some(ref mut tree_data) = self.tree_view_data {
                            let response = self.tree_view_renderer.render(ui, tree_data);
                            
                            // Handle TreeView interactions
                            if let Some(clicked_path) = response.clicked {
                                // Handle item selection
                                log::info!("Selected: {}", clicked_path.display());
                            }
                            
                            if let Some(double_clicked_path) = response.double_clicked {
                                // Handle navigation on double-click
                                log::info!("Navigate to: {}", double_clicked_path.display());
                                // TODO: Implement navigation to subdirectory
                            }
                            
                            if let Some(right_clicked_path) = response.right_clicked {
                                // Handle context menu
                                log::info!("Context menu for: {}", right_clicked_path.display());
                                // TODO: Show context menu
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("No data available for tree view");
                            });
                        }
                    }
                    crate::gui::ViewMode::PieChart => {
                        if let Some(ref mut pie_chart_data) = self.pie_chart_data {
                            let response = self.pie_chart_renderer.render(ui, pie_chart_data);
                            
                            // Handle PieChart interactions
                            if let Some(clicked_path) = response.clicked {
                                // Handle item selection
                                log::info!("Selected: {}", clicked_path.display());
                            }
                            
                            if let Some(double_clicked_path) = response.double_clicked {
                                // Handle navigation on double-click
                                log::info!("Navigate to: {}", double_clicked_path.display());
                                pie_chart_data.navigate_to(double_clicked_path);
                            }
                            
                            if let Some(right_clicked_path) = response.right_clicked {
                                // Handle context menu
                                log::info!("Context menu for: {}", right_clicked_path.display());
                                // TODO: Show context menu
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("No data available for pie chart view");
                            });
                        }
                    }
                }
            } else if self.state.is_scanning {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                        ui.add_space(20.0);
                        ui.heading("Scanning Directory...");
                        
                        if let Some(progress) = &self.state.scan_progress {
                            ui.label(format!("Current: {}", progress.current_path.display()));
                            ui.label(format!("Scanned: {} files, {} directories", 
                                progress.scanned_files, progress.scanned_dirs));
                        }
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Welcome to DiskUsage");
                        ui.add_space(20.0);
                        ui.label("Select a directory to analyze disk usage");
                        ui.add_space(20.0);
                        
                        if ui.button("📁 Open Directory").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.start_scan(path);
                            }
                        }
                    });
                });
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }
}