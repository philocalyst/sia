// Code for generating the svg file

use anyhow::{Error, Result};
use svg::node::element::{
    ClipPath, Definitions, Filter, FilterEffectGaussianBlur, FilterEffectMerge,
    FilterEffectMergeNode, FilterEffectOffset, Group, Rectangle, TSpan, Text,
};
use svg::Document;
use svg::Node;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::utils::get_canvas_height;
use crate::{Colors, FontConfig, Input};

pub(crate) fn code_to_svg(
    theme: &Theme,
    source: &Input,
    font: &FontConfig,
    colors: &Colors,
) -> Result<Document, Error> {
    // Prepare highlighter
    let ss = SyntaxSet::load_defaults_newlines();
    let syntax = ss
        .find_syntax_by_token(&source.ext)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);

    // Highlight each line into Vec<(Style, &str)>
    let lines: Vec<Vec<(Style, &str)>> = LinesWithEndings::from(&source.contents)
        .map(|ln| highlighter.highlight_line(ln, &ss).unwrap())
        .collect();

    // Extract default bg/fg from theme.settings
    let bg = theme.settings.background.unwrap();
    let fg = theme.settings.foreground.unwrap();
    let bg_hex = format!("#{:02X}{:02X}{:02X}", bg.r, bg.g, bg.b);
    let fg_hex = format!("#{:02X}{:02X}{:02X}", fg.r, fg.g, fg.b);

    // a semantic <g> for all text
    let mut g = Group::new()
        .set("font-family", font.font.name().unwrap())
        .set("font-size", font.font_size)
        .set("fill", fg_hex.clone());

    let mut max_width = 0;
    for (i, line) in lines.iter().enumerate() {
        // For some reason 1.2 works better...
        let y_em = (i + 1) as f64 * 1.2;

        // using space preserve otherwise it leads to even weirder space behavior.
        let mut text = Text::new("")
            .set("x", 0)
            .set("y", format!("{:.2}em", y_em))
            .set("xml:space", "preserve");

        let mut segments = String::new();

        for &(ref style, segment) in line {
            // Check if there is style information for the current segment.
            let unstyled = style.foreground == fg && style.font_style.is_empty();

            let mut t = TSpan::new(segment);

            // Only apply the fill if there is style information
            if !unstyled {
                t = t.set(
                    "fill",
                    format!(
                        "#{:02X}{:02X}{:02X}{:02X}", // Ensure that each RGB value converts accurately to a HEX
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                        colors.foreground_alpha.to_u8()
                    ),
                );
            } else {
                // Use the default foreground if no style is found
                t = t.set(
                    "fill",
                    format!(
                        "#{:02X}{:02X}{:02X}{:02X}", // Ensure that each RGB value converts accurately to a HEX
                        fg.r,
                        fg.g,
                        fg.b,
                        colors.background_alpha.to_u8()
                    ),
                );
            }

            use syntect::highlighting::FontStyle;

            if style.font_style.contains(FontStyle::BOLD) {
                t = t.set("font-weight", "bold");
            }

            if style.font_style.contains(FontStyle::ITALIC) {
                t = t.set("font-style", "italic");
            }

            text = text.add(t);
            segments.push_str(segment);
        }

        // Calculate the width for this line
        let width: f32 = segments
            .chars()
            .map(|c| font.font.metrics(c, font.font_size).advance_width)
            .sum();
        max_width = max_width.max(width as u32);

        g = g.add(text);
    }

    let height = get_canvas_height(None, lines.len(), font);

    // Build up the SVG document boilerplate
    let mut doc = Document::new()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("width", format!("{:.0}px", max_width))
        .set("height", format!("{:.0}px", height));

    // Draw background rect
    let bg_rect = Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", bg_hex.clone());
    doc = doc.add(bg_rect);

    doc = doc.add(g);

    Ok(doc)
}

fn add_shadow(elem: Document, id: &str, x_offset: f64, y_offset: f64, blur: f64) -> Document {
    // Gaussian blur the alpha channel
    let gaussian = FilterEffectGaussianBlur::new()
        .set("in", "SourceAlpha")
        .set("stdDeviation", blur);

    // Offset the blurred
    let offset = FilterEffectOffset::new()
        .set("result", "offsetblur")
        .set("dx", x_offset)
        .set("dy", y_offset);

    // Merge the offset blur with the original graphic
    let merge = FilterEffectMerge::new()
        .add(FilterEffectMergeNode::new())
        .add(FilterEffectMergeNode::new().set("in", "SourceGraphic"));

    // Build the <filter> element
    let filter = Filter::new()
        .set("id", id)
        .set("filterUnits", "userSpaceOnUse")
        .add(gaussian)
        .add(offset)
        .add(merge);

    // Wrap it in <defs> and append
    let defs = Definitions::new().add(filter);
    elem.add(defs)
}

/// Adds a <clipPath> definition (with a single <rect>) to the document’s <defs>.
fn add_clip_path(doc: &mut Document, id: &str, x: f64, y: f64, width: f64, height: f64) {
    let clip = ClipPath::new().set("id", id).add(
        Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", width)
            .set("height", height),
    );

    let defs = Definitions::new().add(clip);
    doc.append(defs);
}

/// Returns a new <rect> with corner‐radius applied.
fn add_corner_radius(rect: Rectangle, r: f64) -> Rectangle {
    rect.set("rx", r).set("ry", r)
}

/// Returns a new element shifted to (x,y) with a “px” suffix.
fn move_element<E: Node>(elem: &mut E, x: f64, y: f64) -> &mut E {
    elem.assign("x", format!("{:.2}px", x));
    elem.assign("y", format!("{:.2}px", y));
    elem
}

/// Returns a new element given a stroke outline.
fn add_outline<'a, E: Node>(elem: &'a mut E, width: f64, color: &str) -> &'a mut E {
    elem.assign("stroke", color);
    elem.assign("stroke-width", format!("{:.2}", width));
    elem
}

/// Sets width/height attributes
pub fn set_dimensions<E: Node>(elem: &mut E, width: f64, height: f64) -> &mut E {
    elem.assign("width", width);
    elem.assign("height", height);
    elem
}

/// Reads `width`/`height` attributes (e.g. `"500px"` or `"200"`) and returns integers.
pub fn get_dimensions<E: Node>(elem: &E) -> (u32, u32) {
    let element_attributes = elem.get_attributes().unwrap();
    let w = element_attributes.get("width").unwrap();
    let h = element_attributes.get("height").unwrap();
    (dimension_to_int(w), dimension_to_int(h))
}

/// Helper to strip `"px"` and parse an integer, defaulting to 0.
fn dimension_to_int(s: &str) -> u32 {
    s.trim_end_matches("px").parse::<u32>().unwrap_or(0)
}
