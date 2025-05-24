use crate::{Dimensions, FontConfig};

        .scaled(font.scale)
        .h_metrics()
        .advance_width;

pub fn get_canvas_height(
    pref_dimensions: Option<Dimensions>,
    num_lines: usize,
    font: &FontConfig,
) -> f32 {
    if let Some(dims) = pref_dimensions {
        dims.height as f32
    } else {
        // Get vertical metrics & compute line height
        let v_metrics = font.font.v_metrics(font.scale);
        let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap) * 1.2;

        // Compute total height in px (and add one extra line’s worth of padding)
        line_height * (num_lines as f32 + 1.0)
    }
}

pub fn get_text_info(str: &str) -> (u32, u32) {
    let lines: Vec<String> = str.lines().map(|s| s.to_owned()).collect();

    // Figure out the widest line in “characters”
    let max_chars: u32 = lines.iter().map(|line| line.len()).max().unwrap_or(0) as u32;

    // Total number of lines
    let line_count: u32 = lines.len() as u32;

    (max_chars, line_count)
}
