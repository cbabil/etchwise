/**
 * Shared look of upstream's ImButton / ImButtonRed / ImToggleTab: a 40px
 * button whose background Qt Material insets by 6px top and bottom (so a
 * 28px box with 6px margins), at least 120px wide, 4px radius, 14px ALL
 * CAPS text. Qt's
 * `activeFocus` (tab focus only) maps to `:focus-visible`. No ripple,
 * shadow or transition.
 */
import type { SxProps, Theme } from '@mui/material/styles'

import { colors, fontFamily, fontSize, sizes } from '@/theme/tokens'

const disabledBackground = 'rgba(0, 0, 0, 0.1)'
const disabledForeground = 'rgba(0, 0, 0, 0.3)'

const visibleHeight = sizes.buttonHeight - 2 * sizes.buttonInset

const base = {
  height: visibleHeight,
  my: `${sizes.buttonInset}px`,
  // One line centred vertically (upstream Text.AlignVCenter); block layout
  // keeps the ellipsis for labels too long for the button.
  py: 0,
  lineHeight: `${visibleHeight}px`,
  minWidth: sizes.buttonMinWidth,
  px: 2,
  borderRadius: `${sizes.buttonRadius}px`,
  fontFamily,
  fontSize: fontSize.sm,
  textTransform: 'uppercase',
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  display: 'block',
  boxShadow: 'none',
  transition: 'none',
  '&.Mui-disabled': { backgroundColor: disabledBackground, color: disabledForeground },
} as const

/** White button (upstream ImButton). */
export const whiteButtonSx: SxProps<Theme> = {
  ...base,
  backgroundColor: colors.buttonBackground,
  color: colors.buttonForeground,
  border: `1px solid ${colors.popupBorder}`,
  '&:hover': { backgroundColor: colors.buttonHoveredBackground, boxShadow: 'none' },
  '&.Mui-focusVisible, &:focus-visible': { backgroundColor: colors.buttonFocusedBackground },
  '&.Mui-disabled': { ...base['&.Mui-disabled'], borderColor: 'rgba(0, 0, 0, 0.2)' },
}

/** Red button (upstream ImButtonRed): focus and hover share one look. */
export const redButtonSx: SxProps<Theme> = {
  ...base,
  backgroundColor: colors.button2Background,
  color: colors.button2Foreground,
  border: 'none',
  '&:hover, &.Mui-focusVisible, &:focus-visible': {
    backgroundColor: colors.button2HoveredBackground,
    color: colors.raspberryRed,
    boxShadow: 'none',
  },
}

/** Combines a label with its accessible description, as upstream does. */
export function accessibleName(label: unknown, description?: string): string | undefined {
  if (!description || typeof label !== 'string') return undefined
  return `${label}, ${description}`
}
