import { create } from 'zustand'
import type { OrganizeAction } from '../lib/tauri'

interface ActionStore {
  actions: OrganizeAction[]
  setActions: (a: OrganizeAction[]) => void
  replace: (a: OrganizeAction) => void
}

export const useActionStore = create<ActionStore>((set) => ({
  actions: [],
  setActions: (actions) => set({ actions }),
  replace: (action) => set((s) => ({ actions: s.actions.map((a) => (a.id === action.id ? action : a)) })),
}))
