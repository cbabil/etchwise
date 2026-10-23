/**
 * Upstream `WritingStep.qml` `confirmDialog`: the red "ERASE" warning,
 * a 2-second "Please wait... N" countdown, then Cancel and "I understand,
 * erase and write". Escape closes it.
 */
import { Box } from '@mui/material'
import { useEffect, useState } from 'react'

import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

export const CONFIRM_COUNTDOWN = 2

/** Counts from 2 to 0 once a second while `open`; restarts on reopening. */
function useCountdown(open: boolean): number {
  const [left, setLeft] = useState(CONFIRM_COUNTDOWN)
  const [wasOpen, setWasOpen] = useState(open)
  if (open !== wasOpen) {
    setWasOpen(open)
    setLeft(CONFIRM_COUNTDOWN)
  }
  useEffect(() => {
    if (!open) return
    const timer = setInterval(() => setLeft((n) => Math.max(0, n - 1)), 1000)
    return () => clearInterval(timer)
  }, [open])
  return left
}

export interface EraseConfirmDialogProps {
  open: boolean
  storageName: string
  onAccept: () => void
  onCancel: () => void
}

function Actions({
  left,
  onAccept,
  onCancel,
}: {
  left: number
  onAccept: () => void
  onCancel: () => void
}) {
  const tr = useTr('WritingStep')
  const common = useTr('CommonStrings')
  if (left > 0)
    return (
      <Box
        sx={{
          mt: `${spacing.small}px`,
          textAlign: 'right',
          fontSize: fontSize.sm,
          color: colors.textMetadata,
        }}
      >
        {tr('Please wait... %1', left)}
      </Box>
    )
  return (
    <Box sx={{ mt: `${spacing.small}px` }}>
      <DialogButtons>
        <ImButton
          onClick={onCancel}
          accessibleDescription={tr(
            'Cancel and return to the write summary without erasing the storage device',
          )}
        >
          {common('Cancel')}
        </ImButton>
        <ImButtonRed
          onClick={onAccept}
          sx={{ maxWidth: 'none' }}
          accessibleDescription={tr(
            'Confirm erasure and begin writing the image to the storage device',
          )}
        >
          {tr('I understand, erase and write')}
        </ImButtonRed>
      </DialogButtons>
    </Box>
  )
}

export function EraseConfirmDialog({
  open,
  storageName,
  onAccept,
  onCancel,
}: EraseConfirmDialogProps) {
  const tr = useTr('WritingStep')
  const left = useCountdown(open)
  const warning = tr(
    'You are about to ERASE all data on: %1',
    storageName || tr('the storage device'),
  )
  return (
    <BaseDialog open={open} onClose={onCancel} title={warning}>
      <Box
        component="h2"
        sx={{ m: 0, fontSize: fontSize.md, fontWeight: 700, color: colors.formLabelError }}
      >
        {warning}
      </Box>
      <Box sx={{ fontSize: fontSize.sm, fontWeight: 700, color: colors.formLabel }}>
        {tr('This action is PERMANENT and CANNOT be undone.')}
      </Box>
      <Actions left={left} onAccept={onAccept} onCancel={onCancel} />
      <Box sx={{ height: sizes.cardPadding }} />
    </BaseDialog>
  )
}
