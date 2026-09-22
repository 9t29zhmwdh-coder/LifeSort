import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { t, type Lang } from './i18n'

// ── Types ────────────────────────────────────────────────────

export type FileKind = 'photo' | 'pdf' | 'document' | 'video' | 'audio' | 'archive' | 'installer' | 'code' | 'font' | 'unknown'

export type Category =
  | 'photo_person' | 'photo_landscape' | 'photo_event' | 'photo_screenshot' | 'photo_meme' | 'photo_document' | 'photo_other'
  | 'invoice' | 'contract' | 'guarantee' | 'tax_document' | 'letter' | 'certificate' | 'report'
  | 'installer_app' | 'download_archive' | 'download_asset' | 'download_junk'
  | 'video' | 'audio' | 'code' | 'unknown'

export interface Classification {
  category: Category
  subcategory?: string
  confidence: number
  tags: string[]
  extracted_date?: string
  extracted_amount?: number
  extracted_sender?: string
  ai_summary?: string
  classified_by: 'rules' | 'ai' | 'extension'
}

export interface FileEntry {
  id: string
  path: string
  name: string
  extension?: string
  size: number
  mime_type: string
  kind: FileKind
  hash?: string
  created_at?: string
  modified_at: string
  exif_date?: string
  dimensions?: [number, number]
  camera?: string
  screenshot_marker: boolean
  classification?: Classification
  tags: string[]
  scan_session_id: string
  duplicate_group_id?: string
}

export interface DuplicateGroup {
  id: string
  hash: string
  size: number
  file_ids: string[]
  keep_id?: string
  total_wasted_bytes: number
}

export type ActionKind = 'move' | 'copy' | 'delete' | 'tag' | 'rename'
export type ActionStatus = 'pending' | 'applied' | 'skipped' | { failed: string }

export interface OrganizeAction {
  id: string
  file_id: string
  file_name: string
  kind: ActionKind
  source_path: string
  target_path?: string
  reason: string
  status: ActionStatus
  undoable: boolean
}

export interface ScanSession {
  id: string
  path: string
  file_count: number
}

export interface ScanDone {
  id: string
  count: number
  error?: string
}

export type AiStatus =
  | { state: 'ready' }
  | { state: 'unreachable' }
  | { state: 'missing_models'; models: string[] }

export interface ClassifyStart {
  total: number
  ai: AiStatus
}

export interface ScanStats {
  total_files: number
  total_size_bytes: number
  by_kind: Record<string, number>
  by_category: Record<string, number>
  classified: number
  duplicate_count: number
  wasted_bytes: number
}

export interface AppSettings {
  ollama_url: string
  text_model: string
  vision_model: string
  target_root: string
  auto_classify: boolean
  auto_hash: boolean
  skip_hidden: boolean
}

// ── API ──────────────────────────────────────────────────────

export const api = {
  scanDirectory:    (path: string) => invoke<ScanSession>('scan_directory', { path }),
  getScanResults:   (sessionId: string) => invoke<FileEntry[]>('get_scan_results', { sessionId }),
  classifyBatch:    (sessionId: string) => invoke<ClassifyStart>('classify_batch', { sessionId }),
  findDuplicates:   (sessionId: string) => invoke<DuplicateGroup[]>('find_duplicates', { sessionId }),
  resolveDuplicate: (sessionId: string, group: DuplicateGroup, keepId: string) =>
    invoke<string[]>('resolve_duplicate', { sessionId, group, keepId }),
  proposeActions:   (sessionId: string, lang: Lang) => invoke<OrganizeAction[]>('propose_actions', { sessionId, lang }),
  executeAction:    (actionId: string) => invoke<OrganizeAction>('execute_action', { actionId }),
  undoAction:       (actionId: string) => invoke<OrganizeAction>('undo_action', { actionId }),
  listActions:      () => invoke<OrganizeAction[]>('list_actions'),
  getSettings:      () => invoke<AppSettings>('get_settings'),
  saveSettings:     (settings: AppSettings) => invoke<void>('save_settings', { settings }),
  checkOllama:      () => invoke<AiStatus>('check_ollama'),
  getStats:         (sessionId: string) => invoke<ScanStats>('get_stats', { sessionId }),
}

// ── Events ───────────────────────────────────────────────────

export const events = {
  onScanProgress:     (cb: (n: number) => void) => listen<number>('scan://progress', e => cb(e.payload)),
  onScanDone:         (cb: (done: ScanDone) => void) => listen<ScanDone>('scan://done', e => cb(e.payload)),
  onClassifyProgress: (cb: (done: number, total: number) => void) =>
    listen<[number, number]>('classify://progress', e => cb(e.payload[0], e.payload[1])),
  onClassifyDone:     (cb: () => void) => listen<number>('classify://done', () => cb()),
}

// ── Helpers ──────────────────────────────────────────────────

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1_073_741_824) return `${(bytes / 1_048_576).toFixed(1)} MB`
  return `${(bytes / 1_073_741_824).toFixed(2)} GB`
}

export function categoryLabel(cat: Category): string {
  const map: Record<Category, string> = {
    photo_person: t('catPhotoPerson'), photo_landscape: t('catPhotoLandscape'),
    photo_event: t('catPhotoEvent'), photo_screenshot: t('catPhotoScreenshot'),
    photo_meme: t('catPhotoMeme'), photo_document: t('catPhotoDocument'), photo_other: t('catPhotoOther'),
    invoice: t('catInvoice'), contract: t('catContract'), guarantee: t('catGuarantee'),
    tax_document: t('catTaxDocument'), letter: t('catLetter'),
    certificate: t('catCertificate'), report: t('catReport'),
    installer_app: t('catInstallerApp'), download_archive: t('catDownloadArchive'),
    download_asset: t('catDownloadAsset'), download_junk: t('catDownloadJunk'),
    video: t('catVideo'), audio: t('catAudio'), code: t('catCode'), unknown: t('catUnknown'),
  }
  return map[cat] ?? cat
}

export function kindLabel(kind: FileKind): string {
  const map: Record<FileKind, string> = {
    photo: t('kindPhoto'), pdf: t('kindPdf'), document: t('kindDocument'), video: t('kindVideo'),
    audio: t('kindAudio'), archive: t('kindArchive'), installer: t('kindInstaller'),
    code: t('kindCode'), font: t('kindFont'), unknown: t('kindUnknown'),
  }
  return map[kind] ?? kind
}

export function kindIcon(kind: FileKind): string {
  const map: Record<FileKind, string> = {
    photo: '🖼', pdf: '📄', document: '📝', video: '🎬',
    audio: '🎵', archive: '📦', installer: '💿', code: '💻',
    font: '🔤', unknown: '📎',
  }
  return map[kind] ?? '📎'
}
