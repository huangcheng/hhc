use std::str;

use xml::reader::{
    ParserConfig, Result,
    XmlEvent::{Characters, EndDocument},
};
use xml::EmitterConfig;

use regex::Regex;

use undrift_gps::gcj_to_wgs;

use crate::utils::is_coordinate_in_china;

fn is_coordinate(coordinate: &str) -> bool {
    if let Ok(regex) = Regex::new(r"((\d+).(\d+),){2} *((\d+).(\d+))") {
        if regex.is_match(coordinate) {
            return true;
        }
    }

    false
}

#[test]
fn test_is_coordinate() {
    assert!(is_coordinate("114.3087832333621,30.64590540363425, 19.7"));
}

pub fn convert(document: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = ParserConfig::new()
        .trim_whitespace(true)
        .create_reader(document.as_bytes());

    let mut buffer: Vec<u8> = vec![];

    let mut writer = EmitterConfig::default().create_writer(&mut buffer);

    loop {
        let reader_event = reader.next()?;

        match reader_event {
            Characters(text) => {
                if is_coordinate(&text) {
                    let mut v: Vec<&str> = text.split(",").map(|s| s.trim()).collect();

                    let longitude = v[0].parse::<f64>()?;
                    let latitude = v[1].parse::<f64>()?;

                    if !is_coordinate_in_china(longitude, latitude) {
                        continue;
                    }

                    let (latitude, longitude) = gcj_to_wgs(latitude, longitude);

                    let latitude = latitude.to_string();
                    let longitude = longitude.to_string();

                    v[0] = &longitude;
                    v[1] = &latitude;

                    let coordinate = v.join(", ");

                    let event = xml::writer::XmlEvent::Characters(&coordinate);

                    writer.write(event)?;
                } else {
                    let event = xml::writer::XmlEvent::Characters(&text);

                    writer.write(event)?;
                }
            }

            EndDocument => break,

            other => {
                if let Some(writer_event) = other.as_writer_event() {
                    writer.write(writer_event)?;
                }
            }
        }
    }

    let result = String::from_utf8(buffer)?;

    Ok(result)
}
