/**
 * Upstream's de-chromed summary: "Summary" with the choices grid, then
 * "Customisations to apply:" with a bullet list when any are on.
 */
import { Box } from '@mui/material'

import { useTr } from '@/i18n'
import { sizes, spacing } from '@/theme/tokens'

import { useWizard } from '../../WizardContext'
import { ChoicesGrid, Heading } from './ChoicesGrid'
import { CustomisationList } from './CustomisationList'
import { anyCustomisations, customisationLines } from './customisations'

const section = {
  width: '100%',
  maxWidth: sizes.sectionMaxWidth,
  mx: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: `${spacing.medium}px`,
  minHeight: 0,
} as const

export function WriteSummary() {
  const tr = useTr('WritingStep')
  const wizard = useWizard()
  const items = customisationLines(wizard)
  return (
    <>
      <Box sx={{ ...section, flexShrink: 0 }}>
        <Heading>{tr('Summary')}</Heading>
        <ChoicesGrid context="WritingStep" />
      </Box>
      {anyCustomisations(wizard, false) ? (
        <Box sx={{ ...section, flex: 1 }}>
          <Heading>{tr('Customisations to apply:')}</Heading>
          <CustomisationList items={items} marker="bullet" context="WritingStep" />
        </Box>
      ) : null}
    </>
  )
}
