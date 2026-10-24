/**
 * Upstream `DoneStep.qml` content: "Your choices:" with the choices
 * grid, "Customisations applied:" with check marks (from the completion
 * snapshot), and the eject instruction.
 */
import { Box } from '@mui/material'

import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { InfoText } from '../../dialogs/common/dialogParts'
import { useWizard } from '../../WizardContext'
import { ChoicesGrid, Heading } from '../writing/ChoicesGrid'
import { CustomisationList } from '../writing/CustomisationList'
import { anyCustomisations, customisationLines } from '../writing/customisations'

/** Ubuntu autoinstall images: the one step left on the NUC. */
function AutoinstallHint() {
  const tr = useTr('DoneStep')
  const { autoinstall } = useWizard()
  if (!autoinstall) return null
  return (
    <InfoText color={colors.formLabel} size={fontSize.sm} center>
      {tr(
        'Boot the NUC from the stick. When the installer asks "Continue with autoinstall?", type yes.',
      )}
    </InfoText>
  )
}

export function DoneContent({ autoEject }: { autoEject: boolean }) {
  const tr = useTr('DoneStep')
  const { completionSnapshot } = useWizard()
  const applied = anyCustomisations(completionSnapshot, true)
  return (
    <Box
      sx={{
        flex: 1,
        minHeight: 0,
        p: `${sizes.cardPadding}px`,
        display: 'flex',
        flexDirection: 'column',
        gap: `${spacing.medium}px`,
      }}
    >
      <Heading>{tr('Your choices:')}</Heading>
      <ChoicesGrid context="DoneStep" />
      {applied ? (
        <>
          <Box sx={{ mt: `${spacing.small}px` }}>
            <Heading size={fontSize.sm}>{tr('Customisations applied:')}</Heading>
          </Box>
          <CustomisationList
            items={customisationLines(completionSnapshot)}
            marker="check"
            context="DoneStep"
          />
        </>
      ) : null}
      <AutoinstallHint />
      <Box
        sx={{
          fontSize: fontSize.xs,
          color: colors.textDescription,
          textAlign: 'center',
          flexShrink: 0,
        }}
      >
        {autoEject
          ? tr('The storage device was ejected automatically. You can now remove it safely.')
          : tr('Please eject the storage device before removing it from your computer.')}
      </Box>
    </Box>
  )
}
