// Code for generating the svg file

use quick_xml;
use std::fs;
use std::path::PathBuf;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::SiaError;
fn get_svg_elements(svg_path: PathBuf) -> Result<String, SiaError> {
    let file_contents = fs::read_to_string(svg_path)?;

    let mut reader = Reader::from_str(&file_contents);

    reader.config_mut().trim_text(true);

    Ok("OK".to_string())
}
