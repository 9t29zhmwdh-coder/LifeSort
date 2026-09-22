import { create } from 'zustand'
import type { AiStatus, AppSettings } from '../lib/tauri'

interface SettingsStore {
  settings?: AppSettings
  ai: AiStatus
  setSettings: (s: AppSettings) => void
  setAi: (s: AiStatus) => void
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: undefined,
  ai: { state: 'unreachable' },
  setSettings: (settings) => set({ settings }),
  setAi: (ai) => set({ ai }),
}))
