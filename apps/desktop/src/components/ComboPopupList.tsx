/**
 * Drop-down list of `ImComboBox`: optional search banner ("Search: …",
 * "n of m"), 40px rows with the list highlight colour, "No matches".
 */
import { Box } from '@mui/material'
import { useEffect, useRef } from 'react'

import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import type { FilteredOption } from './comboFilter'

export interface ComboPopupListProps {
  filtered: FilteredOption[]
  total: number
  search: string
  highlighted: number
  indicateError: boolean
  onPick: (filteredIndex: number) => void
}

function SearchBanner({ search, count, total }: { search: string; count: number; total: number }) {
  const tr = useTr('ImComboBox')
  return (
    <Box
      sx={{
        height: 28,
        flexShrink: 0,
        px: `${spacing.tiny}px`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        backgroundColor: colors.buttonFocusedBackground,
        fontStyle: 'italic',
      }}
    >
      <Box component="span" sx={{ fontSize: fontSize.xs, color: colors.textDescription }}>
        {tr('Search: "%1"', search)}
      </Box>
      <Box component="span" sx={{ fontSize: 10, color: colors.textMetadata }}>
        {tr('%1 of %2', count, total)}
      </Box>
    </Box>
  )
}

interface RowProps {
  text: string
  highlighted: boolean
  indicateError: boolean
  onPick: () => void
}

function Row({ text, highlighted, indicateError, onPick }: RowProps) {
  return (
    <Box
      role="option"
      aria-selected={highlighted}
      onClick={onPick}
      sx={{
        height: 40,
        m: '2px',
        px: `${spacing.tiny}px`,
        display: 'flex',
        alignItems: 'center',
        fontSize: fontSize.sm,
        borderRadius: `${sizes.sectionBorderRadius}px`,
        color: indicateError ? colors.formLabelError : colors.textDescription,
        backgroundColor: highlighted ? colors.listViewHighlight : 'transparent',
        cursor: 'default',
        whiteSpace: 'nowrap',
      }}
    >
      {text}
    </Box>
  )
}

function NoMatches() {
  const tr = useTr('ImComboBox')
  return (
    <Box
      sx={{
        position: 'absolute',
        inset: 0,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: fontSize.xs,
        fontStyle: 'italic',
        color: colors.textMetadata,
        pointerEvents: 'none',
      }}
    >
      {tr('No matches')}
    </Box>
  )
}

export function ComboPopupList(props: ComboPopupListProps) {
  const { filtered, total, search, highlighted, indicateError, onPick } = props
  const listRef = useRef<HTMLDivElement>(null)
  useEffect(() => {
    const row = listRef.current?.children[highlighted] as HTMLElement | undefined
    row?.scrollIntoView?.({ block: 'nearest' })
  }, [highlighted])
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {search.length > 0 ? (
        <SearchBanner search={search} count={filtered.length} total={total} />
      ) : null}
      <Box
        ref={listRef}
        role="listbox"
        sx={{ flex: 1, minHeight: 0, overflowY: 'auto', position: 'relative' }}
      >
        {filtered.map((option, index) => (
          <Row
            key={option.originalIndex}
            text={option.text}
            highlighted={index === highlighted}
            indicateError={indicateError}
            onPick={() => onPick(index)}
          />
        ))}
      </Box>
      {filtered.length === 0 && search.length > 0 ? <NoMatches /> : null}
    </Box>
  )
}
