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

fn add_shadow(document: Document, id: &str, x_offset: f64, y_offset: f64, blur: f64) -> Document {
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
    document.add(defs)
}
