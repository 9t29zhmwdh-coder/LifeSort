import { useState } from 'react'
import { api, formatBytes } from '../../lib/tauri'
import { useScanStore } from '../../stores/scanStore'

export function DuplicatesView() {
  const { session, duplicates, setDuplicates, entries } = useScanStore()
  const [loading, setLoading] = useState(false)
  const [resolving, setResolving] = useState<string | null>(null)

  const handleFind = async () => {
    if (!session) return
    setLoading(true)
    try {
      const groups = await api.findDuplicates(session.id)
      setDuplicates(groups)
    } finally {
      setLoading(false)
    }
  }

  const handleResolve = async (groupId: string, keepId: string) => {
    const group = duplicates.find(g => g.id === groupId)
    if (!group) return
    setResolving(groupId)
    try {
      await api.resolveDuplicate(group, keepId)
      setDuplicates(duplicates.filter(g => g.id !== groupId))
    } finally {
      setResolving(null)
    }
  }

  const totalWasted = duplicates.reduce((sum, g) => sum + g.total_wasted_bytes, 0)

  const getEntry = (id: string) => entries.find(e => e.id === id)

  return (
    <div className="p-6 overflow-y-auto h-full">
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={handleFind}
          disabled={!session || loading}
          className="px-4 py-2 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm rounded-lg transition-colors"
        >
          {loading ? 'Suche Duplikate…' : 'Duplikate suchen'}
        </button>
        {duplicates.length > 0 && (
          <span className="text-sm text-[#d29922]">
            {duplicates.length} Gruppen · {formatBytes(totalWasted)} verschwendet
          </span>
        )}
      </div>

      {duplicates.length === 0 && !loading && (
        <div className="text-center text-[#8b949e] py-16">
          <div className="text-3xl mb-2">✅</div>
          <div>Keine Duplikate gefunden</div>
        </div>
      )}

      <div className="space-y-4">
        {duplicates.map(group => (
          <div key={group.id} className="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="text-xs text-[#8b949e]">
                Hash: <span className="font-mono">{group.hash.slice(0, 16)}…</span> ·{' '}
                {formatBytes(group.size)} je Datei ·{' '}
                <span className="text-[#d29922]">{formatBytes(group.total_wasted_bytes)} verschwendet</span>
              </div>
            </div>
            <div className="space-y-2">
              {group.file_ids.map(fileId => {
                const entry = getEntry(fileId)
                return (
                  <div key={fileId} className="flex items-center gap-3 bg-[#0d1117] rounded p-2">
                    <div className="flex-1 min-w-0">
                      <div className="text-sm text-[#e6edf3] truncate">{entry?.name ?? fileId}</div>
                      <div className="text-xs text-[#8b949e] truncate">{entry?.path}</div>
                      {entry?.modified_at && (
                        <div className="text-xs text-[#8b949e]">
                          {new Date(entry.modified_at).toLocaleDateString('de-CH')}
                        </div>
                      )}
                    </div>
                    <button
                      onClick={() => handleResolve(group.id, fileId)}
                      disabled={resolving === group.id}
                      className="flex-shrink-0 px-3 py-1 text-xs bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white rounded transition-colors"
                    >
                      Behalten
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
