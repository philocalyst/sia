// Code for generating the svg file

use quick_xml;
use std::fs;
use std::path::PathBuf;
use svg::node::element::{
    Circle, ClipPath, Definitions, Filter, FilterEffectGaussianBlur, FilterEffectMerge,
    FilterEffectMergeNode, FilterEffectOffset, Group, Rectangle,
};
use svg::Document;
use svg::Node;

use crate::SiaError;
fn get_svg_elements<'a>(
    svg_path: PathBuf,
    contents: &'a mut String,
) -> Result<roxmltree::Document<'a>, SiaError> {
    // Read the file contents into the string buffer
    *contents = fs::read_to_string(svg_path)?;

    // Parse the XML
    let doc =
        roxmltree::Document::parse(contents).map_err(|e| SiaError::XmlParseError(e.to_string()))?;

    // Ensure that it is an svg
    let root_element = doc.root_element();
    if !root_element.has_tag_name("svg") {
        return Err(SiaError::InvalidSvg(
            "Root element is not an SVG element".to_string(),
        ));
    }

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
fn set_dimensions<E: Node>(elem: &mut E, width: f64, height: f64) -> &mut E {
    elem.assign("width", width);
    elem.assign("height", height);
    elem
}

/// Reads `width`/`height` attributes (e.g. `"500px"` or `"200"`) and returns integers.
fn get_dimensions<E: Node>(elem: &E) -> (i32, i32) {
    let element_attributes = elem.get_attributes().unwrap();
    let w = element_attributes.get("width").unwrap();
    let h = element_attributes.get("height").unwrap();
    (dimension_to_int(w), dimension_to_int(h))
}

/// Helper to strip `"px"` and parse an integer, defaulting to 0.
fn dimension_to_int(s: &str) -> i32 {
    s.trim_end_matches("px").parse::<i32>().unwrap_or(0)
}
