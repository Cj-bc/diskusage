use eframe::egui;

pub struct ProgressBar {
    pub progress: f32,
    pub text: Option<String>,
    pub show_percentage: bool,
    pub indeterminate: bool,
}

impl ProgressBar {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            text: None,
            show_percentage: true,
            indeterminate: false,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            progress: 0.0,
            text: None,
            show_percentage: false,
            indeterminate: true,
        }
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn with_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn render(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let height = 20.0;

        let (rect, _response) = ui.allocate_exact_size(
            egui::Vec2::new(available_width, height),
            egui::Sense::hover(),
        );

        // Draw background
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(4.0),
            ui.style().visuals.extreme_bg_color,
        );

        // Draw progress
        if self.indeterminate {
            // Animated indeterminate progress
            let time = ui.input(|i| i.time) as f32;
            let animation_progress = (time * 2.0).sin() * 0.5 + 0.5;
            let bar_width = rect.width() * 0.3;
            let bar_x = rect.min.x + (rect.width() - bar_width) * animation_progress;

            let progress_rect = egui::Rect::from_min_size(
                egui::Pos2::new(bar_x, rect.min.y),
                egui::Vec2::new(bar_width, rect.height()),
            );

            ui.painter().rect_filled(
                progress_rect,
                egui::Rounding::same(4.0),
                ui.style().visuals.selection.bg_fill,
            );
        } else {
            // Normal progress bar
            let progress_width = rect.width() * self.progress;
            let progress_rect =
                egui::Rect::from_min_size(rect.min, egui::Vec2::new(progress_width, rect.height()));

            ui.painter().rect_filled(
                progress_rect,
                egui::Rounding::same(4.0),
                ui.style().visuals.selection.bg_fill,
            );
        }

        // Draw border
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(4.0),
            egui::Stroke::new(1.0_f32, ui.style().visuals.widgets.inactive.fg_stroke.color),
        );

        // Draw text
        if let Some(ref text) = self.text {
            let text_color = ui.style().visuals.text_color();
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                text_color,
            );
        } else if self.show_percentage && !self.indeterminate {
            let percentage_text = format!("{:.0}%", self.progress * 100.0);
            let text_color = ui.style().visuals.text_color();
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                percentage_text,
                egui::FontId::default(),
                text_color,
            );
        }
    }
}
