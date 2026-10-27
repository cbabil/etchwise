/**
 * Popup state of `ImComboBox`: the text in its search field, the filtered
 * options and the highlighted row. Keys follow upstream: Escape closes
 * without change, Enter picks the highlighted row, the arrows move it,
 * wherever focus is in the popup; typing goes to the search field.
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
  /** The search field's text changed. */
  setSearchText: (text: string) => void
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
  const setSearchText = (text: string) => {
    setSearch(text)
    setHighlighted(0)
  }
  const onKeyDown = (event: KeyboardEvent) => {
    // An input method (Japanese, Chinese, Korean) owns Enter and the arrows
    // while it composes; WebKit reports its commit as keyCode 229
    if (event.nativeEvent.isComposing || event.keyCode === 229) return
    if (handleKey(event.key, { filtered, highlighted, close, pick, setHighlighted })) {
      event.preventDefault()
    }
  }
  return { open, search, highlighted, filtered, openPopup, close, pick, setSearchText, onKeyDown }
}

interface KeyContext {
  filtered: ReturnType<typeof filterOptions>
  highlighted: number
  close: () => void
  pick: (i: number) => void
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
    case 'ArrowDown':
      ctx.setHighlighted(Math.min(last, ctx.highlighted + 1))
      return true
    case 'ArrowUp':
      ctx.setHighlighted(Math.max(0, ctx.highlighted - 1))
      return true
    default:
      return false
  }
}
