use crate::models::{Category, Classification, ClassifierKind, FileEntry, FileKind};
use once_cell::sync::Lazy;
use regex::Regex;

static JUNK_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(temp|tmp|cache|thumbs|desktop\.ini|\.ds_store|\.bak|~\d+|copy\s+of|kopie\s+von)").unwrap()
});
static ASSET_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\.(svg|ico|png|jpg|webp|woff|ttf|otf|eot|css|json|xml|yaml|yml)$").unwrap()
});

pub fn classify(entry: &FileEntry) -> Classification {
    let name_lower = entry.name.to_lowercase();

    // Explicit Installer by kind
    if matches!(entry.kind, FileKind::Installer) {
        return Classification {
            category: Category::InstallerApp,
            subcategory: entry.extension.clone(),
            confidence: 0.95,
            tags: vec!["installer".into()],
            ..base()
        };
    }
    if matches!(entry.kind, FileKind::Archive) {
        // Check if it looks like an asset bundle or just a plain archive
        return Classification {
            category: Category::DownloadArchive,
            subcategory: entry.extension.clone(),
            confidence: 0.9,
            tags: vec!["archiv".into()],
            ..base()
        };
    }

    // Junk detection
    if JUNK_PATTERNS.is_match(&entry.name) || entry.name.starts_with("._") {
        return Classification {
            category: Category::DownloadJunk,
            confidence: 0.8,
            tags: vec!["junk".into()],
            ..base()
        };
    }

    // Very small files that are likely cruft
    if entry.size < 1024 && entry.extension.as_deref().map(|e| ["txt","log","tmp"].contains(&e)).unwrap_or(false) {
        return Classification {
            category: Category::DownloadJunk,
            confidence: 0.6,
            tags: vec!["klein".into()],
            ..base()
        };
    }

    // Asset detection
    if ASSET_PATTERNS.is_match(&entry.name) {
        return Classification {
            category: Category::DownloadAsset,
            confidence: 0.7,
            tags: vec!["asset".into()],
            ..base()
        };
    }

    Classification {
        category: Category::Unknown,
        confidence: 0.3,
        tags: vec![],
        ..base()
    }
}

fn base() -> Classification {
    Classification {
        category: Category::Unknown,
        subcategory: None,
        confidence: 0.5,
        tags: vec![],
        extracted_date: None,
        extracted_amount: None,
        extracted_sender: None,
        ai_summary: None,
        classified_by: ClassifierKind::Rules,
    }
}
