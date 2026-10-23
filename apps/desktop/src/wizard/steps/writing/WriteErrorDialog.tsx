/**
 * `main.qml`'s error dialog for a `startWrite` call the backend rejects
 * outright (no `error` signal follows): "Error", the message, Continue.
 */
import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'

export function WriteErrorDialog({ message, onClose }: { message: string; onClose: () => void }) {
  const tr = useTr('main')
  const common = useTr('CommonStrings')
  return (
    <BaseDialog open={message !== ''} onClose={onClose} title={tr('Error')}>
      <DialogHeading>{tr('Error')}</DialogHeading>
      <DialogBody>{message}</DialogBody>
      <DialogButtons>
        <ImButton
          onClick={onClose}
          accessibleDescription={tr('Close the error dialog and continue')}
        >
          {common('Continue')}
        </ImButton>
      </DialogButtons>
    </BaseDialog>
  )
}
