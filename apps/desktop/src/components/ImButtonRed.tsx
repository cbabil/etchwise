/** Upstream `ImButtonRed.qml`: the red primary button. */
import { Button } from '@mui/material'

import { accessibleName, redButtonSx } from './buttonStyles'
import { type ImButtonProps, mergeSx } from './ImButton'

export function ImButtonRed({ accessibleDescription, sx, children, ...rest }: ImButtonProps) {
  const label = accessibleName(children, accessibleDescription)
  return (
    <Button
      disableRipple
      disableFocusRipple
      disableElevation
      {...(label ? { 'aria-label': label } : {})}
      {...rest}
      sx={mergeSx(redButtonSx, sx)}
    >
      {children}
    </Button>
  )
}
