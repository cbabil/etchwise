/** Upstream `WizardSectionContainer.qml` and the steps' centred content column. */
import { Box } from '@mui/material'
import type { ReactNode } from 'react'

import { colors, fontSize, sizes } from '@/theme/tokens'

/** Content column with `sectionPadding` margins, vertically centred when it fits. */
export function CenteredContent({ children }: { children: ReactNode }) {
  return (
    <Box
      sx={{
        my: 'auto',
        p: `${sizes.sectionPadding}px`,
        display: 'flex',
        flexDirection: 'column',
        gap: `${sizes.stepContentSpacing}px`,
      }}
    >
      {children}
    </Box>
  )
}

/** At most 500px wide, centred, 16px between children. */
export function SectionContainer({ children }: { children: ReactNode }) {
  return (
    <Box
      sx={{
        width: '100%',
        maxWidth: sizes.sectionMaxWidth,
        mx: 'auto',
        display: 'flex',
        flexDirection: 'column',
        gap: `${sizes.stepContentSpacing}px`,
      }}
    >
      {children}
    </Box>
  )
}

/** Upstream `WizardFormLabel.qml`: 14px form label. */
export function FormLabel({
  children,
  bold,
  id,
}: {
  children: ReactNode
  bold?: boolean
  id?: string
}) {
  return (
    <Box
      component="label"
      {...(id ? { id } : {})}
      sx={{ fontSize: fontSize.sm, fontWeight: bold ? 700 : 400, color: colors.formLabel }}
    >
      {children}
    </Box>
  )
}

/** Upstream reads booleans saved as either `true` or `"true"`. */
export const isTrue = (value: unknown): boolean => value === true || value === 'true'
