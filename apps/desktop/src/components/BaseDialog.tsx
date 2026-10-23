/**
 * Upstream `BaseDialog.qml`: a modal on a 30% black backdrop, at least
 * 400px wide (at most the window minus 40px), #f5f5f5 with a #dcdcdc
 * border and 8px radius, 20px padding and 15px between children. Escape
 * and a backdrop click close it unless `onClose` is omitted.
 */
import { Dialog, Stack } from '@mui/material'
import type { ReactNode } from 'react'

import { colors, sizes, spacing } from '@/theme/tokens'

export interface BaseDialogProps {
  open: boolean
  /** Called on Escape (upstream `escapePressed`). */
  onClose?: () => void
  /** Accessible name of the dialog. */
  title?: string
  children: ReactNode
}

export function BaseDialog({ open, onClose, title, children }: BaseDialogProps) {
  return (
    <Dialog
      open={open}
      onClose={() => onClose?.()}
      maxWidth={false}
      transitionDuration={150}
      slotProps={{
        backdrop: { sx: { backgroundColor: colors.dialogBackdrop } },
        paper: {
          elevation: 0,
          ...(title ? { 'aria-label': title } : {}),
          sx: {
            m: 0,
            minWidth: sizes.dialogWidth,
            maxWidth: `max(${sizes.dialogWidth}px, calc(100% - ${sizes.cardPadding * 2}px))`,
            backgroundColor: colors.titleBackground,
            border: `1px solid ${colors.popupBorder}`,
            borderRadius: `${sizes.sectionBorderRadius}px`,
            backgroundImage: 'none',
          },
        },
      }}
    >
      <Stack spacing={`${spacing.medium}px`} sx={{ p: `${sizes.dialogPadding}px` }}>
        {children}
      </Stack>
    </Dialog>
  )
}
