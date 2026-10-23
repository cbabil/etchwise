/**
 * Upstream `KeychainPermissionDialog.qml`: asks before the Wi-Fi password
 * is read from the macOS keychain (which prompts for admin credentials).
 * `onResponse(true)` for Yes; No, Escape and a backdrop click give `false`.
 */
import { Box } from '@mui/material'

import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { colors, fontSize } from '@/theme/tokens'

export interface KeychainPermissionDialogProps {
  open: boolean
  onResponse: (granted: boolean) => void
}

/** Upstream `Layout.preferredWidth: 80`. */
const narrow = { minWidth: 80, width: 80 } as const

export function KeychainPermissionDialog({ open, onResponse }: KeychainPermissionDialogProps) {
  const tr = useTr('KeychainPermissionDialog')
  const common = useTr('CommonStrings')
  const title = tr('Keychain Access')
  return (
    <BaseDialog open={open} onClose={() => onResponse(false)} title={title}>
      <DialogHeading>{title}</DialogHeading>
      <DialogBody>
        {tr('Would you like to prefill the Wi‑Fi password from the system keychain?')}
      </DialogBody>
      <Box component="p" sx={{ m: 0, fontSize: fontSize.xs, color: colors.textMetadata }}>
        {tr('This will require administrator authentication on macOS.')}
      </Box>
      <DialogButtons>
        <ImButton
          accessibleDescription={tr('Skip keychain access and manually enter the Wi-Fi password')}
          onClick={() => onResponse(false)}
          sx={narrow}
        >
          {common('No')}
        </ImButton>
        <ImButtonRed
          accessibleDescription={tr(
            'Retrieve the Wi-Fi password from the system keychain using administrator authentication',
          )}
          onClick={() => onResponse(true)}
          sx={narrow}
        >
          {common('Yes')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
