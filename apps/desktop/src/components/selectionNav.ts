/** Keyboard stepping of upstream `SelectionListView.qml`. */

/**
 * Index of the next selectable row from `from` in `direction` (+1/-1),
 * or `from` itself when none is left before the edge.
 */
export function findNextSelectableIndex(
  from: number,
  direction: 1 | -1,
  count: number,
  isSelectable: (index: number) => boolean,
): number {
  for (let next = from + direction; next >= 0 && next < count; next += direction) {
    if (isSelectable(next)) return next
  }
  return from
}

/** Up/Down handling: from no selection both keys go to the first selectable row. */
export function stepIndex(
  current: number,
  key: 'ArrowUp' | 'ArrowDown',
  count: number,
  isSelectable: (index: number) => boolean,
): number {
  if (count === 0) return current
  if (current === -1) {
    const first = findNextSelectableIndex(-1, 1, count, isSelectable)
    return first
  }
  if (key === 'ArrowUp')
    return current > 0 ? findNextSelectableIndex(current, -1, count, isSelectable) : current
  return current < count - 1 ? findNextSelectableIndex(current, 1, count, isSelectable) : current
}
