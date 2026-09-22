use chrono::{DateTime, TimeZone, Utc};
use kamadak_exif::{In, Reader, Tag, Value};
use std::path::Path;

#[derive(Debug, Default)]
pub struct PhotoMeta {
    pub exif_date: Option<DateTime<Utc>>,
    pub dimensions: Option<(u32, u32)>,
    pub camera: Option<String>,
    pub screenshot_marker: bool,
}

pub fn photo_metadata(path: &Path) -> PhotoMeta {
    let mut meta = read_exif(path).unwrap_or_default();
    meta.dimensions = crate::imageprep::dimensions(path);
    meta
}

/// kamadak-exif reads JPEG, TIFF, PNG, WebP and HEIF containers, so this
/// covers iPhone HEIC without the sips detour.
fn read_exif(path: &Path) -> Option<PhotoMeta> {
    let file = std::fs::File::open(path).ok()?;
    let exif = Reader::new()
        .read_from_container(&mut std::io::BufReader::new(file))
        .ok()?;
    let text = |tag| {
        exif.get_field(tag, In::PRIMARY)
            .map(|f| f.display_value().to_string().trim_matches('"').trim().to_string())
            .filter(|s| !s.is_empty())
    };

    let exif_date = exif
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .and_then(|f| match &f.value {
            Value::Ascii(v) => v.first().map(|b| String::from_utf8_lossy(b).into_owned()),
            _ => None,
        })
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S").ok())
        .map(|dt| Utc.from_utc_datetime(&dt));

    let camera = match (text(Tag::Make), text(Tag::Model)) {
        (Some(make), Some(model)) if model.starts_with(&make) => Some(model),
        (Some(make), Some(model)) => Some(format!("{make} {model}")),
        (make, model) => make.or(model),
    };

    let screenshot_marker = exif
        .get_field(Tag::UserComment, In::PRIMARY)
        .map(|f| match &f.value {
            Value::Undefined(bytes, _) => String::from_utf8_lossy(bytes).into_owned(),
            other => format!("{}", other.display_as(Tag::UserComment)),
        })
        .is_some_and(|c| c.to_lowercase().contains("screenshot"));

    Some(PhotoMeta { exif_date, dimensions: None, camera, screenshot_marker })
}
