/**
 * Upstream `ImToggleTab.qml`: a button that looks red while `active` and
 * white otherwise.
 */
import { Button } from '@mui/material'

import { accessibleName, redButtonSx, whiteButtonSx } from './buttonStyles'
import { type ImButtonProps, mergeSx } from './ImButton'

export interface ImToggleTabProps extends ImButtonProps {
  active: boolean
}

export function ImToggleTab({
  active,
  accessibleDescription,
  sx,
  children,
  ...rest
}: ImToggleTabProps) {
  const described = accessibleName(children, accessibleDescription)
  const label =
    active && typeof children === 'string'
      ? `${described ?? children} (currently selected)`
      : described
  return (
    <Button
      disableRipple
      disableFocusRipple
      disableElevation
      aria-pressed={active}
      {...(label ? { 'aria-label': label } : {})}
      {...rest}
      sx={mergeSx(active ? redButtonSx : whiteButtonSx, sx)}
    >
      {children}
    </Button>
  )
}
