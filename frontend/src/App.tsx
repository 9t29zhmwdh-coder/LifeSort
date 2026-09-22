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
import { PhotosView } from './components/Photos/PhotosView'

export type Tab = 'dashboard' | 'files' | 'duplicates' | 'organize' | 'photos' | 'settings'

export default function App() {
  const [tab, setTab] = useState<Tab>('dashboard')
  const [isMac, setIsMac] = useState(false)
  const t = useT()
  const lang = useLangStore((s) => s.lang)
  const toggleLang = useLangStore((s) => s.toggle)

  useEffect(() => {
    api.platform().then((os) => setIsMac(os === 'macos')).catch(() => setIsMac(false))
    const settings = useSettingsStore.getState()
    api.getSettings().then(settings.setSettings).catch(console.error)
    api.checkOllama().then(settings.setAi).catch(() => settings.setAi({ state: 'unreachable' }))

    const scan = useScanStore.getState()
    // Results are loaded when the backend says the work is finished, never
    // after a fixed delay: a large folder takes minutes, not 500 ms.
    const unlisteners = [
      events.onScanProgress((n) => scan.set({ progress: n })),
      events.onScanDone((done) => useScanStore.getState().scanFinished(done)),
      events.onClassifyProgress((done, total) => {
        scan.set({ classifying: true, classifyProgress: [done, total] })
        // Refresh every 25 files so the overview fills while the run goes on.
        if (done % 25 === 0) void useScanStore.getState().refresh()
      }),
      events.onClassifyDone(async () => {
        scan.set({ classifying: false })
        await useScanStore.getState().refresh()
      }),
    ]
    return () => { unlisteners.forEach((p) => p.then((fn) => fn())) }
  }, [])

  const tabs: { id: Tab; label: string }[] = [
    { id: 'dashboard', label: t('navDashboard') },
    { id: 'files', label: t('navFiles') },
    { id: 'duplicates', label: t('navDuplicates') },
    { id: 'organize', label: t('navOrganize') },
    ...(isMac ? [{ id: 'photos' as Tab, label: t('navPhotos') }] : []),
    { id: 'settings', label: t('navSettings') },
  ]

  return (
    <div className="flex flex-col h-screen bg-[#0d1117] text-[#e6edf3]">
      <header className="flex items-center gap-4 px-5 py-3 border-b border-[#30363d] shrink-0">
        <div>
          <span className="text-base font-bold text-[#58a6ff]">LifeSort</span>
          <span className="text-xs text-[#8b949e] ml-2">{t('tagline')}</span>
        </div>
        <nav className="flex gap-1 ml-4">
          {tabs.map((tabItem) => (
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
        <button onClick={toggleLang} className="ml-auto text-xs text-[#8b949e] hover:text-[#e6edf3] px-2 py-1 rounded-sm hover:bg-[#21262d]">
          {lang === 'en' ? 'DE' : 'EN'}
        </button>
        <OllamaStatus />
      </header>

      <main className="flex-1 overflow-hidden">
        {tab === 'dashboard' && <Dashboard onNavigate={setTab} />}
        {tab === 'files' && <FileGrid />}
        {tab === 'duplicates' && <DuplicatesView />}
        {tab === 'organize' && <OrganizerView />}
        {tab === 'settings' && <SettingsView />}
        {/* Kept mounted: a library scan or AI run goes on while another tab is open. */}
        {isMac && (
          <div className={tab === 'photos' ? 'h-full' : 'hidden'}>
            <PhotosView />
          </div>
        )}
      </main>
    </div>
  )
}

function OllamaStatus() {
  const ai = useSettingsStore((s) => s.ai)
  const t = useT()
  const [color, label] =
    ai.state === 'ready' ? ['bg-[#3fb950]', t('ollamaOnline')]
    : ai.state === 'missing_models' ? ['bg-[#d29922]', t('ollamaMissing')]
    : ['bg-[#f85149]', t('ollamaOffline')]
  return (
    <div className="flex items-center gap-1.5 text-xs text-[#8b949e]">
      <span className={`w-1.5 h-1.5 rounded-full ${color}`} />
      Ollama {label}
    </div>
  )
}
