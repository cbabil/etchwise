/**
 * Layout pieces shared by the customisation steps, ported from upstream
 * `wizard/components/`: the vertically centred content column, the
 * 500px `WizardSectionContainer`, the two-column form grid,
 * `WizardFormLabel`, `WizardDescriptionText` and the "ⓘ" info icon.
 */
import { Box, Tooltip } from '@mui/material'
import type { ReactNode } from 'react'

import { colors, fontFamily, fontSize, sizes, spacing } from '@/theme/tokens'

/** Upstream content `ColumnLayout`: centred vertically, 16px margins, scrolls inside StepBase. */
export function StepContent({ children, gap = sizes.stepContentSpacing }: StepContentProps) {
  return (
    <Box
      sx={{
        my: 'auto',
        p: `${sizes.sectionPadding}px`,
        display: 'flex',
        flexDirection: 'column',
        gap: `${gap}px`,
        minWidth: 0,
      }}
    >
      {children}
    </Box>
  )
}

interface StepContentProps {
  children: ReactNode
  gap?: number
}

/** Upstream `WizardSectionContainer`: full width up to 500px, centred, 16px between children. */
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

/** Upstream two-column `GridLayout` (20px column, 15px row spacing). */
export function FormGrid({ children }: { children: ReactNode }) {
  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: 'auto minmax(0, 1fr)',
        columnGap: `${sizes.formColumnSpacing}px`,
        rowGap: `${sizes.formRowSpacing}px`,
        alignItems: 'center',
      }}
    >
      {children}
    </Box>
  )
}

/** A label/control row with 15px spacing (upstream `RowLayout`). */
export function FormRow({ children, gap = spacing.medium }: { children: ReactNode; gap?: number }) {
  return (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: `${gap}px`, minWidth: 0 }}>
      {children}
    </Box>
  )
}

export interface FormLabelProps {
  children: string
  htmlFor?: string
  description?: string
  isError?: boolean
  isDisabled?: boolean
}

/** Upstream `WizardFormLabel`: 14px, black / red / grey. */
export function FormLabel({ children, htmlFor, description, isError, isDisabled }: FormLabelProps) {
  const color = isError
    ? colors.formLabelError
    : isDisabled
      ? colors.formLabelDisabled
      : colors.formLabel
  return (
    <Box
      component="label"
      {...(htmlFor ? { htmlFor } : {})}
      {...(description ? { 'aria-description': description } : {})}
      sx={{ fontFamily, fontSize: fontSize.sm, color, whiteSpace: 'nowrap', flexShrink: 0 }}
    >
      {children}
    </Box>
  )
}

/** Upstream `WizardDescriptionText`: 12px wrapped description text. */
export function DescriptionText({ children }: { children: ReactNode }) {
  return (
    <Box
      component="p"
      sx={{ m: 0, fontFamily, fontSize: fontSize.xs, color: colors.textDescription }}
    >
      {children}
    </Box>
  )
}

export interface InfoIconProps {
  tooltip: string
  /** Accessible name; defaults to the tooltip text. */
  label?: string
  role?: 'button' | 'note'
  cursor?: 'pointer' | 'help'
}

/** The "ⓘ" glyph with a hover tooltip (300ms delay), grey turning dark on hover. */
export function InfoIcon({ tooltip, label, role = 'note', cursor = 'pointer' }: InfoIconProps) {
  return (
    <Tooltip title={tooltip} enterDelay={300} disableInteractive>
      <Box
        component="span"
        role={role}
        aria-label={label ?? tooltip}
        {...(label ? { 'aria-description': tooltip } : {})}
        sx={{
          fontSize: fontSize.sm,
          color: colors.textMetadata,
          cursor,
          flexShrink: 0,
          '&:hover': { color: colors.textDescription },
        }}
      >
        {'ⓘ'}
      </Box>
    </Tooltip>
  )
}
