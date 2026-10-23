/**
 * Upstream `OSSelectionListView.qml` on top of `SelectionList`: Right
 * opens the current group, Left goes back from a sublist, and selections
 * remember whether they came from the keyboard (Enter / Space) so the
 * step can auto-advance. Double-click counts as keyboard, like upstream.
 */
import { Box } from '@mui/material'
import { type KeyboardEvent, useEffect, useRef } from 'react'

import { SelectionList } from '@/components/SelectionList'
import { useTr } from '@/i18n'

import { OsRow, type OsRowContext } from './OsRow'
import { isSublist, type OsEntry } from './types'
import type { OsLevel } from './useOsList'

export interface OsListViewProps {
  level: OsLevel
  depth: number
  ctx: OsRowContext
  onCurrentChange: (index: number) => void
  onSelect: (entry: OsEntry, fromKeyboard: boolean) => void
  onBack: () => void
}

function useListName(depth: number, count: number): string {
  const tr = useTr('OSSelectionStep')
  const n = depth > 0 ? Math.max(0, count - 1) : count
  const amount =
    n === 0
      ? tr('No operating systems')
      : n === 1
        ? tr('1 operating system')
        : tr('%1 operating systems', n)
  const head = depth > 0 ? tr('Operating system category') : tr('Operating system list')
  const hint =
    depth > 0
      ? tr('Use arrow keys to navigate, Enter or Space to select, Left arrow to go back')
      : tr('Use arrow keys to navigate, Enter or Space to select')
  return `${head}. ${amount}. ${hint}`
}

export function OsListView({ level, depth, ctx, ...handlers }: OsListViewProps) {
  const box = useRef<HTMLDivElement>(null)
  const keyboard = useRef(false)
  const shownDepth = useRef(depth)
  const name = useListName(depth, level.entries.length)
  useEffect(() => {
    if (shownDepth.current === depth) return
    shownDepth.current = depth
    box.current?.querySelector<HTMLElement>('[role="listbox"]')?.focus()
  }, [depth])
  const onKeyDownCapture = (event: KeyboardEvent) => {
    keyboard.current = event.key === 'Enter' || event.key === ' '
  }
  const onKeyDown = (event: KeyboardEvent) => {
    const entry = level.entries[level.current]
    if (event.key === 'ArrowRight' && entry && isSublist(entry)) handlers.onSelect(entry, false)
    else if (event.key === 'ArrowLeft' && depth > 0) handlers.onBack()
    else return
    event.preventDefault()
  }
  const select = (entry: OsEntry, fromKeyboard: boolean) => {
    keyboard.current = false
    handlers.onSelect(entry, fromKeyboard)
  }
  return (
    <Box
      ref={box}
      onKeyDownCapture={onKeyDownCapture}
      onKeyDown={onKeyDown}
      // The second click of a double-click must not act on a page that just opened.
      onClickCapture={(event) => event.detail > 1 && event.stopPropagation()}
      sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}
    >
      <SelectionList
        key={depth}
        items={level.entries}
        getKey={(entry, index) => `${index}:${entry.name}:${entry.url ?? ''}`}
        currentIndex={level.current}
        onCurrentIndexChange={handlers.onCurrentChange}
        onItemSelected={(_, entry) => select(entry, keyboard.current)}
        onItemDoubleClicked={(_, entry) => select(entry, true)}
        ariaLabel={name}
        renderItem={(entry, _index, current) => <OsRow entry={entry} current={current} ctx={ctx} />}
        sx={{ flex: 1 }}
      />
    </Box>
  )
}
