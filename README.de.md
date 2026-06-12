<p align="center">
  <img src="RayStudio.png" alt="RayStudio" width="120" />
</p>

<h1 align="center">LifeSort</h1>
<p align="center"><strong>KI-gestützter lokaler Datei-Organizer — offline, privat, plattformübergreifend</strong></p>
<p align="center">
  <a href="README.md">English</a> ·
  <a href="https://github.com/9t29zhmwdh-coder/LifeSort">GitHub</a> ·
  <a href="LICENSE">MIT-Lizenz</a>
</p>

---

## Was ist LifeSort?

LifeSort erkennt, klassifiziert, taggt und sortiert Dateien, Fotos, PDFs und Dokumente automatisch in eine übersichtliche Ordnerstruktur — **vollständig offline**, mit lokalen KI-Modellen.

## Funktionen

| Modul | Beschreibung |
|---|---|
| **Foto-Erkennung** | Erkennt Personen, Orte, Ereignisse, Screenshots, Memes via Vision-Modell |
| **Dokument-Klassifizierung** | Rechnungen, Verträge, Garantien, Steuerunterlagen, Briefe |
| **PDF-Analyse** | Extrahiert Absender, Datum, Betrag, Dokumenttyp via OCR + KI |
| **Download-Sortierung** | Ordnet Installer, Archive, Assets, Müll automatisch ein |
| **Intelligentes Tagging** | KI-generierte Tags pro Datei |
| **Duplikaterkennung** | BLAKE3-Inhalts-Hashing, Bericht über verschwendeten Speicher |
| **Sortier-Vorschläge** | Schlägt Verschiebe-Aktionen vor mit Zielpfad und Begründung |
| **Plugin-System** | Eigene Datei-Typ-Handler über Rust-Trait |

## Ordnerstruktur

```
LifeSort/
├── Fotos/
│   ├── Personen/    Orte/    Ereignisse/{Jahr}/
│   └── Screenshots/ Diverses/
├── Dokumente/
│   ├── Rechnungen/{Jahr}/  Vertraege/  Garantien/
│   └── Steuern/{Jahr}/     Briefe/     Zertifikate/
├── Downloads/
│   ├── Installer/  Archive/  Assets/  Muell/
└── Medien/
    └── Videos/  Audio/
```

## Technologie

- **Core** — Rust (walkdir, infer, blake3, kamadak-exif, lopdf, rayon)
- **Desktop** — Tauri v2
- **Frontend** — React, TypeScript, Tailwind CSS, Recharts
- **KI** — Ollama (llama3 für Text, llava für Bilder) — 100% offline

## Schnellstart

```bash
# Voraussetzungen: Rust, Node.js 18+, Ollama (https://ollama.com)
git clone https://github.com/9t29zhmwdh-coder/LifeSort
cd LifeSort

# KI-Modelle herunterladen
ollama pull llama3
ollama pull llava

npm --prefix frontend install
cargo tauri dev
```

## Datenschutz

LifeSort verarbeitet alle Dateien **lokal auf deinem Gerät**. Es werden keine Daten in die Cloud hochgeladen. Ollama führt die Modelle vollständig offline aus.

---

<p align="right">
  <sub>von <a href="https://github.com/9t29zhmwdh-coder">RayStudio</a> &nbsp;·&nbsp; MIT-Lizenz</sub>
  &nbsp;
  <img src="RayStudio.png" alt="" width="70" align="right" />
</p>
