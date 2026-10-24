/**
 * Which Device-step tab is shown: "Raspberry Pi" for the main
 * repository's devices, "Other" for those merged from an extra repository
 * (`group: "other"`). Rows keep their position in the full list, which the
 * backend's `setHWIndex` and `currentIndex` use.
 */
import { useState } from 'react'

import type { HwDevice } from '../os/types'

export type DeviceTab = 'pi' | 'other'

interface TabRow {
  device: HwDevice
  /** Position in the full device list. */
  index: number
}

const tabOf = (device: HwDevice | undefined): DeviceTab =>
  device?.group === 'other' ? 'other' : 'pi'

export function useDeviceTab(devices: readonly HwDevice[], current: number) {
  const [chosen, setChosen] = useState<DeviceTab | null>(null)
  // Until the user picks a tab, show the one holding the selected device.
  const tab = chosen ?? (current >= 0 ? tabOf(devices[current]) : 'pi')
  const rows: TabRow[] = devices
    .map((device, index) => ({ device, index }))
    .filter((row) => tabOf(row.device) === tab)
  return {
    tab,
    choose: setChosen,
    items: rows.map((row) => row.device),
    /** Row in this tab -> position in the full list (-1 when out of range). */
    indexOf: (row: number) => rows[row]?.index ?? -1,
    /** Position in the full list -> row in this tab (-1 when elsewhere). */
    rowOf: (index: number) => rows.findIndex((r) => r.index === index),
  }
}
