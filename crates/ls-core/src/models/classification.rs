use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    // Photos
    PhotoPerson,
    PhotoLandscape,
    PhotoEvent,
    PhotoScreenshot,
    PhotoMeme,
    PhotoDocument,
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

impl Category {
    /// Returns the relative folder path for this category.
    pub fn folder_path(&self, date: Option<NaiveDate>) -> String {
        let year = date
            .map(|d| d.format("%Y").to_string())
            .unwrap_or_else(|| "Sonstiges".to_string());
        match self {
            Category::PhotoPerson    => "Fotos/Personen".into(),
            Category::PhotoLandscape => "Fotos/Orte".into(),
            Category::PhotoEvent     => format!("Fotos/Ereignisse/{year}"),
            Category::PhotoScreenshot => "Fotos/Screenshots".into(),
            Category::PhotoMeme      => "Fotos/Diverses".into(),
            Category::PhotoDocument  => "Dokumente/Diverses".into(),
            Category::Invoice        => format!("Dokumente/Rechnungen/{year}"),
            Category::Contract       => "Dokumente/Vertraege".into(),
            Category::Guarantee      => "Dokumente/Garantien".into(),
            Category::TaxDocument    => format!("Dokumente/Steuern/{year}"),
            Category::Letter         => "Dokumente/Briefe".into(),
            Category::Certificate    => "Dokumente/Zertifikate".into(),
            Category::Report         => "Dokumente/Berichte".into(),
            Category::InstallerApp   => "Downloads/Installer".into(),
            Category::DownloadArchive => "Downloads/Archive".into(),
            Category::DownloadAsset  => "Downloads/Assets".into(),
            Category::DownloadJunk   => "Downloads/Muell".into(),
            Category::Video          => "Medien/Videos".into(),
            Category::Audio          => "Medien/Audio".into(),
            Category::Code           => "Entwicklung".into(),
            Category::Unknown        => "Sonstiges".into(),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::PhotoPerson    => "Foto — Person",
            Category::PhotoLandscape => "Foto — Ort/Landschaft",
            Category::PhotoEvent     => "Foto — Ereignis",
            Category::PhotoScreenshot => "Screenshot",
            Category::PhotoMeme      => "Meme",
            Category::PhotoDocument  => "Foto — Dokument",
            Category::Invoice        => "Rechnung",
            Category::Contract       => "Vertrag",
            Category::Guarantee      => "Garantie",
            Category::TaxDocument    => "Steuerdokument",
            Category::Letter         => "Brief",
            Category::Certificate    => "Zertifikat",
            Category::Report         => "Bericht",
            Category::InstallerApp   => "Installer",
            Category::DownloadArchive => "Archiv",
            Category::DownloadAsset  => "Asset",
            Category::DownloadJunk   => "Müll",
            Category::Video          => "Video",
            Category::Audio          => "Audio",
            Category::Code           => "Quellcode",
            Category::Unknown        => "Unbekannt",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierKind {
    Rules,
    Ai,
    Ocr,
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
    pub fn unknown(kind: ClassifierKind) -> Self {
        Self {
            category: Category::Unknown,
            subcategory: None,
            confidence: 0.0,
            tags: vec![],
            extracted_date: None,
            extracted_amount: None,
            extracted_sender: None,
            ai_summary: None,
            classified_by: kind,
        }
    }
}
