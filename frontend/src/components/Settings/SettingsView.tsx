import { useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { api, type AppSettings } from '../../lib/tauri'
import { useSettingsStore } from '../../stores/settingsStore'
import { useT } from '../../lib/i18n'

export function SettingsView() {
  const { settings, setSettings, setOllamaOnline } = useSettingsStore()
  const [draft, setDraft] = useState<AppSettings>({ ...settings })
  const [saved, setSaved] = useState(false)
  const [checking, setChecking] = useState(false)
  const [ollamaMsg, setOllamaMsg] = useState('')
  const t = useT()

  const set = <K extends keyof AppSettings>(k: K, v: AppSettings[K]) =>
    setDraft(d => ({ ...d, [k]: v }))

  const handleSave = async () => {
    await api.saveSettings(draft)
    setSettings(draft)
    setSaved(true)
    setTimeout(() => setSaved(false), 1500)
  }

  const handleCheckOllama = async () => {
    setChecking(true); setOllamaMsg('')
    const ok = await api.checkOllama().catch(() => false)
    setOllamaOnline(ok)
    setOllamaMsg(ok ? `${t('ollamaReachable')} ✓` : t('ollamaUnreachable'))
    setChecking(false)
  }

  const handlePickFolder = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (selected && !Array.isArray(selected)) set('target_root', selected)
  }

  return (
    <div className="p-6 max-w-lg overflow-y-auto h-full">
      <h2 className="text-lg font-semibold text-[#e6edf3] mb-6">{t('settingsTitle')}</h2>

      <Section title={t('localAiSection')}>
        <Label>{t('ollamaUrl')}</Label>
        <Input value={draft.ollama_url} onChange={v => set('ollama_url', v)} />
        <Label>{t('textModel')}</Label>
        <Input value={draft.text_model} onChange={v => set('text_model', v)} placeholder="llama3" />
        <Label>{t('visionModel')}</Label>
        <Input value={draft.vision_model} onChange={v => set('vision_model', v)} placeholder="llava" />
        <button
          onClick={handleCheckOllama}
          disabled={checking}
          className="mt-2 px-3 py-1.5 text-xs bg-[#21262d] hover:bg-[#30363d] text-[#8b949e] hover:text-[#e6edf3] rounded transition-colors"
        >
          {checking ? t('testing') : t('testConnection')}
        </button>
        {ollamaMsg && (
          <div className={`mt-1 text-xs ${ollamaMsg.includes('✓') ? 'text-[#3fb950]' : 'text-[#f85149]'}`}>
            {ollamaMsg}
          </div>
        )}
      </Section>

      <Section title={t('targetFolderSection')}>
        <Label>{t('baseDirectory')}</Label>
        <div className="flex gap-2">
          <input
            value={draft.target_root}
            onChange={e => set('target_root', e.target.value)}
            className="flex-1 bg-[#0d1117] border border-[#30363d] rounded-md px-3 py-1.5 text-sm text-[#e6edf3] font-mono focus:outline-none focus:border-[#58a6ff]"
          />
          <button
            onClick={handlePickFolder}
            className="px-3 py-1.5 text-xs bg-[#21262d] hover:bg-[#30363d] text-[#8b949e] hover:text-[#e6edf3] rounded transition-colors"
          >
            {t('choose')}
          </button>
        </div>
        <p className="text-xs text-[#8b949e] mt-1">
          {t('defaultSubfolders')}
        </p>
      </Section>

      <Section title={t('scanOptionsSection')}>
        <Toggle
          label={t('skipHidden')}
          value={draft.skip_hidden}
          onChange={v => set('skip_hidden', v)}
        />
        <Toggle
          label={t('autoClassify')}
          value={draft.auto_classify}
          onChange={v => set('auto_classify', v)}
        />
        <Toggle
          label={t('autoHash')}
          value={draft.auto_hash}
          onChange={v => set('auto_hash', v)}
        />
      </Section>

      <Section title={t('defaultRulesSection')}>
        <div className="text-xs text-[#8b949e] space-y-1">
          {t('folderRulesExample').split('\n').map((line, i) => <div key={i}>{line}</div>)}
        </div>
      </Section>

      <button
        onClick={handleSave}
        className="w-full py-2.5 bg-[#238636] hover:bg-[#2ea043] text-white text-sm rounded-lg transition-colors"
      >
        {saved ? t('saved') : t('saveSettings')}
      </button>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="mb-6 bg-[#161b22] border border-[#30363d] rounded-lg p-4">
      <h3 className="text-xs font-semibold text-[#8b949e] uppercase tracking-wider mb-4">{title}</h3>
      {children}
    </div>
  )
}

function Label({ children }: { children: React.ReactNode }) {
  return <div className="text-xs text-[#8b949e] mb-1.5">{children}</div>
}

function Input({ value, onChange, placeholder }: { value: string; onChange: (v: string) => void; placeholder?: string }) {
  return (
    <input
      value={value}
      onChange={e => onChange(e.target.value)}
      placeholder={placeholder}
      className="w-full bg-[#0d1117] border border-[#30363d] rounded-md px-3 py-1.5 text-sm text-[#e6edf3] font-mono focus:outline-none focus:border-[#58a6ff] mb-3 placeholder-[#484f58]"
    />
  )
}

function Toggle({ label, value, onChange }: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <div className="flex items-center justify-between mb-3">
      <span className="text-sm text-[#c9d1d9]">{label}</span>
      <button
        onClick={() => onChange(!value)}
        className={`w-10 h-5 rounded-full transition-colors relative flex-shrink-0 ${value ? 'bg-[#238636]' : 'bg-[#30363d]'}`}
      >
        <span className={`absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform ${value ? 'left-5' : 'left-0.5'}`} />
      </button>
    </div>
  )
}
