import { create } from 'zustand'
import type { OrganizeAction, ActionStatus } from '../lib/tauri'

interface ActionStore {
  actions: OrganizeAction[]
  setActions: (a: OrganizeAction[]) => void
  updateStatus: (id: string, status: ActionStatus) => void
}

export const useActionStore = create<ActionStore>((set) => ({
  actions: [],
  setActions: (actions) => set({ actions }),
  updateStatus: (id, status) => set(s => ({
    actions: s.actions.map(a => a.id === id ? { ...a, status } : a),
  })),
}))
