import { create } from 'zustand'
import type { FileEntry, ScanSession, ScanStats, DuplicateGroup } from '../lib/tauri'

interface ScanStore {
  session?: ScanSession
  entries: FileEntry[]
  stats?: ScanStats
  duplicates: DuplicateGroup[]
  scanning: boolean
  classifying: boolean
  progress: number
  classifyProgress: [number, number]
  setSession: (s: ScanSession) => void
  setEntries: (e: FileEntry[]) => void
  setStats: (s: ScanStats) => void
  setDuplicates: (d: DuplicateGroup[]) => void
  setScanning: (v: boolean) => void
  setClassifying: (v: boolean) => void
  setProgress: (n: number) => void
  setClassifyProgress: (done: number, total: number) => void
  updateEntry: (id: string, update: Partial<FileEntry>) => void
}

export const useScanStore = create<ScanStore>((set) => ({
  session: undefined,
  entries: [],
  stats: undefined,
  duplicates: [],
  scanning: false,
  classifying: false,
  progress: 0,
  classifyProgress: [0, 0],
  setSession: (session) => set({ session }),
  setEntries: (entries) => set({ entries }),
  setStats: (stats) => set({ stats }),
  setDuplicates: (duplicates) => set({ duplicates }),
  setScanning: (scanning) => set({ scanning }),
  setClassifying: (classifying) => set({ classifying }),
  setProgress: (progress) => set({ progress }),
  setClassifyProgress: (done, total) => set({ classifyProgress: [done, total] }),
  updateEntry: (id, update) => set(s => ({
    entries: s.entries.map(e => e.id === id ? { ...e, ...update } : e),
  })),
}))
