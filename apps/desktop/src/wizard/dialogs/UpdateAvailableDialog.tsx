/**
 * Upstream `UpdateAvailableDialog.qml`: "Update available" with the new
 * version, No (closes) and a wider red Update that opens the download
 * page in the browser. Escape closes it like No.
 */
import { Box } from '@mui/material'

import { call } from '@/api/imager'
import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { sizes, spacing } from '@/theme/tokens'

export interface UpdateAvailableDialogProps {
  open: boolean
  url: string
  version: string
  onClose: () => void
}

export function UpdateAvailableDialog({ open, url, version, onClose }: UpdateAvailableDialogProps) {
  const tr = useTr('UpdateAvailableDialog')
  const common = useTr('CommonStrings')
  const update = () => {
    if (url) void call('openUrl', url)
    onClose()
  }
  const wide = sizes.buttonMinWidth * 1.5
  return (
    <BaseDialog open={open} onClose={onClose} title={tr('Update available')}>
      <DialogHeading>{tr('Update available')}</DialogHeading>
      <DialogBody>
        {version
          ? tr(
              'Imager version %1 is available. Would you like to visit the website to download it?',
              version,
            )
          : tr(
              'There is a newer version of Imager available. Would you like to visit the website to download it?',
            )}
      </DialogBody>
      <Box sx={{ pt: `${spacing.small}px` }}>
        <DialogButtons>
          <ImButton
            onClick={onClose}
            accessibleDescription={tr('Continue using the current version of Raspberry Pi Imager')}
          >
            {common('No')}
          </ImButton>
          <ImButtonRed
            onClick={update}
            accessibleDescription={tr(
              'Open the Raspberry Pi website in your browser to download the latest version',
            )}
            sx={{ minWidth: wide }}
          >
            {tr('Update')}
          </ImButtonRed>
        </DialogButtons>
      </Box>
    </BaseDialog>
  )
}
