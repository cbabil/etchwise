/**
 * Upstream `DeviceSelectionStep.qml` offline placeholder: a large warning
 * sign, heading, explanation and a Retry button that restarts the OS list
 * fetch, centred at 80% of the step width.
 */
import { WarningAmber } from '@mui/icons-material'
import { Box } from '@mui/material'

import { call } from '@/api/imager'
import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

export function DeviceOfflinePlaceholder() {
  const tr = useTr('DeviceSelectionStep')
  return (
    <Box sx={{ flex: 1, minHeight: 0, display: 'flex', alignItems: 'center' }}>
      <Box
        sx={{
          width: '80%',
          mx: 'auto',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          gap: `${spacing.large}px`,
          textAlign: 'center',
        }}
      >
        <WarningAmber aria-hidden sx={{ fontSize: 48, color: colors.textDescription }} />
        <Box
          component="h2"
          sx={{ m: 0, fontSize: fontSize.md, fontWeight: 700, color: colors.formLabel }}
        >
          {tr('Unable to load device list')}
        </Box>
        <Box
          component="p"
          sx={{
            m: 0,
            fontSize: fontSize.xs,
            color: colors.textDescription,
            lineHeight: 1.3,
            whiteSpace: 'pre-line',
          }}
        >
          {tr(
            "The device list could not be downloaded. Please check your internet connection and try again.\n\nYou can still write a local image file by pressing Next and selecting 'Use custom' on the following screen.",
          )}
        </Box>
        <ImButton
          accessibleDescription={tr('Retry downloading the device list')}
          onClick={() => void call('beginOSListFetch')}
        >
          {tr('Retry')}
        </ImButton>
      </Box>
    </Box>
  )
}
