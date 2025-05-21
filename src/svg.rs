// Code for generating the svg file

use quick_xml;
use std::fs;
use std::path::PathBuf;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::SiaError;
fn get_svg_elements(svg_path: PathBuf) -> Result<String, SiaError> {
    // Read file contents
    let file_contents = fs::read_to_string(svg_path)?;

    // Create XML reader
    let mut reader = Reader::from_str(&file_contents);
    reader.config_mut().trim_text(true);

    // Initialize variables for tracking elements
    let root_element = None;
    let mut buf = Vec::new();

    // Parse XML document
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                // Found an element - check if it's the root
                if root_element.is_none() {
                    // This is the first element, store it as the root
                    Some(e.name().as_ref().to_vec());
                }
            }
            Ok(Event::Eof) => break, // End of file reached
            Err(e) => return Err(SiaError::XmlParseError(e.to_string())),
            _ => (), // Skip other events
        }
        buf.clear();
    }

    // Check if we found at least one element
    match root_element {
        Some(tag) => {
            let tag_name =
                String::from_utf8(tag).map_err(|e| SiaError::XmlParseError(e.to_string()))?;
            Ok(tag_name)
        }
        None => Err(SiaError::InvalidSvg(
            "No elements found in SVG file".to_string(),
        )),
    }
}
