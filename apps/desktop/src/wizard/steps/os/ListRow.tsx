/**
 * Row layout shared by upstream's device and OS delegates: highlight
 * (#BACCE7) on the current row, #f5f5f5 on hover, 15px side padding,
 * 10px top / 15px bottom, a 40px icon, then a bold 14px name, a wrapping
 * 12px description and any extra lines, 2px apart.
 */
import { Box } from '@mui/material'
import type { ReactNode } from 'react'

import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { ListIcon } from './ListIcon'

export interface ListRowProps {
  current: boolean
  minHeight: number
  icon: string | undefined
  name: string
  description: string
  children?: ReactNode
}

export function ListRow({ current, minHeight, icon, name, description, children }: ListRowProps) {
  return (
    <Box
      sx={{
        minHeight,
        boxSizing: 'border-box',
        display: 'flex',
        alignItems: 'center',
        gap: `${spacing.medium}px`,
        px: `${sizes.listItemPadding}px`,
        pt: `${spacing.small}px`,
        pb: `${spacing.medium}px`,
        cursor: 'pointer',
        userSelect: 'none',
        backgroundColor: current ? colors.listViewHighlight : colors.listViewRowBackground,
        '&:hover': current ? {} : { backgroundColor: colors.listViewHoverRowBackground },
      }}
    >
      <ListIcon icon={icon} />
      <Box
        sx={{
          flex: 1,
          minWidth: 0,
          display: 'flex',
          flexDirection: 'column',
          gap: `${spacing.xxSmall}px`,
        }}
      >
        <Box sx={{ fontSize: fontSize.sm, fontWeight: 700, color: colors.formLabel }}>{name}</Box>
        <Box
          sx={{ fontSize: fontSize.xs, color: colors.textDescription, overflowWrap: 'anywhere' }}
        >
          {description}
        </Box>
        {children}
      </Box>
    </Box>
  )
}

/** A 12px single-line metadata line (upstream `textMetadataColor`, elided). */
export function MetaLine({ children, color }: { children: ReactNode; color?: string }) {
  return (
    <Box
      sx={{
        fontSize: fontSize.xs,
        color: color ?? colors.textMetadata,
        whiteSpace: 'nowrap',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        minHeight: '1.2em',
      }}
    >
      {children}
    </Box>
  )
}
