import { create } from 'zustand'
import { api, type AiStatus, type FileEntry, type ScanDone, type ScanSession, type ScanStats, type DuplicateGroup } from '../lib/tauri'
import { useSettingsStore } from './settingsStore'

interface ScanStore {
  session?: ScanSession
  entries: FileEntry[]
  stats?: ScanStats
  duplicates: DuplicateGroup[]
  scanning: boolean
  classifying: boolean
  hashing: boolean
  progress: number
  classifyProgress: [number, number]
  /** AI state of the last classification run; shown so a silent rules-only run cannot happen. */
  lastRunAi?: AiStatus
  error: string
  /** A done event that arrived before scanDirectory returned its session id. */
  earlyDone?: ScanDone
  set: (patch: Partial<ScanStore>) => void
  /** Reloads entries and stats of the current session from the backend. */
  refresh: () => Promise<void>
  startClassify: () => Promise<void>
  startScan: (path: string) => Promise<void>
  scanFinished: (done: ScanDone) => Promise<void>
  findDuplicates: () => Promise<void>
}

export const useScanStore = create<ScanStore>((set, get) => ({
  entries: [],
  duplicates: [],
  scanning: false,
  classifying: false,
  hashing: false,
  progress: 0,
  classifyProgress: [0, 0],
  error: '',
  set: (patch) => set(patch),
  refresh: async () => {
    const session = get().session
    if (!session) return
    const [entries, stats] = await Promise.all([api.getScanResults(session.id), api.getStats(session.id)])
    set({ entries, stats })
  },
  startClassify: async () => {
    const session = get().session
    if (!session || get().classifying) return
    const start = await api.classifyBatch(session.id)
    set({ lastRunAi: start.ai, classifying: start.total > 0, classifyProgress: [0, start.total] })
  },
  startScan: async (path) => {
    set({ scanning: true, progress: 0, error: '', entries: [], stats: undefined, duplicates: [], earlyDone: undefined })
    try {
      const session = await api.scanDirectory(path)
      set({ session })
      // A small folder can finish before this await returns.
      const early = get().earlyDone
      if (early?.id === session.id) await get().scanFinished(early)
    } catch (e) {
      set({ scanning: false, error: String(e) })
    }
  },
  scanFinished: async (done) => {
    if (get().session?.id !== done.id) {
      set({ earlyDone: done })
      return
    }
    set({ scanning: false, progress: done.count, error: done.error ?? '', earlyDone: undefined })
    await get().refresh()
    const prefs = useSettingsStore.getState().settings
    if (prefs?.auto_hash) await get().findDuplicates()
    if (prefs?.auto_classify) await get().startClassify()
  },
  findDuplicates: async () => {
    const session = get().session
    if (!session) return
    set({ hashing: true })
    try {
      const duplicates = await api.findDuplicates(session.id)
      set({ duplicates })
      await get().refresh()
    } finally {
      set({ hashing: false })
    }
  },
}))
