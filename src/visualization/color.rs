// Placeholder for color management
use eframe::egui::Color32;

#[derive(Debug, Clone, Default)]
pub enum ColorScheme {
    #[default]
    DepthBased,
    SizeBased,
    TypeBased,
    Custom(Vec<Color32>),
}

