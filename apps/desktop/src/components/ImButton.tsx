/** Upstream `ImButton.qml`: the white secondary button. */
import { Button, type ButtonProps } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'

import { accessibleName, whiteButtonSx } from './buttonStyles'

export interface ImButtonProps extends Omit<ButtonProps, 'variant' | 'color'> {
  /** Appended to the accessible name ("Label, description"). */
  accessibleDescription?: string
}

export function mergeSx(base: SxProps<Theme>, extra: SxProps<Theme> | undefined): SxProps<Theme> {
  if (!extra) return base
  return [base, ...(Array.isArray(extra) ? extra : [extra])] as SxProps<Theme>
}

export function ImButton({ accessibleDescription, sx, children, ...rest }: ImButtonProps) {
  const label = accessibleName(children, accessibleDescription)
  return (
    <Button
      disableRipple
      disableFocusRipple
      disableElevation
      {...(label ? { 'aria-label': label } : {})}
      {...rest}
      sx={mergeSx(whiteButtonSx, sx)}
    >
      {children}
    </Button>
  )
}
