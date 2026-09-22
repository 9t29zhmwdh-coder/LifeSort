import { create } from 'zustand'

export type Lang = 'en' | 'de'

const STORAGE_KEY = 'lifesort_lang'

/** A stored choice wins; otherwise the system language, English as fallback. */
function initialLang(): Lang {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'en' || stored === 'de') return stored
  return navigator.language.toLowerCase().startsWith('de') ? 'de' : 'en'
}

let currentLang: Lang = initialLang()

export function getLang(): Lang {
  return currentLang
}

interface LangState {
  lang: Lang
  toggle: () => void
}

export const useLangStore = create<LangState>((set) => ({
  lang: currentLang,
  toggle: () => {
    currentLang = currentLang === 'en' ? 'de' : 'en'
    localStorage.setItem(STORAGE_KEY, currentLang)
    set({ lang: currentLang })
  },
}))

const en = {
  navDashboard: 'Overview', navFiles: 'Files', navDuplicates: 'Duplicates',
  navOrganize: 'Organize', navSettings: 'Settings', tagline: 'Sorts files by what is inside them',
  ollamaOnline: 'ready', ollamaOffline: 'offline', ollamaMissing: 'model missing',

  scanning: 'Scanning… ({{n}} files)', scanFolder: 'Scan folder', scanError: 'Scan failed: {{msg}}',
  classifying: 'Classifying… {{done}}/{{total}}', classify: 'Classify',
  aiUnreachable: 'Ollama is not reachable, so photos and documents are classified by rules only. Start Ollama for AI classification.',
  aiMissing: 'Ollama is missing {{models}}. Classifying by rules only until you run: ollama pull {{models}}',
  statFiles: 'Files', statSize: 'Size', statClassified: 'Classified', statDuplicates: 'Duplicate copies',
  byFileType: 'By file type', byCategory: 'By category',
  viewFiles: 'View files', cleanUpDuplicates: 'Clean up duplicates', sortSuggestions: 'Sort suggestions',
  pickFolderPrompt: 'Choose a folder to scan',

  searchPlaceholder: 'Search…', all: 'All', allCategories: 'All categories',
  filesCount: '{{n}} files', noFiles: 'No files', duplicateBadge: 'Duplicate',
  size: 'Size', type: 'Type', modified: 'Modified', dimensions: 'Dimensions', camera: 'Camera',
  classification: 'Classification', category: 'Category', confidence: 'Confidence', source: 'Source',
  sourceAi: 'AI', sourceRules: 'Rules', sourceExtension: 'File type',
  date: 'Date', amount: 'Amount', sender: 'Sender',

  kindPhoto: 'Photo', kindPdf: 'PDF', kindDocument: 'Document', kindVideo: 'Video',
  kindAudio: 'Audio', kindArchive: 'Archive', kindInstaller: 'Installer',
  kindCode: 'Code', kindFont: 'Font', kindUnknown: 'Unknown',

  analyzing: 'Analyzing…', createProposals: 'Create suggestions', classifyFirst: 'Classify the scan first; suggestions need a category per file.',
  executing: 'Moving…', executeAction: 'Move {{n}} file', executeActionsPlural: 'Move {{n}} files',
  pendingCount: '{{pending}} pending · {{applied}} done', selectAll: 'Select all', selectNone: 'Select none',
  noActionsYet: 'Create sort suggestions after scanning', undo: 'Undo', undoFailed: 'Undo failed: {{msg}}',

  findDuplicates: 'Find duplicates', searchingDuplicates: 'Comparing contents…',
  duplicateSummary: '{{n}} groups · {{size}} wasted', duplicateSummaryOne: '1 group · {{size}} wasted',
  noDuplicates: 'No duplicates found',
  perFile: 'per file', wasted: 'wasted', keep: 'Keep this one',
  confirmTrash: 'Keep "{{name}}" and move the {{n}} other copies to the Trash? You can restore them from there.',
  confirmTrashOne: 'Keep "{{name}}" and move the other copy to the Trash? You can restore it from there.',
  confirmTrashTitle: 'Remove duplicates', trashYes: 'Move to Trash', trashNo: 'Cancel',

  settingsTitle: 'Settings', localAiSection: 'Local AI (Ollama)',
  ollamaUrl: 'Ollama URL', textModel: 'Text model (documents)', visionModel: 'Vision model (photos)',
  modelHint: 'Which model fits which Mac is listed in the README.',
  testing: 'Testing…', testConnection: 'Test connection',
  ollamaReachable: 'Ollama ready, both models installed', ollamaUnreachable: 'Ollama not reachable',
  targetFolderSection: 'Target folder', baseDirectory: 'Base directory for sorted files', choose: 'Choose',
  scanOptionsSection: 'Scan options', skipHidden: 'Skip hidden files and folders',
  autoClassify: 'Classify automatically after a scan', autoHash: 'Look for duplicates automatically after a scan',
  defaultRulesSection: 'Folders LifeSort creates',
  folderRulesExample: 'Photos/People · Photos/Places · Photos/Events/{Year}\nPhotos/Screenshots · Photos/Memes · Photos/Other\nDocuments/Invoices/{Year} · Documents/Contracts\nDocuments/Guarantees · Documents/Taxes/{Year} · Documents/Letters\nDownloads/Installers · Downloads/Archives · Downloads/Junk\nMedia/Videos · Media/Audio · Code · Other',
  saved: 'Saved', saveSettings: 'Save settings', saveFailed: 'Saving failed: {{msg}}',

  catPhotoPerson: 'Photo: people', catPhotoLandscape: 'Photo: place', catPhotoEvent: 'Photo: event',
  catPhotoScreenshot: 'Screenshot', catPhotoMeme: 'Meme', catPhotoDocument: 'Photo of a document', catPhotoOther: 'Photo: other',
  catInvoice: 'Invoice', catContract: 'Contract', catGuarantee: 'Guarantee',
  catTaxDocument: 'Tax document', catLetter: 'Letter', catCertificate: 'Certificate', catReport: 'Report',
  catInstallerApp: 'Installer', catDownloadArchive: 'Archive', catDownloadAsset: 'Asset', catDownloadJunk: 'Junk',
  catVideo: 'Video', catAudio: 'Audio', catCode: 'Code', catUnknown: 'Unknown',
}

