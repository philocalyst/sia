// Code for generating the svg file

use quick_xml;
use roxmltree;
use std::fs;
use std::path::PathBuf;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

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
