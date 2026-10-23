/**
 * Upstream `SelectionListView.qml`: a focusable list whose current row is
 * highlighted (#BACCE7 while the list has focus, white otherwise). Up and
 * Down skip unselectable rows; Enter, Return and Space select the current
 * row (then run `onNext` when `keyboardAutoAdvance`); a click selects and
 * a double-click reports `onItemDoubleClicked`. The list scrolls inside
 * itself, never the page.
 */
import { Box } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'
import {
  type KeyboardEvent,
  type ReactNode,
  type RefObject,
  useEffect,
  useId,
  useRef,
  useState,
} from 'react'

import { colors } from '@/theme/tokens'

import { stepIndex } from './selectionNav'

export interface SelectionListProps<T> {
  items: readonly T[]
  renderItem: (item: T, index: number, current: boolean) => ReactNode
  getKey?: (item: T, index: number) => string | number
  /** Controlled current row; omit to let the list keep its own. */
  currentIndex?: number
  onCurrentIndexChange?: (index: number) => void
  onItemSelected?: (index: number, item: T) => void
  onItemDoubleClicked?: (index: number, item: T) => void
  isItemSelectable?: (index: number) => boolean
  autoSelectFirst?: boolean
  keyboardAutoAdvance?: boolean
  onNext?: () => void
  ariaLabel?: string
  sx?: SxProps<Theme>
}

const always = () => true

function useCurrent<T>(props: SelectionListProps<T>): [number, (i: number) => void] {
  const [own, setOwn] = useState(-1)
  const current = props.currentIndex ?? own
  const setCurrent = (index: number) => {
    setOwn(index)
    props.onCurrentIndexChange?.(index)
  }
  return [current, setCurrent]
}

function useListKeys<T>(
  props: SelectionListProps<T>,
  current: number,
  setCurrent: (i: number) => void,
) {
  const { items, isItemSelectable = always } = props
  const select = (index: number) => {
    const item = items[index]
    if (item === undefined || !isItemSelectable(index)) return
    setCurrent(index)
    props.onItemSelected?.(index, item)
  }
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
      event.preventDefault()
      setCurrent(stepIndex(current, event.key, items.length, isItemSelectable))
    } else if ((event.key === 'Enter' || event.key === ' ') && current !== -1) {
      event.preventDefault()
      select(current)
      if (props.keyboardAutoAdvance) queueMicrotask(() => props.onNext?.())
    }
  }
  return { select, onKeyDown }
}

function useListEffects(
  listRef: RefObject<HTMLDivElement | null>,
  current: number,
  setCurrent: (i: number) => void,
  autoSelect: boolean,
  count: number,
) {
  useEffect(() => {
    if (autoSelect && current === -1 && count > 0) setCurrent(0)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoSelect, count, current])
  useEffect(() => {
    const row = listRef.current?.children[current] as HTMLElement | undefined
    row?.scrollIntoView?.({ block: 'nearest' })
  }, [listRef, current])
}

export function SelectionList<T>(props: SelectionListProps<T>) {
  const { items, renderItem, getKey } = props
  const [current, setCurrent] = useCurrent(props)
  const [focused, setFocused] = useState(false)
  const listRef = useRef<HTMLDivElement>(null)
  const idPrefix = useId()
  const { select, onKeyDown } = useListKeys(props, current, setCurrent)
  useListEffects(listRef, current, setCurrent, props.autoSelectFirst ?? false, items.length)
  return (
    <Box
      ref={listRef}
      role="listbox"
      tabIndex={0}
      aria-label={props.ariaLabel ?? 'Selection list'}
      aria-description="Use arrow keys to navigate, Enter or Space to select"
      {...(current >= 0 ? { 'aria-activedescendant': `${idPrefix}-${current}` } : {})}
      onKeyDown={onKeyDown}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      sx={
        [
          { overflowY: 'auto', minHeight: 0, outline: 'none' },
          ...(Array.isArray(props.sx) ? props.sx : props.sx ? [props.sx] : []),
        ] as SxProps<Theme>
      }
    >
      {items.map((item, index) => (
        <Box
          key={getKey ? getKey(item, index) : index}
          id={`${idPrefix}-${index}`}
          role="option"
          aria-selected={index === current}
          onClick={() => select(index)}
          onDoubleClick={() => props.onItemDoubleClicked?.(index, item)}
          sx={{
            backgroundColor:
              index === current && focused
                ? colors.listViewHighlight
                : colors.listViewRowBackground,
          }}
        >
          {renderItem(item, index, index === current)}
        </Box>
      ))}
    </Box>
  )
}
