/**
 * Upstream `OSSelectionStep.qml` offline banner: a #f5f5f5 strip with a
 * warning sign, the explanation and a Retry button that restarts the OS
 * list fetch.
 */
import { WarningAmber } from '@mui/icons-material'
import { Box } from '@mui/material'

import { call } from '@/api/imager'
import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

export function OsOfflineBanner() {
  const tr = useTr('OSSelectionStep')
  return (
    <Box
      sx={{
        flexShrink: 0,
        display: 'flex',
        alignItems: 'center',
        gap: `${spacing.medium}px`,
        px: `${spacing.medium}px`,
        py: `${spacing.medium}px`,
        backgroundColor: colors.titleBackground,
      }}
    >
      <WarningAmber aria-hidden sx={{ fontSize: fontSize.sm, color: colors.formLabel }} />
      <Box
        component="p"
        sx={{ m: 0, flex: 1, minWidth: 0, fontSize: fontSize.xs, color: colors.formLabel }}
      >
        {tr('Unable to download OS list. You can still use a local image file.')}
      </Box>
      <ImButton
        accessibleDescription={tr('Retry downloading the OS list')}
        onClick={() => void call('beginOSListFetch')}
      >
        {tr('Retry')}
      </ImButton>
    </Box>
  )
}
