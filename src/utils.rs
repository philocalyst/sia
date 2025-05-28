use crate::{Dimensions, FontConfig};
use rusttype::{self, Scale};
use std::fs;

pub fn get_canvas_height(
    pref_dimensions: Option<Dimensions>,
    num_lines: usize,
    font: &FontConfig,
) -> f32 {
    if let Some(dims) = pref_dimensions {
        dims.height as f32
    } else {
        let bytes = fs::read(font.font_path.clone()).unwrap();
        let font_font = rusttype::Font::try_from_bytes(&bytes).unwrap();
        // Get vertical metrics & compute line height

        let scale = Scale::uniform(font.font_size);
        let v_metrics = font_font.v_metrics(scale);
        let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap) * 1.2;

        // Compute total height in px (and add one extra line’s worth of padding)
        line_height * num_lines as f32
    }
}
