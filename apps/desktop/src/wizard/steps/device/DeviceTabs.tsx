/**
 * The Device step's "Raspberry Pi" | "Other" tabs. Only the active tab is
 * marked (its underline); the tab row has no border of its own.
 */
import { Box, Tab, Tabs } from '@mui/material'

import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

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

/** Shown in place of the list when no extra repository adds devices. */
export function OtherTabHint() {
  const tr = useTr('DeviceSelectionStep')
  return (
    <Box sx={{ p: `${spacing.medium}px`, color: colors.textMetadata, fontSize: fontSize.sm }}>
      {tr('Add devices with App Options > Extra Repository')}
    </Box>
  )
}
