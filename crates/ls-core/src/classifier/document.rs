use crate::ai::AiBackend;
use crate::ai::prompts::DOCUMENT_CLASSIFY;
use crate::models::{Classification, ClassifierKind, FileEntry};

pub async fn classify(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    let text = read_text_content(entry);

    if let (Some(backend), Some(ref text)) = (ai, &text) {
        if !text.trim().is_empty() {
            let truncated = &text[..text.len().min(4000)];
            if let Ok(c) = backend.classify_text(truncated, DOCUMENT_CLASSIFY).await {
                return c;
            }
        }
    }

    if let Some(ref text) = text {
        return crate::classifier::pdf::rule_classify_text_pub(text);
    }

    Classification::unknown(ClassifierKind::Rules)
}

fn read_text_content(entry: &FileEntry) -> Option<String> {
    match entry.mime_type.as_str() {
        "text/plain" | "text/markdown" => {
            std::fs::read_to_string(&entry.path).ok()
        }
        _ => None,
    }
}
