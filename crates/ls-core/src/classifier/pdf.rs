use crate::ai::AiBackend;
use crate::models::{Classification, ClassifierKind, FileEntry};

/// PDFs with a text layer. Scanned PDFs are images without text; LifeSort has
/// no OCR and classifies them as unknown instead of guessing.
pub async fn classify(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    match extract_pdf_text(&entry.path) {
        Some(text) => super::classify_text(&text, ai).await,
        None => Classification {
            tags: vec!["no-text-layer".into()],
            ..Classification::unknown(ClassifierKind::Rules)
        },
    }
}

fn extract_pdf_text(path: &str) -> Option<String> {
    let doc = lopdf::Document::load(path).ok()?;
    let pages: Vec<u32> = doc.get_pages().keys().copied().take(5).collect();
    let text = pages
        .iter()
        .filter_map(|p| doc.extract_text(&[*p]).ok())
        .collect::<Vec<_>>()
        .join("\n");
    (!text.trim().is_empty()).then_some(text)
}
