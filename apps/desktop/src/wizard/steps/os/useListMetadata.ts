/**
 * What upstream `onOsListPreparedHandler` does after a full load of the
 * main list: offer the update dialog (once per run, as main.qml's
 * `updatePopupShown`) when `check_version` is on and the repository
 * advertises a newer Imager (currently off, see `UPDATE_CHECK_ENABLED`),
 * and select `default_os` (plus
 * `embedded_default_os` in embedded mode).
 */
import { useState } from 'react'

import { call } from '@/api/imager'

import type { OsEntry } from './types'

export interface UpdateOffer {
  url: string
  version: string
}

type Metadata = Readonly<Record<string, unknown>>

/**
 * Off: the official list's `latest_version` is Raspberry Pi Imager's, so
 * offering it would send users to install Raspberry Pi's app in place of
 * this one. Turn back on once this app publishes its own releases.
 */
const UPDATE_CHECK_ENABLED = false

let updatePopupShown = false

/** Test hook: forget that the update dialog was shown this run. */
export function resetUpdatePopupShown(): void {
  updatePopupShown = false
}

async function updateOffer(meta: Metadata, embedded: boolean): Promise<UpdateOffer | null> {
  if (!UPDATE_CHECK_ENABLED) return null
  if (updatePopupShown || embedded || !('latest_version' in meta) || !('url' in meta)) return null
  const check = await call<boolean | null>('getBoolSetting', 'check_version')
  const version = String(meta.latest_version ?? '')
  if (!check || !(await call<boolean | null>('isVersionNewer', version))) return null
  updatePopupShown = true
  return { url: String(meta.url ?? ''), version }
}

export function useListMetadata(select: (entry: OsEntry) => void) {
  const [offer, setOffer] = useState<UpdateOffer | null>(null)
  const selectNamed = (name: unknown, entries: readonly OsEntry[]) => {
    const entry = typeof name === 'string' ? entries.find((e) => e.name === name) : undefined
    if (entry) select(entry)
  }
  const onLoaded = async (entries: readonly OsEntry[]) => {
    const [meta, embedded] = await Promise.all([
      call<Metadata | null>('imagerMetadata'),
      call<boolean | null>('isEmbeddedMode'),
    ])
    if (!meta || typeof meta !== 'object') return
    const found = await updateOffer(meta, Boolean(embedded))
    if (found) setOffer(found)
    if ('default_os' in meta) selectNamed(meta.default_os, entries)
    if (embedded && 'embedded_default_os' in meta) selectNamed(meta.embedded_default_os, entries)
  }
  return { offer, closeOffer: () => setOffer(null), onLoaded }
}
