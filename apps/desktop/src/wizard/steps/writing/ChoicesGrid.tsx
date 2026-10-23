/**
 * The two-column Device / Operating system / Storage grid shared by the
 * Writing summary and the Done step (20px columns, 10px rows, 12px text
 * with bold values that elide).
 */
import { Box } from '@mui/material'
import type { ReactNode } from 'react'

import { MarqueeText } from '@/components/MarqueeText'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { useWizard } from '../../WizardContext'

const labelSx = { fontSize: fontSize.xs, color: colors.formLabel, whiteSpace: 'nowrap' } as const
const valueSx = { fontSize: fontSize.xs, fontWeight: 700, color: colors.formLabel } as const

export function Heading({ children, size = fontSize.md }: { children: ReactNode; size?: number }) {
  return (
    <Box component="h2" sx={{ m: 0, fontSize: size, fontWeight: 700, color: colors.formLabel }}>
      {children}
    </Box>
  )
}

/** `context` picks the catalogue upstream used for the OS and storage labels. */
export function ChoicesGrid({ context }: { context: 'WritingStep' | 'DoneStep' }) {
  const tr = useTr(context)
  const common = useTr('CommonStrings')
  const wizard = useWizard()
  const rows: Array<[string, string]> = [
    [common('Device:'), wizard.selectedDeviceName || common('No device selected')],
    [tr('Operating system:'), wizard.selectedOsName || common('No image selected')],
    [
      context === 'DoneStep' ? tr('Storage:') : common('Storage:'),
      wizard.selectedStorageName || common('No storage selected'),
    ],
  ]
  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: 'auto minmax(0, 1fr)',
        columnGap: `${sizes.formColumnSpacing}px`,
        rowGap: `${spacing.small}px`,
      }}
    >
      {rows.map(([label, value]) => (
        <Box key={label} sx={{ display: 'contents' }}>
          <Box sx={labelSx}>{label}</Box>
          <MarqueeText text={value} sx={valueSx} />
        </Box>
      ))}
    </Box>
  )
}
