use crate::ai::AiBackend;
use crate::models::{Category, Classification, ClassifierKind, FileEntry};
use base64::Engine;

pub async fn classify(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    // Fast heuristic: screen-sized images are likely screenshots
    if let Some((w, h)) = entry.dimensions {
        if is_screenshot_dimensions(w, h) {
            return Classification {
                category: Category::PhotoScreenshot,
                subcategory: None,
                confidence: 0.8,
                tags: vec!["screenshot".into()],
                extracted_date: entry.exif_date.map(|d| d.date_naive()),
                extracted_amount: None,
                extracted_sender: None,
                ai_summary: None,
                classified_by: ClassifierKind::Rules,
            };
        }
    }

    // No EXIF → likely not a camera photo
    let is_camera_photo = entry.exif_date.is_some();

    // Try AI vision if available
    if let Some(backend) = ai {
        if let Ok(bytes) = std::fs::read(&entry.path) {
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes[..bytes.len().min(512_000)]);
            if let Ok(mut c) = backend.classify_image(&b64).await {
                if entry.exif_date.is_some() {
                    c.extracted_date = entry.exif_date.map(|d| d.date_naive());
                }
                return c;
            }
        }
    }

    // Rule-based fallback
    let (category, tags) = if is_camera_photo {
        (Category::PhotoEvent, vec!["foto".into(), "kamera".into()])
    } else {
        (Category::PhotoLandscape, vec!["bild".into()])
    };

    Classification {
        category,
        subcategory: None,
        confidence: 0.4,
        tags,
        extracted_date: entry.exif_date.map(|d| d.date_naive()),
        extracted_amount: None,
        extracted_sender: None,
        ai_summary: None,
        classified_by: ClassifierKind::Rules,
    }
}

fn is_screenshot_dimensions(w: u32, h: u32) -> bool {
    // Common screen resolutions / aspect ratios
    let common_widths = [1920, 1440, 1366, 1280, 2560, 3840, 2880, 1080, 390, 430, 414, 375];
    common_widths.contains(&w) || common_widths.contains(&h)
}
