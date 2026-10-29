use eframe::egui::{Color32, Rect};

pub fn calculate_aspect_ratio(rect: Rect) -> f32 {
    let width = rect.width();
    let height = rect.height();

    if height == 0.0 {
        f32::INFINITY
    } else {
        width / height
    }
}

pub fn normalize_values(values: &[f64]) -> Vec<f64> {
    let sum: f64 = values.iter().sum();

    if sum == 0.0 {
        values.to_vec()
    } else {
        values.iter().map(|&v| v / sum).collect()
    }
}

pub fn color_interpolate(start: Color32, end: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv_t = 1.0 - t;

    Color32::from_rgba_unmultiplied(
        (start.r() as f32 * inv_t + end.r() as f32 * t) as u8,
        (start.g() as f32 * inv_t + end.g() as f32 * t) as u8,
        (start.b() as f32 * inv_t + end.b() as f32 * t) as u8,
        (start.a() as f32 * inv_t + end.a() as f32 * t) as u8,
    )
}

pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Color32 {
    let h = hue.rem_euclid(360.0);
    let s = saturation.clamp(0.0, 1.0);
    let v = value.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color32::from_rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_to_rgb_wraps_hue() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), Color32::from_rgb(255, 0, 0));
        assert_eq!(hsv_to_rgb(-120.0, 1.0, 1.0), hsv_to_rgb(240.0, 1.0, 1.0));
        assert_eq!(hsv_to_rgb(480.0, 1.0, 1.0), hsv_to_rgb(120.0, 1.0, 1.0));
    }
}
