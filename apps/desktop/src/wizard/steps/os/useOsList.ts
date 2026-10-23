/**
 * The OS list and its drill-down pages (upstream's `SwipeView` of
 * `OSSelectionListView`s). Level 0 is the main list; each opened group
 * pushes a page whose first row is "Back". Reloads follow upstream's
 * `osListPrepared` / `osListUnavailableChanged` handlers; `onLoaded` runs
 * after each full (re)load of the main list.
 */
import { useCallback, useEffect, useRef, useState } from 'react'

import { call } from '@/api/imager'

import { BACK_ROW, type OsEntry } from './types'
import { useImagerEvent } from './useImagerEvent'

export interface OsLevel {
  entries: readonly OsEntry[]
  current: number
}

export interface OsListState {
  levels: readonly OsLevel[]
  level: OsLevel
  unavailable: boolean
  setCurrent: (index: number) => void
  openGroup: (entry: OsEntry) => Promise<void>
  goBack: () => void
}

type Levels = readonly OsLevel[]
const replaceTop = (levels: Levels, patch: Partial<OsLevel>): Levels =>
  levels.map((l, i) => (i === levels.length - 1 ? { ...l, ...patch } : l))

const fetchOsList = () =>
  Promise.all([call<OsEntry[] | null>('getOSList'), call<boolean | null>('isOsListUnavailable')])

function useLoader(
  setLevels: (change: (levels: Levels) => Levels) => void,
  onLoaded: (entries: readonly OsEntry[]) => void,
) {
  const [unavailable, setUnavailable] = useState(false)
  const loaded = useRef(false)
  const count = useRef(0)
  const latest = useRef(onLoaded)
  useEffect(() => {
    latest.current = onLoaded
  })
  const load = useCallback(
    (full: boolean) =>
      fetchOsList().then(([entries, offline]) => {
        if (full) loaded.current = false
        const list = entries ?? []
        count.current = list.length
        setUnavailable(Boolean(offline))
        setLevels((levels) => levels.map((l, i) => (i === 0 ? { ...l, entries: list } : l)))
        if (!loaded.current && entries) {
          loaded.current = true
          latest.current(list)
        }
      }),
    [setLevels],
  )
  useEffect(() => {
    void load(false)
  }, [load])
  useImagerEvent((event) => {
    if (event.name === 'osListPrepared')
      void call<boolean | null>('isOsListUnavailable').then((offline) => {
        if (!offline) void load(!loaded.current || count.current <= 2)
      })
    if (event.name === 'osListUnavailableChanged') void load(true)
  })
  return unavailable
}

export function useOsList(onLoaded: (entries: readonly OsEntry[]) => void): OsListState {
  const [levels, setLevels] = useState<Levels>([{ entries: [], current: -1 }])
  const unavailable = useLoader(setLevels, onLoaded)
  const setCurrent = useCallback(
    (index: number) => setLevels((all) => replaceTop(all, { current: index })),
    [],
  )
  const openGroup = useCallback(async (entry: OsEntry) => {
    const children = (await call<OsEntry[] | null>('getSublist', entry)) ?? []
    setLevels((all) => [...all, { entries: [BACK_ROW, ...children], current: 0 }])
  }, [])
  const goBack = useCallback(
    () => setLevels((all) => (all.length > 1 ? all.slice(0, -1) : all)),
    [],
  )
  const level = levels[levels.length - 1] as OsLevel
  return { levels, level, unavailable, setCurrent, openGroup, goBack }
}
