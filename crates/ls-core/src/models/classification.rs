use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    // Photos
    PhotoPerson,
    PhotoLandscape,
    PhotoEvent,
    PhotoScreenshot,
    PhotoMeme,
    PhotoDocument,
    PhotoOther,
    // Documents
    Invoice,
    Contract,
    Guarantee,
    TaxDocument,
    Letter,
    Certificate,
    Report,
    // Downloads
    InstallerApp,
    DownloadArchive,
    DownloadAsset,
    DownloadJunk,
    // Media
    Video,
    Audio,
    // Other
    Code,
    Unknown,
}

/// Language of the folder names LifeSort creates on disk.
///
/// Follows the UI language, so a German user gets `Fotos/Screenshots` and an
/// English one `Photos/Screenshots`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FolderLang {
    #[default]
    En,
    De,
}

impl Category {
    /// Returns the relative folder path for this category.
    pub fn folder_path(&self, date: Option<NaiveDate>, lang: FolderLang) -> String {
        let (template, no_year) = match lang {
            FolderLang::En => (self.folder_en(), "Undated"),
            FolderLang::De => (self.folder_de(), "Ohne Datum"),
        };
        let year = date
            .map(|d| d.format("%Y").to_string())
            .unwrap_or_else(|| no_year.to_string());
        template.replace("{year}", &year)
    }

    fn folder_en(&self) -> &'static str {
        match self {
            Category::PhotoPerson => "Photos/People",
            Category::PhotoLandscape => "Photos/Places",
            Category::PhotoEvent => "Photos/Events/{year}",
            Category::PhotoScreenshot => "Photos/Screenshots",
            Category::PhotoMeme => "Photos/Memes",
            Category::PhotoDocument => "Photos/Documents",
            Category::PhotoOther => "Photos/Other",
            Category::Invoice => "Documents/Invoices/{year}",
            Category::Contract => "Documents/Contracts",
            Category::Guarantee => "Documents/Guarantees",
            Category::TaxDocument => "Documents/Taxes/{year}",
            Category::Letter => "Documents/Letters",
            Category::Certificate => "Documents/Certificates",
            Category::Report => "Documents/Reports",
            Category::InstallerApp => "Downloads/Installers",
            Category::DownloadArchive => "Downloads/Archives",
            Category::DownloadAsset => "Downloads/Assets",
            Category::DownloadJunk => "Downloads/Junk",
            Category::Video => "Media/Videos",
            Category::Audio => "Media/Audio",
            Category::Code => "Code",
            Category::Unknown => "Other",
        }
    }

    fn folder_de(&self) -> &'static str {
        match self {
            Category::PhotoPerson => "Fotos/Personen",
            Category::PhotoLandscape => "Fotos/Orte",
            Category::PhotoEvent => "Fotos/Ereignisse/{year}",
            Category::PhotoScreenshot => "Fotos/Screenshots",
            Category::PhotoMeme => "Fotos/Memes",
            Category::PhotoDocument => "Fotos/Dokumente",
            Category::PhotoOther => "Fotos/Diverses",
            Category::Invoice => "Dokumente/Rechnungen/{year}",
            Category::Contract => "Dokumente/Vertraege",
            Category::Guarantee => "Dokumente/Garantien",
            Category::TaxDocument => "Dokumente/Steuern/{year}",
            Category::Letter => "Dokumente/Briefe",
            Category::Certificate => "Dokumente/Zertifikate",
            Category::Report => "Dokumente/Berichte",
            Category::InstallerApp => "Downloads/Installer",
            Category::DownloadArchive => "Downloads/Archive",
            Category::DownloadAsset => "Downloads/Assets",
            Category::DownloadJunk => "Downloads/Muell",
            Category::Video => "Medien/Videos",
            Category::Audio => "Medien/Audio",
            Category::Code => "Code",
            Category::Unknown => "Sonstiges",
        }
    }

    /// Stable snake_case key, identical to the serde representation. The UI
    /// translates it, so the backend never ships display text.
    pub fn key(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "unknown".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierKind {
    Rules,
    Ai,
    Extension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub category: Category,
    pub subcategory: Option<String>,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub extracted_date: Option<NaiveDate>,
    pub extracted_amount: Option<f64>,
    pub extracted_sender: Option<String>,
    pub ai_summary: Option<String>,
    pub classified_by: ClassifierKind,
}

impl Classification {
    /// A classification with only the category, confidence, tags and source
    /// set; everything a document parser would extract stays empty.
    pub fn simple(category: Category, confidence: f32, tags: &[&str], by: ClassifierKind) -> Self {
        Self {
            category,
            subcategory: None,
            confidence,
            tags: tags.iter().map(|t| t.to_string()).collect(),
            extracted_date: None,
            extracted_amount: None,
            extracted_sender: None,
            ai_summary: None,
            classified_by: by,
        }
    }

    pub fn unknown(kind: ClassifierKind) -> Self {
        Self::simple(Category::Unknown, 0.0, &[], kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_names_follow_the_language() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 1);
        assert_eq!(Category::Invoice.folder_path(date, FolderLang::En), "Documents/Invoices/2024");
        assert_eq!(Category::Invoice.folder_path(date, FolderLang::De), "Dokumente/Rechnungen/2024");
        assert_eq!(Category::PhotoEvent.folder_path(None, FolderLang::De), "Fotos/Ereignisse/Ohne Datum");
    }

    #[test]
    fn key_matches_serde() {
        assert_eq!(Category::PhotoScreenshot.key(), "photo_screenshot");
        assert_eq!(Category::TaxDocument.key(), "tax_document");
    }
}
