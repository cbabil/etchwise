/**
 * `main.qml`'s "Storage device removed" dialog: shown after the wizard
 * returned to the Storage step because the selected device vanished.
 */
import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'

import { setRemovedNotice, useRemovedNotice } from './selectedDrive'

export function StorageRemovedDialog() {
  const tr = useTr('main')
  const open = useRemovedNotice()
  const close = () => setRemovedNotice(false)
  return (
    <BaseDialog open={open} onClose={close} title={tr('Storage device removed')}>
      <DialogHeading>{tr('Storage device removed')}</DialogHeading>
      <DialogBody>
        {tr(
          'The storage device was removed while writing, so the operation was cancelled. Please reinsert the device or select a different one to continue.',
        )}
      </DialogBody>
      <DialogButtons>
        <ImButtonRed
          onClick={close}
          accessibleDescription={tr(
            'Close the storage removed notification and return to storage selection',
          )}
        >
          {tr('OK')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
