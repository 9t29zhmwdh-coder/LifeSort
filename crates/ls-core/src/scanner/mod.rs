#![allow(unused_imports)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn schreibe(name: &str, inhalt: &[u8]) -> std::path::PathBuf {
        let verzeichnis = std::env::temp_dir().join(format!("ls-mime-{}", std::process::id()));
        std::fs::create_dir_all(&verzeichnis).unwrap();
        let pfad = verzeichnis.join(name);
        std::fs::File::create(&pfad).unwrap().write_all(inhalt).unwrap();
        pfad
    }

    /// Haelt fest, welchen MIME-Typ `infer` aus den Magic Bytes ableitet.
    ///
    /// Dieser Wert entscheidet, in welchen Ordner eine Datei einsortiert wird.
    /// Meldet eine neue Version einen anderen Typ oder gar keinen mehr, landen
    /// Dateien am falschen Ort. Das ist kein Absturz und faellt beim Bauen
    /// nicht auf, sondern erst, wenn jemand seine Bilder sucht.
    #[test]
    fn magic_bytes_ergeben_denselben_mime_typ() {
        // PNG: 89 50 4E 47 0D 0A 1A 0A
        let png = schreibe(
            "probe.png",
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13],
        );
        assert_eq!(detect_mime(&png, Some("png")), "image/png");

        // JPEG: FF D8 FF
        let jpg = schreibe("probe.jpg", &[0xFF, 0xD8, 0xFF, 0xE0, 0, 16, 0x4A, 0x46]);
        assert_eq!(detect_mime(&jpg, Some("jpg")), "image/jpeg");

        // PDF: %PDF-
        let pdf = schreibe("probe.pdf", b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");
        assert_eq!(detect_mime(&pdf, Some("pdf")), "application/pdf");

        // ZIP: PK\x03\x04
        let zip = schreibe("probe.zip", &[0x50, 0x4B, 0x03, 0x04, 0, 0, 0, 0]);
        assert_eq!(detect_mime(&zip, Some("zip")), "application/zip");
    }

    /// Ohne erkennbare Magic Bytes greift die Endung. Auch dieser Weg muss
    /// erhalten bleiben, sonst bekommt jede Textdatei den Sammeltyp.
    #[test]
    fn ohne_magic_bytes_entscheidet_die_endung() {
        let txt = schreibe("probe.txt", b"nur Text, keine Signatur\n");
        assert_eq!(detect_mime(&txt, Some("txt")), "text/plain");

        let unbekannt = schreibe("probe.xyzzy", b"weder Signatur noch bekannte Endung\n");
        assert_eq!(
            detect_mime(&unbekannt, Some("xyzzy")),
            "application/octet-stream"
        );
    }
}
