/**
 * Upstream `ImRadioButton.qml`: a Material radio in the form-control blue
 * with a 2px focus outline drawn 4px outside the control. Space, Enter
 * and Return select it but never uncheck it.
 */
import { FormControlLabel, Radio } from '@mui/material'
import type { KeyboardEvent } from 'react'

import { colors, fontSize, sizes } from '@/theme/tokens'

export interface ImRadioButtonProps {
  label: string
  checked: boolean
  onSelect: () => void
  disabled?: boolean
  name?: string
  value?: string
  accessibleDescription?: string
}

export function ImRadioButton(props: ImRadioButtonProps) {
  const { label, checked, onSelect, disabled, name, value, accessibleDescription } = props
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      if (!checked) onSelect()
    }
  }
  return (
    <FormControlLabel
      disabled={disabled ?? false}
      label={label}
      sx={{
        mx: 0,
        borderRadius: `${sizes.focusOutlineRadius}px`,
        outline: '2px solid transparent',
        outlineOffset: 4,
        '&:has(.Mui-focusVisible)': { outlineColor: colors.focusOutline },
        '& .MuiFormControlLabel-label': { fontSize: fontSize.sm, color: colors.formLabel },
        '& .MuiFormControlLabel-label.Mui-disabled': { color: colors.formLabelDisabled },
      }}
      control={
        <Radio
          checked={checked}
          disableRipple
          onChange={() => onSelect()}
          onKeyDown={onKeyDown}
          slotProps={{
            input: {
              ...(name ? { name } : {}),
              ...(value ? { value } : {}),
              ...(accessibleDescription ? { 'aria-description': accessibleDescription } : {}),
            },
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
