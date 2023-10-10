use std::error::Error;
use xml::reader::ParserConfig;
use xml::EmitterConfig;

use undrift_gps::gcj_to_wgs;

use super::Doc;

use crate::utils::is_coordinates_in_china;

pub struct Document;

impl Document {
    const ATTRIBUTE_NAME_LATITUDE: &'static str = "lat";
    const ATTRIBUTE_NAME_LONGITUDE: &'static str = "lon";

    const TAG_NAME_TRACK_POINT: &'static str = "trkpt";
}

impl Doc for Document {
    fn convert(document: &str) -> std::result::Result<String, Box<dyn Error>> {
        let mut butter: Vec<u8> = vec![];

        let mut reader = ParserConfig::default()
            .ignore_root_level_whitespace(true)
            .ignore_comments(false)
            .cdata_to_characters(true)
            .coalesce_characters(true)
            .create_reader(document.as_bytes());

        let mut writer = EmitterConfig::default().create_writer(&mut butter);

        loop {
            let reader_event = reader.next()?;

            match reader_event {
                xml::reader::XmlEvent::EndDocument => break,
                xml::reader::XmlEvent::StartElement {
                    name,
                    mut attributes,
                    namespace,
                } => {
                    let name = name.borrow();

                    let attributes = if name.local_name == Self::TAG_NAME_TRACK_POINT {
                        let mut latitude = attributes
                            .iter()
                            .find(|attr| {
                                attr.borrow().name.local_name == Self::ATTRIBUTE_NAME_LATITUDE
                            })
                            .unwrap()
                            .borrow()
                            .value
                            .parse::<f64>()?;

                        let mut longitude = attributes
                            .iter()
                            .find(|attr| {
                                attr.borrow().name.local_name == Self::ATTRIBUTE_NAME_LONGITUDE
                            })
                            .unwrap()
                            .borrow()
                            .value
                            .parse::<f64>()?;

                        if is_coordinates_in_china(longitude, latitude) {
                            (latitude, longitude) = gcj_to_wgs(latitude, longitude);
                        }

                        let latitude = latitude.to_string();
                        let longitude = longitude.to_string();

                        attributes
                            .iter_mut()
                            .map(|attr| {
                                if attr.borrow().name.local_name == Self::ATTRIBUTE_NAME_LATITUDE {
                                    attr.value = latitude.clone();
                                } else if attr.borrow().name.local_name
                                    == Self::ATTRIBUTE_NAME_LONGITUDE
                                {
                                    attr.value = longitude.clone();
                                }

                                attr.borrow()
                            })
                            .collect()
                    } else {
                        attributes.iter().map(|attr| attr.borrow()).collect()
                    };

                    let event = xml::writer::XmlEvent::StartElement {
                        name,
                        namespace: namespace.borrow(),
                        attributes,
                    };

                    writer.write(event)?;
                }

                other => {
                    if let Some(writer_event) = other.as_writer_event() {
                        writer.write(writer_event)?;
                    }
                }
            }
        }

        let result = std::str::from_utf8(&butter)?.to_string();

        Ok(result)
    }
}
