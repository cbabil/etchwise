/** State, initialisation and Apply of upstream `RepositoryDialog.qml`. */
import { useEffect, useState } from 'react'

import { call } from '@/api/imager'
import { useTr } from '@/i18n'

import { useWizard } from '../../WizardContext'
import { pickFile } from '../common/pickFile'
import { isWriteIdle } from '../common/writeIdle'

export type RepoSource = 'official' | 'file' | 'url'

/** Local path to a `file://` URL as the backend fetcher expects it. */
export function fileUrlOf(path: string): string {
  const slashed = path.replace(/\\/g, '/')
  return `file://${slashed.startsWith('/') ? '' : '/'}${encodeURI(slashed)}`
}

/** `UrlFmt.display`: a `file://` URL shown as its local path. */
export function displayUrl(url: string): string {
  if (!url.startsWith('file://')) return url
  const path = decodeURI(url.slice('file://'.length))
  return /^\/[A-Za-z]:\//.test(path) ? path.slice(1) : path
}

interface RepoInit {
  source: RepoSource
  file: string
  uri: string
  original: string
}

async function loadRepo(): Promise<RepoInit & { idle: boolean }> {
  const [custom, url, idle] = await Promise.all([
    call<boolean | null>('customRepo'),
    call<string | null>('osListUrl'),
    isWriteIdle(),
  ])
  const repo = custom ? (url ?? '') : ''
  if (repo.startsWith('file:')) return { source: 'file', file: repo, uri: '', original: repo, idle }
  if (/^https?:/.test(repo)) return { source: 'url', file: '', uri: repo, original: repo, idle }
  return { source: 'official', file: '', uri: '', original: '', idle }
}

const blank: RepoInit = { source: 'official', file: '', uri: '', original: '' }

function useUrlValidity(uri: string): boolean {
  const [valid, setValid] = useState(false)
  useEffect(() => {
    let live = true
    void call<boolean | null>('isValidRepoUrl', uri).then((ok) => {
      if (live) setValid(ok === true)
    })
    return () => {
      live = false
    }
  }, [uri])
  return valid
}

export function useRepository(open: boolean) {
  const tr = useTr('RepositoryDialog')
  const common = useTr('CommonStrings')
  const wizard = useWizard()
  const [repo, setRepo] = useState<RepoInit>(blank)
  const [idle, setIdle] = useState(true)
  const urlValid = useUrlValidity(repo.uri)
  useEffect(() => {
    if (!open) return
    let live = true
    void loadRepo().then(({ idle: writeIdle, ...init }) => {
      if (!live) return
      setRepo(init)
      setIdle(writeIdle)
    })
    return () => {
      live = false
    }
  }, [open])
  const change = (patch: Partial<RepoInit>) => setRepo((r) => ({ ...r, ...patch }))
  const browse = async () => {
    const path = await pickFile({
      title: tr('Select Repository'),
      filters: [
        {
          name: common('Imager Repository Files (*.json *.%1)', 'rpi-imager-manifest'),
          extensions: ['json', 'rpi-imager-manifest'],
        },
        { name: common('All files (*)'), extensions: ['*'] },
      ],
    })
    if (path) change({ file: fileUrlOf(path) })
  }
  const canApply =
    idle &&
    (repo.source === 'official' ||
      (repo.source === 'file' && repo.file !== '') ||
      (repo.source === 'url' && urlValid))
  const apply = () => {
    const target = repo.source === 'file' ? repo.file : repo.uri
    if (repo.source === 'official' && repo.original !== '') void call('refreshOsListFromDefaultUrl')
    else if (repo.source !== 'official' && repo.original !== target)
      void call('refreshOsListFrom', target)
    else return
    wizard.resetWizard()
  }
  return { repo, change, browse, canApply, apply }
}
