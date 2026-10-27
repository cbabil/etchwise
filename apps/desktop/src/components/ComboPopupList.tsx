/**
 * Drop-down list of `ImComboBox`: a search field focused on open, with
 * "n of m" while searching, 40px rows with the list highlight colour,
 * "No matches".
 */
import { Search } from '@mui/icons-material'
import { Box } from '@mui/material'
import { type ChangeEvent, useEffect, useId, useRef } from 'react'

import { useTr } from '@/i18n'
import { colors, fontFamily, fontSize, sizes, spacing } from '@/theme/tokens'

import { type FilteredOption, SEARCH_ROW_HEIGHT } from './comboFilter'

export interface ComboPopupListProps {
  filtered: FilteredOption[]
  total: number
  search: string
  highlighted: number
  indicateError: boolean
  onPick: (filteredIndex: number) => void
  onSearch: (text: string) => void
}

const searchRowSx = {
  height: SEARCH_ROW_HEIGHT,
  flexShrink: 0,
  px: `${spacing.tiny}px`,
  display: 'flex',
  alignItems: 'center',
  gap: `${spacing.tiny}px`,
  borderBottom: `1px solid ${colors.popupBorder}`,
}

const searchInputSx = {
  flex: 1,
  minWidth: 0,
  border: 'none',
  outline: 'none',
  background: 'transparent',
  fontFamily,
  fontSize: fontSize.sm,
  color: colors.textDescription,
  // The webview's own blue clear button clashes with the theme
  '&::-webkit-search-cancel-button': { display: 'none' },
}

function SearchField(props: {
  search: string
  count: number
  total: number
  onSearch: (text: string) => void
  /** The listbox, and the highlighted row in it, for assistive technology. */
  listId: string
  activeId: string | undefined
}) {
  const tr = useTr('ImComboBox')
  const label = tr('Type to search')
  return (
    <Box sx={searchRowSx}>
      <Search sx={{ fontSize: 18, color: colors.textMetadata }} />
      <Box
        component="input"
        type="search"
        autoFocus
        value={props.search}
        placeholder={label}
        aria-label={label}
        onChange={(event: ChangeEvent<HTMLInputElement>) => props.onSearch(event.target.value)}
        aria-controls={props.listId}
        aria-activedescendant={props.activeId}
        sx={searchInputSx}
      />
      {props.search.length > 0 ? (
        <Box
          component="span"
          sx={{ fontSize: 10, color: colors.textMetadata, whiteSpace: 'nowrap' }}
        >
          {tr('%1 of %2', props.count, props.total)}
        </Box>
      ) : null}
    </Box>
  )
}

interface RowProps {
  id: string
  text: string
  highlighted: boolean
  indicateError: boolean
  onPick: () => void
}

function Row({ id, text, highlighted, indicateError, onPick }: RowProps) {
  return (
    <Box
      id={id}
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
      {/* Upstream 2.0.11 (4dcfbaac): long names end in "…", not mid-character */}
      <Box component="span" sx={{ overflow: 'hidden', textOverflow: 'ellipsis' }} title={text}>
        {text}
      </Box>
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
  const listId = useId()
  const rowId = (index: number) => `${listId}-${index}`
  useEffect(() => {
    const row = listRef.current?.children[highlighted] as HTMLElement | undefined
    row?.scrollIntoView?.({ block: 'nearest' })
  }, [highlighted])
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <SearchField
        search={search}
        count={filtered.length}
        total={total}
        onSearch={props.onSearch}
        listId={listId}
        activeId={filtered.length > 0 ? rowId(highlighted) : undefined}
      />
      <Box
        ref={listRef}
        id={listId}
        role="listbox"
        sx={{ flex: 1, minHeight: 0, overflowY: 'auto', position: 'relative' }}
      >
        {filtered.map((option, index) => (
          <Row
            key={option.originalIndex}
            id={rowId(index)}
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
