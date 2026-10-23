/** Upstream "USB Gadget Mode Warning" dialog: accept unlocks after 2 s. */
import { Box } from '@mui/material'

import { call } from '@/api/imager'
import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { colors, fontSize } from '@/theme/tokens'

import { InfoText } from '../../dialogs/common/dialogParts'
import { useDelayedFlag } from '../../dialogs/common/useDelayedFlag'
import { USB_GADGET_HELP_URL } from './IfFeaturesSections'

export const USB_GADGET_ACCEPT_DELAY_MS = 2000

export interface UsbGadgetDialogProps {
  open: boolean
  onCancel: () => void
  onAccept: () => void
}

function DocumentationLine() {
  const tr = useTr('IfAndFeaturesCustomizationStep')
  const [before, after = ''] = tr(
    "Please review the <a href='%1'>documentation</a> before proceeding.",
  )
    .replace(/<a [^>]*>/, '\u0000')
    .split('\u0000')
  const [linkText = '', rest = ''] = after.split('</a>')
  return (
    <InfoText size={fontSize.sm} bold color={colors.formLabel}>
      {before}
      <Box
        component="a"
        href={USB_GADGET_HELP_URL}
        onClick={(event) => {
          event.preventDefault()
          void call('openUrl', USB_GADGET_HELP_URL)
        }}
        sx={{ color: colors.buttonForeground }}
      >
        {linkText}
      </Box>
      {rest}
    </InfoText>
  )
}

export function UsbGadgetDialog({ open, onCancel, onAccept }: UsbGadgetDialogProps) {
  const tr = useTr('IfAndFeaturesCustomizationStep')
  const common = useTr('CommonStrings')
  const allowAccept = useDelayedFlag(open, USB_GADGET_ACCEPT_DELAY_MS)
  return (
    <BaseDialog open={open} onClose={onCancel} title={tr('USB Gadget Mode Warning')}>
      <InfoText size={fontSize.md} bold color={colors.formLabelError}>
        {tr(
          'USB Gadget Mode can change how your device behaves and may impact connectivity and host interaction.',
        )}
      </InfoText>
      <DocumentationLine />
      <InfoText size={fontSize.sm} bold color={colors.formLabelError}>
        {tr('Only continue if you are sure you know what you are doing.')}
      </InfoText>
      <DialogButtons>
        <ImButton
          onClick={onCancel}
          accessibleDescription={tr(
            'Cancel and return to the interfaces and features settings without enabling USB Gadget Mode',
          )}
        >
          {common('Cancel')}
        </ImButton>
        <ImButtonRed
          onClick={onAccept}
          disabled={!allowAccept}
          accessibleDescription={
            allowAccept
              ? tr(
                  'Confirm that you understand the risks and continue with USB Gadget Mode enabled',
                )
              : tr('This button will be enabled after 2 seconds')
          }
        >
          {tr('I understand, continue')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
