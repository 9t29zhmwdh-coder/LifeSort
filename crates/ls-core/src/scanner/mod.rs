pub mod metadata;

use crate::models::{FileEntry, FileKind};
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct ScanOptions {
    pub max_depth: Option<usize>,
    pub skip_hidden: bool,
    pub min_size: u64,
    pub max_size: Option<u64>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self { max_depth: None, skip_hidden: true, min_size: 0, max_size: None }
    }
}

pub fn scan_directory(
    path: &Path,
    session_id: &str,
    opts: &ScanOptions,
    mut on_file: impl FnMut(FileEntry),
) -> Result<usize> {
    let mut walker = WalkDir::new(path).follow_links(false);
    if let Some(d) = opts.max_depth {
        walker = walker.max_depth(d);
    }

    let mut count = 0;
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if opts.skip_hidden && name.starts_with('.') {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = meta.len();
        if size < opts.min_size {
            continue;
        }
        if let Some(max) = opts.max_size {
            if size > max {
                continue;
            }
        }

        let file_path = entry.path();
        let ext = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase());

        // MIME detection: try magic bytes first, fall back to extension
        let mime_type = detect_mime(file_path, ext.as_deref());
        let kind = if mime_type != "application/octet-stream" {
            FileKind::from_mime(&mime_type)
        } else {
            ext.as_deref().map(FileKind::from_extension).unwrap_or(FileKind::Unknown)
        };

        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t| {
                let secs = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                Utc.timestamp_opt(secs as i64, 0).single()
            })
            .unwrap_or_else(Utc::now);

        let created_at = meta
            .created()
            .ok()
            .and_then(|t| {
                let secs = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                Utc.timestamp_opt(secs as i64, 0).single()
            });

        // EXIF date + dimensions for photos
        let (exif_date, dimensions) = if matches!(kind, FileKind::Photo) {
            metadata::photo_metadata(file_path)
        } else {
            (None, None)
        };

        on_file(FileEntry {
            id: Uuid::new_v4().to_string(),
            path: file_path.to_string_lossy().into_owned(),
            name: entry.file_name().to_string_lossy().into_owned(),
            extension: ext.map(|s| s.to_string()),
            size,
            mime_type,
            kind,
            hash: None,
            created_at,
            modified_at,
            exif_date,
            dimensions,
            classification: None,
            tags: vec![],
            scan_session_id: session_id.to_string(),
            duplicate_group_id: None,
        });
        count += 1;
    }
    Ok(count)
}

fn detect_mime(path: &Path, ext: Option<&str>) -> String {
    // Read first 8 KB for magic-byte detection
    let bytes = std::fs::read(path)
        .ok()
        .map(|b| b[..b.len().min(8192)].to_vec())
        .unwrap_or_default();

    if let Some(kind) = infer::get(&bytes) {
        return kind.mime_type().to_string();
    }
    // Fallback to extension-based guess
    ext.and_then(|e| mime_guess::from_ext(e).first())
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string())
}
