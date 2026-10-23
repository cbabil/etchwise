/**
 * Upstream `ImOptionPill.qml`: a bold 14px label (plus an optional 12px
 * help link opened in the browser) on the left and a Material switch on
 * the right; the switch accent turns raspberry red while focused.
 */
import { Box, Stack, Switch } from '@mui/material'
import type { KeyboardEvent } from 'react'

import { call } from '@/api/imager'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

export interface ImOptionPillProps {
  label: string
  checked: boolean
  onToggle: (checked: boolean) => void
  helpLabel?: string
  helpUrl?: string
  accessibleDescription?: string
  disabled?: boolean
}

function HelpLink({ label, url }: { label: string; url: string }) {
  const tr = useTr('ImOptionPill')
  const open = () => void call('openUrl', url)
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      open()
    }
  }
  return (
    <Box
      component="span"
      role="link"
      tabIndex={0}
      aria-description={tr('Opens in browser')}
      onClick={open}
      onKeyDown={onKeyDown}
      sx={{
        fontSize: fontSize.xs,
        color: colors.buttonForeground,
        cursor: 'pointer',
        outline: 'none',
        '&:hover': { textDecoration: 'underline' },
        '&:focus-visible': { color: colors.raspberryRed, textDecoration: 'underline' },
      }}
    >
      {label}
    </Box>
  )
}

export function ImOptionPill(props: ImOptionPillProps) {
  const { label, checked, onToggle, helpLabel, helpUrl, accessibleDescription, disabled } = props
  const description = accessibleDescription || helpLabel
  return (
    <Stack
      direction="row"
      spacing={`${spacing.medium}px`}
      sx={{ alignItems: 'center', minHeight: Math.max(sizes.buttonHeight - 8, 28) }}
    >
      <Stack spacing={`${spacing.xxSmall}px`} sx={{ minWidth: 0, flex: 1 }}>
        <Box
          onClick={() => !disabled && onToggle(!checked)}
          sx={{
            fontSize: fontSize.sm,
            fontWeight: 700,
            color: colors.formLabel,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {label}
        </Box>
        {helpLabel && helpUrl ? <HelpLink label={helpLabel} url={helpUrl} /> : null}
      </Stack>
      <Switch
        checked={checked}
        disabled={disabled ?? false}
        disableRipple
        onChange={(_, value) => onToggle(value)}
        onKeyDown={(event) => event.key === 'Enter' && onToggle(!checked)}
        slotProps={{ input: { 'aria-label': description ? `${label}, ${description}` : label } }}
        sx={{
          '& .MuiSwitch-switchBase.Mui-checked': { color: colors.formControlActive },
          '& .MuiSwitch-switchBase.Mui-checked + .MuiSwitch-track': {
            backgroundColor: colors.formControlActive,
            opacity: 0.5,
          },
          '& .MuiSwitch-switchBase.Mui-focusVisible.Mui-checked': { color: colors.raspberryRed },
          '& .MuiSwitch-switchBase.Mui-focusVisible.Mui-checked + .MuiSwitch-track': {
            backgroundColor: colors.raspberryRed,
          },
        }}
      />
    </Stack>
  )
}
