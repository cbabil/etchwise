/** Layout pieces shared by the options dialogs (upstream BaseDialog content). */
import { Box } from '@mui/material'
import type { ReactNode } from 'react'

import { colors, fontSize, spacing } from '@/theme/tokens'

/** 16px bold centred dialog title (`fontSizeLargeHeading`). */
export function CenteredHeading({ children }: { children: ReactNode }) {
  return (
    <Box
      component="h2"
      sx={{
        m: 0,
        fontSize: fontSize.md,
        fontWeight: 700,
        color: colors.formLabel,
        textAlign: 'center',
      }}
    >
      {children}
    </Box>
  )
}

/**
 * Column of options that scrolls on its own when the window is too short,
 * so the heading and buttons stay visible. `chrome` is the height of
 * everything else in the dialog, in px.
 */
export function DialogScrollArea({ chrome, children }: { chrome: number; children: ReactNode }) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        gap: `${spacing.medium}px`,
        px: `${spacing.small}px`,
        maxHeight: `calc(100vh - ${64 + chrome}px)`,
        overflowY: 'auto',
        minHeight: 0,
      }}
    >
      {children}
    </Box>
  )
}

export interface InfoTextProps {
  children: ReactNode
  /** Text colour; the description colour by default. */
  color?: string
  /** Font size; 12px (`fontSizeDescription`) by default. */
  size?: number
  bold?: boolean
  center?: boolean
}

/** Wrapping body text in Imager's description style. */
export function InfoText({ children, color, size, bold, center }: InfoTextProps) {
  return (
    <Box
      component="p"
      sx={{
        m: 0,
        fontSize: size ?? fontSize.xs,
        fontWeight: bold ? 700 : 400,
        color: color ?? colors.textDescription,
        textAlign: center ? 'center' : 'left',
        whiteSpace: 'pre-line',
        overflowWrap: 'anywhere',
      }}
    >
      {children}
    </Box>
  )
}

/** Removes emoji / pictograph characters from translated text (no emoji in the UI). */
export function stripEmoji(text: string): string {
  return text
    .replace(/\u{FE0F}/gu, '')
    .replace(/[\u{1F000}-\u{1FAFF}\u{2600}-\u{27BF}]/gu, '')
    .trim()
}
