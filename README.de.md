<div align="center">
  <img src="RayStudio.png" alt="RayStudio Logo" width="120"/>

  <h1>LifeSort</h1>
</div>

[🇬🇧 English Version](README.md)

**KI-gestützter lokaler Datei-Organizer. Offline, privat, plattformübergreifend, entwickelt mit Rust und Tauri.**

LifeSort erkennt, klassifiziert, taggt und sortiert Dateien, Fotos, PDFs und Dokumente automatisch in eine übersichtliche Ordnerstruktur; **vollständig offline**, mit lokalen KI-Modellen. Keine Cloud, kein Tracking, kein manuelles Sortieren.

[![CI](https://github.com/9t29zhmwdh-coder/LifeSort/actions/workflows/ci.yml/badge.svg)](https://github.com/9t29zhmwdh-coder/LifeSort/actions) ![Platform](https://img.shields.io/badge/Platform-macOS_%7C_Windows-lightgrey) ![Rust](https://img.shields.io/badge/Rust-CE422B?logo=rust&logoColor=white) ![Tauri](https://img.shields.io/badge/Tauri-24C8D8?logo=tauri&logoColor=white) ![AI | Claude Code](https://img.shields.io/badge/AI-Claude_Code-black?logo=anthropic&logoColor=white) ![AI | Copilot](https://img.shields.io/badge/AI-Copilot-black?logo=github&logoColor=white)
![Plattform](https://img.shields.io/badge/Plattform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![Lizenz](https://img.shields.io/badge/Lizenz-MIT-green)

---

## Funktionen

| Funktion | Beschreibung |
|---|---|
| **Foto-Erkennung** | Erkennt Personen, Orte, Ereignisse, Screenshots, Memes via Vision-Modell |
| **Dokument-Klassifizierung** | Rechnungen, Verträge, Garantien, Steuerunterlagen, Briefe |
| **PDF-Analyse** | Extrahiert Absender, Datum, Betrag, Dokumenttyp via OCR + KI |
| **Download-Sortierung** | Ordnet Installer, Archive, Assets und Müll automatisch ein |
| **Intelligentes Tagging** | KI-generierte Tags pro Datei |
| **Duplikaterkennung** | BLAKE3-Inhalts-Hashing, Bericht über verschwendeten Speicher |
| **Sortier-Vorschläge** | Schlägt Verschiebe-Aktionen mit Zielpfad und Begründung vor: Nutzer bestätigt |
| **Plugin-System** | Eigene Datei-Typ-Handler über Rust-Trait |

---

## Voraussetzungen

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 20+
- [Tauri CLI v2](https://tauri.app/): `cargo install tauri-cli`
- [Ollama](https://ollama.ai): `ollama pull llama3 && ollama pull llava`
- macOS / Windows / Linux

---

## Schnellstart

```bash
git clone https://github.com/9t29zhmwdh-coder/LifeSort
cd LifeSort

ollama pull llama3
ollama pull llava

cd frontend && npm install && cd ..
cargo tauri dev
```

---

## Datenschutz

LifeSort verarbeitet alle Dateien **lokal auf deinem Gerät**. Es werden keine Daten in die Cloud hochgeladen. Ollama führt die Modelle vollständig offline aus; deine Dateien verlassen dein Gerät nie.

---

## Architektur

```
LifeSort/
├── crates/ls-core/      — Rust: Scanner, Klassifizierung, Tagging, DB
├── crates/ls-cli/       — CLI-Binary
├── src-tauri/           — Tauri v2 Backend + IPC-Commands
└── frontend/            — React + TypeScript + Tailwind + Recharts
```

### Ziel-Ordnerstruktur

```
LifeSort/
├── Fotos/       Personen/  Orte/  Ereignisse/{Jahr}/  Screenshots/
├── Dokumente/   Rechnungen/{Jahr}/  Verträge/  Steuern/{Jahr}/
├── Downloads/   Installer/  Archive/  Assets/  Müll/
└── Medien/      Videos/  Audio/
```

---

**Autor:** [Rafael Yilmaz](https://github.com/9t29zhmwdh-coder) · **Status:** Active · v0.1.0 · **Lizenz:** MIT
