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
        let line_height: f32;
        if let Some(v_metrics) = font.font.vertical_line_metrics(font.font_size) {
            line_height = v_metrics.ascent.ceil() + v_metrics.descent.floor();
        } else {
            let metrics = font.font.metrics('A', font.font_size);
            line_height = metrics.height as f32;
        }

        // Compute total height in px
        line_height * num_lines as f32 * 1.52f32
    }
}
