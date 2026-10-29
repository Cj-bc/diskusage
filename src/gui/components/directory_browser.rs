use eframe::egui;
use std::path::PathBuf;

pub struct DirectoryBrowser {
    pub current_path: PathBuf,
    pub favorites: Vec<PathBuf>,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
}

impl DirectoryBrowser {
    pub fn new() -> Self {
        Self {
            current_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            favorites: Self::load_favorites(),
            history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut selected_path = None;

        ui.vertical(|ui| {
            // Navigation buttons
            ui.horizontal(|ui| {
                if ui.button("⬅").clicked() && self.can_go_back() {
                    selected_path = self.go_back();
                }
                
                if ui.button("➡").clicked() && self.can_go_forward() {
                    selected_path = self.go_forward();
                }
                
                if ui.button("⬆").clicked() {
                    if let Some(parent) = self.current_path.parent() {
                        selected_path = Some(parent.to_path_buf());
                    }
                }
                
                ui.separator();
                
                if ui.button("🏠").clicked() {
                    if let Some(home) = dirs::home_dir() {
                        selected_path = Some(home);
                    }
                }
                
                if ui.button("💽").clicked() {
                    // Show drives/mounts
                    self.show_drives_menu(ui);
                }
            });

            // Current path display
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut self.current_path.to_string_lossy().to_string());
            });

            ui.separator();

            // Favorites section
            ui.collapsing("Favorites", |ui| {
                for favorite in &self.favorites.clone() {
                    if ui.button(favorite.file_name().unwrap_or_default().to_string_lossy()).clicked() {
                        selected_path = Some(favorite.clone());
                    }
                }
                
                ui.horizontal(|ui| {
                    if ui.button("Add Current").clicked() {
                        self.add_favorite(self.current_path.clone());
                    }
                    
                    if ui.button("Manage").clicked() {
                        // TODO: Show favorites management dialog
                    }
                });
            });

            ui.separator();

            // Directory listing
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Ok(entries) = std::fs::read_dir(&self.current_path) {
                    let mut dirs: Vec<_> = entries
                        .filter_map(|entry| entry.ok())
                        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                        .collect();
                    
                    dirs.sort_by_key(|a| a.file_name());
                    
                    for entry in dirs {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if ui.selectable_label(false, format!("📁 {}", name)).clicked() {
                            selected_path = Some(entry.path());
                        }
                    }
                }
            });
        });

        if let Some(path) = &selected_path {
            self.navigate_to(path.clone());
        }

        selected_path
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        // Add current path to history
        if self.current_path != path {
            self.history.truncate(self.history_index + 1);
            self.history.push(self.current_path.clone());
            self.history_index = self.history.len() - 1;
        }
        
        self.current_path = path;
    }

    pub fn go_back(&mut self) -> Option<PathBuf> {
        if self.can_go_back() {
            self.history_index -= 1;
            let path = self.history[self.history_index].clone();
            self.current_path = path.clone();
            Some(path)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<PathBuf> {
        if self.can_go_forward() {
            self.history_index += 1;
            let path = self.history[self.history_index].clone();
            self.current_path = path.clone();
            Some(path)
        } else {
            None
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn add_favorite(&mut self, path: PathBuf) {
        if !self.favorites.contains(&path) {
            self.favorites.push(path);
            self.save_favorites();
        }
    }

    fn show_drives_menu(&self, ui: &mut egui::Ui) {
        ui.menu_button("Drives", |ui| {
            #[cfg(windows)]
            {
                // Show Windows drives
                for drive in 'A'..='Z' {
                    let drive_path = format!("{}:\\", drive);
                    let path = PathBuf::from(&drive_path);
                    if path.exists()
                        && ui.button(&drive_path).clicked() {
                            // TODO: Navigate to drive
                            ui.close_menu();
                        }
                }
            }
            
            #[cfg(unix)]
            {
                // Show common Unix mount points
                let mount_points = ["/", "/home", "/tmp", "/usr", "/var"];
                for mount in &mount_points {
                    let path = PathBuf::from(mount);
                    if path.exists() {
                        if ui.button(mount).clicked() {
                            // TODO: Navigate to mount point
                            ui.close_menu();
                        }
                    }
                }
            }
        });
    }

    fn load_favorites() -> Vec<PathBuf> {
        // TODO: Load favorites from file
        vec![
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            dirs::document_dir().unwrap_or_else(|| PathBuf::from("/")),
            dirs::download_dir().unwrap_or_else(|| PathBuf::from("/")),
        ]
    }

    fn save_favorites(&self) {
        // TODO: Save favorites to file
    }
}

impl Default for DirectoryBrowser {
    fn default() -> Self {
        Self::new()
    }
}