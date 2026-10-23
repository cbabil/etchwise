/** Upstream `PiConnectCustomizationStep.qml` "Invalid Token" dialog. */
import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'
import { colors, fontSize } from '@/theme/tokens'

import { InfoText } from '../../dialogs/common/dialogParts'

export interface InvalidTokenDialogProps {
  open: boolean
  /** OK: closes and clears the token. */
  onOk: () => void
  /** Escape: closes only. */
  onDismiss: () => void
}

export function InvalidTokenDialog({ open, onOk, onDismiss }: InvalidTokenDialogProps) {
  const tr = useTr('PiConnectCustomizationStep')
  return (
    <BaseDialog open={open} onClose={onDismiss} title={tr('Invalid Token')}>
      <InfoText size={fontSize.md} bold color={colors.formLabelError}>
        {tr('Invalid Token')}
      </InfoText>
      <InfoText size={fontSize.sm} color={colors.formLabel}>
        {tr(
          "The token you entered is not valid. Please check the token and try again, or use the 'Open Raspberry Pi Connect' button to get a valid token.",
        )}
      </InfoText>
      <DialogButtons>
        <ImButton
          onClick={onOk}
          accessibleDescription={tr('Close this dialog and return to the token field')}
        >
          {tr('OK')}
        </ImButton>
      </DialogButtons>
    </BaseDialog>
  )
}
