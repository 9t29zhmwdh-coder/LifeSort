import { useState, useMemo } from 'react'
import { useScanStore } from '../../stores/scanStore'
import { categoryLabel, formatBytes, kindIcon, type Category, type FileKind } from '../../lib/tauri'

const KIND_LABELS: Record<FileKind, string> = {
  photo: 'Foto', pdf: 'PDF', document: 'Dokument', video: 'Video',
  audio: 'Audio', archive: 'Archiv', installer: 'Installer',
  code: 'Code', font: 'Font', unknown: 'Unbekannt',
}

export function FileGrid() {
  const { entries } = useScanStore()
  const [filterKind, setFilterKind] = useState<FileKind | 'all'>('all')
  const [filterCat, setFilterCat] = useState<string>('all')
  const [search, setSearch] = useState('')
  const [selected, setSelected] = useState<string | null>(null)

  const kinds = useMemo(() => {
    const s = new Set(entries.map(e => e.kind))
    return Array.from(s) as FileKind[]
  }, [entries])

  const cats = useMemo(() => {
    const s = new Set(entries.flatMap(e => e.classification ? [e.classification.category] : []))
    return Array.from(s) as Category[]
  }, [entries])

  const filtered = useMemo(() => entries.filter(e => {
    if (filterKind !== 'all' && e.kind !== filterKind) return false
    if (filterCat !== 'all' && e.classification?.category !== filterCat) return false
    if (search && !e.name.toLowerCase().includes(search.toLowerCase())) return false
    return true
  }), [entries, filterKind, filterCat, search])

  const selectedEntry = selected ? entries.find(e => e.id === selected) : null

  return (
    <div className="flex h-full">
      {/* Main */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Filter bar */}
        <div className="flex items-center gap-3 px-4 py-2 border-b border-[#30363d] flex-wrap">
          <input
            value={search}
            onChange={e => setSearch(e.target.value)}
            placeholder="Suche…"
            className="bg-[#161b22] border border-[#30363d] rounded-md px-3 py-1 text-sm text-[#e6edf3] focus:outline-none focus:border-[#58a6ff] w-48"
          />
          <div className="flex gap-1 flex-wrap">
            <Chip active={filterKind === 'all'} onClick={() => setFilterKind('all')}>Alle</Chip>
            {kinds.map(k => (
              <Chip key={k} active={filterKind === k} onClick={() => setFilterKind(k)}>
                {kindIcon(k)} {KIND_LABELS[k]}
              </Chip>
            ))}
          </div>
          {cats.length > 0 && (
            <div className="flex gap-1 flex-wrap ml-2 border-l border-[#30363d] pl-2">
              <Chip active={filterCat === 'all'} onClick={() => setFilterCat('all')}>Alle Kat.</Chip>
              {cats.map(c => (
                <Chip key={c} active={filterCat === c} onClick={() => setFilterCat(c)}>
                  {categoryLabel(c)}
                </Chip>
              ))}
            </div>
          )}
          <span className="ml-auto text-xs text-[#8b949e]">{filtered.length} Dateien</span>
        </div>

        {/* Grid */}
        <div className="flex-1 overflow-y-auto p-4">
          {filtered.length === 0 ? (
            <div className="text-center text-[#8b949e] py-16">Keine Dateien</div>
          ) : (
            <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-3">
              {filtered.map(entry => (
                <button
                  key={entry.id}
                  onClick={() => setSelected(entry.id === selected ? null : entry.id)}
                  className={`text-left p-3 rounded-lg border transition-colors ${
                    selected === entry.id
                      ? 'border-[#58a6ff] bg-[#1c2128]'
                      : 'border-[#30363d] bg-[#161b22] hover:border-[#484f58]'
                  }`}
                >
                  <div className="text-2xl mb-2">{kindIcon(entry.kind)}</div>
                  <div className="text-xs font-medium text-[#e6edf3] truncate">{entry.name}</div>
                  <div className="text-xs text-[#8b949e] mt-0.5">{formatBytes(entry.size)}</div>
                  {entry.classification && (
                    <div className="mt-1">
                      <span className="text-xs px-1.5 py-0.5 rounded-full bg-[#1f3a5f] text-[#79c0ff]">
                        {categoryLabel(entry.classification.category)}
                      </span>
                    </div>
                  )}
                  {entry.duplicate_group_id && (
                    <div className="mt-1">
                      <span className="text-xs px-1.5 py-0.5 rounded-full bg-[#3d1a00] text-[#ffa657]">Duplikat</span>
                    </div>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Inspector panel */}
      {selectedEntry && (
        <aside className="w-72 flex-shrink-0 border-l border-[#30363d] p-4 overflow-y-auto">
          <div className="text-3xl mb-3">{kindIcon(selectedEntry.kind)}</div>
          <div className="text-sm font-semibold text-[#e6edf3] break-all mb-1">{selectedEntry.name}</div>
          <div className="text-xs text-[#8b949e] mb-4 break-all">{selectedEntry.path}</div>
          <InfoRow label="Grösse" value={formatBytes(selectedEntry.size)} />
          <InfoRow label="Typ" value={selectedEntry.mime_type} />
          <InfoRow label="Geändert" value={new Date(selectedEntry.modified_at).toLocaleDateString('de-CH')} />
          {selectedEntry.dimensions && (
            <InfoRow label="Abmessungen" value={`${selectedEntry.dimensions[0]} × ${selectedEntry.dimensions[1]}`} />
          )}
          {selectedEntry.classification && (
            <>
              <div className="mt-4 mb-2 text-xs font-semibold text-[#8b949e] uppercase tracking-wider">Klassifizierung</div>
              <InfoRow label="Kategorie" value={categoryLabel(selectedEntry.classification.category)} />
              <InfoRow label="Konfidenz" value={`${(selectedEntry.classification.confidence * 100).toFixed(0)}%`} />
              {selectedEntry.classification.extracted_date && (
                <InfoRow label="Datum" value={selectedEntry.classification.extracted_date} />
              )}
              {selectedEntry.classification.extracted_amount != null && (
                <InfoRow label="Betrag" value={`${selectedEntry.classification.extracted_amount}`} />
              )}
              {selectedEntry.classification.extracted_sender && (
                <InfoRow label="Absender" value={selectedEntry.classification.extracted_sender} />
              )}
              {selectedEntry.classification.ai_summary && (
                <div className="mt-2 text-xs text-[#8b949e] italic">{selectedEntry.classification.ai_summary}</div>
              )}
              {selectedEntry.tags.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {selectedEntry.tags.map(t => (
                    <span key={t} className="text-xs px-1.5 py-0.5 rounded-full bg-[#21262d] text-[#8b949e]">{t}</span>
                  ))}
                </div>
              )}
            </>
          )}
        </aside>
      )}
    </div>
  )
}

function Chip({ children, active, onClick }: { children: React.ReactNode; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={`px-2.5 py-0.5 text-xs rounded-full transition-colors ${
        active ? 'bg-[#1f6feb] text-white' : 'bg-[#21262d] text-[#8b949e] hover:text-[#e6edf3]'
      }`}
    >
      {children}
    </button>
  )
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between text-xs mb-1.5">
      <span className="text-[#8b949e]">{label}</span>
      <span className="text-[#e6edf3] text-right max-w-[160px] truncate">{value}</span>
    </div>
  )
}