type TranslationKey = keyof typeof en

const de: Record<TranslationKey, string> = {
  navDashboard: 'Übersicht', navFiles: 'Dateien', navDuplicates: 'Duplikate',
  navOrganize: 'Sortieren', navSettings: 'Einstellungen', tagline: 'Sortiert Dateien nach ihrem Inhalt',
  ollamaOnline: 'bereit', ollamaOffline: 'offline', ollamaMissing: 'Modell fehlt',

  scanning: 'Scanne… ({{n}} Dateien)', scanFolder: 'Ordner scannen', scanError: 'Scan fehlgeschlagen: {{msg}}',
  classifying: 'Klassifiziere… {{done}}/{{total}}', classify: 'Klassifizieren',
  aiUnreachable: 'Ollama ist nicht erreichbar, Fotos und Dokumente werden deshalb nur nach Regeln eingeordnet. Für die KI-Einordnung Ollama starten.',
  aiMissing: 'In Ollama fehlt {{models}}. Bis dahin nur Regeln. Installieren mit: ollama pull {{models}}',
  statFiles: 'Dateien', statSize: 'Grösse', statClassified: 'Klassifiziert', statDuplicates: 'Doppelte Kopien',
  byFileType: 'Nach Dateityp', byCategory: 'Nach Kategorie',
  viewFiles: 'Dateien anzeigen', cleanUpDuplicates: 'Duplikate bereinigen', sortSuggestions: 'Sortier-Vorschläge',
  pickFolderPrompt: 'Wähle einen Ordner zum Scannen',

  searchPlaceholder: 'Suche…', all: 'Alle', allCategories: 'Alle Kategorien',
  filesCount: '{{n}} Dateien', noFiles: 'Keine Dateien', duplicateBadge: 'Duplikat',
  size: 'Grösse', type: 'Typ', modified: 'Geändert', dimensions: 'Abmessungen', camera: 'Kamera',
  classification: 'Einordnung', category: 'Kategorie', confidence: 'Sicherheit', source: 'Quelle',
  sourceAi: 'KI', sourceRules: 'Regeln', sourceExtension: 'Dateityp',
  date: 'Datum', amount: 'Betrag', sender: 'Absender',

  kindPhoto: 'Foto', kindPdf: 'PDF', kindDocument: 'Dokument', kindVideo: 'Video',
  kindAudio: 'Audio', kindArchive: 'Archiv', kindInstaller: 'Installer',
  kindCode: 'Code', kindFont: 'Schrift', kindUnknown: 'Unbekannt',

  analyzing: 'Analysiere…', createProposals: 'Vorschläge erstellen', classifyFirst: 'Zuerst klassifizieren: Vorschläge brauchen eine Kategorie pro Datei.',
  executing: 'Verschiebe…', executeAction: '{{n}} Datei verschieben', executeActionsPlural: '{{n}} Dateien verschieben',
  pendingCount: '{{pending}} ausstehend · {{applied}} erledigt', selectAll: 'Alle auswählen', selectNone: 'Keine auswählen',
  noActionsYet: 'Erstelle Sortier-Vorschläge nach dem Scan', undo: 'Rückgängig', undoFailed: 'Rückgängig fehlgeschlagen: {{msg}}',

  findDuplicates: 'Duplikate suchen', searchingDuplicates: 'Vergleiche Inhalte…',
  duplicateSummary: '{{n}} Gruppen · {{size}} verschwendet', duplicateSummaryOne: '1 Gruppe · {{size}} verschwendet',
  noDuplicates: 'Keine Duplikate gefunden',
  perFile: 'pro Datei', wasted: 'verschwendet', keep: 'Diese behalten',
  confirmTrash: '«{{name}}» behalten und die {{n}} anderen Kopien in den Papierkorb legen? Von dort lassen sie sich wiederherstellen.',
  confirmTrashOne: '«{{name}}» behalten und die andere Kopie in den Papierkorb legen? Von dort lässt sie sich wiederherstellen.',
  confirmTrashTitle: 'Duplikate entfernen', trashYes: 'In den Papierkorb', trashNo: 'Abbrechen',

  settingsTitle: 'Einstellungen', localAiSection: 'Lokale KI (Ollama)',
  ollamaUrl: 'Ollama-URL', textModel: 'Textmodell (Dokumente)', visionModel: 'Vision-Modell (Fotos)',
  modelHint: 'Welches Modell zu welchem Mac passt, steht im README.',
  testing: 'Teste…', testConnection: 'Verbindung testen',
  ollamaReachable: 'Ollama bereit, beide Modelle installiert', ollamaUnreachable: 'Ollama nicht erreichbar',
  targetFolderSection: 'Zielordner', baseDirectory: 'Basisordner für sortierte Dateien', choose: 'Wählen',
  scanOptionsSection: 'Scan-Optionen', skipHidden: 'Versteckte Dateien und Ordner überspringen',
  autoClassify: 'Nach dem Scan automatisch klassifizieren', autoHash: 'Nach dem Scan automatisch Duplikate suchen',
  defaultRulesSection: 'Ordner, die LifeSort anlegt',
  folderRulesExample: 'Fotos/Personen · Fotos/Orte · Fotos/Ereignisse/{Jahr}\nFotos/Screenshots · Fotos/Memes · Fotos/Diverses\nDokumente/Rechnungen/{Jahr} · Dokumente/Vertraege\nDokumente/Garantien · Dokumente/Steuern/{Jahr} · Dokumente/Briefe\nDownloads/Installer · Downloads/Archive · Downloads/Muell\nMedien/Videos · Medien/Audio · Code · Sonstiges',
  saved: 'Gespeichert', saveSettings: 'Einstellungen speichern', saveFailed: 'Speichern fehlgeschlagen: {{msg}}',

  catPhotoPerson: 'Foto: Personen', catPhotoLandscape: 'Foto: Ort', catPhotoEvent: 'Foto: Ereignis',
  catPhotoScreenshot: 'Screenshot', catPhotoMeme: 'Meme', catPhotoDocument: 'Foto eines Dokuments', catPhotoOther: 'Foto: Diverses',
  catInvoice: 'Rechnung', catContract: 'Vertrag', catGuarantee: 'Garantie',
  catTaxDocument: 'Steuerdokument', catLetter: 'Brief', catCertificate: 'Zertifikat', catReport: 'Bericht',
  catInstallerApp: 'Installer', catDownloadArchive: 'Archiv', catDownloadAsset: 'Asset', catDownloadJunk: 'Müll',
  catVideo: 'Video', catAudio: 'Audio', catCode: 'Code', catUnknown: 'Unbekannt',
}

const translations: Record<Lang, Record<TranslationKey, string>> = { en, de }

function interpolate(str: string, vars?: Record<string, string | number>): string {
  if (!vars) return str
  return str.replace(/\{\{(\w+)\}\}/g, (_, key) => String(vars[key] ?? ''))
}

export function t(key: TranslationKey, vars?: Record<string, string | number>): string {
  return interpolate(translations[currentLang][key] ?? key, vars)
}

export function useT() {
  const lang = useLangStore((s) => s.lang)
  return (key: TranslationKey, vars?: Record<string, string | number>) =>
    interpolate(translations[lang][key] ?? key, vars)
}
