use eframe::egui;

pub use crate::utils::SizeFormat;

pub struct SizeDisplay;

impl SizeDisplay {
    pub fn render_size(ui: &mut egui::Ui, size: u64, format: SizeFormat) {
        let formatted = crate::utils::format_size_with_format(size, format);
        ui.label(formatted);
    }

    pub fn render_size_with_percentage(
        ui: &mut egui::Ui,
        size: u64,
        total: u64,
        format: SizeFormat,
    ) {
        let formatted = crate::utils::format_size_with_format(size, format);
        let percentage = if total > 0 {
            (size as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        ui.horizontal(|ui| {
            ui.label(formatted);
            ui.label(format!("({:.1}%)", percentage));
        });
    }

    pub fn render_size_comparison(
        ui: &mut egui::Ui,
        size1: u64,
        size2: u64,
        label1: &str,
        label2: &str,
        format: SizeFormat,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{}: {}",
                label1,
                crate::utils::format_size_with_format(size1, format)
            ));
            ui.separator();
            ui.label(format!(
                "{}: {}",
                label2,
                crate::utils::format_size_with_format(size2, format)
            ));

            if size1 > 0 && size2 > 0 {
                let ratio = size1 as f64 / size2 as f64;
                ui.label(format!(
                    "({}x)",
                    if ratio >= 1.0 {
                        format!("{:.1}", ratio)
                    } else {
                        format!("1/{:.1}", 1.0 / ratio)
                    }
                ));
            }
        });
    }
}
