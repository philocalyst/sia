use crate::{Dimensions, FontConfig};
pub fn get_canvas_size(
    pref_dimensions: Option<Dimensions>,
    largest_line_length: u32,
    num_lines: u32,
    font: &FontConfig,
) -> (Dimensions, f32) {
    // Were using A as a reference width char as it's a good average.
    let advance_width = font
        .font
        .glyph('A')
        .scaled(font.scale)
        .h_metrics()
        .advance_width;

    if let Some(dims) = pref_dimensions {
        return (dims, advance_width);
    } else {
        // Calculate the total width in px
        let width_px = largest_line_length * advance_width as u32;

        // Get vertical metrics & compute line height
        let v_metrics = font.font.v_metrics(font.scale);
        let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap) * 1.2;

        // Compute total height in px (and add one extra line’s worth of padding)
        let height_px = line_height as u32 * (num_lines as u32 + 1);

        (
            Dimensions {
                width: width_px,
                height: height_px,
            },
            advance_width,
        )
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
