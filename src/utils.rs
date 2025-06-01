use crate::FontConfig;
use rusttype::{self, Scale};
use std::fs;

pub fn get_canvas_height(num_lines: usize, font: &FontConfig) -> f32 {
    // Read into RUSTTYPE as fontdue sucks at height
    let font_font = rusttype::Font::try_from_bytes(&font.font_data).unwrap();

    // Get vertical metrics & find individual line height
    let scale = Scale::uniform(font.font_size);
    let v_metrics = font_font.v_metrics(scale);
    let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap) * 1.2;

    line_height * num_lines as f32
}
