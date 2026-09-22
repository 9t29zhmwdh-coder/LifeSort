use crate::ai::AiBackend;
use crate::models::{Classification, ClassifierKind, FileEntry};
use std::io::Read;

/// Plain text and Markdown. Office formats would need a zip and XML parser
/// each; they are classified as unknown and left where they are.
pub async fn classify(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    match read_text_content(entry) {
        Some(text) => super::classify_text(&text, ai).await,
        None => Classification::unknown(ClassifierKind::Rules),
    }
}

fn read_text_content(entry: &FileEntry) -> Option<String> {
    if !matches!(entry.mime_type.as_str(), "text/plain" | "text/markdown") {
        return None;
    }
    // A 2 GB log file must not end up in memory to classify its first page.
    let mut bytes = Vec::new();
    std::fs::File::open(&entry.path).ok()?.take(64 * 1024).read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}
