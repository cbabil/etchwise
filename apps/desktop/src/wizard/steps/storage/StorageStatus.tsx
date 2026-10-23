/**
 * Upstream's enumeration error banner (red, white text, warning icon)
 * and the visually hidden status label screen readers announce when no
 * device can be selected.
 */
import WarningAmberIcon from '@mui/icons-material/WarningAmber'
import { Box } from '@mui/material'

import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import type { StorageStatus } from './drives'

export function EnumerationErrorBanner({ error }: { error: string }) {
  const tr = useTr('StorageSelectionStep')
  if (!error) return null
  return (
    <Box
      role="alert"
      aria-label={tr('Error: Could not list storage devices. %1', error)}
      sx={{
        flexShrink: 0,
        display: 'flex',
        alignItems: 'center',
        gap: `${spacing.small}px`,
        p: `${spacing.medium}px`,
        backgroundColor: colors.formLabelError,
        opacity: 0.9,
        color: '#ffffff',
      }}
    >
      <WarningAmberIcon sx={{ fontSize: fontSize.sm + 4 }} />
      <Box sx={{ flex: 1, fontSize: fontSize.xs, overflowWrap: 'anywhere' }}>
        {tr('Could not list storage devices: %1', error)}
      </Box>
    </Box>
  )
}

const hidden = {
  position: 'absolute',
  width: 1,
  height: 1,
  overflow: 'hidden',
  clipPath: 'inset(50%)',
  whiteSpace: 'nowrap',
} as const

/** Upstream `noDevicesLabel` (opacity 0, read by screen readers). */
export function NoDevicesStatus({ status, error }: { status: StorageStatus; error: string }) {
  const tr = useTr('StorageSelectionStep')
  const text: Record<StorageStatus, string> = {
    ok: '',
    error: tr('Could not list storage devices: %1', error),
    none: tr('No storage devices found'),
    readOnlyFiltered: tr(
      "All visible devices are read-only.\nTry connecting a new device, or uncheck\n'Exclude system drives' below.",
    ),
    readOnly: tr('All devices are read-only.\nPlease connect a writable storage device.'),
    hidden: tr(
      "All devices are hidden by the filter.\nUncheck 'Exclude system drives' below\nto show system drives.",
    ),
  }
  return (
    <Box role="status" sx={hidden}>
      {text[status]}
    </Box>
  )
}
