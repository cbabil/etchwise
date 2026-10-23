/**
 * Upstream's customisation lists: bullets on the Writing summary, check
 * marks on the Done step. The list scrolls inside itself, capped at 40% of
 * the window height (upstream: 40% of the step) like its `ScrollView`.
 */
import CheckIcon from '@mui/icons-material/Check'
import { Box } from '@mui/material'

import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import { customisationsAccessibleName } from './customisations'

export interface CustomisationListProps {
  /** `CommonStrings` source texts. */
  items: readonly string[]
  marker: 'bullet' | 'check'
  /** Translation context of "customization(s)". */
  context: 'WritingStep' | 'DoneStep'
}

export function CustomisationList({ items, marker, context }: CustomisationListProps) {
  const tr = useTr(context)
  const common = useTr('CommonStrings')
  const texts = items.map((item) => common(item))
  return (
    <Box
      component="ul"
      tabIndex={0}
      aria-label={customisationsAccessibleName(texts, tr)}
      sx={{
        m: 0,
        p: 0,
        listStyle: 'none',
        maxHeight: '40vh',
        minHeight: 0,
        flexShrink: 1,
        overflowY: 'auto',
        outline: 'none',
        display: 'flex',
        flexDirection: 'column',
        gap: marker === 'bullet' ? `${spacing.xSmall}px` : 0,
      }}
    >
      {texts.map((text) => (
        <Box
          component="li"
          key={text}
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
            fontSize: fontSize.xs,
            color: colors.formLabel,
            flexShrink: 0,
          }}
        >
          {marker === 'check' ? (
            <CheckIcon aria-hidden sx={{ fontSize: fontSize.xs + 2 }} />
          ) : (
            <span aria-hidden>{'•'}</span>
          )}
          {text}
        </Box>
      ))}
    </Box>
  )
}
