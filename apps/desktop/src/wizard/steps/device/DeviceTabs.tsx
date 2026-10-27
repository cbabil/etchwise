/**
 * The Device step's "Raspberry Pi" | "Other" tabs. Only the active tab is
 * marked (its underline); the tab row has no border of its own.
 */
import { Tab, Tabs } from '@mui/material'

import { useTr } from '@/i18n'
import { fontSize } from '@/theme/tokens'

import type { DeviceTab } from './useDeviceTab'

const tabSx = { textTransform: 'none', fontSize: fontSize.sm, minHeight: 36, py: 0 } as const

export function DeviceTabs({
  tab,
  onChange,
}: {
  tab: DeviceTab
  onChange: (tab: DeviceTab) => void
}) {
  const tr = useTr('DeviceSelectionStep')
  return (
    <Tabs
      value={tab}
      onChange={(_, next: DeviceTab) => onChange(next)}
      aria-label={tr('Device groups')}
      sx={{ minHeight: 36, flexShrink: 0, borderBottom: 'none' }}
    >
      <Tab value="pi" label={tr('Raspberry Pi')} sx={tabSx} />
      <Tab value="other" label={tr('Other')} sx={tabSx} />
    </Tabs>
  )
}
