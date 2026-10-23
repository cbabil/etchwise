/**
 * Upstream `DebugOptionsDialog.qml` (Cmd+Option+S): I/O, logging, network,
 * workaround and rpiboot switches with a live status summary; Apply sends
 * them to the backend in one `setDebugOptions` call.
 */
import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { colors, sizes } from '@/theme/tokens'

import { CenteredHeading, DialogScrollArea, InfoText, stripEmoji } from './common/dialogParts'
import { useDebugOptions } from './debug/debugOptions'
import { DebugOptionsList, WARNING_SIGN } from './debug/DebugOptionsList'

export interface DebugOptionsDialogProps {
  open: boolean
  onClose: () => void
}

/** Padding, heading, warning (two lines), gaps and buttons. */
const CHROME_PX = 40 + 20 + 15 + 30 + 15 + 15 + 40

export function DebugOptionsDialog({ open, onClose }: DebugOptionsDialogProps) {
  const tr = useTr('DebugOptionsDialog')
  const common = useTr('CommonStrings')
  const debug = useDebugOptions(open)
  const width = { minWidth: sizes.buttonMinWidth }
  const apply = () => {
    debug.apply()
    onClose()
  }
  return (
    <BaseDialog open={open} onClose={onClose} title={tr('Debug Options')}>
      <CenteredHeading>{tr('Debug Options')}</CenteredHeading>
      <InfoText center color={colors.formLabelError}>
        {stripEmoji(
          tr(
            `${WARNING_SIGN} These options are for debugging and testing. Changing them may affect performance and data integrity.`,
          ),
        )}
      </InfoText>
      <DialogScrollArea chrome={CHROME_PX}>
        <DebugOptionsList options={debug.options} onChange={debug.change} />
      </DialogScrollArea>
      <DialogButtons>
        <ImButton
          onClick={onClose}
          sx={width}
          accessibleDescription={tr('Close the debug options dialog without saving any changes')}
        >
          {common('Cancel')}
        </ImButton>
        <ImButtonRed
          onClick={apply}
          sx={width}
          accessibleDescription={tr('Apply the selected debug options')}
        >
          {tr('Apply')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
