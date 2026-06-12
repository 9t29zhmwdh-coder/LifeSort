import { useState } from 'react'
import { api, type OrganizeAction } from '../../lib/tauri'
import { useScanStore } from '../../stores/scanStore'
import { useActionStore } from '../../stores/actionStore'

export function OrganizerView() {
  const { session } = useScanStore()
  const { actions, setActions, updateStatus } = useActionStore()
  const [loading, setLoading] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [selected, setSelected] = useState<Set<string>>(new Set())

  const handlePropose = async () => {
    if (!session) return
    setLoading(true)
    try {
      const proposed = await api.proposeActions(session.id)
      setActions(proposed)
      setSelected(new Set(proposed.map(a => a.id)))
    } finally {
      setLoading(false)
    }
  }

  const handleExecuteSelected = async () => {
    setExecuting(true)
    try {
      for (const action of actions.filter(a => selected.has(a.id) && a.status === 'pending')) {
        const status = await api.executeAction(action.id)
        updateStatus(action.id, status)
      }
    } finally {
      setExecuting(false)
    }
  }

  const handleUndo = async (id: string) => {
    const ok = await api.undoAction(id)
    if (ok) updateStatus(id, 'pending')
  }

  const toggle = (id: string) => {
    const next = new Set(selected)
    if (next.has(id)) next.delete(id); else next.add(id)
    setSelected(next)
  }

  const pending = actions.filter(a => a.status === 'pending')
  const applied = actions.filter(a => a.status === 'applied')

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center gap-3 px-6 py-3 border-b border-[#30363d]">
        <button
          onClick={handlePropose}
          disabled={!session || loading}
          className="px-4 py-2 bg-[#1f6feb] hover:bg-[#388bfd] disabled:opacity-50 text-white text-sm rounded-lg transition-colors"
        >
          {loading ? 'Analysiere…' : 'Vorschläge erstellen'}
        </button>
        {pending.length > 0 && (
          <button
            onClick={handleExecuteSelected}
            disabled={executing || selected.size === 0}
            className="px-4 py-2 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm rounded-lg transition-colors"
          >
            {executing
              ? 'Führe aus…'
              : `${selected.size} Aktion${selected.size !== 1 ? 'en' : ''} ausführen`}
          </button>
        )}
        {actions.length > 0 && (
          <span className="text-xs text-[#8b949e]">
            {pending.length} ausstehend · {applied.length} erledigt
          </span>
        )}
        {pending.length > 0 && (
          <button
            onClick={() => setSelected(new Set(pending.map(a => a.id)))}
            className="ml-auto text-xs text-[#8b949e] hover:text-[#e6edf3]"
          >
            Alle auswählen
          </button>
        )}
      </div>

      {/* Action list */}
      <div className="flex-1 overflow-y-auto p-6">
        {actions.length === 0 ? (
          <div className="text-center text-[#8b949e] py-16">
            <div className="text-3xl mb-2">📋</div>
            <div className="text-sm">Erstelle Sortier-Vorschläge nach dem Scan</div>
          </div>
        ) : (
          <div className="space-y-2">
            {actions.map(action => (
              <ActionRow
                key={action.id}
                action={action}
                checked={selected.has(action.id)}
                onToggle={() => toggle(action.id)}
                onUndo={() => handleUndo(action.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

function ActionRow({
  action, checked, onToggle, onUndo,
}: {
  action: OrganizeAction
  checked: boolean
  onToggle: () => void
  onUndo: () => void
}) {
  const isPending = action.status === 'pending'
  const isApplied = action.status === 'applied'
  const isFailed = typeof action.status === 'object'

  return (
    <div className={`flex items-start gap-3 p-3 rounded-lg border ${
      isApplied ? 'border-[#238636] bg-[#0a1a0e]' :
      isFailed  ? 'border-[#f85149] bg-[#2d0f0f]' :
      'border-[#30363d] bg-[#161b22]'
    }`}>
      {isPending && (
        <input
          type="checkbox"
          checked={checked}
          onChange={onToggle}
          className="mt-0.5 accent-[#1f6feb]"
        />
      )}
      {isApplied && <span className="text-[#3fb950] mt-0.5">✓</span>}
      {isFailed && <span className="text-[#f85149] mt-0.5">✗</span>}

      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium text-[#e6edf3] truncate">{action.file_name}</div>
        <div className="text-xs text-[#8b949e] truncate mt-0.5">{action.reason}</div>
        <div className="text-xs font-mono text-[#8b949e] truncate mt-0.5">
          {action.source_path} → {action.target_path}
        </div>
      </div>

      {isApplied && action.undoable && (
        <button
          onClick={onUndo}
          className="flex-shrink-0 text-xs text-[#8b949e] hover:text-[#e6edf3] px-2 py-1 rounded hover:bg-[#21262d]"
        >
          Rückgängig
        </button>
      )}
    </div>
  )
}
