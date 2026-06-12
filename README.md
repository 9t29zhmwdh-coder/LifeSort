<p align="center">
  <img src="RayStudio.png" alt="RayStudio" width="120" />
</p>

<h1 align="center">LifeSort</h1>
<p align="center"><strong>AI-powered local file organizer — offline, private, cross-platform</strong></p>
<p align="center">
  <a href="README.de.md">Deutsch</a> ·
  <a href="https://github.com/9t29zhmwdh-coder/LifeSort">GitHub</a> ·
  <a href="LICENSE">MIT License</a>
</p>

---

## What is LifeSort?

LifeSort automatically recognizes, classifies, tags, and sorts files, photos, PDFs and documents into a clean folder structure — **fully offline**, using local AI models.

## Features

| Module | Description |
|---|---|
| **Photo Recognition** | Detects people, places, events, screenshots, memes via vision model |
| **Document Classification** | Invoices, contracts, guarantees, tax documents, letters |
| **PDF Analysis** | Extracts sender, date, amount, document type via OCR + AI |
| **Download Sorting** | Automatically categorizes installers, archives, assets, junk |
| **Smart Tagging** | AI-generated tags per file |
| **Duplicate Detection** | BLAKE3 content hashing, wasted space report |
| **Organization Proposals** | Suggests move actions, shows target path + reason |
| **Plugin System** | Custom file type handlers via Rust trait |

## Folder Structure

```
LifeSort/
├── Fotos/
│   ├── Personen/    Orte/    Ereignisse/{Year}/
│   └── Screenshots/ Diverses/
├── Dokumente/
│   ├── Rechnungen/{Year}/  Vertraege/  Garantien/
│   └── Steuern/{Year}/     Briefe/     Zertifikate/
├── Downloads/
│   ├── Installer/  Archive/  Assets/  Muell/
└── Medien/
    └── Videos/  Audio/
```

## Tech Stack

- **Core** — Rust (walkdir, infer, blake3, kamadak-exif, lopdf, rayon)
- **Desktop** — Tauri v2
- **Frontend** — React, TypeScript, Tailwind CSS, Recharts
- **AI** — Ollama (llama3 for text, llava for vision) — 100% offline

## Getting Started

```bash
# Prerequisites: Rust, Node.js 18+, Ollama (https://ollama.com)
git clone https://github.com/9t29zhmwdh-coder/LifeSort
cd LifeSort

# Pull AI models
ollama pull llama3
ollama pull llava

npm --prefix frontend install
cargo tauri dev
```

## Privacy

LifeSort processes all files **locally on your machine**. No data is uploaded to the cloud. Ollama runs the models entirely offline.

---

<p align="right">
  <sub>by <a href="https://github.com/9t29zhmwdh-coder">RayStudio</a> &nbsp;·&nbsp; MIT License</sub>
  &nbsp;
  <img src="RayStudio.png" alt="" width="70" align="right" />
</p>
