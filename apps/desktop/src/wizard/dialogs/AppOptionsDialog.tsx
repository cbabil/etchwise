/**
 * Upstream `AppOptionsDialog.qml`: sound / eject / telemetry / disable
 * warnings switches, the content repository editor, the Secure Boot RSA
 * key, clearing saved customisation, and Cancel / Save. Options load
 * fresh on every open so Cancel discards changes.
 */
import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { sizes } from '@/theme/tokens'

import { ConfirmClearSettings, ConfirmDisableWarnings } from './appOptions/AppOptionsConfirms'
import { AppOptionsList } from './appOptions/AppOptionsList'
import { useAppOptionsFlow } from './appOptions/useAppOptionsFlow'
import { CenteredHeading, DialogScrollArea, InfoText } from './common/dialogParts'
import { RepositoryDialog } from './RepositoryDialog'

export interface AppOptionsDialogProps {
  open: boolean
  onClose: () => void
}

/** Dialog padding, heading, gaps and button row around the options list. */
const CHROME_PX = 40 + 20 + 15 + 15 + 40

function Footer({ onCancel, onSave }: { onCancel: () => void; onSave: () => void }) {
  const tr = useTr('AppOptionsDialog')
  const common = useTr('CommonStrings')
  const width = { minWidth: sizes.buttonMinWidth }
  return (
    <DialogButtons>
      <ImButton
        onClick={onCancel}
        sx={width}
        accessibleDescription={tr('Close the options dialog without saving any changes')}
      >
        {common('Cancel')}
      </ImButton>
      <ImButtonRed
        onClick={onSave}
        sx={width}
        accessibleDescription={tr(
          'Save the selected options and apply them to Raspberry Pi Imager',
        )}
      >
        {tr('Save')}
      </ImButtonRed>
    </DialogButtons>
  )
}

export function AppOptionsDialog({ open, onClose }: AppOptionsDialogProps) {
  const tr = useTr('AppOptionsDialog')
  const flow = useAppOptionsFlow(open, onClose)
  const { options, warnings, clear } = flow
  const save = () => {
    options.save()
    onClose()
  }
  return (
    <>
      <BaseDialog open={open} onClose={onClose} title={tr('App Options')}>
        <CenteredHeading>{tr('App Options')}</CenteredHeading>
        <DialogScrollArea chrome={CHROME_PX + (options.env.showVersion ? 30 : 0)}>
          <AppOptionsList
            values={options.values}
            env={options.env}
            onChange={options.change}
            onDisableWarnings={warnings.toggle}
            onEditRepository={flow.editRepository}
            onSelectKey={() => void flow.selectKey()}
            onClearSettings={clear.start}
          />
        </DialogScrollArea>
        {options.env.showVersion ? (
          <InfoText center>{tr('Version: %1', options.env.version)}</InfoText>
        ) : null}
        <Footer onCancel={onClose} onSave={save} />
      </BaseDialog>
      <ConfirmDisableWarnings
        open={warnings.open}
        onCancel={warnings.cancel}
        onConfirm={warnings.confirm}
      />
      <ConfirmClearSettings open={clear.open} onCancel={clear.cancel} onConfirm={clear.confirm} />
      <RepositoryDialog open={flow.repoOpen} onClose={flow.closeRepo} />
    </>
  )
}
