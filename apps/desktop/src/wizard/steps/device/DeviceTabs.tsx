/**
 * The Device step's "Raspberry Pi" | "Other" tabs. Only the active tab is
 * marked (its underline); the tab row has no border of its own.
 */
import { Box, Tab, Tabs } from '@mui/material'

import { colors, fontSize } from '@/theme/tokens'

import type { DeviceTab } from './useDeviceTab'

const tabSx = { textTransform: 'none', fontSize: fontSize.sm, minHeight: 36, py: 0 } as const

export function DeviceTabs({
  tab,
  onChange,
}: {
  tab: DeviceTab
  onChange: (tab: DeviceTab) => void
}) {
  return (
    <Tabs
      value={tab}
      onChange={(_, next: DeviceTab) => onChange(next)}
      aria-label="Device groups"
      sx={{ minHeight: 36, flexShrink: 0, borderBottom: 'none' }}
    >
      <Tab value="pi" label="Raspberry Pi" sx={tabSx} />
      <Tab value="other" label="Other" sx={tabSx} />
    </Tabs>
  )
}

/** Shown in place of the list when no extra repository adds devices. */
export function OtherTabHint() {
  return (
    <Box sx={{ p: 2, color: colors.textMetadata, fontSize: fontSize.sm }}>
      Add devices with App Options &gt; Extra Repository
    </Box>
  )
}
