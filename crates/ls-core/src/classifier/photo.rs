use crate::ai::AiBackend;
use crate::models::{Category, Classification, ClassifierKind, FileEntry};
use base64::Engine;
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn classify(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    let date = entry.exif_date.map(|d| d.date_naive());
    let tags = messenger_tag(&entry.name);

    if let Some(confidence) = screenshot_confidence(entry) {
        let mut c = Classification::simple(Category::PhotoScreenshot, confidence, &["screenshot"], ClassifierKind::Rules);
        c.extracted_date = date;
        c.tags.extend(tags);
        return c;
    }

    if let Some(backend) = ai {
        let path = entry.path.clone();
        // Decoding a 48-megapixel photo takes a noticeable moment; keep it off
        // the async runtime so progress events keep flowing.
        let jpeg = tokio::task::spawn_blocking(move || crate::imageprep::prepare_for_model(std::path::Path::new(&path)))
            .await
            .ok()
            .flatten();
        if let Some(jpeg) = jpeg {
            let b64 = base64::engine::general_purpose::STANDARD.encode(jpeg);
            if let Ok(mut c) = backend.classify_image(&b64).await {
                c.extracted_date = date;
                c.tags.extend(tags);
                return c;
            }
        }
    }

    let (category, rule_tags): (Category, &[&str]) = if entry.camera.is_some() {
        (Category::PhotoEvent, &["camera"])
    } else {
        (Category::PhotoOther, &["image"])
    };
    let mut c = Classification::simple(category, 0.4, rule_tags, ClassifierKind::Rules);
    c.extracted_date = date;
    c.tags.extend(tags);
    c
}

/// Certain screenshot signals first, then exact screen sizes.
///
/// Size alone is only trusted when no camera wrote EXIF: a camera photo never
/// comes out at exactly 1179 × 2556, but a photo forwarded by a messenger has
/// lost its EXIF and might, which is why that case gets less confidence.
pub fn screenshot_confidence(entry: &FileEntry) -> Option<f32> {
    static NAME: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)(screenshot|screen shot|bildschirmfoto|capture d.écran|schermata)").unwrap());
    if entry.screenshot_marker || NAME.is_match(&entry.name) {
        return Some(0.95);
    }
    let (w, h) = entry.dimensions?;
    let (short, long) = (w.min(h), w.max(h));
    (entry.camera.is_none() && SCREEN_SIZES.contains(&(short, long))).then_some(0.75)
}

/// Native screenshot resolutions in pixels, short side first.
const SCREEN_SIZES: &[(u32, u32)] = &[
    // iPhone
    (640, 1136), (750, 1334), (1242, 2208), (1125, 2436), (828, 1792), (1242, 2688),
    (1080, 2340), (1170, 2532), (1284, 2778), (1179, 2556), (1290, 2796), (1206, 2622),
    (1320, 2868), (1260, 2736),
    // iPad
    (1536, 2048), (1620, 2160), (1640, 2360), (1668, 2224), (1668, 2388), (2048, 2732),
    (1488, 2266), (2064, 2752),
    // Android, common panels
    (1080, 1920), (1080, 2400), (1080, 2412), (1440, 3200), (1440, 3120), (1440, 3088),
    (1220, 2712), (1260, 2800),
    // Mac and PC displays
    (768, 1366), (900, 1440), (800, 1280), (1200, 1920), (1440, 2560),
    (1600, 2560), (1664, 2560), (1800, 2880), (1864, 2880), (1964, 3024), (2234, 3456),
    (1912, 2940), (2224, 3420), (2160, 3840),
];

/// Tags images that came through a messenger, recognisable by the file names
/// WhatsApp and Telegram give them.
fn messenger_tag(name: &str) -> Option<String> {
    static WHATSAPP: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)(-WA\d{4}|^WhatsApp Image )").unwrap());
    static TELEGRAM: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^photo_\d{4}-\d{2}-\d{2}_").unwrap());
    if WHATSAPP.is_match(name) {
        Some("whatsapp".into())
    } else if TELEGRAM.is_match(name) {
        Some("telegram".into())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FileKind;
    use chrono::Utc;

    fn photo(name: &str, dims: (u32, u32), camera: Option<&str>) -> FileEntry {
        FileEntry {
            id: "1".into(),
            path: format!("/nowhere/{name}"),
            name: name.into(),
            extension: None,
            size: 1,
            mime_type: "image/png".into(),
            kind: FileKind::Photo,
            hash: None,
            created_at: None,
            modified_at: Utc::now(),
            exif_date: None,
            dimensions: Some(dims),
            camera: camera.map(String::from),
            screenshot_marker: false,
            classification: None,
            tags: vec![],
            scan_session_id: String::new(),
            duplicate_group_id: None,
        }
    }

    /// The old list held 390, 430 and 375: iPhone sizes in points, not
    /// pixels. No real iPhone screenshot ever matched.
    #[test]
    fn iphone_screenshots_are_recognised_in_pixels() {
        assert!(screenshot_confidence(&photo("IMG_0815.PNG", (1179, 2556), None)).is_some());
        assert!(screenshot_confidence(&photo("IMG_0816.PNG", (2796, 1290), None)).is_some());
    }

    #[test]
    fn camera_photo_with_screen_size_is_not_a_screenshot() {
        assert!(screenshot_confidence(&photo("IMG_1.JPG", (1080, 1920), Some("Apple iPhone 15"))).is_none());
    }

    /// 1080 was on the old list on its own, so every 1080 × 1350 Instagram or
    /// WhatsApp picture counted as a screenshot.
    #[test]
    fn a_single_matching_edge_is_not_enough() {
        assert!(screenshot_confidence(&photo("x.jpg", (1080, 1350), None)).is_none());
        assert!(screenshot_confidence(&photo("x.jpg", (1600, 1200), None)).is_none());
    }

    #[test]
    fn names_and_marker() {
        assert_eq!(screenshot_confidence(&photo("Bildschirmfoto 2024-01-01 um 10.00.00.png", (10, 10), None)), Some(0.95));
        let mut marked = photo("IMG_2.PNG", (10, 10), None);
        marked.screenshot_marker = true;
        assert_eq!(screenshot_confidence(&marked), Some(0.95));
    }

    #[test]
    fn messenger_names() {
        assert_eq!(messenger_tag("IMG-20240101-WA0007.jpg").as_deref(), Some("whatsapp"));
        assert_eq!(messenger_tag("WhatsApp Image 2024-01-01 at 10.00.00.jpeg").as_deref(), Some("whatsapp"));
        assert_eq!(messenger_tag("IMG_0001.HEIC"), None);
    }
}
