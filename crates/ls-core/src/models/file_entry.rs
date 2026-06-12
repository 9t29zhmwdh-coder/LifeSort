use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::Classification;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Photo,
    Document,
    Pdf,
    Video,
    Audio,
    Archive,
    Installer,
    Code,
    Font,
    Unknown,
}

impl FileKind {
    pub fn from_mime(mime: &str) -> Self {
        match mime {
            m if m.starts_with("image/") => FileKind::Photo,
            m if m.starts_with("video/") => FileKind::Video,
            m if m.starts_with("audio/") => FileKind::Audio,
            "application/pdf" => FileKind::Pdf,
            "application/zip" | "application/x-tar" | "application/gzip"
            | "application/x-7z-compressed" | "application/x-rar-compressed"
            | "application/x-bzip2" | "application/x-xz" => FileKind::Archive,
            "application/x-msdownload" | "application/x-msi"
            | "application/x-apple-diskimage" | "application/vnd.debian.binary-package" => {
                FileKind::Installer
            }
            m if m.starts_with("text/") => FileKind::Document,
            "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-powerpoint"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            | "application/rtf" => FileKind::Document,
            _ => FileKind::Unknown,
        }
    }

    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "heif" | "tiff" | "bmp"
            | "raw" | "cr2" | "nef" | "arw" | "dng" => FileKind::Photo,
            "mp4" | "mov" | "avi" | "mkv" | "m4v" | "wmv" | "flv" | "webm" => FileKind::Video,
            "mp3" | "flac" | "aac" | "wav" | "ogg" | "m4a" | "opus" => FileKind::Audio,
            "pdf" => FileKind::Pdf,
            "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "xls" | "xlsx" | "csv"
            | "ods" | "ppt" | "pptx" | "odp" => FileKind::Document,
            "zip" | "tar" | "gz" | "bz2" | "7z" | "rar" | "xz" | "zst" => FileKind::Archive,
            "dmg" | "pkg" | "exe" | "msi" | "deb" | "rpm" | "appimage" | "flatpak" => {
                FileKind::Installer
            }
            "rs" | "ts" | "js" | "py" | "go" | "swift" | "kt" | "java" | "c" | "cpp"
            | "h" | "rb" | "php" | "sh" | "bash" => FileKind::Code,
            "ttf" | "otf" | "woff" | "woff2" | "eot" => FileKind::Font,
            _ => FileKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub mime_type: String,
    pub kind: FileKind,
    pub hash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: DateTime<Utc>,
    pub exif_date: Option<DateTime<Utc>>,
    pub dimensions: Option<(u32, u32)>,
    pub classification: Option<Classification>,
    pub tags: Vec<String>,
    pub scan_session_id: String,
    pub duplicate_group_id: Option<String>,
}

impl FileEntry {
    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}
