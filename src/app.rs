use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::data::{ScanProgress, ScanResult};
use crate::fs::{ScanOptions, ScanUpdate, Scanner};
use crate::gui::views::{
    PieChartData, PieChartRenderer, TreeMapData, TreeMapRenderer, TreeViewData, TreeViewRenderer,
};
use crate::gui::{AppState, ViewMode};

/// How often to repaint while a scan runs, so progress keeps updating
/// without user input.
const SCAN_REPAINT_INTERVAL: Duration = Duration::from_millis(100);

/// Where the app is in the scan lifecycle. Keeping the per-view data inside
/// `Loaded` guarantees it always belongs to the result being shown.
enum ScanState {
    Idle,
    Scanning { progress: Option<ScanProgress> },
    Loaded(Box<LoadedScan>),
}

struct LoadedScan {
    result: Arc<ScanResult>,
    tree_view: TreeViewData,
    tree_map: TreeMapData,
    pie_chart: PieChartData,
}

impl LoadedScan {
    fn new(result: ScanResult) -> Self {
        let result = Arc::new(result);
        Self {
            tree_view: TreeViewData::new(Arc::clone(&result)),
            tree_map: TreeMapData::new(Arc::clone(&result)),
            pie_chart: PieChartData::new(Arc::clone(&result)),
            result,
        }
    }
}

/// Interactions reported by any of the views.
struct ViewInteraction {
    clicked: Option<PathBuf>,
    double_clicked: Option<PathBuf>,
    right_clicked: Option<PathBuf>,
}

pub struct DiskUsageApp {
    state: AppState,
    scanner: Scanner,
    scan: ScanState,
    show_about: bool,
    tree_view_renderer: TreeViewRenderer,
    tree_map_renderer: TreeMapRenderer,
    pie_chart_renderer: PieChartRenderer,
}

