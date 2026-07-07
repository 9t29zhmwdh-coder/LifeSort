import { useEffect, useState } from 'react'
import { api, events } from './lib/tauri'
import { useScanStore } from './stores/scanStore'
import { useSettingsStore } from './stores/settingsStore'
import { useT, useLangStore } from './lib/i18n'
import { Dashboard } from './components/Dashboard/Dashboard'
import { FileGrid } from './components/FileGrid/FileGrid'
import { DuplicatesView } from './components/Duplicates/DuplicatesView'
import { OrganizerView } from './components/Organizer/OrganizerView'
import { SettingsView } from './components/Settings/SettingsView'

type Tab = 'dashboard' | 'files' | 'duplicates' | 'organize' | 'settings'

export default function App() {
  const [tab, setTab] = useState<Tab>('dashboard')
  const { setSettings, setOllamaOnline } = useSettingsStore()
  const { setScanning, setProgress, setClassifying, setClassifyProgress } = useScanStore()
  const t = useT()
  const lang = useLangStore((s) => s.lang)
  const toggleLang = useLangStore((s) => s.toggle)

  useEffect(() => {
    api.getSettings().then(setSettings).catch(console.error)
    api.checkOllama().then(setOllamaOnline).catch(() => setOllamaOnline(false))

    const unlisteners = [
      events.onScanProgress(n => { setScanning(true); setProgress(n) }),
      events.onScanDone((_, count) => { setScanning(false); setProgress(count) }),
      events.onClassifyProgress((done, total) => { setClassifying(true); setClassifyProgress(done, total) }),
      events.onClassifyDone(() => setClassifying(false)),
    ]
    return () => { unlisteners.forEach(p => p.then(fn => fn())) }
  }, [])

  const tabs: { id: Tab; label: string; badge?: number }[] = [
    { id: 'dashboard',  label: t('navDashboard') },
    { id: 'files',      label: t('navFiles') },
    { id: 'duplicates', label: t('navDuplicates') },
    { id: 'organize',   label: t('navOrganize') },
    { id: 'settings',   label: t('navSettings') },
  ]

  return (
    <div className="flex flex-col h-screen bg-[#0d1117] text-[#e6edf3]">
      {/* Top bar */}
      <header className="flex items-center gap-4 px-5 py-3 border-b border-[#30363d] flex-shrink-0">
        <div>
          <span className="text-base font-bold text-[#58a6ff]">LifeSort</span>
          <span className="text-xs text-[#8b949e] ml-2">AI File Organizer</span>
        </div>
        <nav className="flex gap-1 ml-4">
          {tabs.map(tabItem => (
            <button
              key={tabItem.id}
              onClick={() => setTab(tabItem.id)}
              className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
                tab === tabItem.id
                  ? 'bg-[#1c2128] text-[#e6edf3]'
                  : 'text-[#8b949e] hover:text-[#e6edf3] hover:bg-[#161b22]'
              }`}
            >
              {tabItem.label}
            </button>
          ))}
        </nav>
        <button onClick={toggleLang} className="ml-auto text-xs text-[#8b949e] hover:text-[#e6edf3] px-2 py-1 rounded hover:bg-[#21262d]">
          {lang === 'en' ? 'DE' : 'EN'}
        </button>
        <OllamaStatus />
      </header>

      {/* Content */}
      <main className="flex-1 overflow-hidden">
        {tab === 'dashboard'  && <Dashboard onNavigate={setTab} />}
        {tab === 'files'      && <FileGrid />}
        {tab === 'duplicates' && <DuplicatesView />}
        {tab === 'organize'   && <OrganizerView />}
        {tab === 'settings'   && <SettingsView />}
      </main>
    </div>
  )
}

function OllamaStatus() {
  const { ollamaOnline } = useSettingsStore()
  const t = useT()
  return (
    <div className="flex items-center gap-1.5 text-xs text-[#8b949e]">
      <span className={`w-1.5 h-1.5 rounded-full ${ollamaOnline ? 'bg-[#3fb950]' : 'bg-[#f85149]'}`} />
      Ollama {ollamaOnline ? t('ollamaOnline') : t('ollamaOffline')}
    </div>
  )
}
