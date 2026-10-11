use crate::ai::AiBackend;
use crate::models::{Classification, ClassifierKind, FileEntry};
use lopdf::LoadOptions;

/// PDFs come from arbitrary folders (downloads, mail attachments). Without these
/// limits a small file with nested Flate streams can inflate to gigabytes.
const MAX_PDF_FILE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_DECOMPRESSED_BYTES: usize = 8 * 1024 * 1024;

/// PDFs with a text layer. Scanned PDFs are images without text; LifeSort has
/// no OCR and classifies them as unknown instead of guessing.
pub async fn classify(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    let path = entry.path.clone();
    // Parsing is CPU and memory heavy, keep it off the async runtime threads.
    let text = tokio::task::spawn_blocking(move || extract_pdf_text(&path)).await.ok().flatten();
    match text {
        Some(text) => super::classify_text(&text, ai).await,
        None => Classification {
            tags: vec!["no-text-layer".into()],
            ..Classification::unknown(ClassifierKind::Rules)
        },
    }
}

fn extract_pdf_text(path: &str) -> Option<String> {
    if std::fs::metadata(path).ok()?.len() > MAX_PDF_FILE_BYTES {
        return None;
    }
    let options = LoadOptions::with_max_decompressed_size(MAX_DECOMPRESSED_BYTES);
    let doc = lopdf::Document::load_with_options(path, options).ok()?;
    let pages: Vec<u32> = doc.get_pages().keys().copied().take(5).collect();
    let text = pages
        .iter()
        .filter_map(|p| doc.extract_text_with_limit(&[*p], MAX_DECOMPRESSED_BYTES).ok())
        .collect::<Vec<_>>()
        .join("\n");
    (!text.trim().is_empty()).then_some(text)
}
