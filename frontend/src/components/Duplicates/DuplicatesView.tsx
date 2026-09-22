import { useState } from 'react'
import { ask } from '@tauri-apps/plugin-dialog'
import { api, formatBytes, type DuplicateGroup } from '../../lib/tauri'
import { useScanStore } from '../../stores/scanStore'
import { useT, getLang } from '../../lib/i18n'

export function DuplicatesView() {
  const { session, duplicates, entries, hashing, findDuplicates, refresh, set } = useScanStore()
  const [resolving, setResolving] = useState<string | null>(null)
  const [searched, setSearched] = useState(false)
  const t = useT()

  const getEntry = (id: string) => entries.find((e) => e.id === id)

  // Nothing is removed without an explicit yes, and removed copies go to the
  // Trash, not into oblivion.
  const handleKeep = async (group: DuplicateGroup, keepId: string) => {
    if (!session) return
    const name = getEntry(keepId)?.name ?? keepId
    const others = group.file_ids.length - 1
    const text = others === 1 ? t('confirmTrashOne', { name }) : t('confirmTrash', { name, n: others })
    const confirmed = await ask(text, {
      title: t('confirmTrashTitle'),
      kind: 'warning',
      okLabel: t('trashYes'),
      cancelLabel: t('trashNo'),
    })
    if (!confirmed) return
    setResolving(group.id)
    try {
      await api.resolveDuplicate(session.id, group, keepId)
      set({ duplicates: useScanStore.getState().duplicates.filter((g) => g.id !== group.id) })
      await refresh()
    } finally {
      setResolving(null)
    }
  }

  const totalWasted = duplicates.reduce((sum, g) => sum + g.total_wasted_bytes, 0)
  const locale = getLang() === 'de' ? 'de-CH' : 'en-US'

  return (
    <div className="p-6 overflow-y-auto h-full">
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={() => void findDuplicates().then(() => setSearched(true))}
          disabled={!session || hashing}
          className="px-4 py-2 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm rounded-lg transition-colors"
        >
          {hashing ? t('searchingDuplicates') : t('findDuplicates')}
        </button>
        {duplicates.length > 0 && (
          <span className="text-sm text-[#d29922]">
            {duplicates.length === 1
              ? t('duplicateSummaryOne', { size: formatBytes(totalWasted) })
              : t('duplicateSummary', { n: duplicates.length, size: formatBytes(totalWasted) })}
          </span>
        )}
      </div>

      {duplicates.length === 0 && !hashing && searched && (
        <div className="text-center text-[#8b949e] py-16">{t('noDuplicates')}</div>
      )}

      <div className="space-y-4">
        {duplicates.map((group) => (
          <div key={group.id} className="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
            <div className="text-xs text-[#8b949e] mb-3">
              {formatBytes(group.size)} {t('perFile')} ·{' '}
              <span className="text-[#d29922]">{formatBytes(group.total_wasted_bytes)} {t('wasted')}</span>
            </div>
            <div className="space-y-2">
              {group.file_ids.map((fileId) => {
                const entry = getEntry(fileId)
                return (
                  <div key={fileId} className="flex items-center gap-3 bg-[#0d1117] rounded-sm p-2">
                    <div className="flex-1 min-w-0">
                      <div className="text-sm text-[#e6edf3] truncate">{entry?.name ?? fileId}</div>
                      <div className="text-xs text-[#8b949e] truncate">{entry?.path}</div>
                      {entry?.modified_at && (
                        <div className="text-xs text-[#8b949e]">{new Date(entry.modified_at).toLocaleDateString(locale)}</div>
                      )}
                    </div>
                    <button
                      onClick={() => void handleKeep(group, fileId)}
                      disabled={resolving === group.id}
                      className="shrink-0 px-3 py-1 text-xs bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white rounded-sm transition-colors"
                    >
                      {t('keep')}
                    </button>
                  </div>
                )
              })}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
