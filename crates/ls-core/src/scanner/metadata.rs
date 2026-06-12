use chrono::{DateTime, TimeZone, Utc};
use std::path::Path;

pub fn photo_metadata(path: &Path) -> (Option<DateTime<Utc>>, Option<(u32, u32)>) {
    let exif_date = exif_date(path);
    let dims = image_dimensions(path);
    (exif_date, dims)
}

fn exif_date(path: &Path) -> Option<DateTime<Utc>> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(file);
    let exif = kamadak_exif::Reader::new()
        .read_from_container(&mut bufreader)
        .ok()?;

    let field = exif.get_field(kamadak_exif::Tag::DateTimeOriginal, kamadak_exif::In::PRIMARY)?;
    let dt_str = match &field.value {
        kamadak_exif::Value::Ascii(v) => v.first()?.iter().map(|&b| b as char).collect::<String>(),
        _ => return None,
    };
    // EXIF format: "YYYY:MM:DD HH:MM:SS"
    let dt = chrono::NaiveDateTime::parse_from_str(&dt_str, "%Y:%m:%d %H:%M:%S").ok()?;
    Some(Utc.from_utc_datetime(&dt))
}

fn image_dimensions(path: &Path) -> Option<(u32, u32)> {
    image::image_dimensions(path).ok()
}
