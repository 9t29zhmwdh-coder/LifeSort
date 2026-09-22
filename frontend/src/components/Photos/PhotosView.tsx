import { useEffect, useState } from 'react'
import { api, events, formatBytes, formatDuration, type AiStatus, type PhotoAccess, type PhotoGroup, type PhotoGroupKey } from '../../lib/tauri'
import { useT } from '../../lib/i18n'

const GROUP_LABEL: Record<PhotoGroupKey, 'groupVideos' | 'groupScreenshots' | 'groupBurstExtras' | 'groupMemes' | 'groupPhotographedDocuments'> = {
  videos: 'groupVideos',
  screenshots: 'groupScreenshots',
  burst_extras: 'groupBurstExtras',
  memes: 'groupMemes',
  photographed_documents: 'groupPhotographedDocuments',
}

export function PhotosView() {
  const t = useT()
  const [access, setAccess] = useState<PhotoAccess>()
  const [reading, setReading] = useState<[number, number] | null>(null)
  const [summary, setSummary] = useState<{ count: number; bytes: number } | null>(null)
  const [groups, setGroups] = useState<PhotoGroup[]>([])
  const [classifying, setClassifying] = useState<[number, number] | null>(null)
  const [aiNotice, setAiNotice] = useState<AiStatus | null>(null)
  const [error, setError] = useState('')

  const refreshGroups = () => api.photosGroups().then(setGroups).catch((e) => setError(String(e)))

  useEffect(() => {
    api.photosAccess(false).then(setAccess).catch(() => setAccess('unsupported'))
    const unlisteners = [
      events.onPhotosProgress((done, total) => setReading([done, total])),
      events.onPhotosDone((done) => {
        setReading(null)
        if (done.error) setError(t('photosError', { msg: done.error }))
        else setSummary({ count: done.count, bytes: done.bytes })
        void refreshGroups()
      }),
      events.onPhotosClassifyProgress((done, total) => {
        setClassifying([done, total])
        if (done % 20 === 0) void refreshGroups()
      }),
      events.onPhotosClassifyDone(() => {
        setClassifying(null)
        void refreshGroups()
      }),
    ]
    return () => { unlisteners.forEach((p) => p.then((fn) => fn())) }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const handleRead = async () => {
    setError('')
    const status = await api.photosAccess(true)
    setAccess(status)
    if (status !== 'authorized' && status !== 'limited') return
    setReading([0, 0])
    await api.photosScan()
  }

  const handleClassify = async () => {
    setAiNotice(null)
    const start = await api.photosClassify()
    if (start.ai.state !== 'ready') setAiNotice(start.ai)
    else if (start.total > 0) setClassifying([0, start.total])
  }

  if (access === 'unsupported') {
    return <div className="p-6 text-sm text-[#8b949e]">{t('photosUnsupported')}</div>
  }

  return (
    <div className="p-6 overflow-y-auto h-full">
      <p className="text-sm text-[#c9d1d9] mb-5 max-w-4xl">{t('photosIntro')}</p>

      <div className="flex items-center gap-4 mb-5 flex-wrap">
        <button
          onClick={() => void handleRead()}
          disabled={reading !== null || classifying !== null}
          className="px-5 py-2.5 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white text-sm font-medium rounded-lg transition-colors"
        >
          {reading ? t('photosReading', { done: reading[0], total: reading[1] }) : summary ? t('photosRereading') : t('photosRead')}
        </button>
        {summary && !reading && (
          classifying ? (
            <>
              <span className="text-sm text-[#c9d1d9]">{t('photosClassifying', { done: classifying[0], total: classifying[1] })}</span>
              <button onClick={() => void api.photosCancel()} className="px-3 py-1.5 text-sm bg-[#21262d] hover:bg-[#30363d] text-[#e6edf3] rounded-md">
                {t('photosCancel')}
              </button>
            </>
          ) : (
            <button
              onClick={() => void handleClassify()}
              className="px-5 py-2.5 bg-[#1f6feb] hover:bg-[#388bfd] text-white text-sm font-medium rounded-lg transition-colors"
            >
              {t('photosClassify')}
            </button>
          )
        )}
        {summary && (
          <span className="text-sm text-[#8b949e]">{t('photosSummary', { count: summary.count, size: formatBytes(summary.bytes) })}</span>
        )}
      </div>

      {access === 'denied' && <Notice>{t('photosDenied')}</Notice>}
      {access === 'limited' && <Notice>{t('photosLimited')}</Notice>}
      {error && <div className="text-sm text-[#f85149] mb-4">{error}</div>}
      {aiNotice && (
        <Notice>{aiNotice.state === 'missing_models' ? t('aiMissing', { models: aiNotice.models.join(' ') }) : t('aiUnreachable')}</Notice>
      )}
      {summary && !classifying && <p className="text-xs text-[#8b949e] mb-5 max-w-4xl">{t('photosClassifyHint')}</p>}

      {summary && groups.length === 0 && !reading && <div className="text-sm text-[#8b949e]">{t('photosNothing')}</div>}

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
        {groups.map((g) => <GroupCard key={g.key} group={g} />)}
      </div>
    </div>
  )
}

function GroupCard({ group }: { group: PhotoGroup }) {
  const t = useT()
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState('')
  const title = t('albumPrefix') + t(GROUP_LABEL[group.key])

  const handleAlbum = async () => {
    setBusy(true)
    setMessage('')
    try {
      const n = await api.photosAddAlbum(group.key, title)
      setMessage(t('albumCreated', { title, n }))
    } catch (e) {
      setMessage(String(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-4">
      <div className="flex items-baseline justify-between gap-3 mb-3">
        <div>
          <div className="text-base font-semibold text-[#e6edf3]">{t(GROUP_LABEL[group.key])}</div>
          <div className="text-sm text-[#8b949e]">{group.count} · {formatBytes(group.bytes)}</div>
        </div>
        <button
          onClick={() => void handleAlbum()}
          disabled={busy}
          className="shrink-0 px-3 py-1.5 text-sm bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white rounded-md"
        >
          {busy ? t('creatingAlbum') : t('createAlbum')}
        </button>
      </div>
      {message && (
        <div className="flex items-center gap-3 text-sm text-[#3fb950] mb-3">
          <span>{message}</span>
          <button onClick={() => void api.photosOpenApp()} className="text-[#58a6ff] hover:underline">{t('openPhotos')}</button>
        </div>
      )}
      <div className="text-xs font-semibold text-[#8b949e] uppercase tracking-wider mb-1">{t('largest')}</div>
      <div className="space-y-1">
        {group.top.map((a) => (
          <div key={a.id} className="flex justify-between gap-3 text-sm">
            <span className="text-[#c9d1d9] truncate">{a.filename || a.id}</span>
            <span className="text-[#8b949e] shrink-0">
              {a.kind === 'video' ? `${formatDuration(a.duration_s)} · ` : ''}{formatBytes(a.bytes)}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}

function Notice({ children }: { children: React.ReactNode }) {
  return <div className="text-sm text-[#d29922] bg-[#2d1b00] border border-[#d29922] rounded-md px-3 py-2 mb-4 max-w-4xl">{children}</div>
}
