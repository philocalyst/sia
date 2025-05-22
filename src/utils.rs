use crate::{Dimensions, FontConfig};
pub fn get_canvas_size(
    pref_dimensions: Option<Dimensions>,
    largest_line_length: u32,
    num_lines: usize,
    font: &FontConfig,
) -> Dimensions {
    if let Some(dims) = pref_dimensions {
        return dims;
    } else {
        // Were using A as a reference width char as it's a good average.
        let advance_width = font
            .font_family
            .glyph('A')
            .scaled(font.scale)
            .h_metrics()
            .advance_width;

        // Calculate the total width in px
        let width_px = largest_line_length * advance_width as u32;

        // Get vertical metrics & compute line height
        let v_metrics = font.font_family.v_metrics(font.scale);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        // Compute total height in px (and add one extra line’s worth of padding)
        let height_px = line_height as u32 * (num_lines + 1);

        Dimensions {
            width: width_px,
            height: height_px,
        }
    }
}
