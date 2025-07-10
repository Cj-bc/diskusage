// Placeholder for color management
use eframe::egui::Color32;

#[derive(Debug, Clone)]
pub enum ColorScheme {
    DepthBased,
    SizeBased,
    TypeBased,
    Custom(Vec<Color32>),
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme::DepthBased
    }
}