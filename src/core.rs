pub trait Doc {
    fn convert(document: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub mod gpx;
pub mod kml;
pub mod tcx;
