pub mod document;
pub mod download;
pub mod pdf;
pub mod photo;

use crate::ai::AiBackend;
use crate::models::{Classification, ClassifierKind, FileEntry, FileKind};
use anyhow::Result;

pub async fn classify_entry(
    entry: &FileEntry,
    ai: Option<&dyn AiBackend>,
) -> Classification {
    match entry.kind {
        FileKind::Photo => photo::classify(entry, ai).await,
        FileKind::Pdf   => pdf::classify(entry, ai).await,
        FileKind::Document => document::classify(entry, ai).await,
        FileKind::Archive | FileKind::Installer | FileKind::Unknown => {
            download::classify(entry)
        }
        FileKind::Video => crate::models::Classification {
            category: crate::models::Category::Video,
            subcategory: None,
            confidence: 1.0,
            tags: vec!["video".into()],
            extracted_date: None,
            extracted_amount: None,
            extracted_sender: None,
            ai_summary: None,
            classified_by: ClassifierKind::Extension,
        },
        FileKind::Audio => crate::models::Classification {
            category: crate::models::Category::Audio,
            subcategory: None,
            confidence: 1.0,
            tags: vec!["audio".into()],
            extracted_date: None,
            extracted_amount: None,
            extracted_sender: None,
            ai_summary: None,
            classified_by: ClassifierKind::Extension,
        },
        FileKind::Code => crate::models::Classification {
            category: crate::models::Category::Code,
            subcategory: entry.extension.clone(),
            confidence: 1.0,
            tags: vec!["code".into()],
            extracted_date: None,
            extracted_amount: None,
            extracted_sender: None,
            ai_summary: None,
            classified_by: ClassifierKind::Extension,
        },
        FileKind::Font => Classification::unknown(ClassifierKind::Extension),
    }
}
