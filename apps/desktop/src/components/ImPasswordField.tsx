/**
 * Upstream `ImPasswordField.qml`: a password field whose eye toggle only
 * appears once the user has typed something. F2 toggles visibility from
 * the keyboard; the eye button itself is not in the tab order.
 */
import { Visibility, VisibilityOff } from '@mui/icons-material'
import { IconButton, InputAdornment } from '@mui/material'
import { type KeyboardEvent, useState } from 'react'

import { useTr } from '@/i18n'
import { colors } from '@/theme/tokens'

import { ImTextField, type ImTextFieldProps } from './ImTextField'

export interface ImPasswordFieldProps extends Omit<
  ImTextFieldProps,
  'type' | 'onChange' | 'value'
> {
  value: string
  onChange: (value: string) => void
  showToggle?: boolean
  accessibleDescription?: string
}

export function ImPasswordField(props: ImPasswordFieldProps) {
  const { value, onChange, showToggle = true, accessibleDescription, slotProps, ...rest } = props
  const tr = useTr('ImPasswordField')
  const [visible, setVisible] = useState(false)
  const toggleVisible = showToggle && value.length > 0
  const hint = visible
    ? tr('Password is visible. Press F2 to hide.')
    : tr('Password is hidden. Press F2 to show.')
  const description = [accessibleDescription, toggleVisible ? hint : ''].filter(Boolean).join(' ')
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'F2' && toggleVisible) {
      event.preventDefault()
      setVisible((v) => !v)
    }
  }
  const toggleLabel = visible
    ? tr('Password is currently visible. Activate to hide it.')
    : tr('Password is currently hidden. Activate to show it.')
  const adornment = toggleVisible ? (
    <InputAdornment position="end">
      <IconButton
        tabIndex={-1}
        size="small"
        aria-label={toggleLabel}
        onClick={() => setVisible((v) => !v)}
      >
        {visible ? <VisibilityOff fontSize="small" /> : <Visibility fontSize="small" />}
      </IconButton>
    </InputAdornment>
  ) : null
  return (
    <ImTextField
      {...rest}
      type={visible ? 'text' : 'password'}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      onKeyDown={onKeyDown}
      slotProps={{
        ...slotProps,
        htmlInput: description ? { 'aria-description': description } : {},
        input: { endAdornment: adornment },
      }}
      sx={{ '& .MuiIconButton-root': { color: colors.textMetadata } }}
    />
  )
}
