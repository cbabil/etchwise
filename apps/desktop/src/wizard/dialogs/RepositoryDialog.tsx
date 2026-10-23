/**
 * Upstream `RepositoryDialog.qml`: choose the official repository, a
 * local JSON / manifest file or a custom URL; "Apply & Restart" switches
 * the OS list and restarts the wizard when the choice changed.
 */
import { Box } from '@mui/material'

import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { sizes, spacing } from '@/theme/tokens'

import { FormLabel } from '../steps/custom2/layout'
import { CenteredHeading, DialogScrollArea } from './common/dialogParts'
import { RepoSourceFields } from './repository/RepoSourceFields'
import { useRepository } from './repository/useRepository'

export interface RepositoryDialogProps {
  open: boolean
  onClose: () => void
}

const CHROME_PX = 40 + 20 + 15 + 15 + 40

export function RepositoryDialog({ open, onClose }: RepositoryDialogProps) {
  const tr = useTr('RepositoryDialog')
  const common = useTr('CommonStrings')
  const repository = useRepository(open)
  const width = { minWidth: sizes.buttonMinWidth }
  const apply = () => {
    repository.apply()
    onClose()
  }
  return (
    <BaseDialog open={open} onClose={onClose} title={tr('Content Repository')}>
      <CenteredHeading>{tr('Content Repository')}</CenteredHeading>
      <DialogScrollArea chrome={CHROME_PX}>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.xSmall}px` }}>
          <FormLabel>{tr('Repository source:')}</FormLabel>
          <RepoSourceFields repository={repository} />
        </Box>
      </DialogScrollArea>
      <DialogButtons>
        <ImButton
          onClick={onClose}
          sx={width}
          accessibleDescription={tr(
            'Close the repository dialog without changing the content source',
          )}
        >
          {common('Cancel')}
        </ImButton>
        <ImButtonRed
          onClick={apply}
          disabled={!repository.canApply}
          sx={width}
          accessibleDescription={tr(
            'Apply the new content repository and restart the wizard from the beginning',
          )}
        >
          {tr('Apply & Restart')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
