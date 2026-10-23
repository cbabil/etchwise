/**
 * Upstream `ImCloseButton.qml`: a bold "X" pinned 25px from the right and
 * 10px from the top of its positioned parent.
 */
import { Box } from '@mui/material'
import type { KeyboardEvent } from 'react'

import { fontFamily, fontSize } from '@/theme/tokens'

export interface CloseButtonProps {
  onClick: () => void
}

export function CloseButton({ onClick }: CloseButtonProps) {
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      onClick()
    }
  }
  return (
    <Box
      role="button"
      tabIndex={0}
      aria-label="Close"
      aria-description="Close dialog"
      onClick={onClick}
      onKeyDown={onKeyDown}
      sx={{
        position: 'absolute',
        right: 25,
        top: 10,
        fontFamily,
        fontSize: fontSize.sm,
        fontWeight: 700,
        cursor: 'default',
        userSelect: 'none',
      }}
    >
      X
    </Box>
  )
}
