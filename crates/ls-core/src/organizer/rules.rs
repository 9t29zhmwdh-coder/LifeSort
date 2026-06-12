use crate::models::Category;

/// Human-readable label for folder rules shown in Settings UI.
pub fn default_rules() -> Vec<(Category, &'static str)> {
    vec![
        (Category::PhotoPerson,     "Fotos/Personen"),
        (Category::PhotoLandscape,  "Fotos/Orte"),
        (Category::PhotoEvent,      "Fotos/Ereignisse/{Jahr}"),
        (Category::PhotoScreenshot, "Fotos/Screenshots"),
        (Category::PhotoMeme,       "Fotos/Diverses"),
        (Category::Invoice,         "Dokumente/Rechnungen/{Jahr}"),
        (Category::Contract,        "Dokumente/Vertraege"),
        (Category::Guarantee,       "Dokumente/Garantien"),
        (Category::TaxDocument,     "Dokumente/Steuern/{Jahr}"),
        (Category::Letter,          "Dokumente/Briefe"),
        (Category::Certificate,     "Dokumente/Zertifikate"),
        (Category::Report,          "Dokumente/Berichte"),
        (Category::InstallerApp,    "Downloads/Installer"),
        (Category::DownloadArchive, "Downloads/Archive"),
        (Category::DownloadAsset,   "Downloads/Assets"),
        (Category::DownloadJunk,    "Downloads/Muell"),
        (Category::Video,           "Medien/Videos"),
        (Category::Audio,           "Medien/Audio"),
        (Category::Code,            "Entwicklung"),
        (Category::Unknown,         "Sonstiges"),
    ]
}
