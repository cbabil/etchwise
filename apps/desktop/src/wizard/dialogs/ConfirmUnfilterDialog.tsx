/**
 * Upstream `ConfirmUnfilterDialog.qml`: stern warning before system
 * drives are shown. "KEEP FILTER ON" (red) and Escape cancel; "SHOW
 * SYSTEM DRIVES" confirms.
 */
import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { sizes } from '@/theme/tokens'

import { richText } from '../steps/storage/richText'

export interface ConfirmUnfilterDialogProps {
  open: boolean
  onConfirm: () => void
  onCancel: () => void
}

/** Upstream lets both buttons grow to fit their label. */
const fit = { minWidth: sizes.buttonMinWidth, maxWidth: 'none' }

export function ConfirmUnfilterDialog({ open, onConfirm, onCancel }: ConfirmUnfilterDialogProps) {
  const tr = useTr('ConfirmUnfilterDialog')
  const common = useTr('CommonStrings')
  return (
    <BaseDialog open={open} onClose={onCancel} title={tr('SHOW SYSTEM DRIVES')}>
      <DialogBody>
        {richText(
          tr(
            'By disabling system drive filtering, <b>system drives will be shown</b> in the list.',
          ),
        )}
        <br />
        <br />
        {common(
          'System drives typically contain files essential to the operation of your computer, and may include your personal files (photos, videos, documents).',
        )}
        <br />
        <br />
        {common(
          'Selecting the wrong drive will permanently erase data and can render your computer inoperable.',
        )}
        <br />
        <br />
        {common('Only proceed if you fully understand the risks.')}
      </DialogBody>
      <DialogButtons>
        <ImButtonRed
          onClick={onCancel}
          sx={fit}
          accessibleDescription={tr(
            'Keep system drives hidden to prevent accidental damage to your operating system',
          )}
        >
          {tr('KEEP FILTER ON')}
        </ImButtonRed>
        <ImButton
          onClick={onConfirm}
          sx={fit}
          accessibleDescription={tr(
            'Remove the safety filter and display system drives in the storage device list',
          )}
        >
          {tr('SHOW SYSTEM DRIVES')}
        </ImButton>
      </DialogButtons>
    </BaseDialog>
  )
}
