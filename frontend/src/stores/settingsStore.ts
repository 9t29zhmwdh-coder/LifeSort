import { create } from 'zustand'
import type { AppSettings } from '../lib/tauri'

interface SettingsStore {
  settings: AppSettings
  ollamaOnline: boolean
  setSettings: (s: AppSettings) => void
  setOllamaOnline: (v: boolean) => void
}

const defaults: AppSettings = {
  ollama_url: 'http://localhost:11434',
  text_model: 'llama3',
  vision_model: 'llava',
  target_root: '',
  auto_classify: true,
  auto_hash: true,
  skip_hidden: true,
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: defaults,
  ollamaOnline: false,
  setSettings: (settings) => set({ settings }),
  setOllamaOnline: (ollamaOnline) => set({ ollamaOnline }),
}))
