/**
 * Upstream `ImCheckBox.qml`: a Material checkbox in the form-control blue,
 * 14px label (grey when disabled), Enter/Return/Space toggle, and a 2px
 * dark-red underline under the label while keyboard-focused.
 */
import { Checkbox, FormControlLabel } from '@mui/material'
import type { KeyboardEvent } from 'react'

import { colors, fontSize } from '@/theme/tokens'

export interface ImCheckBoxProps {
  label: string
  checked: boolean
  onChange: (checked: boolean) => void
  disabled?: boolean
  accessibleDescription?: string
}

export function ImCheckBox({
  label,
  checked,
  onChange,
  disabled,
  accessibleDescription,
}: ImCheckBoxProps) {
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter') {
      event.preventDefault()
      onChange(!checked)
    }
  }
  return (
    <FormControlLabel
      disabled={disabled ?? false}
      label={label}
      sx={{
        mx: 0,
        alignItems: 'center',
        '& .MuiFormControlLabel-label': {
          fontSize: fontSize.sm,
          color: colors.formLabel,
          borderBottom: '2px solid transparent',
          pt: '2px',
        },
        '& .MuiFormControlLabel-label.Mui-disabled': { color: colors.formLabelDisabled },
        '&:has(.Mui-focusVisible) .MuiFormControlLabel-label': {
          borderBottomColor: colors.button2FocusedBackground,
        },
      }}
      control={
        <Checkbox
          checked={checked}
          disableRipple
          onChange={(_, value) => onChange(value)}
          onKeyDown={onKeyDown}
          slotProps={{
            input: accessibleDescription ? { 'aria-description': accessibleDescription } : {},
          }}
          sx={{
            p: 1,
            color: colors.textMetadata,
            '&.Mui-checked': { color: colors.formControlActive },
          }}
        />
      }
    />
  )
}
