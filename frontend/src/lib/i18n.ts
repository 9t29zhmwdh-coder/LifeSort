import { create } from 'zustand'

export type Lang = 'en' | 'de'

const STORAGE_KEY = 'lifesort_lang'

let currentLang: Lang = (localStorage.getItem(STORAGE_KEY) as Lang) || 'en'

export function getLang(): Lang {
  return currentLang
}

interface LangState {
  lang: Lang
  setLang: (l: Lang) => void
  toggle: () => void
}

export const useLangStore = create<LangState>((set) => ({
  lang: currentLang,
  setLang: (l) => {
    currentLang = l
    localStorage.setItem(STORAGE_KEY, l)
    set({ lang: l })
  },
  toggle: () => {
    const next: Lang = currentLang === 'en' ? 'de' : 'en'
    currentLang = next
    localStorage.setItem(STORAGE_KEY, next)
    set({ lang: next })
  },
}))

const translations = {
  en: {
    navDashboard: 'Overview', navFiles: 'Files', navDuplicates: 'Duplicates',
    navOrganize: 'Organize', navSettings: 'Settings',
    ollamaOnline: 'online', ollamaOffline: 'offline',

    scanning: 'Scanning… ({{n}} files)', scanFolder: 'Scan folder',
    aiClassifying: 'AI classifying… {{done}}/{{total}}', classifyWithAi: 'Classify with AI',
    statFiles: 'Files', statSize: 'Size', statClassified: 'Classified', statDuplicates: 'Duplicates',
    byFileType: 'By file type', byCategory: 'By category',
    viewFiles: 'View files', cleanUpDuplicates: 'Clean up duplicates', sortSuggestions: 'Sort suggestions',
    pickFolderPrompt: 'Choose a folder to scan',

    searchPlaceholder: 'Search…', all: 'All', allCategories: 'All categories',
    filesCount: '{{n}} files', noFiles: 'No files', duplicateBadge: 'Duplicate',
    size: 'Size', type: 'Type', modified: 'Modified', dimensions: 'Dimensions',
    classification: 'Classification', category: 'Category', confidence: 'Confidence',
    date: 'Date', amount: 'Amount', sender: 'Sender',

    kindPhoto: 'Photo', kindPdf: 'PDF', kindDocument: 'Document', kindVideo: 'Video',
    kindAudio: 'Audio', kindArchive: 'Archive', kindInstaller: 'Installer',
    kindCode: 'Code', kindFont: 'Font', kindUnknown: 'Unknown',

    analyzing: 'Analyzing…', createProposals: 'Create suggestions',
    executing: 'Executing…', executeAction: 'Execute {{n}} action', executeActionsPlural: 'Execute {{n}} actions',
    pendingCount: '{{pending}} pending · {{applied}} done', selectAll: 'Select all',
    noActionsYet: 'Create sort suggestions after scanning', undo: 'Undo',

    settingsTitle: 'Settings', localAiSection: 'Local AI (Ollama)',
    ollamaUrl: 'Ollama URL', textModel: 'Text model', visionModel: 'Vision model (for images)',
    testing: 'Testing…', testConnection: 'Test connection',
    ollamaReachable: 'Ollama reachable', ollamaUnreachable: 'Ollama not reachable',
    targetFolderSection: 'Target folder', baseDirectory: 'Base directory for sorted files', choose: 'Choose',
    defaultSubfolders: 'Default subfolders: Photos/, Documents/, Downloads/, Media/…',
    scanOptionsSection: 'Scan options', skipHidden: 'Skip hidden files',
    autoClassify: 'Automatically classify after scan', autoHash: 'Automatically hash (for duplicate detection)',
    defaultRulesSection: 'Default folder rules',
    folderRulesExample: 'Photos/Person · Photos/Place · Photos/Event/{Year}\nPhotos/Screenshots · Photos/Misc\nDocuments/Invoices/{Year} · Documents/Contracts\nDocuments/Guarantees · Documents/Taxes/{Year}\nDownloads/Installers · Downloads/Archives · Downloads/Junk\nMedia/Videos · Media/Audio · Code · Other',
    saved: 'Saved!', saveSettings: 'Save settings',

    catPhotoPerson: 'Photo: person', catPhotoLandscape: 'Photo: place', catPhotoEvent: 'Photo: event',
    catPhotoScreenshot: 'Screenshot', catPhotoMeme: 'Meme', catPhotoDocument: 'Photo: document',
    catInvoice: 'Invoice', catContract: 'Contract', catGuarantee: 'Guarantee',
    catTaxDocument: 'Tax document', catLetter: 'Letter', catCertificate: 'Certificate', catReport: 'Report',
    catInstallerApp: 'Installer', catDownloadArchive: 'Archive', catDownloadAsset: 'Asset', catDownloadJunk: 'Junk',
    catVideo: 'Video', catAudio: 'Audio', catCode: 'Code', catUnknown: 'Unknown',
  },
  de: {
    navDashboard: 'Übersicht', navFiles: 'Dateien', navDuplicates: 'Duplikate',
    navOrganize: 'Sortieren', navSettings: 'Einstellungen',
    ollamaOnline: 'online', ollamaOffline: 'offline',

    scanning: 'Scanne… ({{n}} Dateien)', scanFolder: 'Ordner scannen',
    aiClassifying: 'AI klassifiziert… {{done}}/{{total}}', classifyWithAi: 'Mit AI klassifizieren',
    statFiles: 'Dateien', statSize: 'Grösse', statClassified: 'Klassifiziert', statDuplicates: 'Duplikate',
    byFileType: 'Nach Dateityp', byCategory: 'Nach Kategorie',
    viewFiles: 'Dateien anzeigen', cleanUpDuplicates: 'Duplikate bereinigen', sortSuggestions: 'Sortier-Vorschläge',
    pickFolderPrompt: 'Wähle einen Ordner zum Scannen',

    searchPlaceholder: 'Suche…', all: 'Alle', allCategories: 'Alle Kategorien',
    filesCount: '{{n}} Dateien', noFiles: 'Keine Dateien', duplicateBadge: 'Duplikat',
    size: 'Grösse', type: 'Typ', modified: 'Geändert', dimensions: 'Abmessungen',
    classification: 'Klassifizierung', category: 'Kategorie', confidence: 'Konfidenz',
    date: 'Datum', amount: 'Betrag', sender: 'Absender',

    kindPhoto: 'Foto', kindPdf: 'PDF', kindDocument: 'Dokument', kindVideo: 'Video',
    kindAudio: 'Audio', kindArchive: 'Archiv', kindInstaller: 'Installer',
    kindCode: 'Code', kindFont: 'Font', kindUnknown: 'Unbekannt',

    analyzing: 'Analysiere…', createProposals: 'Vorschläge erstellen',
    executing: 'Führe aus…', executeAction: '{{n}} Aktion ausführen', executeActionsPlural: '{{n}} Aktionen ausführen',
    pendingCount: '{{pending}} ausstehend · {{applied}} erledigt', selectAll: 'Alle auswählen',
    noActionsYet: 'Erstelle Sortier-Vorschläge nach dem Scan', undo: 'Rückgängig',

    settingsTitle: 'Einstellungen', localAiSection: 'Lokale AI (Ollama)',
    ollamaUrl: 'Ollama URL', textModel: 'Text-Modell', visionModel: 'Vision-Modell (für Bilder)',
    testing: 'Teste…', testConnection: 'Verbindung testen',
    ollamaReachable: 'Ollama erreichbar', ollamaUnreachable: 'Ollama nicht erreichbar',
    targetFolderSection: 'Zielordner', baseDirectory: 'Basis-Verzeichnis für sortierte Dateien', choose: 'Wählen',
    defaultSubfolders: 'Standard-Unterordner: Fotos/, Dokumente/, Downloads/, Medien/…',
    scanOptionsSection: 'Scan-Optionen', skipHidden: 'Versteckte Dateien überspringen',
    autoClassify: 'Automatisch klassifizieren nach Scan', autoHash: 'Automatisch hashen (für Duplikaterkennung)',
    defaultRulesSection: 'Standard-Ordner-Regeln',
    folderRulesExample: 'Fotos/Personen · Fotos/Orte · Fotos/Ereignisse/{Jahr}\nFotos/Screenshots · Fotos/Diverses\nDokumente/Rechnungen/{Jahr} · Dokumente/Verträge\nDokumente/Garantien · Dokumente/Steuern/{Jahr}\nDownloads/Installer · Downloads/Archive · Downloads/Müll\nMedien/Videos · Medien/Audio · Entwicklung · Sonstiges',
    saved: 'Gespeichert!', saveSettings: 'Einstellungen speichern',

    catPhotoPerson: 'Foto: Person', catPhotoLandscape: 'Foto: Ort', catPhotoEvent: 'Foto: Ereignis',
    catPhotoScreenshot: 'Screenshot', catPhotoMeme: 'Meme', catPhotoDocument: 'Foto: Dokument',
    catInvoice: 'Rechnung', catContract: 'Vertrag', catGuarantee: 'Garantie',
    catTaxDocument: 'Steuerdokument', catLetter: 'Brief', catCertificate: 'Zertifikat', catReport: 'Bericht',
    catInstallerApp: 'Installer', catDownloadArchive: 'Archiv', catDownloadAsset: 'Asset', catDownloadJunk: 'Müll',
    catVideo: 'Video', catAudio: 'Audio', catCode: 'Code', catUnknown: 'Unbekannt',
  },
} as const

type TranslationKey = keyof typeof translations.en

function interpolate(str: string, vars?: Record<string, string | number>): string {
  if (!vars) return str
  return str.replace(/\{\{(\w+)\}\}/g, (_, key) => String(vars[key] ?? ''))
}

export function t(key: TranslationKey, vars?: Record<string, string | number>): string {
  const str = translations[currentLang][key] ?? key
  return interpolate(str, vars)
}

export function useT() {
  const lang = useLangStore((s) => s.lang)
  return (key: TranslationKey, vars?: Record<string, string | number>) => {
    const str = translations[lang][key] ?? key
    return interpolate(str, vars)
  }
}
