/**
 * Popup state of `ImComboBox`: the search string typed while open, the
 * filtered options and the highlighted row. Keys follow upstream: Escape
 * closes without change, Enter picks, Backspace trims the search, Space
 * only extends a non-empty search, other printable keys type.
 */
import { type KeyboardEvent, useMemo, useState } from 'react'

import { filterOptions } from './comboFilter'

export interface ComboPopup {
  open: boolean
  search: string
  highlighted: number
  filtered: ReturnType<typeof filterOptions>
  openPopup: () => void
  close: () => void
  pick: (filteredIndex: number) => void
  onKeyDown: (event: KeyboardEvent) => void
}

export function useComboPopup(
  options: readonly string[],
  value: number,
  onChange: (index: number) => void,
): ComboPopup {
  const [open, setOpen] = useState(false)
  const [search, setSearch] = useState('')
  const [highlighted, setHighlighted] = useState(0)
  const filtered = useMemo(() => filterOptions(options, search), [options, search])

  const openPopup = () => {
    setSearch('')
    setHighlighted(Math.max(0, value))
    setOpen(true)
  }
  const close = () => {
    setOpen(false)
    setSearch('')
  }
  const pick = (filteredIndex: number) => {
    const option = filtered[filteredIndex]
    if (option) onChange(option.originalIndex)
    close()
  }
  const typeText = (text: string) => {
    setSearch((s) => s + text.toLowerCase())
    setHighlighted(0)
  }
  const onKeyDown = (event: KeyboardEvent) => {
    const handled = handleKey(event.key, {
      search,
      filtered,
      highlighted,
      close,
      pick,
      typeText,
      setSearch,
      setHighlighted,
    })
    if (handled) event.preventDefault()
  }
  return { open, search, highlighted, filtered, openPopup, close, pick, onKeyDown }
}

interface KeyContext {
  search: string
  filtered: ReturnType<typeof filterOptions>
  highlighted: number
  close: () => void
  pick: (i: number) => void
  typeText: (t: string) => void
  setSearch: (s: string) => void
  setHighlighted: (i: number) => void
}

function handleKey(key: string, ctx: KeyContext): boolean {
  const last = ctx.filtered.length - 1
  switch (key) {
    case 'Escape':
      ctx.close()
      return true
    case 'Enter':
      ctx.pick(ctx.highlighted)
      return true
    case 'Backspace':
      if (ctx.search.length > 0) {
        ctx.setSearch(ctx.search.slice(0, -1))
        ctx.setHighlighted(0)
      }
      return true
    case 'ArrowDown':
      ctx.setHighlighted(Math.min(last, ctx.highlighted + 1))
      return true
    case 'ArrowUp':
      ctx.setHighlighted(Math.max(0, ctx.highlighted - 1))
      return true
    case 'Tab':
      return false
    case ' ':
      if (ctx.search.length > 0) ctx.typeText(' ')
      return true
    default:
      if (key.length !== 1) return false
      ctx.typeText(key)
      return true
  }
}