impl DiskUsageApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        Self {
            state,
            scanner: Scanner::new(),
            scan: ScanState::Idle,
            show_about: false,
            tree_view_renderer: TreeViewRenderer::new(),
            tree_map_renderer: TreeMapRenderer::new(),
            pie_chart_renderer: PieChartRenderer::new(),
        }
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        if self.scanner.is_scanning() {
            ctx.request_repaint_after(SCAN_REPAINT_INTERVAL);
        }

        match self.scanner.poll() {
            None => {}
            Some(ScanUpdate::Progress(latest)) => {
                if let ScanState::Scanning { progress } = &mut self.scan {
                    *progress = Some(latest);
                }
            }
            Some(ScanUpdate::Finished(Ok(result))) => {
                if result.has_errors() {
                    log::warn!("Scan finished with {} skipped entries", result.errors.len());
                }
                self.scan = ScanState::Loaded(Box::new(LoadedScan::new(result)));
                self.state.clear_error();
            }
            Some(ScanUpdate::Finished(Err(error))) => {
                self.scan = ScanState::Idle;
                self.state.set_error(format!("Scan failed: {}", error));
            }
        }
    }

    fn start_scan(&mut self, path: PathBuf) {
        log::info!("Starting scan of: {}", path.display());
        let options = ScanOptions {
            follow_symlinks: self.state.settings.follow_symlinks,
            max_depth: self.state.settings.max_scan_depth,
        };
        self.scanner.start_scan(path.clone(), options);
        self.state.set_current_path(path);
        self.state.clear_error();
        self.scan = ScanState::Scanning { progress: None };
    }

    fn cancel_scan(&mut self) {
        self.scanner.cancel();
        self.scan = ScanState::Idle;
    }

    fn pick_and_scan(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.start_scan(path);
        }
    }

    fn show_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Directory...").clicked() {
                    ui.close_menu();
                    self.pick_and_scan();
                }

                ui.separator();

                let mut recent_choice = None;
                ui.add_enabled_ui(!self.state.recent_paths.is_empty(), |ui| {
                    ui.menu_button("Recent", |ui| {
                        for path in &self.state.recent_paths {
                            if ui.button(path.display().to_string()).clicked() {
                                recent_choice = Some(path.clone());
                                ui.close_menu();
                            }
                        }
                    });
                });
                if let Some(path) = recent_choice {
                    self.start_scan(path);
                }

                ui.separator();

                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("View", |ui| {
                for mode in ViewMode::ALL {
                    if ui
                        .radio(self.state.view_mode == mode, mode.label())
                        .clicked()
                    {
                        self.state.set_view_mode(mode);
                        ui.close_menu();
                    }
                }
            });

            ui.menu_button("Tools", |ui| {
                let refresh_path = self.state.current_path.clone();
                if ui
                    .add_enabled(refresh_path.is_some(), egui::Button::new("Refresh"))
                    .clicked()
                {
                    ui.close_menu();
                    if let Some(path) = refresh_path {
                        self.start_scan(path);
                    }
                }

                ui.separator();

                ui.add_enabled(false, egui::Button::new("Export..."))
                    .on_disabled_hover_text("Not implemented yet");

                ui.separator();

                ui.add_enabled(false, egui::Button::new("Settings"))
                    .on_disabled_hover_text("Not implemented yet");
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    self.show_about = true;
                    ui.close_menu();
                }
            });
        });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📁 Open Directory").clicked() {
                self.pick_and_scan();
            }

            ui.separator();

            match &self.state.current_path {
                Some(path) => ui.label(format!("Path: {}", path.display())),
                None => ui.label("No directory selected"),
            };

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Right-to-left layout, so add the modes in reverse order.
                for mode in ViewMode::ALL.into_iter().rev() {
                    ui.selectable_value(&mut self.state.view_mode, mode, mode.label());
                }
            });
        });
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            match &self.scan {
                ScanState::Scanning { progress } => {
                    ui.spinner();
                    ui.label("Scanning...");

                    if let Some(progress) = progress {
                        ui.label(format!(
                            "Files: {}, Dirs: {}, Size: {}",
                            progress.scanned_files,
                            progress.scanned_dirs,
                            crate::utils::format_size(progress.total_size)
                        ));
                    }

                    if ui.button("Cancel").clicked() {
                        self.cancel_scan();
                    }
                }
                ScanState::Loaded(loaded) => {
                    let result = &loaded.result;
                    ui.label(format!(
                        "Total: {} | Files: {} | Dirs: {} | Size: {} | Took: {}",
                        result.total_files + result.total_dirs,
                        result.total_files,
                        result.total_dirs,
                        crate::utils::format_size(result.total_size),
                        crate::utils::format_duration(result.scan_duration),
                    ));

                    if result.has_errors() {
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            format!("⚠ {} skipped", result.errors.len()),
                        )
                        .on_hover_text(skipped_entries_summary(&result.errors));
                    }
                }
                ScanState::Idle => {
                    ui.label("Ready");
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(error) = &self.state.error_message {
                    ui.colored_label(ui.visuals().error_fg_color, format!("Error: {}", error));
                }
            });
        });
    }

    fn show_central_panel(&mut self, ui: &mut egui::Ui) {
        match &mut self.scan {
            ScanState::Loaded(loaded) => {
                let interaction = match self.state.view_mode {
                    ViewMode::TreeMap => {
                        let response = self.tree_map_renderer.render(ui, &mut loaded.tree_map);
                        if let Some(path) = &response.double_clicked {
                            loaded.tree_map.navigate_to(path.clone());
                        }
                        ViewInteraction {
                            clicked: response.clicked,
                            double_clicked: response.double_clicked,
                            right_clicked: response.right_clicked,
                        }
                    }
                    ViewMode::TreeView => {
                        let response = self.tree_view_renderer.render(ui, &mut loaded.tree_view);
                        ViewInteraction {
                            clicked: response.clicked,
                            double_clicked: response.double_clicked,
                            right_clicked: response.right_clicked,
                        }
                    }
                    ViewMode::PieChart => {
                        let response = self.pie_chart_renderer.render(ui, &mut loaded.pie_chart);
                        if let Some(path) = &response.double_clicked {
                            loaded.pie_chart.navigate_to(path.clone());
                        }
                        ViewInteraction {
                            clicked: response.clicked,
                            double_clicked: response.double_clicked,
                            right_clicked: response.right_clicked,
                        }
                    }
                };
                log_interaction(&interaction);
            }
            ScanState::Scanning { progress } => {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Spinner::new().size(64.0));
                        ui.add_space(20.0);
                        ui.heading("Scanning Directory...");

                        if let Some(progress) = progress {
                            ui.label(format!("Current: {}", progress.current_path.display()));
                            ui.label(format!(
                                "Scanned: {} files, {} directories",
                                progress.scanned_files, progress.scanned_dirs
                            ));
                        }
                    });
                });
            }
            ScanState::Idle => {
                let mut open_clicked = false;
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Welcome to DiskUsage");
                        ui.add_space(20.0);
                        ui.label("Select a directory to analyze disk usage");
                        ui.add_space(20.0);
                        open_clicked = ui.button("📁 Open Directory").clicked();
                    });
                });
                if open_clicked {
                    self.pick_and_scan();
                }
            }
        }
    }

    fn show_about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About DiskUsage")
            .open(&mut self.show_about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("DiskUsage v{}", env!("CARGO_PKG_VERSION")));
                ui.label(env!("CARGO_PKG_DESCRIPTION"));
                ui.hyperlink(env!("CARGO_PKG_REPOSITORY"));
            });
    }
}

fn log_interaction(interaction: &ViewInteraction) {
    let events = [
        ("Selected", &interaction.clicked),
        ("Opened", &interaction.double_clicked),
        ("Context menu for", &interaction.right_clicked),
    ];
    for (action, path) in events {
        if let Some(path) = path {
            log::info!("{}: {}", action, path.display());
        }
    }
}

/// Hover text listing the first few entries a scan had to skip.
fn skipped_entries_summary(errors: &[crate::data::ScanError]) -> String {
    const SHOWN: usize = 10;
    let mut lines: Vec<String> = errors.iter().take(SHOWN).map(ToString::to_string).collect();
    if errors.len() > SHOWN {
        lines.push(format!("... and {} more", errors.len() - SHOWN));
    }
    lines.join("\n")
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
            self.show_central_panel(ui);
        });

        self.show_about_window(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }
}
