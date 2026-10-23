/**
 * Upstream `PasswordlessSudoWarningDialog.qml`: a red "Passwordless Sudo"
 * heading, the risk explanation, and CANCEL / ENABLE. Escape, a backdrop
 * click and CANCEL all report `onCancel`; ENABLE reports `onConfirm`.
 */
import { Box } from '@mui/material'

import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { colors, fontSize } from '@/theme/tokens'

export interface PasswordlessSudoWarningDialogProps {
  open: boolean
  onConfirm: () => void
  onCancel: () => void
}

export function PasswordlessSudoWarningDialog({
  open,
  onConfirm,
  onCancel,
}: PasswordlessSudoWarningDialogProps) {
  const tr = useTr('PasswordlessSudoWarningDialog')
  const title = tr('Passwordless Sudo')
  return (
    <BaseDialog open={open} onClose={onCancel} title={title}>
      <Box
        component="h2"
        sx={{ m: 0, fontSize: fontSize.md, fontWeight: 700, color: colors.formLabelError }}
      >
        {title}
      </Box>
      <DialogBody>
        {tr(
          'Enabling passwordless sudo allows any process running as this user to gain full root privileges without authentication. This significantly weakens the security of your system.',
        )}
      </DialogBody>
      <Box component="p" sx={{ m: 0, fontSize: fontSize.xs, color: colors.textMetadata }}>
        {tr(
          'Only enable this if you understand the risks and have a specific need, such as automated scripts or headless operation.',
        )}
      </Box>
      <DialogButtons>
        <ImButton
          accessibleDescription={tr('Cancel and keep sudo requiring a password')}
          onClick={onCancel}
        >
          {tr('CANCEL')}
        </ImButton>
        <ImButtonRed
          accessibleDescription={tr('Enable passwordless sudo for this user account')}
          onClick={onConfirm}
        >
          {tr('ENABLE')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
