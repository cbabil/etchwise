/**
 * The device list of upstream `DeviceSelectionStep.qml`: loads once, and
 * reloads when `osListPrepared` arrives while empty or when the OS list
 * becomes available again. The backend pre-selects the repository's
 * default device, which is highlighted here.
 */
import { useCallback, useEffect, useState } from 'react'

import { call } from '@/api/imager'

import type { HwDevice, HwList } from '../os/types'
import { useImagerEvent } from '../os/useImagerEvent'

export interface HwListState {
  devices: readonly HwDevice[]
  /** Highlighted row; -1 for none. */
  current: number
  setCurrent: (index: number) => void
  unavailable: boolean
}

const fetchHwList = () =>
  Promise.all([call<HwList | null>('getHWList'), call<boolean | null>('isOsListUnavailable')])

export function useHwList(): HwListState {
  const [devices, setDevices] = useState<readonly HwDevice[]>([])
  const [current, setCurrent] = useState(-1)
  const [unavailable, setUnavailable] = useState(false)
  const load = useCallback(
    () =>
      fetchHwList().then(([list, offline]) => {
        setUnavailable(Boolean(offline))
        setDevices(list?.devices ?? [])
        if (list && list.currentIndex >= 0) setCurrent(list.currentIndex)
      }),
    [],
  )
  useEffect(() => {
    void load()
  }, [load])
  useImagerEvent((event) => {
    if (event.name === 'osListPrepared' && devices.length === 0) void load()
    if (event.name === 'osListUnavailableChanged') void load()
  })
  return { devices, current, setCurrent, unavailable }
}
