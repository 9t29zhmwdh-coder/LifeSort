import { useEffect, useState } from 'react'
import { api } from '../../lib/tauri'
import type { OrganizeAction } from '../../lib/tauri'
import { useScanStore } from '../../stores/scanStore'
import { useActionStore } from '../../stores/actionStore'
import { useT, useLangStore } from '../../lib/i18n'

export function OrganizerView() {
  const { session, entries, refresh } = useScanStore()
  const { actions, setActions, replace } = useActionStore()
  const lang = useLangStore((s) => s.lang)
  const [loading, setLoading] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [error, setError] = useState('')
  const t = useT()
  const classified = entries.some((e) => e.classification)

  // The journal from earlier runs, so a move from yesterday can still be undone.
  useEffect(() => { api.listActions().then(setActions).catch(console.error) }, [setActions])

  const handlePropose = async () => {
    if (!session) return
    setLoading(true)
    setError('')
    try {
      await api.proposeActions(session.id, lang)
      const all = await api.listActions()
      setActions(all)
      setSelected(new Set(all.filter((a) => a.status === 'pending').map((a) => a.id)))
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const handleExecuteSelected = async () => {
    setExecuting(true)
    try {
      for (const action of actions.filter((a) => selected.has(a.id) && a.status === 'pending')) {
        replace(await api.executeAction(action.id))
      }
      setSelected(new Set())
      await refresh()
    } finally {
      setExecuting(false)
    }
  }

  const handleUndo = async (id: string) => {
    try {
      replace(await api.undoAction(id))
      await refresh()
    } catch (e) {
      setError(t('undoFailed', { msg: String(e) }))
    }
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
          disabled={!session || !classified || loading}
          className="px-4 py-2 bg-[#1f6feb] hover:bg-[#388bfd] disabled:opacity-50 text-white text-sm rounded-lg transition-colors"
        >
          {loading ? t('analyzing') : t('createProposals')}
        </button>
        {pending.length > 0 && (
          <button
            onClick={handleExecuteSelected}
            disabled={executing || selected.size === 0}
            className="px-4 py-2 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm rounded-lg transition-colors"
          >
            {executing
              ? t('executing')
              : selected.size === 1
                ? t('executeAction', { n: selected.size })
                : t('executeActionsPlural', { n: selected.size })}
          </button>
        )}
        {actions.length > 0 && (
          <span className="text-xs text-[#8b949e]">
            {t('pendingCount', { pending: pending.length, applied: applied.length })}
          </span>
        )}
        {pending.length > 0 && (
          <button
            onClick={() => setSelected(selected.size > 0 ? new Set() : new Set(pending.map(a => a.id)))}
            className="ml-auto text-xs text-[#8b949e] hover:text-[#e6edf3]"
          >
            {selected.size > 0 ? t('selectNone') : t('selectAll')}
          </button>
        )}
      </div>
      {session && !classified && <div className="px-6 pt-3 text-xs text-[#8b949e]">{t('classifyFirst')}</div>}
      {error && <div className="px-6 pt-3 text-xs text-[#f85149]">{error}</div>}

      {/* Action list */}
      <div className="flex-1 overflow-y-auto p-6">
        {actions.length === 0 ? (
          <div className="text-center text-[#8b949e] py-16">
            <div className="text-3xl mb-2">📋</div>
            <div className="text-sm">{t('noActionsYet')}</div>
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
  const t = useT()

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
        <div className="text-xs text-[#8b949e] truncate mt-0.5">→ {action.reason}</div>
        {isFailed && typeof action.status === 'object' && (
          <div className="text-xs text-[#f85149] mt-0.5">{action.status.failed}</div>
        )}
        <div className="text-xs font-mono text-[#8b949e] truncate mt-0.5">
          {action.source_path} → {action.target_path}
        </div>
      </div>

      {isApplied && action.undoable && (
        <button
          onClick={onUndo}
          className="shrink-0 text-xs text-[#8b949e] hover:text-[#e6edf3] px-2 py-1 rounded-sm hover:bg-[#21262d]"
        >
          {t('undo')}
        </button>
      )}
    </div>
  )
}
