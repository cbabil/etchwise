/**
 * Upstream `DoneStep.qml`: what was written, the eject instruction, and
 * custom buttons WRITE ANOTHER (same image to another device) and FINISH
 * (closes the app; REBOOT in embedded mode).
 */
import { Box } from '@mui/material'

import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { sizes } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { DoneContent } from './done/DoneContent'
import { finish, useDoneInfo } from './done/useDoneInfo'

const buttonSx = { minWidth: sizes.buttonMinWidth, height: sizes.buttonHeight }

function DoneButtons({ embedded }: { embedded: boolean }) {
  const tr = useTr('DoneStep')
  const common = useTr('CommonStrings')
  const { resetToWriteStep } = useWizard()
  return (
    <>
      <Box sx={{ flex: 1 }} />
      <ImButton
        onClick={resetToWriteStep}
        sx={buttonSx}
        accessibleDescription={tr(
          'Return to storage selection to write the same image to another storage device',
        )}
      >
        {tr('Write Another')}
      </ImButton>
      <ImButtonRed
        onClick={() => void finish(embedded)}
        sx={buttonSx}
        accessibleDescription={
          embedded
            ? tr('Reboot the system to apply changes')
            : tr('Close Raspberry Pi Imager and exit the application')
        }
      >
        {embedded ? tr('Reboot') : common('Finish')}
      </ImButtonRed>
    </>
  )
}

export function DoneStep() {
  const tr = useTr('DoneStep')
  const info = useDoneInfo()
  return (
    <StepBase
      title={tr('Write complete!')}
      showBack={false}
      showNext={false}
      customButtons={<DoneButtons embedded={info.embedded} />}
    >
      <DoneContent autoEject={info.autoEject} />
    </StepBase>
  )
}
