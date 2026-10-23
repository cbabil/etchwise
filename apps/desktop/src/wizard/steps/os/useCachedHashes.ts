/**
 * Which rows' images are in the download cache (`imageWriter.isCached`),
 * re-asked whenever the rows change or `cacheStatusChanged` arrives.
 */
import { useEffect, useMemo, useState } from 'react'

import { call } from '@/api/imager'

import type { OsEntry } from './types'
import { useImagerEvent } from './useImagerEvent'

export function useCachedHashes(entries: readonly OsEntry[]): ReadonlySet<string> {
  const [cached, setCached] = useState<ReadonlySet<string>>(new Set())
  const [version, setVersion] = useState(0)
  useImagerEvent((event) => {
    if (event.name === 'cacheStatusChanged') setVersion((v) => v + 1)
  })
  const hashes = useMemo(
    () => [...new Set(entries.map((e) => e.extract_sha256 ?? '').filter(Boolean))],
    [entries],
  )
  const key = hashes.join(',')
  useEffect(() => {
    let live = true
    void Promise.all(hashes.map(async (h) => ((await call<boolean>('isCached', h)) ? h : ''))).then(
      (hits) => {
        if (live) setCached(new Set(hits.filter(Boolean)))
      },
    )
    return () => {
      live = false
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, version])
  return cached
}
