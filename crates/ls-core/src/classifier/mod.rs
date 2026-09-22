pub mod document;
pub mod download;
pub mod pdf;
pub mod photo;
pub mod text_rules;

use crate::ai::AiBackend;
use crate::models::{Category, Classification, ClassifierKind, FileEntry, FileKind};

/// How much document text goes to the language model. Enough for letterhead,
/// subject and totals, which is where the category shows.
pub const MAX_TEXT_BYTES: usize = 4000;

pub async fn classify_entry(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    match entry.kind {
        FileKind::Photo => photo::classify(entry, ai).await,
        FileKind::Pdf => pdf::classify(entry, ai).await,
        FileKind::Document => document::classify(entry, ai).await,
        FileKind::Archive | FileKind::Installer | FileKind::Unknown | FileKind::Font => {
            download::classify(entry)
        }
        FileKind::Video => Classification::simple(Category::Video, 1.0, &["video"], ClassifierKind::Extension),
        FileKind::Audio => Classification::simple(Category::Audio, 1.0, &["audio"], ClassifierKind::Extension),
        FileKind::Code => Classification {
            subcategory: entry.extension.clone(),
            ..Classification::simple(Category::Code, 1.0, &["code"], ClassifierKind::Extension)
        },
    }
}

/// Cuts at a character boundary. Slicing at a fixed byte index panicked as
/// soon as that index fell inside a multi-byte character such as ä or €.
pub fn truncate_utf8(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Text classification shared by PDFs and plain text files: the language
/// model when it answers, the keyword rules otherwise.
pub async fn classify_text(text: &str, ai: Option<&dyn AiBackend>) -> Classification {
    if text.trim().is_empty() {
        return Classification::unknown(ClassifierKind::Rules);
    }
    if let Some(backend) = ai {
        let excerpt = truncate_utf8(text, MAX_TEXT_BYTES);
        if let Ok(mut c) = backend.classify_text(excerpt, crate::ai::prompts::DOCUMENT_CLASSIFY).await {
            // The model sometimes leaves out what the rules find reliably.
            let rules = text_rules::classify(text);
            c.extracted_date = c.extracted_date.or(rules.extracted_date);
            c.extracted_amount = c.extracted_amount.or(rules.extracted_amount);
            return c;
        }
    }
    text_rules::classify(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_never_splits_a_character() {
        // 3999 ASCII bytes, then "ä" occupies bytes 3999 and 4000.
        let text = format!("{}ä und mehr", "a".repeat(3999));
        let cut = truncate_utf8(&text, MAX_TEXT_BYTES);
        assert_eq!(cut.len(), 3999);
        let euro = "€".repeat(2000);
        assert!(truncate_utf8(&euro, MAX_TEXT_BYTES).len() <= MAX_TEXT_BYTES);
    }
}
