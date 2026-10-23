/** Heading (16px bold) and body (12px description colour) text used inside `BaseDialog`s. */
import { Box } from '@mui/material'
import type { ReactNode } from 'react'

import { colors, fontSize } from '@/theme/tokens'

export function DialogHeading({ children }: { children: ReactNode }) {
  return (
    <Box
      component="h2"
      sx={{ m: 0, fontSize: fontSize.md, fontWeight: 700, color: colors.formLabel }}
    >
      {children}
    </Box>
  )
}

export function DialogBody({ children }: { children: ReactNode }) {
  return (
    <Box
      component="p"
      sx={{ m: 0, fontSize: fontSize.xs, color: colors.textDescription, overflowWrap: 'anywhere' }}
    >
      {children}
    </Box>
  )
}

/** Right-aligned button row with 15px gaps. */
export function DialogButtons({ children }: { children: ReactNode }) {
  return <Box sx={{ display: 'flex', justifyContent: 'flex-end', gap: '15px' }}>{children}</Box>
}
