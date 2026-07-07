import { useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { api, formatBytes } from '../../lib/tauri'
import { useScanStore } from '../../stores/scanStore'
import { useT } from '../../lib/i18n'
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer } from 'recharts'

type Tab = 'dashboard' | 'files' | 'duplicates' | 'organize' | 'settings'

const KIND_COLORS: Record<string, string> = {
  photo: '#58a6ff', pdf: '#f78166', document: '#d2a8ff',
  video: '#3fb950', audio: '#e3b341', archive: '#8b949e',
  installer: '#ffa657', code: '#79c0ff', unknown: '#484f58',
}

export function Dashboard({ onNavigate }: { onNavigate: (tab: Tab) => void }) {
  const { session, scanning, classifying, progress, classifyProgress, setSession, setEntries, setStats, setScanning, stats } = useScanStore()
  const [error, setError] = useState('')
  const t = useT()

  const handleScan = async () => {
    setError('')
    const selected = await open({ directory: true, multiple: false })
    if (!selected || Array.isArray(selected)) return
    setScanning(true)
    try {
      const sess = await api.scanDirectory(selected)
      setSession(sess)
      // Wait for scan://done event, then load results
      setTimeout(async () => {
        const entries = await api.getScanResults(sess.id)
        setEntries(entries)
        const s = await api.getStats(sess.id)
        setStats(s)
      }, 500)
    } catch (e) {
      setError(String(e))
      setScanning(false)
    }
  }

  const handleClassify = async () => {
    if (!session) return
    await api.classifyBatch(session.id)
    setTimeout(async () => {
      const entries = await api.getScanResults(session.id)
      setEntries(entries)
      const s = await api.getStats(session.id)
      setStats(s)
    }, 500)
  }

  const pieData = stats
    ? Object.entries(stats.by_kind).map(([name, value]) => ({ name, value }))
    : []

  return (
    <div className="p-6 overflow-y-auto h-full">
      {/* Scan CTA */}
      <div className="flex items-center gap-4 mb-8">
        <button
          onClick={handleScan}
          disabled={scanning}
          className="px-5 py-2.5 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm font-medium rounded-lg transition-colors"
        >
          {scanning ? t('scanning', { n: progress }) : t('scanFolder')}
        </button>
        {session && !scanning && (
          <button
            onClick={handleClassify}
            disabled={classifying}
            className="px-5 py-2.5 bg-[#1f6feb] hover:bg-[#388bfd] disabled:opacity-50 text-white text-sm font-medium rounded-lg transition-colors"
          >
            {classifying
              ? t('aiClassifying', { done: classifyProgress[0], total: classifyProgress[1] })
              : t('classifyWithAi')}
          </button>
        )}
        {session && (
          <span className="text-sm text-[#8b949e]">
            {session.path}
          </span>
        )}
      </div>

      {error && <div className="text-sm text-[#f85149] mb-4">{error}</div>}

      {stats && (
        <>
          {/* Stats bar */}
          <div className="grid grid-cols-4 gap-4 mb-8">
            <StatCard label={t('statFiles')} value={stats.total_files} />
            <StatCard label={t('statSize')} value={formatBytes(stats.total_size_bytes)} />
            <StatCard label={t('statClassified')} value={`${stats.classified} / ${stats.total_files}`} />
            <StatCard label={t('statDuplicates')} value={stats.duplicate_count} warn={stats.duplicate_count > 0} />
          </div>

          {/* Pie chart + category list */}
          <div className="grid grid-cols-2 gap-6 mb-8">
            <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
              <div className="text-sm font-semibold text-[#8b949e] mb-3">{t('byFileType')}</div>
              <ResponsiveContainer width="100%" height={200}>
                <PieChart>
                  <Pie data={pieData} cx="50%" cy="50%" outerRadius={80} dataKey="value" label={({ name }) => name}>
                    {pieData.map((entry) => (
                      <Cell key={entry.name} fill={KIND_COLORS[entry.name] ?? '#8b949e'} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{ background: '#161b22', border: '1px solid #30363d', borderRadius: '6px', color: '#e6edf3' }}
                  />
                </PieChart>
              </ResponsiveContainer>
            </div>

            <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
              <div className="text-sm font-semibold text-[#8b949e] mb-3">{t('byCategory')}</div>
              <div className="space-y-1 overflow-y-auto max-h-[200px]">
                {Object.entries(stats.by_category)
                  .sort((a, b) => b[1] - a[1])
                  .map(([cat, count]) => (
                    <div key={cat} className="flex items-center justify-between text-sm">
                      <span className="text-[#c9d1d9]">{cat}</span>
                      <span className="text-[#8b949e]">{count}</span>
                    </div>
                  ))}
              </div>
            </div>
          </div>

          {/* Quick actions */}
          <div className="flex gap-3">
            <QuickAction label={t('viewFiles')} onClick={() => onNavigate('files')} />
            <QuickAction label={t('cleanUpDuplicates')} onClick={() => onNavigate('duplicates')} warn />
            <QuickAction label={t('sortSuggestions')} onClick={() => onNavigate('organize')} primary />
          </div>
        </>
      )}

      {!stats && !scanning && (
        <div className="flex flex-col items-center justify-center h-64 text-[#8b949e]">
          <div className="text-4xl mb-3">📁</div>
          <div className="text-sm">{t('pickFolderPrompt')}</div>
        </div>
      )}
    </div>
  )
}

function StatCard({ label, value, warn = false }: { label: string; value: string | number; warn?: boolean }) {
  return (
    <div className="bg-[#161b22] border border-[#30363d] rounded-lg px-4 py-3">
      <div className="text-xs text-[#8b949e] mb-1">{label}</div>
      <div className={`text-xl font-bold ${warn && Number(value) > 0 ? 'text-[#d29922]' : 'text-[#e6edf3]'}`}>{value}</div>
    </div>
  )
}

function QuickAction({ label, onClick, warn, primary }: { label: string; onClick: () => void; warn?: boolean; primary?: boolean }) {
  const cls = primary
    ? 'bg-[#238636] hover:bg-[#2ea043] text-white'
    : warn
    ? 'bg-[#161b22] border border-[#d29922] text-[#d29922] hover:bg-[#2d1b00]'
    : 'bg-[#161b22] border border-[#30363d] text-[#c9d1d9] hover:bg-[#21262d]'
  return (
    <button onClick={onClick} className={`px-4 py-2 text-sm rounded-md transition-colors ${cls}`}>
      {label}
    </button>
  )
}
