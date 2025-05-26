use crate::{Dimensions, FontConfig};

pub fn get_canvas_height(
    pref_dimensions: Option<Dimensions>,
    num_lines: usize,
    font: &FontConfig,
) -> f32 {
    if let Some(dims) = pref_dimensions {
        dims.height as f32
    } else {
        // Get vertical metrics & compute line height
        let v_metrics = font.font.metrics('A', font.font_size);
        let line_height = v_metrics.height;

        // Compute total height in px (and add one extra line’s worth of padding)
        line_height as f32 * (num_lines as f32 + 1.0) * 1.58f32
    }
}
