use std::str;

use xml::reader::{
    ParserConfig,
    XmlEvent::{Characters, EndDocument, EndElement, StartElement},
};
use xml::EmitterConfig;

use undrift_gps::gcj_to_wgs;

use super::Doc;

use crate::utils::is_coordinates_in_china;

pub struct Document;

impl Document {
    const TAG_NAME_LATITUDE: &'static str = "LatitudeDegrees";
    const TAG_NAME_LONGITUDE: &'static str = "LongitudeDegrees";

    fn collect_coordinates(
        document: &str,
    ) -> Result<(Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
        let mut reader = ParserConfig::new()
            .trim_whitespace(true)
            .create_reader(document.as_bytes());

        let mut latitude_points: Vec<f64> = vec![];
        let mut longitude_points: Vec<f64> = vec![];

        let mut tag_name = String::new();

        loop {
            let reader_event = reader.next()?;

            match reader_event {
                StartElement { name, .. } => {
                    tag_name = name.local_name.to_string();
                }
                Characters(text) => {
                    if tag_name == Self::TAG_NAME_LATITUDE {
                        latitude_points.push(text.parse::<f64>()?);
                    } else if tag_name == Self::TAG_NAME_LONGITUDE {
                        longitude_points.push(text.parse::<f64>()?);
                    }
                }

                EndDocument => break,

                _ => {}
            }
        }

        assert_eq!(latitude_points.len(), longitude_points.len());

        Ok((latitude_points, longitude_points))
    }

    fn convert_coordinates(
        latitude_points: &[f64],
        longitude_points: &[f64],
    ) -> (Vec<String>, Vec<String>) {
        let mut latitude_vec: Vec<String> =
            (0..latitude_points.len()).map(|_| "".to_string()).collect();
        let mut longitude_vec: Vec<String> = (0..longitude_points.len())
            .map(|_| "".to_string())
            .collect();

        for i in 0..latitude_points.len() {
            let (latitude, longitude) =
                if is_coordinates_in_china(longitude_points[i], latitude_points[i]) {
                    gcj_to_wgs(latitude_points[i], longitude_points[i])
                } else {
                    (latitude_points[i], longitude_points[i])
                };

            latitude_vec[i] = latitude.to_string();
            longitude_vec[i] = longitude.to_string();
        }

        (latitude_vec, longitude_vec)
    }
}

impl Doc for Document {
    fn convert(document: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
        let mut reader = ParserConfig::new()
            .trim_whitespace(true)
            .create_reader(document.as_bytes());

        let mut buffer: Vec<u8> = vec![];

        let mut writer = EmitterConfig::default().create_writer(&mut buffer);

        let (latitude_points, longitude_points) = Self::collect_coordinates(document)?;
        let (latitude_points, longitude_points) =
            Self::convert_coordinates(&latitude_points, &longitude_points);

        let mut tag_name = String::new();
        let mut index: usize = 0;

        loop {
            let reader_event = reader.next()?;

            match reader_event {
                StartElement {
                    name,
                    attributes,
                    namespace,
                } => {
                    tag_name = name.local_name.clone();

                    let event = xml::writer::XmlEvent::StartElement {
                        name: name.borrow(),
                        namespace: namespace.borrow(),
                        attributes: attributes.iter().map(|attr| attr.borrow()).collect(),
                    };
                    writer.write(event)?;
                }
                Characters(text) => {
                    let event = if tag_name == Self::TAG_NAME_LATITUDE {
                        let text = &latitude_points[index];

                        xml::writer::XmlEvent::Characters(text)
                    } else if tag_name == Self::TAG_NAME_LONGITUDE {
                        let text = &longitude_points[index];

                        xml::writer::XmlEvent::Characters(text)
                    } else {
                        xml::writer::XmlEvent::Characters(&text)
                    };

                    writer.write(event)?;
                }

                EndElement { name } => {
                    if name.local_name == Self::TAG_NAME_LONGITUDE {
                        index += 1;
                    }

                    let event = xml::writer::XmlEvent::EndElement {
                        name: Some(name.borrow()),
                    };

                    writer.write(event)?;
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
}
